/*
backend/crates/infra/src/postgres/file_content_repository.rs
postgresのFileContentRepository実体を定義
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;

// 内部ライブラリ
use identity::NodeId;
use node::model::{FileContent, FileContentStatus};
use repository::{FileContentRepository, RepoError, RepoResult};

pub struct PgFileContentRepository {
  pool: PgPool,
}

impl PgFileContentRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl FileContentRepository for PgFileContentRepository {
  async fn find_by_node_id(&self, node_id: &NodeId) -> RepoResult<Option<FileContent>> {
    let row = sqlx::query!(
      "SELECT node_id, stored_filename, mime_type, size_bytes, status,
                    created_at, updated_at
             FROM file_contents WHERE node_id = $1",
      node_id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(row.map(|r| FileContent {
      node_id: NodeId::from_uuid(r.node_id),
      stored_filename: r.stored_filename,
      mime_type: r.mime_type,
      size_bytes: r.size_bytes,
      status: if r.status == "active" {
        FileContentStatus::Active
      } else {
        FileContentStatus::Pending
      },
      created_at: r.created_at,
      updated_at: r.updated_at,
    }))
  }

  async fn create(&self, content: &FileContent) -> RepoResult<()> {
    sqlx::query!(
      "INSERT INTO file_contents
                (node_id, stored_filename, mime_type, size_bytes, status,
                 created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
      content.node_id.as_uuid(),
      content.stored_filename,
      content.mime_type,
      content.size_bytes,
      content.status.as_str(),
      content.created_at,
      content.updated_at,
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(())
  }

  async fn update(&self, content: &FileContent) -> RepoResult<()> {
    let affected = sqlx::query!(
      "UPDATE file_contents
             SET stored_filename = $2, mime_type = $3, size_bytes = $4,
                 status = $5, updated_at = $6
             WHERE node_id = $1",
      content.node_id.as_uuid(),
      content.stored_filename,
      content.mime_type,
      content.size_bytes,
      content.status.as_str(),
      content.updated_at,
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?
    .rows_affected();

    if affected == 0 {
      return Err(RepoError::NotFound);
    }
    Ok(())
  }

  async fn hard_delete(&self, node_id: &NodeId) -> RepoResult<()> {
    sqlx::query!(
      "DELETE FROM file_contents WHERE node_id = $1",
      node_id.as_uuid()
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(())
  }
}
