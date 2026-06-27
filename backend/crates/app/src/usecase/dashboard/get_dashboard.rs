/*
backend/crates/app/src/usecase/dashboard/get_dashbord.rs
ダッシュボードを取得するユースケース
*/

// 内部ライブラリ
use identity::UserId;
use repository::{FileContentRepository, NodeRepository, UserRepository};

// 自クレート
use crate::error::{AppError, AppResult};

pub struct DashboardInput {
  pub user_id: UserId,
}

pub struct MimeStatOutput {
  pub mime_type: String,
  pub count: i64,
}

pub struct DashboardOutput {
  /// active ファイルの使用容量合計（ゴミ箱内を除く）
  pub used_bytes: i64,
  /// ユーザの容量上限
  pub limit_bytes: i64,
  /// active ファイル数（ゴミ箱内を除く）
  pub file_count: i64,
  /// active フォルダ数（ゴミ箱内を除く）
  pub folder_count: i64,
  /// MIME Type ごとのファイル数
  pub mime_stats: Vec<MimeStatOutput>,
}

pub struct GetDashboardUseCase<'a> {
  user_repo: &'a dyn UserRepository,
  node_repo: &'a dyn NodeRepository,
  file_content_repo: &'a dyn FileContentRepository,
}

impl<'a> GetDashboardUseCase<'a> {
  pub fn new(
    user_repo: &'a dyn UserRepository,
    node_repo: &'a dyn NodeRepository,
    file_content_repo: &'a dyn FileContentRepository,
  ) -> Self {
    Self {
      user_repo,
      node_repo,
      file_content_repo,
    }
  }

  pub async fn execute(&self, input: DashboardInput) -> AppResult<DashboardOutput> {
    // ユーザ情報（容量上限）を取得する
    let user = self
      .user_repo
      .find_by_id(&input.user_id)
      .await?
      .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    // ファイル使用統計を取得する
    let usage = self
      .file_content_repo
      .get_usage_stats(&input.user_id)
      .await?;

    // フォルダ数を取得する
    let folder_count = self.node_repo.count_active_folders(&input.user_id).await?;

    Ok(DashboardOutput {
      used_bytes: usage.total_bytes,
      limit_bytes: *user.storage_limit_bytes(),
      file_count: usage.file_count,
      folder_count,
      mime_stats: usage
        .mime_stats
        .into_iter()
        .map(|m| MimeStatOutput {
          mime_type: m.mime_type,
          count: m.count,
        })
        .collect(),
    })
  }
}
