/*
backend/crates/repository/src/error.rs
repositoryクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

/// repositoryクレートのエラー型
#[derive(Debug, Error)]
pub enum RepoError {
  #[error("not found")]
  NotFound,

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("database error: {0}")]
  Database(String),

  #[error("node error: {0}")]
  Node(String),
}

/// repositoryクレートのリザルト
pub type RepoResult<T> = Result<T, RepoError>;
