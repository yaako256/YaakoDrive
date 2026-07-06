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

  #[error("already deleted")]
  AlreadyDeleted,

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
      repository::RepoError::Node(msg) => AppError::Node(msg),
      repository::RepoError::Auth(msg) => AppError::Auth(msg),
    }
  }
}

// AuthError → AppError
impl From<auth::AuthError> for AppError {
  fn from(e: auth::AuthError) -> Self {
    match e {
      auth::AuthError::InvalidCredentials
      | auth::AuthError::TokenExpired
      | auth::AuthError::InvalidToken => AppError::Auth(e.to_string()),
      // ハッシュ処理失敗はクライアント起因ではないためサーバーエラー扱い
      auth::AuthError::HashError(_) => AppError::Repository(e.to_string()),
      // バリデーション系はクライアントの入力不備
      auth::AuthError::InvalidRole(msg)
      | auth::AuthError::InvalidUsername(msg)
      | auth::AuthError::InvalidPassword(msg) => AppError::InvalidInput(msg),
    }
  }
}

// StorageError → AppError
impl From<storage::StorageError> for AppError {
  fn from(e: storage::StorageError) -> Self {
    match e {
      // ファイル実体がディスク上にない＝サーバー側のデータ不整合として扱う
      storage::StorageError::NotFound(msg) => AppError::Repository(msg),
      storage::StorageError::Io(e) => AppError::Repository(e.to_string()),
    }
  }
}

// NodeError → AppError
impl From<node::NodeError> for AppError {
  fn from(e: node::NodeError) -> Self {
    match e {
      node::NodeError::InvalidName(msg) => AppError::InvalidInput(msg),
      node::NodeError::CircularMove => {
        AppError::InvalidInput("cannot move a folder into its own descendant".to_string())
      }
      node::NodeError::MoveConflict => {
        AppError::AlreadyExists("same name already exists".to_string())
      }
      node::NodeError::NotFound => AppError::NotFound("node not found".to_string()),
      node::NodeError::AlreadyActive
      | node::NodeError::AlreadyDeleted
      | node::NodeError::UnknownNodeType(_)
      | node::NodeError::UnknownStatus(_) => AppError::Repository(e.to_string()),
    }
  }
}
