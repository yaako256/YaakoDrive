pub mod login;
pub mod logout;
pub mod refresh;

use sha2::{Digest, Sha256};

// 複数のユースケースで使うため、mod.rsに書く
/// Refresh Tokenのハッシュ化
/// argon2は不要。SHA-256で十分
/// (推測不能な乱数を短時間ハッシュするだけ)
/// DefaultHasherは衝突耐性がないため使わない
pub(crate) fn hash_token(token: &str) -> String {
  // hasherの設定
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());

  // ハッシュ化をする
  hex::encode(hasher.finalize())
}
