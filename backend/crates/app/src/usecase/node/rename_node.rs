/*
backend/crates/app/src/usecase/node/rename_node.rs
フォルダやファイルのリネームをするユースケース
*/

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::Node;
use repository::NodeRepository;

// 自クレート
use crate::{
  error::{AppError, AppResult},
  usecase::node::map_name_conflict,
};

pub struct RenameNodeInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
  pub new_name: String,
}

// Nodeをリネームするユースケース構造体
pub struct RenameNodeUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> RenameNodeUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  /// リネームを実行
  pub async fn execute(&self, input: RenameNodeInput) -> AppResult<Node> {
    // NodeIdからNode型を取得
    let mut node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or(AppError::NotFound("node not found".to_string()))?;

    // 他ユーザのNodeはリネームできない
    if node.is_owner(&input.requester_user_id) {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    // 名前を更新
    node.rename(input.new_name)?;

    // Nodeの更新(リネームの実行)
    self
      .node_repo
      .update(&node)
      .await
      .map_err(map_name_conflict)?;

    // 更新後のNodeを返す
    Ok(node)
  }
}
