/*
backend/crates/auth/src/token.rs
Refresh Token生成
*/

// 外部クレート
// 乱数生成
use rand;
// 16進数
use hex;
// ハッシュ
use sha2::{Digest, Sha256};

/// 推測不能なRefresh Token文字列を生成する
/// 保存時はこの値をハッシュ化してtoken_hashに保存する
pub fn generate_refresh_token() -> String {
  // 32個の完全ランダムな数字を取得
  let bytes: [u8; 32] = rand::random();
  // 16進数に変換する
  hex::encode(bytes)
}

/// Refresh Tokenのハッシュ化
/// argon2は不要。SHA-256で十分
/// (推測不能な乱数を短時間ハッシュするだけ)
/// DefaultHasherは衝突耐性がないため使わない
pub fn hash_token(token: &str) -> String {
  // hasherの設定
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());

  // ハッシュ化をする
  hex::encode(hasher.finalize())
}
