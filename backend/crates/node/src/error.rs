/*
backend/crates/node/src/error.rs
nodeクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

/// nodeクレートのエラー型
#[derive(Debug, Error)]
pub enum NodeError {
  #[error("node not found")]
  NotFound,

  #[error("a node with the same name already exists in the destination folder")]
  MoveConflict,

  #[error("cannot move a folder into its own descendant")]
  CircularMove,

  #[error("unknown node type: {0}")]
  UnknownNodeType(String),

  #[error("unknown node status: {0}")]
  UnknownStatus(String),

  #[error("invalid name: {0}")]
  InvalidName(String),

  #[error("node is already active")]
  AlreadyActive,

  #[error("node is already deleted")]
  AlreadyDeleted,
}

// nodeクレートのリザルト
pub type NodeResult<T> = Result<T, NodeError>;
