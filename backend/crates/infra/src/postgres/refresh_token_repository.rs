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

pub struct PgRefreshTokenRepository {
  pool: PgPool,
}

impl PgRefreshTokenRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl RefreshTokenRepository for PgRefreshTokenRepository {
  async fn find_by_token_hash(&self, hash: &str) -> RepoResult<Option<RefreshToken>> {
    let row = sqlx::query!(
      "SELECT id, user_id, token_hash, user_agent, expires_at, created_at, revoked_at
             FROM refresh_tokens WHERE token_hash = $1",
      hash
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(row.map(|r| RefreshToken {
      id: RefreshTokenId::from_uuid(r.id),
      user_id: UserId::from_uuid(r.user_id),
      token_hash: r.token_hash,
      user_agent: r.user_agent,
      expires_at: r.expires_at,
      created_at: r.created_at,
      revoked_at: r.revoked_at,
    }))
  }

  async fn create(&self, token: &RefreshToken) -> RepoResult<()> {
    sqlx::query!(
      "INSERT INTO refresh_tokens
                (id, user_id, token_hash, user_agent, expires_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
      token.id.as_uuid(),
      token.user_id.as_uuid(),
      token.token_hash,
      token.user_agent,
      token.expires_at,
      token.created_at,
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(())
  }

  async fn revoke(&self, id: &RefreshTokenId) -> RepoResult<()> {
    let affected = sqlx::query!(
      "UPDATE refresh_tokens SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL",
      id.as_uuid(),
      Utc::now(),
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?
    .rows_affected();

    if affected == 0 {
      return Err(RepoError::NotFound);
    }
    Ok(())
  }

  async fn revoke_all_for_user(&self, user_id: &UserId) -> RepoResult<()> {
    sqlx::query!(
      "UPDATE refresh_tokens SET revoked_at = $2
             WHERE user_id = $1 AND revoked_at IS NULL",
      user_id.as_uuid(),
      Utc::now(),
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(())
  }

  async fn delete_expired(&self) -> RepoResult<u64> {
    let affected = sqlx::query!("DELETE FROM refresh_tokens WHERE expires_at < now()")
      .execute(&self.pool)
      .await
      .map_err(|e| RepoError::Database(e.to_string()))?
      .rows_affected();

    Ok(affected)
  }
}
