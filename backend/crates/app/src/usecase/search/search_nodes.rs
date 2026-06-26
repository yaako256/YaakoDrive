/*
backend/crates/app/src/usecase/search_node/search_nodes.rs
検索機能のユースケース
*/

// 内部ライブラリ
use identity::UserId;
use node::model::Node;
use repository::NodeRepository;

// 自クレート
use crate::error::{AppError, AppResult};

pub struct SearchNodesInput {
  pub owner_user_id: UserId,
  pub query: String,
}

pub struct SearchNodesUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> SearchNodesUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  pub async fn execute(&self, input: SearchNodesInput) -> AppResult<Vec<Node>> {
    if input.query.is_empty() {
      return Err(AppError::InvalidInput("query is empty".to_string()));
    }
    if input.query.len() > 200 {
      return Err(AppError::InvalidInput("query is too long".to_string()));
    }

    let nodes = self
      .node_repo
      .search_by_name(&input.owner_user_id, &input.query)
      .await?;

    Ok(nodes)
  }
}
