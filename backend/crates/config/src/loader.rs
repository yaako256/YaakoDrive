/*
backend/crates/config/src/loader.rs
configをapp.tomlや.envからloadする関数の定義
`.config/`は実行場所(カレントディレクトリ)と同じ階層(実行場所)に置く
*/

// 外部クレート
// config用
use config::{Config, Environment, File};

// 自クレート
// モデル定義
use crate::models::AppConfig;
// リザルト型
use crate::error::ConfigResult;

/// configを設定ファイルからロードする
pub fn load() -> ConfigResult<AppConfig> {
  // 環境変数から環境ラベルを読み込む
  let env = std::env::var("APP_ENV")?;

  // 環境変数から設定ファイルがあるディレクトリパスを読み込む
  let config_dir = std::env::var("APP_CONFIG_DIR")?;

  // 設定ファイルをロードする
  let settings = Config::builder()
    // 基本設定の読み込み(TOML)
    .add_source(File::with_name(&format!("{config_dir}/config.default.toml")).required(true))
    // 環境ラベル別上書き設定の読み込み(TOML)
    .add_source(File::with_name(&format!("{config_dir}/config.{env}.toml")).required(true))
    // 2. ENV上書き（APP__AAAAA__BBBB_BBB_BBBB形式）
    .add_source(Environment::with_prefix("APP").separator("__"))
    .build()?;

  // AppConfigにデシリアライズ
  let config = settings.try_deserialize::<AppConfig>()?;

  Ok(config)
}
