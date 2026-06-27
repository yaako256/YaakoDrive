/*
backend/crates/api/src/state.rs
api 層でハンドラ間の共有状態を定義
*/

// 標準ライブラリ
use std::sync::Arc;

// 内部クレート
use auth::jwt::JwtService;
use config::AppConfig;
use repository::{
  FileContentRepository, NodeRepository, RefreshTokenRepository, UnitOfWork, UserRepository,
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
  pub user_repo: Arc<dyn UserRepository>,
  pub refresh_token_repo: Arc<dyn RefreshTokenRepository>,
  pub node_repo: Arc<dyn NodeRepository>,
  pub file_content_repo: Arc<dyn FileContentRepository>,
  pub uow: Arc<dyn UnitOfWork>,
  // file
  pub storage: Arc<dyn StorageService>,
  pub download_tokens: DownloadTokenStore,
}
