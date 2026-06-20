/*
backend/crates/auth/src/lib.rs
*/
mod error;
pub mod jwt;
pub mod model;
pub mod password;
pub mod token;

// error型だけ再エクスポート
pub use error::AuthError;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_verify_password() {
    let hash = password::hash_password("secret123").unwrap();
    assert!(password::verify_password("secret123", &hash).is_ok());
    assert!(password::verify_password("wrong", &hash).is_err());
  }

  #[test]
  fn test_refresh_token_validity() {
    // is_valid / is_disabled などのロジックをここでテストできる
  }
}
