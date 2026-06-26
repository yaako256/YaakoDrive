/*
backend/crates/repository/src/node_repository.rs
NodeRepositoryのトレイトを定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;
// 時間型
use chrono::{DateTime, Utc};

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

  /// ゴミ箱のルート一覧を取得する。
  /// 「直接ゴミ箱に入れたノード」= 削除されているが親が active、
  /// またはルート直下のノードを返す。
  async fn list_trash_roots(&self, owner_user_id: &UserId) -> RepoResult<Vec<Node>>;

  /// ゴミ箱内フォルダの子ノード一覧を取得する。
  async fn list_deleted_children(
    &self,
    owner_user_id: &UserId,
    parent_id: &NodeId,
  ) -> RepoResult<Vec<Node>>;

  /// 指定した名前の active ノードが同フォルダ内に存在するか確認する。
  /// 復元時の同名衝突チェックに使う。
  /// parent_id が None の場合はルート直下を確認する。
  async fn exists_active_with_name(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
    name: &str,
  ) -> RepoResult<bool>;

  /// 指定ノードと配下の deleted_at を NULL に戻す(論理削除解除)。
  async fn restore_with_descendants(
    &self,
    id: &NodeId,
    updated_at: DateTime<Utc>,
  ) -> RepoResult<()>;

  /// 名前の部分一致検索(active かつ未削除のみ、大文字小文字を無視)。
  async fn search_by_name(&self, owner_user_id: &UserId, query: &str) -> RepoResult<Vec<Node>>;

  /// 指定ノードと配下の NodeId を全件返す(自身を含む)。
  /// 物理削除前に削除対象 ID を収集するために使う。
  async fn collect_descendant_ids(&self, id: &NodeId) -> RepoResult<Vec<NodeId>>;

  /// 複数の NodeId を一括物理削除する。
  /// file_contents は ON DELETE CASCADE で連動削除される。
  async fn hard_delete_many(&self, ids: &[NodeId]) -> RepoResult<()>;

  /// active フォルダ数を返す。Dashboard 集計に使う。
  async fn count_active_folders(&self, owner_user_id: &UserId) -> RepoResult<i64>;
}
