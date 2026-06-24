pub mod create_folder;
pub mod delete_node;
pub mod get_node;
pub mod list_children;
pub mod move_node;
pub mod rename_node;

use crate::error::{AppError, AppResult};

// 複数のユースケースで使うため、mod.rsに書く
/// 名前バリデーション共通関数
pub fn validate_name(name: &str) -> AppResult<()> {
  // 空の名前は使えない
  if name.is_empty() {
    return Err(AppError::InvalidInput("name is empty".to_string()));
  }
  // 長すぎる名前は使えない
  if name.len() > 255 {
    return Err(AppError::InvalidInput("name is too long".to_string()));
  }
  // パス区切り文字を禁止
  if name.contains('/') || name.contains('\\') {
    return Err(AppError::InvalidInput(
      "name contains invalid characters".to_string(),
    ));
  }
  Ok(())
}
