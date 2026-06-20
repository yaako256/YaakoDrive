/*
backend/crates/config/src/error.rs
Configクレートのエラー型の定義
*/

// 外部クレート
// エラー型作成用
use thiserror::Error;
// dotenvy::Error型用
use dotenvy;
// config::ConfigError型用
use config;

/// Configクレートのエラー型
#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("envロード失敗: {0}")]
  Env(#[from] dotenvy::Error),
  #[error("Config失敗: {0}")]
  Config(#[from] config::ConfigError),
}

/// Configクレートのリザルト
pub type ConfigResult<T> = Result<T, ConfigError>;
