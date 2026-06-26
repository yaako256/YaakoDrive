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

  #[error("name already exists in this folder")]
  NameConflict,

  #[error("cannot move folder into its own descendant")]
  CircularMove,

  #[error("invalid name: {0}")]
  InvalidName(String),

  #[error("operation not allowed on deleted node")]
  AlreadyDeleted,
}

// nodeクレートのリザルト
// 現状未使用のためコメントアウト
pub type NodeResult<T> = Result<T, NodeError>;
