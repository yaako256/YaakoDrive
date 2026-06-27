// api/src/handlers/file.rs
use axum::{
  Json,
  body::Body,
  extract::{Multipart, Path, State},
  http::header,
  response::IntoResponse,
};
use futures_util::StreamExt;
use serde::Serialize;
use uuid::Uuid;

// 内部ライブラリ
use identity::NodeId;

use app::usecase::file::{
  download::{DownloadFileInput, DownloadFileUseCase},
  download_url::{GetDownloadUrlInput, GetDownloadUrlUseCase},
  upload::{UploadFileInput, UploadFileUseCase},
};

use crate::{
  error::ApiAppError,
  extractor::AuthenticatedUser,
  handlers::common::{NodeResponse, parse_user_id},
  response::ApiResponse,
  state::AppState,
};

// ─── multipart 共通処理 ───────────────────────────────────

/// multipart から最初のファイルフィールドを取り出す共通ヘルパー
async fn extract_upload_field(
  multipart: &mut Multipart,
) -> Result<(String, bytes::Bytes), ApiAppError> {
  let field = multipart
    .next_field()
    .await
    .map_err(|e| ApiAppError::from(app::AppError::InvalidInput(e.to_string())))?
    .ok_or_else(|| ApiAppError::from(app::AppError::InvalidInput("no file field".to_string())))?;

  let filename = field.file_name().unwrap_or("unknown").to_string();
  let data = field
    .bytes()
    .await
    .map_err(|e| ApiAppError::from(app::AppError::InvalidInput(e.to_string())))?;

  Ok((filename, data))
}

// ─── POST /api/nodes/{id}/upload ─────────────────────────

pub async fn upload_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(parent_id): Path<Uuid>,
  mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;
  let (filename, data) = extract_upload_field(&mut multipart).await?;

  let usecase = UploadFileUseCase::new(
    state.node_repo.as_ref(),
    state.uow.as_ref(),
    state.storage.as_ref(),
    state.config.upload.max_size_bytes,
  );

  let output = usecase
    .execute(UploadFileInput {
      owner_user_id: user_id,
      parent_id: Some(NodeId::from_uuid(parent_id)),
      filename,
      data,
    })
    .await
    .map_err(ApiAppError::from)?;

  Ok(Json(ApiResponse::ok(NodeResponse::from(output.node))))
}

// ─── POST /api/nodes/upload (ルート直下) ─────────────────

pub async fn upload_root_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;
  let (filename, data) = extract_upload_field(&mut multipart).await?;

  let usecase = UploadFileUseCase::new(
    state.node_repo.as_ref(),
    state.uow.as_ref(),
    state.storage.as_ref(),
    state.config.upload.max_size_bytes,
  );

  let output = usecase
    .execute(UploadFileInput {
      owner_user_id: user_id,
      parent_id: None,
      filename,
      data,
    })
    .await
    .map_err(ApiAppError::from)?;

  Ok(Json(ApiResponse::ok(NodeResponse::from(output.node))))
}

// ─── GET /api/nodes/{id}/download-url ────────────────────

#[derive(Serialize)]
struct DownloadUrlResponse {
  url: String,
}

pub async fn download_url_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;
  let node_id = NodeId::from_uuid(id);

  // 権限とファイル存在を確認してからトークンを発行
  let usecase =
    GetDownloadUrlUseCase::new(state.node_repo.as_ref(), state.file_content_repo.as_ref());
  usecase
    .execute(GetDownloadUrlInput {
      node_id,
      requester_user_id: user_id,
    })
    .await
    .map_err(ApiAppError::from)?;

  let token = state.download_tokens.issue(node_id);
  let url = format!("/api/files/download/{}", token);

  Ok(Json(ApiResponse::ok(DownloadUrlResponse { url })))
}

// ─── GET /api/files/download/{token} ─────────────────────
pub async fn download_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(token): Path<String>,
) -> Result<impl IntoResponse, ApiAppError> {
  let user_id = parse_user_id(&claims.sub)?;
  let node_id = state.download_tokens.consume(&token).ok_or_else(|| {
    ApiAppError::from(app::AppError::NotFound(
      "invalid or expired token".to_string(),
    ))
  })?;

  // 権限とファイル存在を確認してからトークンを発行
  let usecase =
    DownloadFileUseCase::new(state.node_repo.as_ref(), state.file_content_repo.as_ref());
  let output = usecase
    .execute(DownloadFileInput {
      node_id,
      requester_user_id: user_id,
    })
    .await
    .map_err(ApiAppError::from)?;

  // storageの方はaxumを使っていてWeb感があるため
  // UseCaseではなくこっちで定義
  let stream = state
    .storage
    .open_stream(&output.stored_filename)
    .await
    .map_err(|e| ApiAppError::from(app::AppError::from(e)))?;

  let body = Body::from_stream(
    stream.map(|r| r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
  );

  let filename_encoded = urlencoding::encode(&output.original_name).to_string();

  Ok((
    [
      (header::CONTENT_TYPE, output.mime_type),
      (
        header::CONTENT_DISPOSITION,
        format!("attachment; filename*=UTF-8''{}", filename_encoded),
      ),
    ],
    body,
  ))
}
