/*
backend/crates/infra/src/unit_of_work.rs
postgresのTransactionContext実体を定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;
// sqlx
use sqlx::{PgPool, Postgres, Transaction};

// 内部ライブラリ
// トレイト型
use repository::{
  FileContentRepository, NodeRepository, RefreshTokenRepository, RepoResult, TransactionContext,
  UnitOfWork, UserRepository,
};

// 自クレート
use crate::postgres::{
  file_content_repository::PgFileContentRepository, node_repository::PgNodeRepository,
  refresh_token_repository::PgRefreshTokenRepository, user_repository::PgUserRepository,
};

// pgのトラジェクション定義
pub struct PgTransactionContext {
  tx: Transaction<'static, Postgres>,
}

#[async_trait]
impl TransactionContext for PgTransactionContext {
  fn users(&self) -> &dyn UserRepository {
    // トランザクション内のRepositoryはStep 6で実装する
    // ここでは一旦コンパイルが通る形にしておく
    todo!()
  }

  fn refresh_tokens(&self) -> &dyn RefreshTokenRepository {
    todo!()
  }

  fn nodes(&self) -> &dyn NodeRepository {
    todo!()
  }

  fn file_contents(&self) -> &dyn FileContentRepository {
    todo!()
  }

  async fn commit(self: Box<Self>) -> RepoResult<()> {
    self
      .tx
      .commit()
      .await
      .map_err(|e| repository::RepoError::Database(e.to_string()))
  }

  async fn rollback(self: Box<Self>) -> RepoResult<()> {
    self
      .tx
      .rollback()
      .await
      .map_err(|e| repository::RepoError::Database(e.to_string()))
  }
}

/// pgのUnitOfWork
pub struct PgUnitOfWork {
  pool: PgPool,
}

impl PgUnitOfWork {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
  async fn begin(&self) -> RepoResult<Box<dyn TransactionContext>> {
    let tx = self
      .pool
      .begin()
      .await
      .map_err(|e| repository::RepoError::Database(e.to_string()))?;
    Ok(Box::new(PgTransactionContext { tx }))
  }
}
