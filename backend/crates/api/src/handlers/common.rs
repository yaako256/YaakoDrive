/*
backend/crates/api/src/handlers/common.rs
NodeResponse と parse_user_id が node.rs と file.rs で重複するため共通モジュールへ移動。
*/

// 外部クレート
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// 内部ライブラリ
use identity::UserId;
use node::model::Node;

// 自クレート
use crate::error::ApiAppError;

/// Node 系 API の共通レスポンス型
#[derive(Serialize)]
pub struct NodeResponse {
  pub id: String,
  pub parent_id: Option<String>,
  pub name: String,
  pub node_type: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<node::model::Node> for NodeResponse {
  fn from(n: Node) -> Self {
    Self {
      id: n.id().to_string(),
      parent_id: n.parent_id().map(|id| id.to_string()),
      name: n.name().to_string(),
      node_type: n.node_type().as_str().to_string(),
      created_at: n.created_at(),
      updated_at: n.updated_at(),
    }
  }
}

/// JWT claims の sub 文字列を UserId に変換する共通ヘルパー
pub fn parse_user_id(sub: &str) -> Result<UserId, ApiAppError> {
  // uuid(文字列)をUUID型にパースする
  let uuid = Uuid::parse_str(sub).map_err(|_| ApiAppError::from(app::AppError::Unauthorized))?;

  // uuidをUserId型にして返す
  Ok(UserId::from_uuid(uuid))
}
