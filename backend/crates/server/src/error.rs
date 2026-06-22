/*
backend/crates/server/src/error.rs
serverクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use sqlx;
use thiserror::Error;

// 内部ライブラリ
use config;

/// Configクレートのエラー型
#[derive(Debug, Error)]
pub enum ServerError {
  #[error("[config] {0}")]
  Config(#[from] config::ConfigError),

  #[error("DBエラー: {0}")]
  Database(#[from] sqlx::Error),

  // axumは標準のstd::ioのエラー
  // axum以外でこのエラーが出ないため、そのまま使う
  #[error("axumエラー: {0}")]
  Axum(#[from] std::io::Error),
}

/// Configクレートのリザルト
pub(crate) type ServerResult<T> = Result<T, ServerError>;
