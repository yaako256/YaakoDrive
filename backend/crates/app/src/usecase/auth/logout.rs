/*
backend/crates/app/src/usecase/auth/logout.rs
ログアウトのユースケース
*/

// 内部クレート
use auth::token::hash_token;
use repository::RefreshTokenRepository;

// 自クレート
use crate::error::{AppError, AppResult};

// ログアウトの入力
pub struct LogoutInput {
  pub refresh_token: String,
}

/// ログアウトのユースケース構造体
pub struct LogoutUseCase<'a> {
  refresh_token_repo: &'a dyn RefreshTokenRepository,
}

impl<'a> LogoutUseCase<'a> {
  /// コンストラクタ
  pub fn new(refresh_token_repo: &'a dyn RefreshTokenRepository) -> Self {
    Self { refresh_token_repo }
  }

  // ログアウト処理の実行
  pub async fn execute(&self, input: LogoutInput) -> AppResult<()> {
    // refresh_tokenをハッシュ化
    let hash = hash_token(&input.refresh_token);

    // refresh_tokenからRefreshToken型を取得
    let token = self
      .refresh_token_repo
      .find_by_token_hash(&hash)
      .await?
      .ok_or(AppError::Unauthorized)?;

    // 失効させる(DB処理)
    self.refresh_token_repo.revoke(token.id()).await?;

    Ok(())
  }
}
