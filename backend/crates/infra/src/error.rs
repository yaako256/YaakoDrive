/*
backend/crates/infra/src/error.rs
infraクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

// 内部ライブラリ
// Repoクレートのエラー型
use repository::RepoError;

/// infraクレートのエラー型
#[derive(Debug, Error)]
pub enum InfraError {
  #[error("database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("not found")]
  NotFound,

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("invalid data: {0}")]
  InvalidData(String),
}

impl From<InfraError> for RepoError {
  // infraエラーをRepoErrorに変換
  fn from(e: InfraError) -> Self {
    match e {
      InfraError::NotFound => RepoError::NotFound,
      InfraError::Conflict(msg) => RepoError::Conflict(msg),
      InfraError::Database(e) => RepoError::Database(e.to_string()),
      // DBから不正な値が来た＝DBエラーとして扱う
      InfraError::InvalidData(msg) => RepoError::Database(msg),
    }
  }
}

impl From<String> for InfraError {
  fn from(e: String) -> Self {
    InfraError::InvalidData(e)
  }
}

/// nodeクレートのリザルト
pub type InfraResult<T> = Result<T, InfraError>;
