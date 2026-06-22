/*
backend/crates/app/src/usecase/admin/create_admin.rs
管理者ユーザを作成するユースケース
*/

// 内部ライブラリ
use auth::{
  model::{Role, User},
  password::hash_password,
};
use chrono::Utc;
use identity::UserId;
use repository::UserRepository;

// 自クレート
// エラー型
use crate::error::{AppError, AppResult};

// 管理者を作るときの入力情報の構造体
pub struct CreateAdminInput {
  pub username: String,
  pub password: String,
}

/// 管理者を作成する構造体
pub struct CreateAdminUseCase<'a> {
  user_repo: &'a dyn UserRepository,
}

impl<'a> CreateAdminUseCase<'a> {
  /// コンストラクタ
  pub fn new(user_repo: &'a dyn UserRepository) -> Self {
    Self { user_repo }
  }

  /// 管理者を作成する構造体
  pub async fn execute(&self, input: CreateAdminInput) -> AppResult<()> {
    // 入力検証
    if input.username.is_empty() {
      return Err(AppError::InvalidInput("username is empty".to_string()));
    }
    if input.password.len() < 8 {
      return Err(AppError::InvalidInput(
        "password must be at least 8 characters".to_string(),
      ));
    }

    // 重複チェック
    let existing = self.user_repo.find_by_username(&input.username).await?;
    if existing.is_some() {
      return Err(AppError::AlreadyExists(format!(
        "username '{}' already exists",
        input.username
      )));
    }

    // パスワードハッシュ化
    let password_hash = hash_password(&input.password)?;

    // 現在時刻取得
    let now = Utc::now();

    // User型を作成
    let user = User {
      id: UserId::new(),
      username: input.username,
      password_hash,
      role: Role::Admin,
      storage_limit_bytes: 10737418240, // 10GB
      created_at: now,
      updated_at: now,
      disabled_at: None,
    };

    // DBに保存
    self.user_repo.create(&user).await?;

    Ok(())
  }
}
