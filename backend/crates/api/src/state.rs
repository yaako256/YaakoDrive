/*
backend/crates/api/src/state.rs
api 層でハンドラ間の共有状態を定義
*/

// 標準ライブラリ
use std::sync::Arc;

// 内部クレート
use auth::jwt::JwtService;
use config::AppConfig;
use infra::postgres::{
  file_content_repository::PgFileContentRepository, node_repository::PgNodeRepository,
  refresh_token_repository::PgRefreshTokenRepository, unit_of_work::PgUnitOfWork,
  user_repository::PgUserRepository,
};
use storage::StorageService;

// 自クレート
use crate::download_token::DownloadTokenStore;

/// axumのState。Arcで包んでclone可能にする
#[derive(Clone)]
pub struct AppState {
  // 設定
  pub config: Arc<AppConfig>,
  // 認証
  pub jwt_service: Arc<JwtService>,
  // DB
  pub user_repo: Arc<PgUserRepository>,
  pub refresh_token_repo: Arc<PgRefreshTokenRepository>,
  pub node_repo: Arc<PgNodeRepository>,
  pub file_content_repo: Arc<PgFileContentRepository>,
  pub uow: Arc<PgUnitOfWork>,
  // file
  pub storage: Arc<dyn StorageService>,
  pub download_tokens: DownloadTokenStore,
}
