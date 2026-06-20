/*
backend/crates/config/src/models.rs
configのモデルを定義
*/
// 外部クレート
// デシリアライズ用
use serde::Deserialize;

/// configまとめ
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub jwt: JwtConfig,
  pub cookie: CookieConfig,
  pub storage: StorageConfig,
  pub upload: UploadConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
  // サーバのホストアドレス
  pub host: String,
  // サーバの公開ポート
  pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
  // DatabaseのURL
  pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
  pub secret: String,
  // AccessTokenの期限[秒]
  pub access_token_expires_secs: u64,
  // RefreshTokenの期限[秒]
  pub refresh_token_expires_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CookieConfig {
  // Cookie認証をするか(開発中:false、本番:true)
  pub secure: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
  // 物理ファイル保存場所(Path)
  pub data_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadConfig {
  // 最大アップロードサイズ
  pub max_size_bytes: u64,
}
