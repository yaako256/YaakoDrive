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
  refresh_token_repository::PgRefreshTokenRepository, user_repository::PgUserRepository,
};

/// axumのState。Arcで包んでclone可能にする
#[derive(Clone)]
pub struct AppState {
  pub config: Arc<AppConfig>,
  pub jwt_service: Arc<JwtService>,
  pub user_repo: Arc<PgUserRepository>,
  pub refresh_token_repo: Arc<PgRefreshTokenRepository>,
}
