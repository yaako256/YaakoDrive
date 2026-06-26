/*
backend/crates/api/src/handlers/node.rs
Node関連のハンドラ
*/

// ─── ハントラの定義 ──────────────────────────────────────
// 外部ライブラリ
use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

// 内部ライブラリ
use app::usecase::node::{
  create_folder::{CreateFolderInput, CreateFolderUseCase},
  delete_node::{DeleteNodeInput, DeleteNodeUseCase},
  get_node::{GetNodeInput, GetNodeUseCase},
  list_children::{ListChildrenInput, ListChildrenUseCase},
  move_node::{MoveNodeInput, MoveNodeUseCase},
  rename_node::{RenameNodeInput, RenameNodeUseCase},
};
use identity::NodeId;

// 自クレート
use crate::handlers::common::{NodeResponse, parse_user_id};
use crate::{
  error::ApiAppError, extractor::AuthenticatedUser, response::ApiResponse, state::AppState,
};

// ─── GET /api/nodes ──────────────────────────────────────
// ルートのNode一覧を取得するハンドラ
pub async fn list_root_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // ルートの子一覧を取得
  let usecase = ListChildrenUseCase::new(state.node_repo.as_ref());
  let nodes = usecase
    .execute(ListChildrenInput {
      owner_user_id: user_id,
      parent_id: None,
    })
    .await
    .map_err(ApiAppError::from)?;

  // レスポンス形式に合わせる
  let response: Vec<NodeResponse> = nodes.into_iter().map(NodeResponse::from).collect();

  // Jsonに変換してレスポンス
  Ok(Json(ApiResponse::ok(response)))
}

// ─── GET /api/nodes/{id}/children ────────────────────────
// フォルダの子Node一覧を取得するハンドラ
pub async fn list_children_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // フォルダの子一覧を取得
  let usecase = ListChildrenUseCase::new(state.node_repo.as_ref());
  let nodes = usecase
    .execute(ListChildrenInput {
      owner_user_id: user_id,
      parent_id: Some(NodeId::from_uuid(id)),
    })
    .await
    .map_err(ApiAppError::from)?;

  // レスポンス形式に合わせる
  let response: Vec<NodeResponse> = nodes.into_iter().map(NodeResponse::from).collect();

  // Jsonに変換してレスポンス
  Ok(Json(ApiResponse::ok(response)))
}

// ─── GET /api/nodes/{id} ─────────────────────────────────
// Nodeを取得するハンドラ
pub async fn get_node_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // Nodeを取得
  let usecase = GetNodeUseCase::new(state.node_repo.as_ref());
  let node = usecase
    .execute(GetNodeInput {
      node_id: NodeId::from_uuid(id),
      requester_user_id: user_id,
    })
    .await
    .map_err(ApiAppError::from)?;

  // レスポンス形式に合わせる
  // Jsonに変換してレスポンス
  Ok(Json(ApiResponse::ok(NodeResponse::from(node))))
}

// ─── POST /api/nodes/{id}/folders ────────────────────────
#[derive(Deserialize)]
pub struct CreateFolderRequest {
  pub name: String,
}

// フォルダを作成するハンドラ
pub async fn create_folder_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
  Json(req): Json<CreateFolderRequest>,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // 新規フォルダの作成
  let usecase = CreateFolderUseCase::new(state.node_repo.as_ref());
  let node = usecase
    .execute(CreateFolderInput {
      owner_user_id: user_id,
      parent_id: Some(NodeId::from_uuid(id)),
      name: req.name,
    })
    .await
    .map_err(ApiAppError::from)?;

  // レスポンス形式に合わせる
  // Jsonに変換してレスポンス
  Ok(Json(ApiResponse::ok(NodeResponse::from(node))))
}

// ─── POST /api/nodes/root/folders ────────────────────────
// フォルダを作成するハンドラ
// ルート直下にフォルダ作成する場合
pub async fn create_root_folder_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Json(req): Json<CreateFolderRequest>,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // 新規フォルダの作成(ルート)
  let usecase = CreateFolderUseCase::new(state.node_repo.as_ref());
  let node = usecase
    .execute(CreateFolderInput {
      owner_user_id: user_id,
      parent_id: None,
      name: req.name,
    })
    .await
    .map_err(ApiAppError::from)?;

  // レスポンス形式に合わせる
  // Jsonに変換してレスポンス
  Ok(Json(ApiResponse::ok(NodeResponse::from(node))))
}

// ─── PATCH /api/nodes/{id}/rename ────────────────────────
#[derive(Deserialize)]
pub struct RenameRequest {
  pub name: String,
}

// Nodeをリネームするハンドラ
pub async fn rename_node_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
  Json(req): Json<RenameRequest>,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // リネームをする
  let usecase = RenameNodeUseCase::new(state.node_repo.as_ref());
  let node = usecase
    .execute(RenameNodeInput {
      node_id: NodeId::from_uuid(id),
      requester_user_id: user_id,
      new_name: req.name,
    })
    .await
    .map_err(ApiAppError::from)?;

  // レスポンス形式に合わせる
  // Jsonに変換してレスポンス
  Ok(Json(ApiResponse::ok(NodeResponse::from(node))))
}

// ─── PATCH /api/nodes/{id}/move ──────────────────────────
#[derive(Deserialize)]
pub struct MoveRequest {
  /// None を渡すとルート直下へ移動
  pub new_parent_id: Option<Uuid>,
}

// Nodeを移動するハンドラ
pub async fn move_node_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
  Json(req): Json<MoveRequest>,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // Nodeの移動をする
  let usecase = MoveNodeUseCase::new(state.node_repo.as_ref());
  let node = usecase
    .execute(MoveNodeInput {
      node_id: NodeId::from_uuid(id),
      requester_user_id: user_id,
      new_parent_id: req.new_parent_id.map(NodeId::from_uuid),
    })
    .await
    .map_err(ApiAppError::from)?;

  // レスポンス形式に合わせる
  // Jsonに変換してレスポンス
  Ok(Json(ApiResponse::ok(NodeResponse::from(node))))
}

// ─── DELETE /api/nodes/{id} ──────────────────────────────
// Nodeを論理削除する
pub async fn delete_node_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiAppError> {
  // UserId型の取得
  let user_id = parse_user_id(&claims.sub)?;

  // 論理削除をする
  let usecase = DeleteNodeUseCase::new(state.node_repo.as_ref());
  usecase
    .execute(DeleteNodeInput {
      node_id: NodeId::from_uuid(id),
      requester_user_id: user_id,
    })
    .await
    .map_err(ApiAppError::from)?;

  // 空のOkレスポンスを返す
  Ok(Json(ApiResponse::ok(())))
}
