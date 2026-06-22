/*
backend/crates/api/src/router.rs
ルータを定義する
*/

// 外部クレート
// ルータ用
use axum::{
  Router,
  routing::{get, post},
};

// 自クレート
// ハンドラ達
use crate::handlers::{
  auth::{login_handler, logout_handler, refresh_handler},
  health::health_handler,
};
use crate::state::AppState;

/// サーバのRouter型を返す
pub fn create_router(state: AppState) -> Router {
  Router::new()
    .route("/api/health", get(health_handler))
    .route("/api/auth/login", post(login_handler))
    .route("/api/auth/refresh", post(refresh_handler))
    .route("/api/auth/logout", post(logout_handler))
    .with_state(state)
}
