/*
backend/crates/repository/src/unit_of_work.rs
UnitOfWorkのトレイトを定義
アップロード時に nodes と file_contents を同一トランザクションで扱うための仕組み
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;

// 自クレート
use crate::error::RepoResult;
use crate::{
  file_content_repository::FileContentRepository, node_repository::NodeRepository,
  refresh_token_repository::RefreshTokenRepository, user_repository::UserRepository,
};

/// トランザクション内で使えるRepositoryをまとめたコンテキスト
#[async_trait]
pub trait TransactionContext: Send + Sync {
  fn users(&self) -> &dyn UserRepository;
  fn refresh_tokens(&self) -> &dyn RefreshTokenRepository;
  fn nodes(&self) -> &dyn NodeRepository;
  fn file_contents(&self) -> &dyn FileContentRepository;

  async fn commit(self: Box<Self>) -> RepoResult<()>;
  async fn rollback(self: Box<Self>) -> RepoResult<()>;
}

#[async_trait]
pub trait UnitOfWork: Send + Sync {
  async fn begin(&self) -> RepoResult<Box<dyn TransactionContext>>;
}
