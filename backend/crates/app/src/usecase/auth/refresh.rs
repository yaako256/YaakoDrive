/*
backend/crates/app/src/usecase/auth/refresh.rs
RefreshTokenの更新をするユースケースを定義
*/

// 内部ライブラリ
use auth::token::hash_token;
use auth::{jwt::JwtService, model::RefreshToken};
use repository::{RefreshTokenRepository, UserRepository};

// 自クレート
use crate::error::{AppError, AppResult};

pub struct RefreshInput {
  pub refresh_token: String,
  pub user_agent: Option<String>,
  pub refresh_token_expires_secs: u64,
}

pub struct RefreshOutput {
  pub access_token: String,
  pub refresh_token: String,
}

pub struct RefreshUseCase<'a> {
  user_repo: &'a dyn UserRepository,
  refresh_token_repo: &'a dyn RefreshTokenRepository,
  jwt_service: &'a JwtService,
}

impl<'a> RefreshUseCase<'a> {
  pub fn new(
    user_repo: &'a dyn UserRepository,
    refresh_token_repo: &'a dyn RefreshTokenRepository,
    jwt_service: &'a JwtService,
  ) -> Self {
    Self {
      user_repo,
      refresh_token_repo,
      jwt_service,
    }
  }

  // RefreshToken更新処理を実行
  pub async fn execute(&self, input: RefreshInput) -> AppResult<RefreshOutput> {
    // refresh_tokenをハッシュ化
    let hash = hash_token(&input.refresh_token);

    // DBからRefresh Token取得
    let token = self
      .refresh_token_repo
      .find_by_token_hash(&hash)
      .await?
      .ok_or(AppError::Unauthorized)?;

    // 有効性チェック
    if !token.is_valid() {
      return Err(AppError::Unauthorized);
    }

    // ユーザ取得
    let user = self
      .user_repo
      .find_by_id(token.user_id())
      .await?
      .ok_or(AppError::Unauthorized)?;

    if user.is_disabled() {
      return Err(AppError::Unauthorized);
    }

    // 旧Refresh Tokenをrevoke(ローテーション)
    self.refresh_token_repo.revoke(token.id()).await?;

    // 新Access Token生成
    let access_token = self
      .jwt_service
      .generate_access_token(user.id(), user.role().as_str())?;

    // 新Refresh Token生成
    // RefreshToken型を作成
    let (raw_token, refresh_token) = RefreshToken::new(
      *user.id(),
      input.user_agent,
      input.refresh_token_expires_secs,
    );

    // DBに保存
    self.refresh_token_repo.create(&refresh_token).await?;

    Ok(RefreshOutput {
      access_token,
      refresh_token: raw_token,
    })
  }
}
