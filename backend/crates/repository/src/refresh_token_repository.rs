/*
backend/crates/repository/src/refresh_token_repository.rs
RefreshTokenRepositoryのトレイトを定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;

// 内部ライブラリ
use auth::model::RefreshToken;
use identity::{RefreshTokenId, UserId};

// 自クレート
use crate::error::RepoResult;

/// RefreshTokenのテーブルを管理
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
  // ハッシュ化されたトークンからRefreshTokenを取得
  async fn find_by_token_hash(&self, hash: &str) -> RepoResult<Option<RefreshToken>>;

  // 新規トークンの作成
  async fn create(&self, token: &RefreshToken) -> RepoResult<()>;

  // トークンの無効化
  async fn revoke(&self, id: &RefreshTokenId) -> RepoResult<()>;

  // 全ユーザのトークンの無効化
  async fn revoke_all_for_user(&self, user_id: &UserId) -> RepoResult<()>;

  // 期限が切れてからの時間
  async fn delete_expired(&self) -> RepoResult<u64>;
}
