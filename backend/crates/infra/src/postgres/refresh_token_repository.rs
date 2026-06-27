/*
backend/crates/infra/src/postgres/refresh_token_repository.rs
postgresのRefreshTokenRepository実体を定義
*/

// 外部クレート
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;

// 内部ライブラリ
use auth::model::RefreshToken;
use identity::{RefreshTokenId, UserId};
use repository::{RefreshTokenRepository, RepoError, RepoResult};

// 自クレート
// エラー型伝搬用
use crate::error::{InfraError, InfraResult};
// RefreshTokenRow用
use crate::postgres::refresh_token_row::RefreshTokenRow;

/// postgreSQLのRefreshTokenRepository実装
pub struct PgRefreshTokenRepository {
  /// DBコネクションプール
  pool: PgPool,
}

impl PgRefreshTokenRepository {
  /// コンストラクタ
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

// エラー型を伝搬させるため、
// 内部実装ブロックで分け、
// トレイト実装はすべて .map_err(RepoError::from) で委譲するだけ
#[async_trait]
impl RefreshTokenRepository for PgRefreshTokenRepository {
  /// ハッシュ化されたRefreshTokenからRefreshTokenを取得する
  async fn find_by_token_hash(&self, hash: &str) -> RepoResult<Option<RefreshToken>> {
    self
      .find_by_token_hash_impl(hash)
      .await
      .map_err(RepoError::from)
  }

  /// 新規RefreshToken行を作成する
  async fn create(&self, token: &RefreshToken) -> RepoResult<()> {
    self.create_impl(token).await.map_err(RepoError::from)
  }

  /// 対象RefreshTokenIdを失効させる
  async fn revoke(&self, id: &RefreshTokenId) -> RepoResult<()> {
    self.revoke_impl(id).await.map_err(RepoError::from)
  }

  /// 対象ユーザの全RefreshTokenを失効させる
  async fn revoke_all_for_user(&self, user_id: &UserId) -> RepoResult<()> {
    self
      .revoke_all_for_user_impl(user_id)
      .await
      .map_err(RepoError::from)
  }

  /// 有効期限切れのRefreshTokenを削除し、削除した件数を返す
  async fn delete_expired(&self) -> RepoResult<u64> {
    self.delete_expired_impl().await.map_err(RepoError::from)
  }
}

// エラー型を伝搬させるため、
// 内部実装ブロックで分け、
// トレイト実装はすべて .map_err(RepoError::from) で委譲するだけ
impl PgRefreshTokenRepository {
  /// ハッシュ化されたRefreshTokenからRefreshTokenを取得find_by_token_hashの内部実装
  async fn find_by_token_hash_impl(&self, hash: &str) -> InfraResult<Option<RefreshToken>> {
    // RefreshTokenの取得
    let row = sqlx::query_as!(
      RefreshTokenRow,
      r#"
      SELECT
        id,
        user_id,
        token_hash,
        user_agent,
        expires_at,
        created_at,
        revoked_at
      FROM
        refresh_tokens
      WHERE
        token_hash = $1
      "#,
      hash
    )
    .fetch_optional(&self.pool)
    .await?;

    // query_asで得たNodeRow型ををNode型に変換
    let refresh_token = row.map(RefreshToken::try_from).transpose()?;

    // InfraResultで返す
    Ok(refresh_token)
  }

  /// 新規RefreshToken行を作成するcreateの内部実装
  async fn create_impl(&self, token: &RefreshToken) -> InfraResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO refresh_tokens (
        id,
        user_id,
        token_hash,
        user_agent,
        expires_at,
        created_at
      )
      VALUES ($1, $2, $3, $4, $5, $6)
      "#,
      token.id().as_uuid(),
      token.user_id().as_uuid(),
      token.token_hash(),
      token.user_agent(),
      token.expires_at(),
      token.created_at(),
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  /// 対象RefreshTokenIdを失効させるrevokeの内部実装
  async fn revoke_impl(&self, id: &RefreshTokenId) -> InfraResult<()> {
    // revoked_atに現在時間を入力
    // 確認のために更新件数を取得
    let affected = sqlx::query!(
      r#"
      UPDATE
        refresh_tokens
      SET
        revoked_at = $2
      WHERE
        id = $1
        AND revoked_at IS NULL
      "#,
      id.as_uuid(),
      Utc::now(),
    )
    .execute(&self.pool)
    .await?
    .rows_affected();

    // 取得失敗したらNotFoundエラー
    if affected == 0 {
      return Err(InfraError::NotFound);
    }

    Ok(())
  }

  /// 対象ユーザの全RefreshTokenを失効させる
  async fn revoke_all_for_user_impl(&self, user_id: &UserId) -> InfraResult<()> {
    // revoked_atに現在時刻を入力
    sqlx::query!(
      r#"
      UPDATE
        refresh_tokens
      SET
        revoked_at = $2
      WHERE user_id = $1
        AND revoked_at IS NULL
      "#,
      user_id.as_uuid(),
      Utc::now(),
    )
    .execute(&self.pool)
    .await?;

    // sqlx::query!はPgQueryResult
    // InfraResultとして返す
    Ok(())
  }

  /// 有効期限切れのRefreshTokenを削除し、削除した件数を返すdelete_expiredの内部実装
  async fn delete_expired_impl(&self) -> InfraResult<u64> {
    let affected = sqlx::query!(
      r#"
      DELETE FROM
        refresh_tokens
      WHERE
        expires_at < now()
      "#
    )
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(affected)
  }
}
