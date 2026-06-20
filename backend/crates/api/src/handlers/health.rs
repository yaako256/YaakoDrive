/*
backend/crates/api/src/handlers/health.rs
ヘルスチェックレスポンスのハンドラ
*/

// 外部クレート
// Json用
use axum::Json;
// シリアライズ用
use serde::Serialize;

// 自クレート
// レスポンス共通型
use crate::response::ApiResponse;

#[derive(Serialize)]
pub struct HealthData {
  pub status: &'static str,
}

pub async fn health_handler() -> Json<ApiResponse<HealthData>> {
  Json(ApiResponse::ok(HealthData { status: "ok" }))
}
