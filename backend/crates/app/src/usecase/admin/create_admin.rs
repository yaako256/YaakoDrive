/*
backend/crates/app/src/usecase/admin/create_admin.rs
管理者ユーザを作成するユースケース
*/

// 内部ライブラリ
use auth::model::User;
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
    // ユーザ名重複チェック
    let existing = self.user_repo.find_by_username(&input.username).await?;
    if existing.is_some() {
      return Err(AppError::AlreadyExists(format!(
        "username '{}' already exists",
        input.username
      )));
    }

    // User型を作成
    let user = User::new_admin(input.username, input.password)?;

    // DBに保存
    self.user_repo.create(&user).await?;

    Ok(())
  }
}
