/*
backend/crates/app/src/usecase/file/download.rs
ダウンロードユースケース
*/

// 外部クレート
use identity::{NodeId, UserId};
use repository::{FileContentRepository, NodeRepository};

// 自クレート
use crate::error::{AppError, AppResult};

pub struct GetDownloadInfoInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
}

pub struct GetDownloadInfoOutput {
  pub stored_filename: String,
  pub original_name: String,
  pub mime_type: String,
}

pub struct GetDownloadInfoUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
  file_content_repo: &'a dyn FileContentRepository,
}

impl<'a> GetDownloadInfoUseCase<'a> {
  pub fn new(
    node_repo: &'a dyn NodeRepository,
    file_content_repo: &'a dyn FileContentRepository,
  ) -> Self {
    Self {
      node_repo,
      file_content_repo,
    }
  }

  pub async fn execute(&self, input: GetDownloadInfoInput) -> AppResult<GetDownloadInfoOutput> {
    let node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or_else(|| AppError::NotFound("node not found".to_string()))?;

    if node.owner_user_id != input.requester_user_id {
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

    Ok(GetDownloadInfoOutput {
      stored_filename: content.stored_filename,
      original_name: node.name,
      mime_type: content.mime_type,
    })
  }
}
