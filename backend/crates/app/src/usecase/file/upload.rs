/*
backend/crates/app/src/usecase/file/upload.rs
アップロードユースケース
*/

// 外部クレート
use bytes::Bytes;
use chrono::Utc;
use uuid::Uuid;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::{FileContent, FileContentStatus, Node, NodeStatus, NodeType};
use repository::{NodeRepository, RepoError, /*TransactionContext,*/ UnitOfWork};
use storage::service::{StorageService, TempFile};

// 自クレート
use crate::error::{AppError, AppResult};

pub struct UploadFileInput {
  pub owner_user_id: UserId,
  /// None はルート直下
  pub parent_id: Option<NodeId>,
  pub filename: String,
  pub data: Bytes,
}

pub struct UploadFileOutput {
  pub node: Node,
}

pub struct UploadFileUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
  uow: &'a dyn UnitOfWork,
  storage: &'a dyn StorageService,
  max_size_bytes: u64,
}

impl<'a> UploadFileUseCase<'a> {
  pub fn new(
    node_repo: &'a dyn NodeRepository,
    uow: &'a dyn UnitOfWork,
    storage: &'a dyn StorageService,
    max_size_bytes: u64,
  ) -> Self {
    Self {
      node_repo,
      uow,
      storage,
      max_size_bytes,
    }
  }

  pub async fn execute(&self, input: UploadFileInput) -> AppResult<UploadFileOutput> {
    let UploadFileInput {
      owner_user_id,
      parent_id,
      filename,
      data,
    } = input;

    // サイズチェック（メモリ上で確認できるため先に行う）
    if data.len() as u64 > self.max_size_bytes {
      return Err(AppError::StorageLimitExceeded);
    }

    // 親フォルダの確認
    if let Some(ref pid) = parent_id {
      let parent = self
        .node_repo
        .find_by_id(pid)
        .await?
        .ok_or_else(|| AppError::NotFound("parent folder not found".to_string()))?;

      if parent.owner_user_id != owner_user_id {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }
      if parent.is_deleted() {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }
      if !parent.is_folder() {
        return Err(AppError::InvalidInput("parent is not a folder".to_string()));
      }
    }

    // ① 一時ファイルへ保存
    let temp = self.storage.save_temp(data, &filename).await?;

    // 以降で失敗した場合は必ず一時ファイルを削除する
    match self
      .commit_upload(owner_user_id, parent_id, &filename, &temp)
      .await
    {
      Ok(output) => Ok(output),
      Err(e) => {
        let _ = self.storage.delete_temp(&temp.filename).await;
        Err(e)
      }
    }
  }

  /// トランザクションを伴うアップロード処理。
  ///
  /// 各ステップで失敗した場合の cleanup:
  /// - promote 前のエラー: tx をロールバック（一時ファイルは呼び出し元が削除）
  /// - promote 後のエラー: tx をロールバック＋正式ファイルを削除
  /// - commit 失敗: 正式ファイルを削除（tx は Drop で自動ロールバック）
  async fn commit_upload(
    &self,
    owner_user_id: UserId,
    parent_id: Option<NodeId>,
    filename: &str,
    temp: &TempFile,
  ) -> AppResult<UploadFileOutput> {
    let now = Utc::now();
    let node_id = NodeId::new();

    // 正式ファイル名: UUID + 元の拡張子
    let stored_filename = {
      let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
      if ext.is_empty() {
        format!("{}.dat", Uuid::new_v4())
      } else {
        format!("{}.{}", Uuid::new_v4(), ext)
      }
    };

    let mime_type = mime_guess::from_path(filename)
      .first_or_octet_stream()
      .to_string();

    // pending 状態のレコードを準備
    let node = Node {
      id: node_id,
      owner_user_id,
      parent_id,
      name: filename.to_string(),
      node_type: NodeType::File,
      status: NodeStatus::Pending,
      deleted_at: None,
      created_at: now,
      updated_at: now,
    };
    let file_content = FileContent {
      node_id,
      stored_filename: stored_filename.clone(),
      mime_type,
      size_bytes: temp.size_bytes as i64,
      status: FileContentStatus::Pending,
      created_at: now,
      updated_at: now,
    };

    // ② トランザクション開始
    let mut tx = self.uow.begin().await?;

    // ③ pending で DB 登録
    if let Err(e) = tx.insert_node(&node).await {
      tx.rollback().await.ok();
      return Err(match e {
        RepoError::Conflict(_) => AppError::AlreadyExists("same name already exists".to_string()),
        other => AppError::from(other),
      });
    }

    if let Err(e) = tx.insert_file_content(&file_content).await {
      tx.rollback().await.ok();
      return Err(AppError::from(e));
    }

    // ④ 一時ファイルを正式配置へ移動
    if let Err(e) = self.storage.promote(&temp.filename, &stored_filename).await {
      tx.rollback().await.ok();
      return Err(AppError::from(e));
    }

    // ⑤ active に更新（promote 成功後のエラーは正式ファイルも削除する）
    //
    // Node / FileContent は Clone を持たないため struct update syntax で move する。
    // insert_node / insert_file_content での &node / &file_content の借用は
    // .await 完了時点で解放されているため、ここで move しても問題ない。
    let active_node = Node {
      status: NodeStatus::Active,
      updated_at: Utc::now(),
      ..node // node を move する
    };

    if let Err(e) = tx.update_node(&active_node).await {
      tx.rollback().await.ok();
      let _ = self.storage.delete(&stored_filename).await;
      return Err(AppError::from(e));
    }

    let active_content = FileContent {
      status: FileContentStatus::Active,
      updated_at: Utc::now(),
      ..file_content // file_content を move する
    };

    if let Err(e) = tx.update_file_content(&active_content).await {
      tx.rollback().await.ok();
      let _ = self.storage.delete(&stored_filename).await;
      return Err(AppError::from(e));
    }

    // ⑥ コミット（tx を消費する）
    if let Err(e) = tx.commit().await {
      // commit 失敗時は tx が Drop されロールバックされるため明示的 rollback は不要
      let _ = self.storage.delete(&stored_filename).await;
      return Err(AppError::from(e));
    }

    Ok(UploadFileOutput { node: active_node })
  }
}
