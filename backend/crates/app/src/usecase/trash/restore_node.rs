/*
backend/crates/app/src/usecase/trash/restore_node.rs
ゴミ箱から復元するユースケース
deleted_atをNoneにする
*/

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::Node;
use node::name::validate_name;
use repository::NodeRepository;

// 自クレート
use crate::error::{AppError, AppResult};

pub struct RestoreNodeInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
  /// 同名衝突時に指定する別名。None の場合は元の名前で復元を試みる。
  pub new_name: Option<String>,
}

pub struct RestoreNodeUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> RestoreNodeUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  pub async fn execute(&self, input: RestoreNodeInput) -> AppResult<Node> {
    let node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or_else(|| AppError::NotFound("node not found".to_string()))?;

    // 権限チェック
    if node.owner_user_id() != &input.requester_user_id {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    // 復元後の名前を決定する
    let restore_name = input.new_name.as_deref().unwrap_or(node.name());

    // 名前チェック
    validate_name(restore_name)?;

    // 復元先に同名の active ノードが存在するか確認
    let conflict = self
      .node_repo
      .exists_active_with_name(&input.requester_user_id, node.parent_id(), &restore_name)
      .await?;

    if conflict {
      return Err(AppError::AlreadyExists(format!(
        "'{}' already exists in the destination",
        restore_name
      )));
    }

    // Node型をゴミ箱から戻す
    let mut restored_node = node;
    // 名前を変更する場合はする
    if let Some(name) = input.new_name {
      restored_node.rename(name)?;
      // 名前更新のためにupdate
      self.node_repo.update(&restored_node).await?;
    }
    // Node型をゴミ箱から戻す
    restored_node.restore()?;

    // deleted_at を NULL に戻す（配下含む）
    self
      .node_repo
      .restore_with_descendants(restored_node.id(), restored_node.updated_at())
      .await?;

    Ok(restored_node)
  }
}
