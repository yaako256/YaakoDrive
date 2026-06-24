/*
backend/crates/app/src/usecase/node/list_children.rs
あるフォルダの子一覧を取得するユースケース
ユーザが実行するというより、操作中に実行される。
フォルダ表示などで使われる。
*/

// 内部クレート
use identity::{NodeId, UserId};
use node::model::Node;
use repository::NodeRepository;

// 自クレート
use crate::error::AppResult;

pub struct ListChildrenInput {
  pub owner_user_id: UserId,
  pub parent_id: Option<NodeId>,
}

// 子一覧を取得するユースケース構造体
pub struct ListChildrenUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> ListChildrenUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  pub async fn execute(&self, input: ListChildrenInput) -> AppResult<Vec<Node>> {
    // 子のNode型をVecで取得
    let nodes = self
      .node_repo
      .list_children(&input.owner_user_id, input.parent_id.as_ref())
      .await?;

    // 子一覧を返す
    Ok(nodes)
  }
}
