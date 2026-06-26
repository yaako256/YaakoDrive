/*
backend/crates/app/src/usecase/node/get_node.rs
Nodeを取得するユースケース
ユーザが実行するというより、操作中に実行される。
Node情報などを表示したいときなどに使う。
*/

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::Node;
use repository::NodeRepository;

// 自クレート
use crate::error::{AppError, AppResult};

pub struct GetNodeInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
}

// Nodeを取得するユースケース構造体
pub struct GetNodeUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> GetNodeUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  // Node取得を実行
  pub async fn execute(&self, input: GetNodeInput) -> AppResult<Node> {
    // NodeIdからNode型を取得
    let node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or(AppError::NotFound("node not found".to_string()))?;

    // 他ユーザのノードは見えない
    if node.owner_user_id() != &input.requester_user_id {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    Ok(node)
  }
}
