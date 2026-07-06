/*
backend/crates/cli/src/commands/migrate.rs
DBマイグレーションを実行するコマンド
*/

// 標準ライブラリ
use std::path::Path;

// 外部クレート
// migration用
use sqlx::PgPool;
use sqlx::migrate::Migrator;
// ログ
use tracing::info;

// 自クレート
use crate::error::{CliError, CliResult};

pub async fn run(pool: PgPool, migrations_path: &Path) -> CliResult<()> {
  info!("マイグレーションを開始します: {:?}", migrations_path);

  let migrator = Migrator::new(migrations_path)
    .await
    .map_err(|e| CliError::Database(e.into()))?;

  migrator
    .run(&pool)
    .await
    .map_err(|e| CliError::Database(e.into()))?;

  info!("マイグレーションが完了しました");

  Ok(())
}
