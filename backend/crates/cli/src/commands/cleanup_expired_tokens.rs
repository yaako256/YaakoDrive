/*
backend/crates/cli/src/commands/cleanup_expired_tokens.rs
全ユーザのRefreshTokenを失効させるコマンドの処理を定義
*/

// 外部クレート
use sqlx::PgPool;
use tracing::info;

// 自クレート
// エラー型
use crate::error::CliResult;

pub async fn run(_pool: PgPool) -> CliResult<()> {
  info!("cleanup-expired-tokens: Phase 7以降で実装予定");
  println!("cleanup-expired-tokens は Phase 7以降で実装します");
  Ok(())
}
