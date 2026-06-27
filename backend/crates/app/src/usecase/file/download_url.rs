/*
backend/crates/app/src/usecase/file/download_url.rs
ダウンロードURLを発行するユースケース
*/

// 外部クレート
use identity::{NodeId, UserId};
use repository::{FileContentRepository, NodeRepository};

// 自クレート
use crate::error::{AppError, AppResult};

pub struct GetDownloadUrlInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
}

pub struct GetDownloadUrlOutput {
  pub stored_filename: String,
  pub original_name: String,
  pub mime_type: String,
}

pub struct GetDownloadUrlUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
  file_content_repo: &'a dyn FileContentRepository,
}

impl<'a> GetDownloadUrlUseCase<'a> {
  pub fn new(
    node_repo: &'a dyn NodeRepository,
    file_content_repo: &'a dyn FileContentRepository,
  ) -> Self {
    Self {
      node_repo,
      file_content_repo,
    }
  }

  pub async fn execute(&self, input: GetDownloadUrlInput) -> AppResult<GetDownloadUrlOutput> {
    let node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or_else(|| AppError::NotFound("node not found".to_string()))?;

    if !node.is_owner(&input.requester_user_id) {
      return Err(AppError::NotFound("node not found".to_string()));
    }
    if node.is_deleted() {
      return Err(AppError::NotFound("node not found".to_string()));
    }
    if !node.is_file() {
      return Err(AppError::InvalidInput("node is not a file".to_string()));
    }

    let content = self
      .file_content_repo
      .find_by_node_id(&input.node_id)
      .await?
      .ok_or_else(|| AppError::NotFound("file content not found".to_string()))?;

    Ok(GetDownloadUrlOutput {
      stored_filename: content.stored_filename().to_string(),
      original_name: node.name().to_string(),
      mime_type: content.mime_type().to_string(),
    })
  }
}
