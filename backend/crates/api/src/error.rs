/*
backend/crates/api/src/error.rs
apiクレートのエラー型の定義
AppError を HTTP レスポンスに変換する
*/

// 外部クレート
use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};

// 内部ライブラリ
// ユースケースのエラー型用
use app::AppError;

// 自クレート
use crate::response::ApiResponse;

pub struct ApiAppError(pub AppError);

impl From<AppError> for ApiAppError {
  fn from(e: AppError) -> Self {
    Self(e)
  }
}

impl IntoResponse for ApiAppError {
  fn into_response(self) -> Response {
    let (status, code, message) = match self.0 {
      AppError::Unauthorized => (
        StatusCode::UNAUTHORIZED,
        "unauthorized",
        "認証が必要です".to_string(),
      ),
      AppError::Forbidden => (
        StatusCode::FORBIDDEN,
        "forbidden",
        "権限がありません".to_string(),
      ),
      AppError::StorageLimitExceeded => (
        StatusCode::CONFLICT,
        "storage_limit_exceeded",
        "ストレージ容量の上限に達しています".to_string(),
      ),
      AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
      AppError::AlreadyExists(msg) => (StatusCode::CONFLICT, "already_exists", msg),
      AppError::InvalidInput(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_request", msg),
      AppError::Repository(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
      AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
    };

    let body = Json(ApiResponse::<()>::err(code, &message));
    (status, body).into_response()
  }
}
