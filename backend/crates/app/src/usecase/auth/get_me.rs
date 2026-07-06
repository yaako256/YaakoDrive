/*
backend/crates/app/src/usecase/auth/get_me.rs
認証済みデバイスがログインスキップするためのユースケース
*/

// 内部ライブラリ
use identity::UserId;
use repository::UserRepository;

// 自クレート
use crate::error::{AppError, AppResult};

pub struct GetMeInput {
  pub user_id: UserId,
}

pub struct GetMeOutput {
  pub username: String,
}

pub struct GetMeUseCase<'a> {
  user_repo: &'a dyn UserRepository,
}

impl<'a> GetMeUseCase<'a> {
  pub fn new(user_repo: &'a dyn UserRepository) -> Self {
    Self { user_repo }
  }

  pub async fn execute(&self, input: GetMeInput) -> AppResult<GetMeOutput> {
    let user = self
      .user_repo
      .find_by_id(&input.user_id)
      .await?
      .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    Ok(GetMeOutput {
      username: user.username().to_string(),
    })
  }
}
