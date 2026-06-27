/*
backend/crates/app/src/usecase/trash/hard_deleate_node.rs
物理削除するユースケース
*/

use crate::error::{AppError, AppResult};
use identity::{NodeId, UserId};
use repository::{FileContentRepository, NodeRepository};
use storage::StorageService;
use tracing::warn;

pub struct HardDeleteNodeInput {
  pub node_id: NodeId,
  pub requester_user_id: UserId,
}

pub struct HardDeleteNodeUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
  file_content_repo: &'a dyn FileContentRepository,
  storage: &'a dyn StorageService,
}

impl<'a> HardDeleteNodeUseCase<'a> {
  pub fn new(
    node_repo: &'a dyn NodeRepository,
    file_content_repo: &'a dyn FileContentRepository,
    storage: &'a dyn StorageService,
  ) -> Self {
    Self {
      node_repo,
      file_content_repo,
      storage,
    }
  }

  pub async fn execute(&self, input: HardDeleteNodeInput) -> AppResult<()> {
    let node = self
      .node_repo
      .find_by_id(&input.node_id)
      .await?
      .ok_or_else(|| AppError::NotFound("node not found".to_string()))?;

    // 権限チェック
    if !node.is_owner(&input.requester_user_id) {
      return Err(AppError::NotFound("node not found".to_string()));
    }

    // ゴミ箱内にあることを確認(物理削除はゴミ箱からのみ)
    if !node.is_deleted() {
      return Err(AppError::InvalidInput(
        "node must be in trash before hard delete".to_string(),
      ));
    }

    // 自身と配下の NodeId を収集する
    let descendant_ids = self
      .node_repo
      .collect_descendant_ids(&input.node_id)
      .await?;

    // 削除対象の実ファイル名を収集する(DB 削除前に取得する必要がある)
    let stored_filenames = self
      .file_content_repo
      .find_stored_filenames_by_node_ids(&descendant_ids)
      .await?;

    // DB から nodes を物理削除する
    // file_contents は ON DELETE CASCADE で自動削除される
    self.node_repo.hard_delete_many(&descendant_ids).await?;

    // 実ファイルを削除する
    // DB 削除成功後のファイル削除失敗は孤立ファイルとして残る。
    // CLI の check-storage-consistency で検出・修復する設計のためエラーにしない。
    for filename in &stored_filenames {
      if let Err(e) = self.storage.delete(filename).await {
        warn!(
          "物理削除後の実ファイル削除に失敗しました: {} - {}",
          filename, e
        );
      }
    }

    Ok(())
  }
}
