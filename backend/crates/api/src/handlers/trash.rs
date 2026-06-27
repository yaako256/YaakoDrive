/*
backend/crates/api/src/handlers/trash.rs
ゴミ箱関連のハンドラ
*/
// 外部ライブラリ
use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

// 内部ライブラリ
use app::usecase::trash::{
  hard_delete_node::{HardDeleteNodeInput, HardDeleteNodeUseCase},
  list_trash::{ListTrashChildrenInput, ListTrashInput, ListTrashUseCase},
  restore_node::{RestoreNodeInput, RestoreNodeUseCase},
};
use identity::NodeId;

// 自クレート
use crate::{
  error::ApiAppError,
  extractor::AuthenticatedUser,
  handlers::common::{NodeResponse, parse_user_id},
  response::ApiResponse,
  state::AppState,
};

// ─── GET /api/trash ───────────────────────────────────────

pub async fn list_trash_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;

  let usecase = ListTrashUseCase::new(state.node_repo.as_ref());
  let nodes = usecase
    .execute(ListTrashInput {
      owner_user_id: user_id,
    })
    .await
    .map_err(ApiAppError::from)?;

  let response: Vec<NodeResponse> = nodes.into_iter().map(NodeResponse::from).collect();
  Ok(Json(ApiResponse::ok(response)))
}

// ─── GET /api/trash/{id}/children ────────────────────────

pub async fn list_trash_children_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;
  let node_id = NodeId::from_uuid(id);

  // 子を取得
  let usecase = ListTrashUseCase::new(state.node_repo.as_ref());
  let nodes = usecase
    .execute_children(ListTrashChildrenInput {
      owner_user_id: user_id,
      parent_id: node_id,
    })
    .await
    .map_err(ApiAppError::from)?;

  let response: Vec<NodeResponse> = nodes.into_iter().map(NodeResponse::from).collect();

  Ok(Json(ApiResponse::ok(response)))
}

// ─── POST /api/trash/{id}/restore ────────────────────────

#[derive(Deserialize)]
pub struct RestoreRequest {
  /// 同名衝突時に指定する別名
  pub new_name: Option<String>,
}

pub async fn restore_node_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
  Json(req): Json<RestoreRequest>,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;

  let usecase = RestoreNodeUseCase::new(state.node_repo.as_ref());
  let node = usecase
    .execute(RestoreNodeInput {
      node_id: NodeId::from_uuid(id),
      requester_user_id: user_id,
      new_name: req.new_name,
    })
    .await
    .map_err(ApiAppError::from)?;

  Ok(Json(ApiResponse::ok(NodeResponse::from(node))))
}

// ─── DELETE /api/trash/{id} ──────────────────────────────

pub async fn hard_delete_node_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;

  let usecase = HardDeleteNodeUseCase::new(
    state.node_repo.as_ref(),
    state.file_content_repo.as_ref(),
    state.storage.as_ref(),
  );
  usecase
    .execute(HardDeleteNodeInput {
      node_id: NodeId::from_uuid(id),
      requester_user_id: user_id,
    })
    .await
    .map_err(ApiAppError::from)?;

  Ok(Json(ApiResponse::ok(())))
}
