/*
backend/crates/node/src/model.rs
nodeの型定義
*/

// 外部クレート
// 時間型用
use chrono::{DateTime, Utc};

// 内部ライブラリ
// Id型
use identity::{NodeId, UserId};

/// Node種類の列挙
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
  File,
  Folder,
}

impl NodeType {
  /// NodeTypeを文字列変換する
  pub fn as_str(&self) -> &'static str {
    match self {
      NodeType::File => "file",
      NodeType::Folder => "folder",
    }
  }
}

impl TryFrom<&str> for NodeType {
  type Error = String;

  // 文字列からNodeType型の取得
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    match s {
      "file" => Ok(NodeType::File),
      "folder" => Ok(NodeType::Folder),
      other => Err(format!("unknown node_type: {}", other)),
    }
  }
}

/// Nodeの状態定義
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
  // 現在ロード中(or 異常)
  Pending,
  // 正常(ロード完了)
  Active,
}

impl NodeStatus {
  /// NodeStatusを文字列変換する
  pub fn as_str(&self) -> &'static str {
    match self {
      NodeStatus::Pending => "pending",
      NodeStatus::Active => "active",
    }
  }
}

impl TryFrom<&str> for NodeStatus {
  type Error = String;

  /// NodeStatus文字列からNodeStatus型の取得を文字列変換する
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    match s {
      "pending" => Ok(NodeStatus::Pending),
      "active" => Ok(NodeStatus::Active),
      other => Err(format!("unknown status: {}", other)),
    }
  }
}

/// Node構造体
#[derive(Debug, Clone)]
pub struct Node {
  pub id: NodeId,
  pub owner_user_id: UserId,
  pub parent_id: Option<NodeId>,
  pub name: String,
  pub node_type: NodeType,
  pub status: NodeStatus,
  pub deleted_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Node {
  /// フォルダーかどうか
  pub fn is_folder(&self) -> bool {
    self.node_type == NodeType::Folder
  }

  /// ファイルかどうか
  pub fn is_file(&self) -> bool {
    self.node_type == NodeType::File
  }

  /// 削除されているかどうか
  pub fn is_deleted(&self) -> bool {
    self.deleted_at.is_some()
  }

  /// activeかどうか
  pub fn is_active(&self) -> bool {
    self.status == NodeStatus::Active && !self.is_deleted()
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileContentStatus {
  Pending,
  Active,
}

impl FileContentStatus {
  pub fn as_str(&self) -> &'static str {
    match self {
      FileContentStatus::Pending => "pending",
      FileContentStatus::Active => "active",
    }
  }
}

#[derive(Debug, Clone)]
pub struct FileContent {
  pub node_id: NodeId,
  pub stored_filename: String,
  pub mime_type: String,
  pub size_bytes: i64,
  pub status: FileContentStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
