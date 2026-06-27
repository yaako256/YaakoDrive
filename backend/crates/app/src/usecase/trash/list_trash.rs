/*
backend/crates/app/src/usecase/trash/list_trash.rs
ゴミ箱の中身一覧を表示するユースケース
*/

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::Node;
use repository::NodeRepository;

// 自クレート
use crate::error::{AppError, AppResult};

pub struct ListTrashInput {
  pub owner_user_id: UserId,
}

pub struct ListTrashUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

pub struct ListTrashChildrenInput {
  pub owner_user_id: UserId,
  pub parent_id: NodeId,
}

impl<'a> ListTrashUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  pub async fn execute(&self, input: ListTrashInput) -> AppResult<Vec<Node>> {
    let nodes = self
      .node_repo
      .list_trash_roots(&input.owner_user_id)
      .await?;
    Ok(nodes)
  }

  pub async fn execute_children(&self, input: ListTrashChildrenInput) -> AppResult<Vec<Node>> {
    let parent = self
      .node_repo
      .find_by_id(&input.parent_id)
      .await?
      .ok_or_else(|| AppError::NotFound("node not found".to_string()))?;

    // 権限チェック
    if !parent.is_owner(&input.owner_user_id) {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    // ゴミ箱内のフォルダであることを確認
    if !parent.is_deleted() {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    if !parent.is_folder() {
      return Err(AppError::InvalidInput("node is not a folder".to_string()));
    }

    let nodes = self
      .node_repo
      .list_deleted_children(&input.owner_user_id, &input.parent_id)
      .await?;

    Ok(nodes)
  }
}
