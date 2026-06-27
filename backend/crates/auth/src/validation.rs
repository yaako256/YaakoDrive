/*
backend/crates/auth/src/validation.rs
ユーザ名とパスワードのバリテーションの定義
*/

// 自クレート
use crate::error::{AuthError, AuthResult};

/// ユーザ名の検証
// 今後、短すぎる/長すぎる/文字化けするユーザ名禁止処理などを入れる
pub fn validate_username(username: &str) -> AuthResult<()> {
  // 空の名前は使えない
  if username.is_empty() {
    return Err(AuthError::InvalidUsername("username is empty".to_string()));
  }

  Ok(())
}

/// パスワードの検証
// 今後、数字を含むとかの処理を入れる
pub fn validate_password(password: &str) -> AuthResult<()> {
  // 空のパスワードは使えない
  if password.is_empty() {
    return Err(AuthError::InvalidPassword("password is empty".to_string()));
  }
  // 8文字以下のパスワードは使えない
  if password.len() < 8 {
    return Err(AuthError::InvalidPassword(
      "password must be at least 8 characters".to_string(),
    ));
  }

  Ok(())
}
