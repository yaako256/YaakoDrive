/*
backend/crates/api/src/handlers/dashboard.rs
ダッシュボード関連のハンドラ
*/

// 外部クレート
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

// 内部ライブラリ
use app::usecase::dashboard::get_dashboard::{DashboardInput, GetDashboardUseCase};

// 自クレート
use crate::{
  error::ApiAppError, extractor::AuthenticatedUser, handlers::common::parse_user_id,
  response::ApiResponse, state::AppState,
};

#[derive(Serialize)]
pub struct MimeStatResponse {
  pub mime_type: String,
  pub count: i64,
}

#[derive(Serialize)]
pub struct DashboardResponse {
  pub used_bytes: i64,
  pub limit_bytes: i64,
  pub file_count: i64,
  pub folder_count: i64,
  pub mime_stats: Vec<MimeStatResponse>,
}

// ─── GET /api/dashboard ──────────────────────────────────

pub async fn dashboard_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;

  let usecase = GetDashboardUseCase::new(
    state.user_repo.as_ref(),
    state.node_repo.as_ref(),
    state.file_content_repo.as_ref(),
  );
  let output = usecase
    .execute(DashboardInput { user_id })
    .await
    .map_err(ApiAppError::from)?;

  Ok(Json(ApiResponse::ok(DashboardResponse {
    used_bytes: output.used_bytes,
    limit_bytes: output.limit_bytes,
    file_count: output.file_count,
    folder_count: output.folder_count,
    mime_stats: output
      .mime_stats
      .into_iter()
      .map(|m| MimeStatResponse {
        mime_type: m.mime_type,
        count: m.count,
      })
      .collect(),
  })))
}
