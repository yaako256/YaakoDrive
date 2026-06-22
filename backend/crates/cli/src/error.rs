/*
backend/crates/cli/src/error.rs
cliクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

// 内部クレート
// エラー型伝搬用
use app::AppError;

/// cliクレートのエラー型
#[derive(Debug, Error)]
pub enum CliError {
  #[error("設定エラー: {0}")]
  Config(#[from] config::ConfigError),

  #[error("DBエラー: {0}")]
  Database(sqlx::Error),

  #[error("処理エラー: {0}")]
  App(#[from] AppError),
}

pub type CliResult<T> = Result<T, CliError>;
