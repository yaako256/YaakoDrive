/*
backend/crates/app/src/usecase/node/move_node.rs
フォルダやファイルの位置を移動するユースケース
移動時の循環チェックが重要。絶対に子孫には移動させない。
*/

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::Node;
use repository::NodeRepository;

// 自クレート
use crate::error::{AppError, AppResult};

pub struct MoveNodeInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
  /// None はルート直下へ移動
  pub new_parent_id: Option<NodeId>,
}

/// Node移動のユースケース構造体
pub struct MoveNodeUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> MoveNodeUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  /// Node移動を実行
  pub async fn execute(&self, input: MoveNodeInput) -> AppResult<Node> {
    // NodeIdからNode型を取得
    let mut node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or(AppError::NotFound("node not found".to_string()))?;

    // 他ユーザのNodeは移動できない
    if node.owner_user_id() != &input.requester_user_id {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    // 循環チェック用: 移動先が自分の子孫でないか確認
    // 自身の子孫には移動できない
    let mut ancestor_ids: Vec<NodeId> = Vec::new();

    // 移動先の確認
    if let Some(ref new_parent_id) = input.new_parent_id {
      // 自分自身への移動は禁止
      if new_parent_id == &input.node_id {
        return Err(AppError::InvalidInput(
          "cannot move node into itself".to_string(),
        ));
      }

      // NodeIdから親フォルダのNode型を取得
      let new_parent =
        self
          .node_repo
          .find_by_id(new_parent_id)
          .await?
          .ok_or(AppError::NotFound(
            "destination folder not found".to_string(),
          ))?;

      // 他ユーザのフォルダには移動できない
      if new_parent.owner_user_id() != &input.requester_user_id {
        return Err(AppError::NotFound(
          "destination folder not found".to_string(),
        ));
      }

      // 削除済みのフォルダには移動できない
      if new_parent.is_deleted() {
        return Err(AppError::NotFound(
          "destination folder not found".to_string(),
        ));
      }

      // フォルダではないものには移動できない
      if !new_parent.is_folder() {
        return Err(AppError::InvalidInput(
          "destination is not a folder".to_string(),
        ));
      }

      // 循環チェック: 移動先が自分の子孫でないか確認
      // 自身の子孫には移動できない
      ancestor_ids = self.node_repo.find_ancestor_ids(new_parent_id).await?;
    }

    // 親NodeIdの置き換え
    node.move_node(input.new_parent_id, &ancestor_ids)?;

    // Nodeの更新(移動の実行)
    self.node_repo.update(&node).await.map_err(|e| match e {
      repository::RepoError::Conflict(_) => {
        AppError::AlreadyExists("same name already exists in destination".to_string())
      }
      other => AppError::from(other),
    })?;

    // 更新後のNodeを返す
    Ok(node)
  }
}
