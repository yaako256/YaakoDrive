/*
backend/crates/infra/src/postgres/node_repository.rs
postgresのNodeRepository実体を定義
*/

// 外部クレート
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::{Node, NodeStatus, NodeType};
use repository::{NodeRepository, RepoError, RepoResult};

/// postgreSQLのNodeRepository実装
pub struct PgNodeRepository {
  /// DBコネクションプール
  pool: PgPool,
}

impl PgNodeRepository {
  /// コンストラクタ
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl NodeRepository for PgNodeRepository {
  /// -----------------------
  /// iDからNodeを取得
  /// -----------------------
  async fn find_by_id(&self, id: &NodeId) -> RepoResult<Option<Node>> {
    // sqlx::query! → コンパイル時にSQLを検証し、"匿名構造体（Record）" を生成する
    // Node列からidが一致するものを持ってくる
    let row = sqlx::query!(
      r#"SELECT id, owner_user_id, parent_id, name, node_type, status, deleted_at, created_at, updated_at
        FROM nodes 
        WHERE id = $1"#,
      id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await?;

    row
      .map(|r| {
        map_node_row(
          r.id,
          r.owner_user_id,
          r.parent_id,
          r.name,
          r.node_type,
          r.status,
          r.deleted_at,
          r.created_at,
          r.updated_at,
        )
      })
      .transpose()
  }

  async fn list_children(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
  ) -> RepoResult<Vec<Node>> {
    let rows = match parent_id {
      Some(pid) => sqlx::query!(
        "SELECT id, owner_user_id, parent_id, name, node_type, status,
                        deleted_at, created_at, updated_at
                 FROM nodes
                 WHERE owner_user_id = $1 AND parent_id = $2
                   AND deleted_at IS NULL AND status = 'active'
                 ORDER BY node_type DESC, name ASC",
        owner_user_id.as_uuid(),
        pid.as_uuid()
      )
      .fetch_all(&self.pool)
      .await
      .map_err(|e| RepoError::Database(e.to_string()))?,

      None => sqlx::query!(
        "SELECT id, owner_user_id, parent_id, name, node_type, status,
                        deleted_at, created_at, updated_at
                 FROM nodes
                 WHERE owner_user_id = $1 AND parent_id IS NULL
                   AND deleted_at IS NULL AND status = 'active'
                 ORDER BY node_type DESC, name ASC",
        owner_user_id.as_uuid()
      )
      .fetch_all(&self.pool)
      .await
      .map_err(|e| RepoError::Database(e.to_string()))?,
    };

    rows
      .into_iter()
      .map(|r| {
        map_node_row(
          r.id,
          r.owner_user_id,
          r.parent_id,
          r.name,
          r.node_type,
          r.status,
          r.deleted_at,
          r.created_at,
          r.updated_at,
        )
      })
      .collect()
  }

  async fn create(&self, node: &Node) -> RepoResult<()> {
    sqlx::query!(
      "INSERT INTO nodes
                (id, owner_user_id, parent_id, name, node_type, status,
                 deleted_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
      node.id.as_uuid(),
      node.owner_user_id.as_uuid(),
      node.parent_id.as_ref().map(|id| *id.as_uuid()),
      node.name,
      node.node_type.as_str(),
      node.status.as_str(),
      node.deleted_at,
      node.created_at,
      node.updated_at,
    )
    .execute(&self.pool)
    .await
    .map_err(|e| {
      // UNIQUE制約違反を Conflict に変換する
      if let sqlx::Error::Database(ref db_err) = e {
        if db_err.code().as_deref() == Some("23505") {
          return RepoError::Conflict("name already exists".to_string());
        }
      }
      RepoError::Database(e.to_string())
    })?;

    Ok(())
  }

  async fn update(&self, node: &Node) -> RepoResult<()> {
    let affected = sqlx::query!(
      "UPDATE nodes
             SET parent_id = $2, name = $3, status = $4,
                 deleted_at = $5, updated_at = $6
             WHERE id = $1",
      node.id.as_uuid(),
      node.parent_id.as_ref().map(|id| *id.as_uuid()),
      node.name,
      node.status.as_str(),
      node.deleted_at,
      node.updated_at,
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

  async fn soft_delete_with_descendants(
    &self,
    id: &NodeId,
    deleted_at: DateTime<Utc>,
  ) -> RepoResult<()> {
    // CTEで再帰的に子孫を取得してまとめてdeleted_atを設定する
    sqlx::query!(
      "WITH RECURSIVE descendants AS (
                SELECT id FROM nodes WHERE id = $1
                UNION ALL
                SELECT n.id FROM nodes n
                INNER JOIN descendants d ON n.parent_id = d.id
            )
            UPDATE nodes SET deleted_at = $2, updated_at = $2
            WHERE id IN (SELECT id FROM descendants)
              AND deleted_at IS NULL",
      id.as_uuid(),
      deleted_at,
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(())
  }

  async fn hard_delete(&self, id: &NodeId) -> RepoResult<()> {
    sqlx::query!("DELETE FROM nodes WHERE id = $1", id.as_uuid())
      .execute(&self.pool)
      .await
      .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(())
  }

  async fn find_ancestor_ids(&self, id: &NodeId) -> RepoResult<Vec<NodeId>> {
    let rows = sqlx::query!(
      "WITH RECURSIVE ancestors AS (
                SELECT parent_id FROM nodes WHERE id = $1
                UNION ALL
                SELECT n.parent_id FROM nodes n
                INNER JOIN ancestors a ON n.id = a.parent_id
            )
            SELECT parent_id FROM ancestors WHERE parent_id IS NOT NULL",
      id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(
      rows
        .into_iter()
        .filter_map(|r| r.parent_id)
        .map(NodeId::from_uuid)
        .collect(),
    )
  }
}

fn map_node_row(
  id: Uuid,
  owner_user_id: Uuid,
  parent_id: Option<Uuid>,
  name: String,
  node_type: String,
  status: String,
  deleted_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
) -> RepoResult<Node> {
  let node_type = NodeType::try_from(node_type.as_str()).map_err(|e| RepoError::Database(e))?;
  let status = NodeStatus::try_from(status.as_str()).map_err(|e| RepoError::Database(e))?;

  Ok(Node {
    id: NodeId::from_uuid(id),
    owner_user_id: UserId::from_uuid(owner_user_id),
    parent_id: parent_id.map(NodeId::from_uuid),
    name,
    node_type,
    status,
    deleted_at,
    created_at,
    updated_at,
  })
}
