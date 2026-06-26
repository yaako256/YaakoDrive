pub mod create_folder;
pub mod delete_node;
pub mod get_node;
pub mod list_children;
pub mod move_node;
pub mod rename_node;

use crate::error::AppError;
use repository::RepoError;

// エラー伝搬の定義
pub fn map_name_conflict(e: RepoError) -> AppError {
  match e {
    repository::RepoError::Conflict(_) => {
      AppError::AlreadyExists("same name already exists".to_string())
    }
    other => AppError::from(other),
  }
}
