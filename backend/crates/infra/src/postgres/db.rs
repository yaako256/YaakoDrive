/*
backend/crates/infra/src/db.rs
DB接続プールを作る関数を定義
serverとcliの両方から呼ばれる
*/

// DB接続プール用
use sqlx::PgPool;

/// DB接続プールを作成する
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
  sqlx::postgres::PgPoolOptions::new()
    .max_connections(10)
    .connect(database_url)
    .await
}
