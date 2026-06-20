/*
backend/crates/repository/src/user_repository.rs
UserRepositoryのトレイトを定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;

// 内部ライブラリ
use auth::model::User;
use identity::UserId;

// 自クレート
use crate::error::RepoResult;

// ユーザのテーブルを管理
#[async_trait]
pub trait UserRepository: Send + Sync {
  /// idからユーザを取得する
  async fn find_by_id(&self, id: &UserId) -> RepoResult<Option<User>>;

  /// ユーザ名からユーザを取得する
  async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>>;

  /// 新規ユーザの作成
  async fn create(&self, user: &User) -> RepoResult<()>;

  /// 既存ユーザ情報の更新
  async fn update(&self, user: &User) -> RepoResult<()>;

  // 全ユーザをVecでリスト取得
  async fn list_all(&self) -> RepoResult<Vec<User>>;
}
