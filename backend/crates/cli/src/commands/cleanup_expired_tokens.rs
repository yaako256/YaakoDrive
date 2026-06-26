/*
backend/crates/cli/src/commands/cleanup_expired_tokens.rs
期限切れのRefreshTokenを削除するCLIコマンド
*/

// 外部クレート
use sqlx::PgPool;
use tracing::info;

// 内部ライブラリ
// トレイト型
use repository::RefreshTokenRepository;
// 実構造体
use infra::postgres::refresh_token_repository::PgRefreshTokenRepository;

// 自クレート
// エラー型
use crate::error::{CliError, CliResult};

//
pub async fn run(pool: PgPool) -> CliResult<()> {
  // PgRefreshTokenRepositoryのインスタンス
  let repo = PgRefreshTokenRepository::new(pool);

  // 期限切れのRefreshTokenを削除
  // 削除数を取得
  let deleted = repo
    .delete_expired()
    .await
    .map_err(|e| CliError::App(app::AppError::Repository(e.to_string())))?;

  // ログ出力
  info!("期限切れRefreshTokenを{}件削除しました", deleted);
  println!("✓ 期限切れRefreshTokenを{}件削除しました", deleted);

  Ok(())
}
