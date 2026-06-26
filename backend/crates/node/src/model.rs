/*
backend/crates/node/src/model.rs
nodeの型定義
*/

// 外部クレート
// 時間型用
use chrono::{DateTime, Utc};
use uuid::Uuid;

// 内部ライブラリ
// Id型
use identity::{NodeId, UserId};

// 自クレート
use crate::error::{NodeError, NodeResult};
use crate::name::validate_name;

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
  type Error = NodeError;

  // 文字列からNodeType型の取得
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    match s {
      "file" => Ok(NodeType::File),
      "folder" => Ok(NodeType::Folder),
      other => Err(NodeError::UnknownNodeType(other.to_string())),
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
  type Error = NodeError;

  /// NodeStatus文字列からNodeStatus型の取得を文字列変換する
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    match s {
      "pending" => Ok(NodeStatus::Pending),
      "active" => Ok(NodeStatus::Active),
      other => Err(NodeError::UnknownStatus(other.to_string())),
    }
  }
}

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

  /// リネームをする
  pub fn rename(&mut self, new_name: String) -> NodeResult<()> {
    // 名前の検証
    validate_name(&new_name)?;

    // 既に削除されているかチェック
    if self.deleted_at.is_some() {
      return Err(NodeError::AlreadyDeleted);
    }

    // 名前の更新
    self.name = new_name;

    // 更新時間も更新
    self.updated_at = Utc::now();

    Ok(())
  }
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
