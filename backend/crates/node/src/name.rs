/*
backend/crates/node/src/name.rs
名前規則を定義
*/

// 自クレート
use crate::error::{NodeError, NodeResult};

/// 名前バリデーション共通関数
pub fn validate_name(name: &str) -> NodeResult<()> {
  // 空の名前は使えない
  if name.is_empty() {
    return Err(NodeError::InvalidName("name is empty".to_string()));
  }
  // 長すぎる名前は使えない
  // name.len() は UTF-8 のバイト数を返すため、Unicode 文字数で判定する
  // name.chars().count()で文字数にする。絵文字とかは複数文字判定される
  // OSの制限ではなく、CloudDrive独自の文字数制限
  if name.chars().count() > 255 {
    return Err(NodeError::InvalidName("name is too long".to_string()));
  }
  // パス区切り文字を禁止
  if name.contains('/') || name.contains('\\') {
    return Err(NodeError::InvalidName(
      "name contains invalid characters".to_string(),
    ));
  }

  Ok(())
}
