/*
backend/crates/cli/src/main.rs
CLIのエントリポイント
*/
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod error;

use error::{CliError, CliResult};

#[derive(Parser)]
#[command(name = "yaakodrive-cli", about = "YaakoDrive 管理コマンド")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// 管理者ユーザを作成する
  CreateAdmin {
    #[arg(long)]
    username: String,
  },
  /// 期限切れRefresh Tokenを削除する
  CleanupExpiredTokens,
}

#[tokio::main]
async fn main() -> CliResult<()> {
  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    .with(tracing_subscriber::fmt::layer())
    .init();

  if let Err(e) = run().await {
    eprintln!("[cli] {}", e);
    return Err(e);
  }

  Ok(())
}

async fn run() -> CliResult<()> {
  let cli = Cli::parse();

  // config読み込み
  let config = config::load()?;

  // DB接続
  let pool = infra::postgres::db::create_pool(&config.database.url)
    .await
    .map_err(CliError::Database)?;

  match cli.command {
    Commands::CreateAdmin { username } => {
      commands::create_admin::run(pool, username).await?;
    }
    Commands::CleanupExpiredTokens => {
      commands::cleanup_expired_tokens::run(pool).await?;
    }
  }

  Ok(())
}
