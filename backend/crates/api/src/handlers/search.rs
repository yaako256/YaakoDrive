/*
backend/crates/api/src/handlers/search.rs
検索機能のハンドラ
*/

// 外部クレート
use axum::{
  Json,
  extract::{Query, State},
  response::IntoResponse,
};
use serde::Deserialize;

// 内部ライブラリ
use app::usecase::search::search_nodes::{SearchNodesInput, SearchNodesUseCase};

// 自クレート
use crate::{
  error::ApiAppError,
  extractor::AuthenticatedUser,
  handlers::common::{NodeResponse, parse_user_id},
  response::ApiResponse,
  state::AppState,
};

#[derive(Deserialize)]
pub struct SearchQuery {
  pub q: String,
}

// ─── GET /api/search?q={keyword} ─────────────────────────

pub async fn search_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;

  let usecase = SearchNodesUseCase::new(state.node_repo.as_ref());
  let nodes = usecase
    .execute(SearchNodesInput {
      owner_user_id: user_id,
      query: params.q,
    })
    .await
    .map_err(ApiAppError::from)?;

  let response: Vec<NodeResponse> = nodes.into_iter().map(NodeResponse::from).collect();
  Ok(Json(ApiResponse::ok(response)))
}
