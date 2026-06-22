/*
backend/crates/app/src/usecase/auth/login.rs
ログインのユースケース
*/

// 外部クレート
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};

// 内部ライブラリ
use auth::{
  jwt::JwtService, model::RefreshToken, password::verify_password, token::generate_refresh_token,
};
use identity::RefreshTokenId;
use repository::{RefreshTokenRepository, UserRepository};

// 自クレート
use crate::error::{AppError, AppResult};

// ログイン認証の入力
pub struct LoginInput {
  pub username: String,
  pub password: String,
  pub user_agent: Option<String>,
  /// config から渡す。秒単位
  pub refresh_token_expires_secs: u64,
}

///　ログイン認証の出力
pub struct LoginOutput {
  pub access_token: String,
  /// フロントエンドには渡さない。Cookieセットのためにapi層へ返す
  pub refresh_token: String,
}

/// ログイン認証のユースケース構造体
pub struct LoginUseCase<'a> {
  user_repo: &'a dyn UserRepository,
  refresh_token_repo: &'a dyn RefreshTokenRepository,
  jwt_service: &'a JwtService,
}

impl<'a> LoginUseCase<'a> {
  /// コンストラクタ
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

  // ログイン認証処理の実行
  pub async fn execute(&self, input: LoginInput) -> AppResult<LoginOutput> {
    // ユーザ取得
    let user = self
      .user_repo
      .find_by_username(&input.username)
      .await?
      .ok_or(AppError::Unauthorized)?;

    // disabledチェック
    if user.is_disabled() {
      return Err(AppError::Unauthorized);
    }

    // パスワード検証
    verify_password(&input.password, &user.password_hash)?;

    // Access Token生成
    let access_token = self
      .jwt_service
      .generate_access_token(&user.id, user.role.as_str())?;

    // Refresh Token生成
    let raw_token = generate_refresh_token();
    let token_hash = hash_token(&raw_token);

    // 現在時刻取得
    let now = Utc::now();
    // 失効時間を設定
    let expires_at = now + Duration::seconds(input.refresh_token_expires_secs as i64);

    // RefreshToken型を作成
    let refresh_token = RefreshToken {
      id: RefreshTokenId::new(),
      user_id: user.id,
      token_hash,
      user_agent: input.user_agent,
      expires_at,
      created_at: now,
      revoked_at: None,
    };

    // DBに保存
    self.refresh_token_repo.create(&refresh_token).await?;

    // ログイン認証の結果を返す
    Ok(LoginOutput {
      access_token,
      refresh_token: raw_token,
    })
  }
}

/// Refresh Tokenのハッシュ化
/// argon2は不要。SHA-256で十分
/// (推測不能な乱数を短時間ハッシュするだけ)
/// DefaultHasherは衝突耐性がないため使わない
pub(crate) fn hash_token(token: &str) -> String {
  // hasherの設定
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());

  // ハッシュ化をする
  hex::encode(hasher.finalize())
}
