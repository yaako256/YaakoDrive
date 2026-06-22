/*
backend/crates/cli/src/commands/creat_admin.rs
管理者を作成するコマンドの処理を定義
*/

// 外部クレート
use infra::postgres::user_repository::PgUserRepository;
use sqlx::PgPool;
use tracing::info;

// 内部ライブラリ
// ユースケース用
use app::usecase::admin::create_admin::{CreateAdminInput, CreateAdminUseCase};

// 自クレート
// エラー型
use crate::error::{CliError, CliResult};

/// 管理者ユーザを作成する
pub async fn run(pool: PgPool, username: String) -> CliResult<()> {
  // パスワードを対話入力する
  // 平文をコマンド引数に渡すとシェル履歴に残るため対話入力にする
  let password = rpassword::prompt_password("パスワード: ")
    .map_err(|e| CliError::App(app::AppError::InvalidInput(e.to_string())))?;

  let confirm = rpassword::prompt_password("パスワード(確認): ")
    .map_err(|e| CliError::App(app::AppError::InvalidInput(e.to_string())))?;

  if password != confirm {
    return Err(CliError::App(app::AppError::InvalidInput(
      "パスワードが一致しません".to_string(),
    )));
  }

  let user_repo = PgUserRepository::new(pool);
  let usecase = CreateAdminUseCase::new(&user_repo);

  usecase
    .execute(CreateAdminInput {
      username: username.clone(),
      password,
    })
    .await?;

  info!("管理者ユーザ '{}' を作成しました", username);
  println!("✓ 管理者ユーザ '{}' を作成しました", username);

  Ok(())
}
