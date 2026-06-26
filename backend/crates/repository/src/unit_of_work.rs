/*
backend/crates/repository/src/unit_of_work.rs
UnitOfWorkのトレイトを定義
アップロード時に nodes と file_contents を同一トランザクションで扱うための仕組み
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;
use node::model::{FileContent, Node};

// 自クレート
use crate::error::RepoResult;

/// トランザクション内で必要な操作だけを定義する
/// &dyn Repository を返す設計は自己参照問題で成立しないため、
/// TransactionContext 自体がトランザクション操作のメソッドを持つ設計にする
#[async_trait]
pub trait TransactionContext: Send + Sync {
  // Node 操作
  // 新規Nodeの作成
  async fn insert_node(&mut self, node: &Node) -> RepoResult<()>;
  // Nodeの更新
  async fn update_node(&mut self, node: &Node) -> RepoResult<()>;

  // FileContent 操作
  // 新規FileContentの作成
  async fn insert_file_content(&mut self, content: &FileContent) -> RepoResult<()>;
  // FileContentの更新
  async fn update_file_content(&mut self, content: &FileContent) -> RepoResult<()>;

  // トランザクション制御
  async fn commit(self: Box<Self>) -> RepoResult<()>;
  async fn rollback(self: Box<Self>) -> RepoResult<()>;
}

// トラジェクションを開始する
#[async_trait]
pub trait UnitOfWork: Send + Sync {
  async fn begin(&self) -> RepoResult<Box<dyn TransactionContext>>;
}
