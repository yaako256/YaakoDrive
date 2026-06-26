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

/// Node構造体
#[derive(Debug, Clone)]
pub struct Node {
  id: NodeId,
  owner_user_id: UserId,
  parent_id: Option<NodeId>,
  name: String,
  node_type: NodeType,
  status: NodeStatus,
  deleted_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

impl Node {
  // ---- コンストラクタ系 ----

  /// 新規fileの作成
  /// pending状態で作成
  pub fn new_file(
    node_id: NodeId,
    owner_user_id: UserId,
    parent_id: Option<NodeId>,
    filename: &str,
  ) -> NodeResult<Self> {
    // 名前の検証
    validate_name(&filename)?;

    Ok(Self {
      id: node_id,
      owner_user_id: owner_user_id,
      parent_id: parent_id,
      name: filename.to_string(),
      node_type: NodeType::File,
      status: NodeStatus::Pending,
      deleted_at: None,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    })
  }

  /// 新規フォルダの作成
  pub fn new_folder(
    owner_user_id: UserId,
    parent_id: Option<NodeId>,
    name: String,
  ) -> NodeResult<Self> {
    // 名前の検証
    validate_name(&name)?;

    Ok(Self {
      id: NodeId::new(),
      owner_user_id: owner_user_id,
      parent_id: parent_id,
      name: name,
      node_type: NodeType::Folder,
      status: NodeStatus::Active,
      deleted_at: None,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    })
  }

  // ---- ゲッター関数 or 真偽関数 ----
  // 後で必要なゲッター関数を作る

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

  /// updated_atを更新する
  fn touch(&mut self) {
    self.updated_at = Utc::now();
  }

  /// statusをActiveにする
  pub fn activate(&mut self) -> NodeResult<()> {
    // 既に削除されているか
    if self.is_deleted() {
      return Err(NodeError::AlreadyDeleted);
    }

    // 既にactiveかどうかの確認
    if self.is_active() {
      return Err(NodeError::AlreadyActive);
    }

    // statusをActiveにする
    self.status = NodeStatus::Active;

    // 更新時間更新
    self.touch();

    Ok(())
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

    // 更新時間更新
    self.touch();

    Ok(())
  }

  /// 移動する
  // moveは予約語なので使えない
  pub fn move_node(&mut self, new_parent: Option<NodeId>, ancestors: &[NodeId]) -> NodeResult<()> {
    // 祖先との循環を防ぐ
    if let Some(parent) = new_parent {
      if ancestors.contains(&parent) {
        return Err(NodeError::CircularMove);
      }
    }

    // 親のNodeIdを更新
    self.parent_id = new_parent;

    // 更新時間更新
    self.touch();

    Ok(())
  }

  /// ゴミ箱に入れる(論理削除)
  pub fn soft_delete(&mut self) -> NodeResult<()> {
    // まだ削除されていないかチェック
    if self.is_deleted() {
      return Err(NodeError::AlreadyDeleted);
    }

    // deleted_atを記入
    self.deleted_at = Some(Utc::now());

    // 更新時間更新
    self.touch();

    Ok(())
  }

  /// ゴミ箱から戻す
  pub fn restore(&mut self) -> NodeResult<()> {
    // 既に削除されているかのチェック
    if !self.is_deleted() {
      return Err(NodeError::AlreadyActive);
    }

    // deleted_atをNoneにする
    self.deleted_at = None;

    // 更新時間更新
    self.touch();

    Ok(())
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
  node_id: NodeId,
  stored_filename: String,
  mime_type: String,
  size_bytes: i64,
  status: FileContentStatus,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}
impl FileContent {
  /// 新規fileの作成
  /// pendingで作成する
  pub fn new_file_content(
    node_id: NodeId,
    mime_type: String,
    size_bytes: i64,
    stored_filename: String,
  ) -> Self {
    Self {
      node_id: node_id,
      stored_filename: stored_filename.clone(),
      mime_type: mime_type,
      size_bytes: size_bytes,
      status: FileContentStatus::Pending,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }

  /// Activeかの確認
  pub fn is_active(&self) -> bool {
    self.status == FileContentStatus::Active
  }

  /// updated_atを更新する
  fn touch(&mut self) {
    self.updated_at = Utc::now();
  }

  /// statusをActiveにする
  pub fn activate(&mut self) -> NodeResult<()> {
    // 既にactiveかどうかの確認
    if self.is_active() {
      return Err(NodeError::AlreadyActive);
    }

    // statusをActiveにする
    self.status = FileContentStatus::Active;

    // 更新時間更新
    self.touch();

    Ok(())
  }
}
