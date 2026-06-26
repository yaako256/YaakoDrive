/*
backend/crates/auth/src/error.rs
authクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

/// authクレートのエラー型
#[derive(Debug, Error)]
pub enum AuthError {
  #[error("invalid credentials")]
  InvalidCredentials,

  #[error("token expired")]
  TokenExpired,

  #[error("invalid token")]
  InvalidToken,

  #[error("password hash error: {0}")]
  HashError(String),
}

/// authクレートのリザルト
pub(crate) type AuthResult<T> = Result<T, AuthError>;
