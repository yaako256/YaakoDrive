/*
backend/crates/api/src/router.rs
ルータを定義する
*/

// 外部クレート
// ルータ用
use axum::{Router, routing::get};

// 自クレート
// ハンドラ達
use crate::handlers::health::health_handler;

/// サーバのRouter型を返す
pub fn create_router() -> Router {
  Router::new().route("/api/health", get(health_handler))
}
