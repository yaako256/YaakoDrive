/*
backend/crates/repository/src/file_content_repository.rs
FileContentRepositoryのトレイトを定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::FileContent;

// 自クレート
use crate::error::RepoResult;

/// MIME Type ごとのファイル数統計
pub struct MimeStat {
  pub mime_type: String,
  pub count: i64,
}

/// Dashboard 向けのファイル使用統計
pub struct UsageStats {
  pub total_bytes: i64,
  pub file_count: i64,
  pub mime_stats: Vec<MimeStat>,
}

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

  /// 複数 NodeId の stored_filename を取得する。
  /// 物理削除前に実ファイル名を収集するために使う。
  async fn find_stored_filenames_by_node_ids(&self, node_ids: &[NodeId])
  -> RepoResult<Vec<String>>;

  /// Dashboard 向けの使用統計を取得する。
  /// active かつ deleted_at IS NULL のファイルのみ集計する。
  async fn get_usage_stats(&self, owner_user_id: &UserId) -> RepoResult<UsageStats>;
}
