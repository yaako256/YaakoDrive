/*
backend/crates/repository/src/file_content_repository.rs
FileContentRepositoryのトレイトを定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;

// 内部ライブラリ
use identity::NodeId;
use node::model::FileContent;

// 自クレート
use crate::error::RepoResult;

// ファイル中身のテーブルを管理
#[async_trait]
pub trait FileContentRepository: Send + Sync {
  /// NodeIdからFileContentを取得
  async fn find_by_node_id(&self, node_id: &NodeId) -> RepoResult<Option<FileContent>>;

  /// 新規FileContentの作成
  async fn create(&self, content: &FileContent) -> RepoResult<()>;

  /// 既存FileContentの更新
  async fn update(&self, content: &FileContent) -> RepoResult<()>;

  /// 完全削除
  async fn hard_delete(&self, node_id: &NodeId) -> RepoResult<()>;
}
