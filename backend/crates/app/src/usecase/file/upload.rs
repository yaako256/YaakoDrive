/*
backend/crates/app/src/usecase/file/upload.rs
アップロードユースケース
*/

// 外部クレート
use bytes::Bytes;
use uuid::Uuid;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::{FileContent, Node};
use repository::{FileContentRepository, NodeRepository, UnitOfWork, UserRepository};
use storage::service::{StorageService, TempFile};

// 自クレート
use crate::error::{AppError, AppResult};
use crate::usecase::node::map_name_conflict;

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
  file_content_repo: &'a dyn FileContentRepository,
  user_repo: &'a dyn UserRepository,
  uow: &'a dyn UnitOfWork,
  storage: &'a dyn StorageService,
  max_size_bytes: u64,
}

impl<'a> UploadFileUseCase<'a> {
  pub fn new(
    node_repo: &'a dyn NodeRepository,
    file_content_repo: &'a dyn FileContentRepository,
    user_repo: &'a dyn UserRepository,
    uow: &'a dyn UnitOfWork,
    storage: &'a dyn StorageService,
    max_size_bytes: u64,
  ) -> Self {
    Self {
      node_repo,
      file_content_repo,
      user_repo,
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

    // ユーザごとのストレージ上限チェック
    // ユーザの統計情報取得
    let usage = self
      .file_content_repo
      .get_usage_stats(&owner_user_id)
      .await?;
    // id からユーザ型を取得
    let user = self
      .user_repo
      .find_by_id(&owner_user_id)
      .await?
      .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
    // ストレージ上限チェック
    if usage.total_bytes + data.len() as i64 > *user.storage_limit_bytes() {
      return Err(AppError::StorageLimitExceeded);
    }

    // アップロード上限サイズチェック（メモリ上で確認できるため先に行う）
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

      // 自分のフォルダじゃなかったらエラー
      if !parent.is_owner(&owner_user_id) {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }
      // 削除済みならエラー
      if parent.is_deleted() {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }
      // フォルダじゃなかったらエラー
      if !parent.is_folder() {
        return Err(AppError::InvalidInput("parent is not a folder".to_string()));
      }
    }

    // 一時ファイルへ保存
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

    // MIME Typeの測定
    let mime_type = mime_guess::from_path(filename)
      .first_or_octet_stream()
      .to_string();

    // 新規NodeIdの作成
    let node_id = NodeId::new();

    // pending 状態のレコードを準備
    let node = Node::new_file(node_id, owner_user_id, parent_id, filename.to_string())?;
    let file_content = FileContent::new_file_content(
      node_id,
      stored_filename.clone(),
      mime_type,
      temp.size_bytes as i64,
    )?;

    // トランザクション開始
    let mut tx = self.uow.begin().await?;

    // pending で DB 登録
    if let Err(e) = tx.insert_node(&node).await {
      tx.rollback().await.ok();
      return Err(map_name_conflict(e));
    }

    if let Err(e) = tx.insert_file_content(&file_content).await {
      tx.rollback().await.ok();
      return Err(AppError::from(e));
    }

    // 一時ファイルを正式配置へ移動
    if let Err(e) = self.storage.promote(&temp.filename, &stored_filename).await {
      tx.rollback().await.ok();
      return Err(AppError::from(e));
    }

    // node を active に更新
    // (promote 成功後のエラーは正式ファイルも削除する)
    let mut active_node = node;
    active_node.activate()?;

    if let Err(e) = tx.update_node(&active_node).await {
      tx.rollback().await.ok();
      let _ = self.storage.delete(&stored_filename).await;
      return Err(AppError::from(e));
    }

    // file_content を active に更新
    let mut active_content = file_content;
    active_content.activate()?;

    if let Err(e) = tx.update_file_content(&active_content).await {
      tx.rollback().await.ok();
      let _ = self.storage.delete(&stored_filename).await;
      return Err(AppError::from(e));
    }

    // コミット（tx を消費する）
    if let Err(e) = tx.commit().await {
      // commit 失敗時は tx が Drop されロールバックされるため明示的 rollback は不要
      let _ = self.storage.delete(&stored_filename).await;
      return Err(AppError::from(e));
    }

    Ok(UploadFileOutput { node: active_node })
  }
}
