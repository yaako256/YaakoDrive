/*
backend/crates/app/src/usecase/node/delete_node.rs
フォルダやファイルを削除(論理削除)をするユースケース
*/

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use identity::{NodeId, UserId};
use repository::NodeRepository;

// 自クレート
use crate::error::{AppError, AppResult};

pub struct DeleteNodeInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
}

/// Node削除のユースケース構造体
pub struct DeleteNodeUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> DeleteNodeUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  /// Node削除を実行
  pub async fn execute(&self, input: DeleteNodeInput) -> AppResult<()> {
    // NodeIdからNode型を取得
    let node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or(AppError::NotFound("node not found".to_string()))?;

    // 他ユーザのNodeは削除できない
    if node.owner_user_id != input.requester_user_id {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    // 既に削除済みのものは削除できない
    if node.is_deleted() {
      return Err(AppError::InvalidInput(
        "node is already deleted".to_string(),
      ));
    }

    // 現在時刻を取得
    let deleted_at = Utc::now();

    // 論理削除(deleted_atの設定)をする
    // フォルダの場合は配下ノードすべてにdeleted_atを設定する
    self
      .node_repo
      .soft_delete_with_descendants(&input.node_id, deleted_at)
      .await?;

    Ok(())
  }
}
