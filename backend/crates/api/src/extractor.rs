/*
backend/crates/api/src/handlers/extractor.rs
JWT認証ミドルウェア
認証が必要なエンドポイントで使うExtractorを定義
*/

// 外部クレート
use auth::jwt::AccessTokenClaims;
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

// 自クレート
use crate::{cookie::ACCESS_TOKEN_COOKIE, error::ApiAppError, state::AppState};

/// 認証済みユーザ情報。ハンドラの引数に加えるだけで認証チェックできる
pub struct AuthenticatedUser(pub AccessTokenClaims);

impl FromRequestParts<AppState> for AuthenticatedUser {
  type Rejection = ApiAppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let jar = CookieJar::from_request_parts(parts, state)
      .await
      .map_err(|_| ApiAppError::from(app::AppError::Unauthorized))?;

    let token = jar
      .get(ACCESS_TOKEN_COOKIE)
      .map(|c| c.value().to_string())
      .ok_or_else(|| ApiAppError::from(app::AppError::Unauthorized))?;

    let claims = state
      .jwt_service
      .verify_access_token(&token)
      .map_err(|_| ApiAppError::from(app::AppError::Unauthorized))?;

    Ok(AuthenticatedUser(claims))
  }
}
