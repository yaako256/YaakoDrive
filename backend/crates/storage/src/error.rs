/*
backend/crates/storage/src/error.rs
storageクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

/// storageクレートのエラー型
#[derive(Debug, Error)]
pub enum StorageError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("file not found: {0}")]
  NotFound(String),

  #[error("storage error: {0}")]
  Other(String),
}

/// Configクレートのリザルト
pub(crate) type StorageResult<T> = Result<T, StorageError>;
