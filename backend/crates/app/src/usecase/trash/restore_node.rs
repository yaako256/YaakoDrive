/*
backend/crates/app/src/usecase/trash/restore_node.rs
ゴミ箱から復元するユースケース
deleted_atをNoneにする
*/

// 外部クレート

use chrono::Utc;

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
    let mut node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or_else(|| AppError::NotFound("node not found".to_string()))?;

    // 権限チェック
    if node.owner_user_id() != &input.requester_user_id {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    // 復元後の名前を決定する
    let restore_name = match input.new_name {
      Some(ref name) => {
        validate_name(name)?;
        name.clone()
      }
      None => node.name().clone(),
    };

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
    // 名前に変更の必要があったら変更する
    if &restore_name != restored_node.name() {
      // 名前を変更しながらゴミ箱から戻す
      restored_node.restore(Some(restore_name))?;
      // 名前更新のためにupdate
      self.node_repo.update(&restored_node).await?;
    } else {
      // ゴミ箱から戻す
      restored_node.restore(None)?;
    }

    // deleted_at を NULL に戻す（配下含む）
    self
      .node_repo
      .restore_with_descendants(restored_node.id(), restored_node.updated_at())
      .await?;

    Ok(restored_node)
  }
}
