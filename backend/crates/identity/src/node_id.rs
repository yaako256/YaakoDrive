/*
backend/crates/identity/src/node_id.rs
NodeId型の定義
*/

// 外部クレート
// シリアライズ用
use serde::{Deserialize, Serialize};
// UUID用
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
  /// NodeIdをインスタンス
  /// uuidは自動生成される
  pub fn new() -> Self {
    Self(Uuid::new_v4())
  }

  /// uuidからNodeId型を作る
  pub fn from_uuid(uuid: Uuid) -> Self {
    Self(uuid)
  }

  /// 自身のuuidを返す
  pub fn as_uuid(&self) -> &Uuid {
    &self.0
  }
}

impl Default for NodeId {
  /// defaultの定義
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for NodeId {
  /// 表示の定義
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
