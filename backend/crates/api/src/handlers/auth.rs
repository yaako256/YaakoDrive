/*
backend/crates/api/src/handlers/auth.rs
認証ハンドラ
*/

// 外部クレート
use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use time;
use tracing::info;

// 内部ライブラリ
use app::usecase::auth::{
  login::{LoginInput, LoginUseCase},
  logout::{LogoutInput, LogoutUseCase},
  refresh::{RefreshInput, RefreshUseCase},
};

// 自クレート
use crate::{
  cookie::{ACCESS_TOKEN_COOKIE, REFRESH_TOKEN_COOKIE},
  error::ApiAppError,
  response::ApiResponse,
  state::AppState,
};

// ─── Login ───────────────────────────────────────────────
#[derive(Deserialize)]
pub struct LoginRequest {
  pub username: String,
  pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
  pub username: String,
}

pub async fn login_handler(
  State(state): State<AppState>,
  jar: CookieJar,
  Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiAppError> {
  // Phase 7ではUser-Agentは省略
  // 後でログインの一部に含める
  let user_agent = None;

  // LoginUseCaseをインスタンス
  let usecase = LoginUseCase::new(
    state.user_repo.as_ref(),
    state.refresh_token_repo.as_ref(),
    state.jwt_service.as_ref(),
  );

  // ログイン処理を実行
  let output = usecase
    .execute(LoginInput {
      username: req.username.clone(),
      password: req.password,
      user_agent,
      refresh_token_expires_secs: state.config.jwt.refresh_token_expires_secs,
    })
    .await
    .map_err(ApiAppError::from)?;

  info!("ログイン成功: {}", req.username);

  let secure = state.config.cookie.secure;

  // Access Token Cookie
  let access_cookie = Cookie::build((ACCESS_TOKEN_COOKIE, output.access_token))
    .http_only(true)
    .secure(secure)
    .same_site(SameSite::Strict)
    .path("/api")
    .build();

  // Refresh Token Cookie（パスを /api/auth/refresh に限定）
  let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, output.refresh_token))
    .http_only(true)
    .secure(secure)
    .same_site(SameSite::Strict)
    .path("/api/auth")
    .build();

  // CookieJarに追加
  let jar = jar.add(access_cookie).add(refresh_cookie);

  Ok((
    jar,
    Json(ApiResponse::ok(LoginResponse {
      username: req.username,
    })),
  ))
}

// ─── Refresh ─────────────────────────────────────────────
pub async fn refresh_handler(
  State(state): State<AppState>,
  jar: CookieJar,
) -> Result<impl IntoResponse, ApiAppError> {
  // Refresh Token CookieからTokenを取り出す
  let refresh_token = jar
    .get(REFRESH_TOKEN_COOKIE)
    .map(|c| c.value().to_string())
    .ok_or_else(|| ApiAppError::from(app::AppError::Unauthorized))?;

  // RefreshTokenをインスタンス
  let usecase = RefreshUseCase::new(
    state.user_repo.as_ref(),
    state.refresh_token_repo.as_ref(),
    state.jwt_service.as_ref(),
  );

  // RefreshTokenの更新
  let output = usecase
    .execute(RefreshInput {
      refresh_token,
      user_agent: None,
      refresh_token_expires_secs: state.config.jwt.refresh_token_expires_secs,
    })
    .await
    .map_err(ApiAppError::from)?;

  // configからクッキー認証を有効にするかのboolを取得
  let secure = state.config.cookie.secure;

  // AccessTokenのCookieを構築
  let access_cookie = Cookie::build((ACCESS_TOKEN_COOKIE, output.access_token))
    .http_only(true)
    .secure(secure)
    .same_site(SameSite::Strict)
    .path("/api")
    .build();

  // RefreshTokenのCookieを構築
  let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, output.refresh_token))
    .http_only(true)
    .secure(secure)
    .same_site(SameSite::Strict)
    .path("/api/auth")
    .build();

  // CookieJarに追加
  let jar = jar.add(access_cookie).add(refresh_cookie);

  Ok((jar, Json(ApiResponse::ok(()))))
}

// ─── Logout ──────────────────────────────────────────────
pub async fn logout_handler(
  State(state): State<AppState>,
  jar: CookieJar,
) -> Result<impl IntoResponse, ApiAppError> {
  // RefreshTokenをCookieJarから取得
  let refresh_token = jar
    .get(REFRESH_TOKEN_COOKIE)
    .map(|c| c.value().to_string())
    .ok_or_else(|| ApiAppError::from(app::AppError::Unauthorized))?;

  // LogoutUseCaseのインスタンス
  let usecase = LogoutUseCase::new(state.refresh_token_repo.as_ref());

  // ログアウト処理実行
  usecase
    .execute(LogoutInput { refresh_token })
    .await
    .map_err(ApiAppError::from)?;

  // Cookieを削除する（空文字・過去日時でmax-ageを0にする）
  // AccessToken削除
  let remove_access = Cookie::build((ACCESS_TOKEN_COOKIE, ""))
    .http_only(true)
    .secure(state.config.cookie.secure)
    .same_site(SameSite::Strict)
    .path("/api")
    .max_age(time::Duration::ZERO)
    .build();
  // RefreshToken削除
  let remove_refresh = Cookie::build((REFRESH_TOKEN_COOKIE, ""))
    .secure(state.config.cookie.secure)
    .same_site(SameSite::Strict)
    .http_only(true)
    .path("/api/auth")
    .max_age(time::Duration::ZERO)
    .build();

  // 削除済みCookieをJarに追加
  let jar = jar.add(remove_access).add(remove_refresh);

  Ok((jar, Json(ApiResponse::ok(()))))
}
