/*
backend/crates/server/src/error.rs
Configクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;

// 内部ライブラリ
use config;

/// Configクレートのエラー型
#[derive(Debug, Error)]
pub enum ServerError {
  #[error("[Configエラー]: {0}")]
  Config(#[from] config::ConfigError),

  // axumは標準のstd::ioのエラー
  // axum以外でこのエラーが出ないため、そのまま使う
  #[error("[Configエラー]: {0}")]
  Axum(#[from] std::io::Error),
}

/// Configクレートのリザルト
pub type ServerResult<T> = Result<T, ServerError>;
