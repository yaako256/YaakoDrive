/*
backend/crates/api/src/router.rs
ルータを定義する
*/

// 外部クレート
// ルータ用
use axum::{
  Router,
  routing::{delete, get, patch, post},
};

// 自クレート
// ハンドラ達
use crate::handlers::{
  auth::{login_handler, logout_handler, refresh_handler},
  health::health_handler,
  node::{
    create_folder_handler, create_root_folder_handler, delete_node_handler, get_node_handler,
    list_children_handler, list_root_handler, move_node_handler, rename_node_handler,
  },
};
// AppState
use crate::state::AppState;

/// サーバのRouter型を返す
pub fn create_router(state: AppState) -> Router {
  Router::new()
    // health
    .route("/api/health", get(health_handler))
    // auth
    .route("/api/auth/login", post(login_handler))
    .route("/api/auth/refresh", post(refresh_handler))
    .route("/api/auth/logout", post(logout_handler))
    // nodes
    .route("/api/nodes", get(list_root_handler))
    .route("/api/nodes/folders", post(create_root_folder_handler))
    .route("/api/nodes/{id}", get(get_node_handler))
    .route("/api/nodes/{id}/children", get(list_children_handler))
    .route("/api/nodes/{id}/folders", post(create_folder_handler))
    .route("/api/nodes/{id}/rename", patch(rename_node_handler))
    .route("/api/nodes/{id}/move", patch(move_node_handler))
    .route("/api/nodes/{id}", delete(delete_node_handler))
    .with_state(state)
}

/*
※注意
axumはルートを登録順に評価するため、
`/api/nodes/folders`
を
`/api/nodes/{id}`
より先に登録する必要がある
*/
