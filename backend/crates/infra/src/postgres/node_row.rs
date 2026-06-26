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
    Ok(Node::reconstitute(
      NodeId::from_uuid(row.id),
      UserId::from_uuid(row.owner_user_id),
      row.parent_id.map(NodeId::from_uuid),
      row.name,
      NodeType::try_from(row.node_type.as_str())?,
      NodeStatus::try_from(row.status.as_str())?,
      row.deleted_at,
      row.created_at,
      row.updated_at,
    ))
  }
}
