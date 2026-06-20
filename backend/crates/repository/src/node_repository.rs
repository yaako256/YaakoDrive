/*
backend/crates/repository/src/node_repository.rs
NodeRepositoryのトレイトを定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::Node;

// 自クレート
use crate::error::RepoResult;

/// ノードのテーブルを管理
#[async_trait]
pub trait NodeRepository: Send + Sync {
  /// IdからNode型を取得
  async fn find_by_id(&self, id: &NodeId) -> RepoResult<Option<Node>>;

  /// 子NodeをVecでリスト取得
  async fn list_children(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
  ) -> RepoResult<Vec<Node>>;

  /// 新規Nodeの作成
  async fn create(&self, node: &Node) -> RepoResult<()>;

  /// 既存Nodeの更新
  async fn update(&self, node: &Node) -> RepoResult<()>;

  /// 論理削除
  async fn soft_delete_with_descendants(
    &self,
    id: &NodeId,
    deleted_at: chrono::DateTime<chrono::Utc>,
  ) -> RepoResult<()>;

  /// 完全削除
  async fn hard_delete(&self, id: &NodeId) -> RepoResult<()>;

  /// 循環移動チェック用。idの祖先一覧を返す
  async fn find_ancestor_ids(&self, id: &NodeId) -> RepoResult<Vec<NodeId>>;
}
