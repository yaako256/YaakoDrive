/*
backend/crates/auth/src/password.rs
パスワードのハッシュ化関数などを定義
*/

// 外部クレート
// ハッシュ用
use argon2::{
  Argon2,
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

// 自クレート
use crate::error::{AuthError, AuthResult};

/// パスワードのハッシュ化
pub fn hash_password(password: &str) -> AuthResult<String> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  argon2
    .hash_password(password.as_bytes(), &salt)
    .map(|h| h.to_string())
    .map_err(|e| AuthError::HashError(e.to_string()))
}

/// パスワードの認証
pub fn verify_password(password: &str, hash: &str) -> AuthResult<()> {
  let parsed_hash = PasswordHash::new(hash).map_err(|e| AuthError::HashError(e.to_string()))?;
  Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .map_err(|_| AuthError::InvalidCredentials)
}
