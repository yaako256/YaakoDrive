/*
backend/crates/server/src/main.rs
YaakoDriveの基本エントリポイント
サーバーが起動される
*/

// 外部クレート
// 非同期処理/低レイヤー通信
use tokio;
// Webフレームワーク/高レイヤー通信
use axum;
// ログ出力・整形用
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// ログ用
use tracing::info;

// 内部ライブラリ
// 設定ファイル読み込み用
use config;
// api用
use api;

// 自クレート
mod error;
use error::ServerResult;

#[tokio::main]
async fn main() -> ServerResult<()> {
  // tracing初期化
  // ログレベルは環境変数(RUST_LOG)で設定できる
  tracing_subscriber::registry()
    // ログレベルの設定(デフォルトでinfo)
    .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    // 画面出力の設定
    .with(tracing_subscriber::fmt::layer())
    .init();

  // サーバ起動処理
  if let Err(e) = run().await {
    // 成形済エラー出力
    eprintln!("[server] {}\n", e);
    // エラー終了
    return Err(e);
  }

  Ok(())
}

/// サーバの起動処理
///
/// main() で整形済みエラーメッセージを出力した後に
/// エラーを再伝搬させるため、実処理をこの関数に分離している。
async fn run() -> ServerResult<()> {
  // config読み込み（失敗したら即終了）
  let config = config::load()?;

  info!("Starting YaakoDrive server");
  info!("Listening on {}:{}", config.server.host, config.server.port);

  // Router組み立て(api crateから)
  let router = api::router::create_router();

  // 起動用のアドレス(ホスト:ポート)を構築
  let addr = format!("{}:{}", config.server.host, config.server.port);

  // 指定したアドレスでTCPリスナー(通信窓口)をバインド
  let listener = tokio::net::TcpListener::bind(&addr).await?;

  // Axumサーバーを起動してリクエストの待機を開始
  axum::serve(listener, router).await?;

  Ok(())
}
