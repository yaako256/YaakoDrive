/*
backend/crates/config/src/loader.rs
configをapp.tomlや.envからloadする関数の定義
`.config/`は実行場所(カレントディレクトリ)と同じ階層(実行場所)に置く
*/

// 外部クレート
// envファイル読み込み用
use dotenvy;
// config用
use config::{Config, Environment, File};

// 自クレート
// モデル定義
use crate::models::AppConfig;
// リザルト型
use crate::error::ConfigResult;

/// configを設定ファイルからロードする
pub fn load() -> ConfigResult<AppConfig> {
  // .env を読む
  // 読み込み失敗はエラー(env上書き必須のものがあるため)
  dotenvy::dotenv()?;

  // 環境変数から環境ラベルを読み込む
  let env = std::env::var("APP_ENV")?;

  // 設定ファイルをロードする
  let settings = Config::builder()
    // 基本設定の読み込み(TOML)
    .add_source(File::with_name(".config/config.toml").required(false))
    // 環境ラベル別上書き設定の読み込み(TOML)
    .add_source(File::with_name(&format!(".config/{env}.toml")).required(false))
    // 2. ENV上書き（APP__AAAAA__BBBB_BBB_BBBB形式）
    .add_source(
      Environment::with_prefix("APP")
        .separator("__")
        .try_parsing(true)
        .list_separator(","),
    )
    .build()?;

  // AppConfigにデシリアライズ
  let config = settings.try_deserialize::<AppConfig>()?;

  Ok(config)
}
