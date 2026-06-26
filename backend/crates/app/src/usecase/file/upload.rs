/*
backend/crates/app/src/usecase/file/upload.rs
アップロードユースケース
*/

// 外部クレート
use chrono::Utc;
use mime_guess::from_path;
use uuid::Uuid;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::{FileContent, FileContentStatus, Node, NodeStatus, NodeType};
use repository::{NodeRepository, TransactionContext, UnitOfWork};
use storage::StorageService;

// 自クレート
use crate::error::{AppError, AppResult};

// アップロードの入力
pub struct UploadFileInput {
  pub owner_user_id: UserId,
  pub parent_id: Option<NodeId>,
  pub filename: String,
  pub content_length: Option<u64>,
  pub stream: storage::service::ByteStream,
}

// アップロードの出力
pub struct UploadFileOutput {
  pub node: Node,
  pub size_bytes: i64,
}

// アップロードのユースケース構造体
pub struct UploadFileUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
  uow: &'a dyn UnitOfWork,
  storage: &'a dyn StorageService,
  max_size_bytes: u64,
}

impl<'a> UploadFileUseCase<'a> {
  /// コンストラクタ
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
    // ファイルサイズの事前チェック（Content-Length がある場合）
    if let Some(len) = input.content_length {
      if len > self.max_size_bytes {
        return Err(AppError::StorageLimitExceeded);
      }
    }

    // 親フォルダの存在確認
    if let Some(ref parent_id) = input.parent_id {
      let parent = self
        .node_repo
        .find_by_id(parent_id)
        .await?
        .ok_or(AppError::NotFound("parent folder not found".to_string()))?;

      if parent.owner_user_id != input.owner_user_id {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }
      if parent.is_deleted() {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }
      if !parent.is_folder() {
        return Err(AppError::InvalidInput("parent is not a folder".to_string()));
      }
    }

    // ① 一時ファイルへストリーミング保存
    let temp_filename = self
      .storage
      .save_temp_stream(input.stream, &input.filename)
      .await?;

    // 以降はエラー時に一時ファイルを削除する必要がある
    let result = self
      .upload_inner(
        &temp_filename,
        input.owner_user_id,
        input.parent_id,
        &input.filename,
      )
      .await;

    if let Err(ref _e) = result {
      // 一時ファイルのクリーンアップ（失敗しても無視）
      let _ = self.storage.delete_temp_file(&temp_filename).await;
    }

    result
  }

  async fn upload_inner(
    &self,
    temp_filename: &str,
    owner_user_id: UserId,
    parent_id: Option<NodeId>,
    filename: &str,
  ) -> AppResult<UploadFileOutput> {
    // ファイルサイズを取得（一時ファイルから）
    // promote後のサイズ確認は不要なので一時ファイルで計測
    let size_bytes = self.storage.get_file_size(temp_filename).await? as i64;

    if size_bytes > self.max_size_bytes as i64 {
      return Err(AppError::StorageLimitExceeded);
    }

    let mime_type = from_path(filename).first_or_octet_stream().to_string();

    // 正式ファイル名（UUID.拡張子）
    let ext = std::path::Path::new(filename)
      .extension()
      .and_then(|e| e.to_str())
      .unwrap_or("");
    let stored_filename = if ext.is_empty() {
      format!("{}.dat", Uuid::new_v4())
    } else {
      format!("{}.{}", Uuid::new_v4(), ext)
    };

    let now = Utc::now();
    let node_id = NodeId::new();

    // pending 状態の Node
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

    // pending 状態の FileContent
    let file_content = FileContent {
      node_id,
      stored_filename: stored_filename.clone(),
      mime_type,
      size_bytes,
      status: FileContentStatus::Pending,
      created_at: now,
      updated_at: now,
    };

    // ② トランザクション開始
    let mut tx = self.uow.begin().await?;

    // ③ pending で DB 登録
    tx.insert_node(&node).await.map_err(|e| match e {
      repository::RepoError::Conflict(_) => {
        AppError::AlreadyExists("same name already exists".to_string())
      }
      other => AppError::from(other),
    })?;
    tx.insert_file_content(&file_content).await?;

    // ④ 一時ファイルを正式配置へ移動
    if let Err(e) = self
      .storage
      .promote_file(temp_filename, &stored_filename)
      .await
    {
      tx.rollback().await.ok();
      return Err(AppError::from(e));
    }

    // ⑤ active に更新
    let mut active_node = node.clone();
    active_node.status = NodeStatus::Active;
    active_node.updated_at = Utc::now();

    let mut active_content = file_content.clone();
    active_content.status = FileContentStatus::Active;
    active_content.updated_at = Utc::now();

    tx.update_node(&active_node).await?;
    tx.update_file_content(&active_content).await?;

    // ⑥ コミット
    tx.commit().await?;

    Ok(UploadFileOutput {
      node: active_node,
      size_bytes,
    })
  }
}
