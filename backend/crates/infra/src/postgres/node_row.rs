/*
backend/crates/infra/src/postgres/node_row.rs
query_as用のNodeRowを定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use uuid::Uuid;

// 内部ライブラリ
// Id型
use identity::{NodeId, UserId};
// node型
use node::NodeError;
use node::model::{Node, NodeStatus, NodeType};

// query_asにする用の構造体
// fromで変換し安全性を高める
#[derive(Debug)]
pub struct NodeRow {
  pub id: Uuid,
  pub owner_user_id: Uuid,
  pub parent_id: Option<Uuid>,
  pub name: String,
  pub node_type: String,
  pub status: String,
  pub deleted_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl TryFrom<NodeRow> for Node {
  type Error = NodeError;

  fn try_from(row: NodeRow) -> Result<Self, Self::Error> {
    Ok(Node {
      id: NodeId::from_uuid(row.id),
      owner_user_id: UserId::from_uuid(row.owner_user_id),
      parent_id: row.parent_id.map(NodeId::from_uuid),
      name: row.name,
      node_type: NodeType::try_from(row.node_type.as_str())?,
      status: NodeStatus::try_from(row.status.as_str())?,
      deleted_at: row.deleted_at,
      created_at: row.created_at,
      updated_at: row.updated_at,
    })
  }
}
