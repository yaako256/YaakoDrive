/*
backend/crates/app/src/usecase/trash/list_trash.rs
ゴミ箱の中身一覧を表示するユースケース
*/

// 内部ライブラリ
use identity::UserId;
use node::model::Node;
use repository::NodeRepository;

// 自クレート
use crate::error::AppResult;

pub struct ListTrashInput {
  pub owner_user_id: UserId,
}

pub struct ListTrashUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
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
}
