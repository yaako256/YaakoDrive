/*
backend/crates/app/src/error.rs
appクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

/// appクレートのエラー型
#[derive(Debug, Error)]
pub enum AppError {
  #[error("already exists: {0}")]
  AlreadyExists(String),

  #[error("not found: {0}")]
  NotFound(String),

  #[error("unauthorized")]
  Unauthorized,

  #[error("forbidden")]
  Forbidden,

  #[error("invalid input: {0}")]
  InvalidInput(String),

  #[error("repository error: {0}")]
  Repository(String),

  #[error("auth error: {0}")]
  Auth(String),

  #[error("storage error: {0}")]
  Storage(String),

  #[error("node error: {0}")]
  Node(String),

  #[error("storage limit exceeded")]
  StorageLimitExceeded,
}

// 今後apiクレートで使いたくなるかもしれないため、(crate)を外す
//pub(crate) type AppResult<T> = Result<T, AppError>;
pub type AppResult<T> = Result<T, AppError>;

// RepoError → AppError
impl From<repository::RepoError> for AppError {
  fn from(e: repository::RepoError) -> Self {
    match e {
      repository::RepoError::NotFound => AppError::NotFound("resource".to_string()),
      repository::RepoError::Conflict(msg) => AppError::AlreadyExists(msg),
      repository::RepoError::Database(msg) => AppError::Repository(msg),
    }
  }
}

// AuthError → AppError
impl From<auth::AuthError> for AppError {
  fn from(e: auth::AuthError) -> Self {
    AppError::Auth(e.to_string())
  }
}

// StorageError → AppError
impl From<storage::StorageError> for AppError {
  fn from(e: storage::StorageError) -> Self {
    AppError::Storage(e.to_string())
  }
}

// NodeError → AppError
impl From<node::NodeError> for AppError {
  fn from(e: node::NodeError) -> Self {
    AppError::Node(e.to_string())
  }
}
