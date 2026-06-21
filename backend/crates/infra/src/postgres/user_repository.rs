/*
backend/crates/infra/src/postgres/user_repository.rs
postgresのUserRepository実体を定義
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use auth::model::{Role, User};
use identity::UserId;
use repository::{RepoError, RepoResult, UserRepository};

pub struct PgUserRepository {
  pool: PgPool,
}

impl PgUserRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UserRepository for PgUserRepository {
  async fn find_by_id(&self, id: &UserId) -> RepoResult<Option<User>> {
    let row = sqlx::query!(
      "SELECT id, username, password_hash, role, storage_limit_bytes,
                    created_at, updated_at, disabled_at
             FROM users WHERE id = $1",
      id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    row
      .map(|r| {
        map_user_row(
          r.id,
          r.username,
          r.password_hash,
          r.role,
          r.storage_limit_bytes,
          r.created_at,
          r.updated_at,
          r.disabled_at,
        )
      })
      .transpose()
  }

  async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>> {
    let row = sqlx::query!(
      "SELECT id, username, password_hash, role, storage_limit_bytes,
                    created_at, updated_at, disabled_at
             FROM users WHERE username = $1",
      username
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    row
      .map(|r| {
        map_user_row(
          r.id,
          r.username,
          r.password_hash,
          r.role,
          r.storage_limit_bytes,
          r.created_at,
          r.updated_at,
          r.disabled_at,
        )
      })
      .transpose()
  }

  async fn create(&self, user: &User) -> RepoResult<()> {
    sqlx::query!(
      "INSERT INTO users
                (id, username, password_hash, role, storage_limit_bytes,
                 created_at, updated_at, disabled_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
      user.id.as_uuid(),
      user.username,
      user.password_hash,
      user.role.as_str(),
      user.storage_limit_bytes,
      user.created_at,
      user.updated_at,
      user.disabled_at,
    )
    .execute(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(())
  }

  async fn update(&self, user: &User) -> RepoResult<()> {
    let affected = sqlx::query!(
      "UPDATE users
             SET username = $2, password_hash = $3, role = $4,
                 storage_limit_bytes = $5, updated_at = $6, disabled_at = $7
             WHERE id = $1",
      user.id.as_uuid(),
      user.username,
      user.password_hash,
      user.role.as_str(),
      user.storage_limit_bytes,
      user.updated_at,
      user.disabled_at,
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

  async fn list_all(&self) -> RepoResult<Vec<User>> {
    let rows = sqlx::query!(
      "SELECT id, username, password_hash, role, storage_limit_bytes,
                    created_at, updated_at, disabled_at
             FROM users ORDER BY created_at ASC"
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    rows
      .into_iter()
      .map(|r| {
        map_user_row(
          r.id,
          r.username,
          r.password_hash,
          r.role,
          r.storage_limit_bytes,
          r.created_at,
          r.updated_at,
          r.disabled_at,
        )
      })
      .collect()
  }
}

fn map_user_row(
  id: Uuid,
  username: String,
  password_hash: String,
  role: String,
  storage_limit_bytes: i64,
  created_at: chrono::DateTime<chrono::Utc>,
  updated_at: chrono::DateTime<chrono::Utc>,
  disabled_at: Option<chrono::DateTime<chrono::Utc>>,
) -> RepoResult<User> {
  let role = Role::try_from(role.as_str()).map_err(|e| RepoError::Database(e))?;

  Ok(User {
    id: UserId::from_uuid(id),
    username,
    password_hash,
    role,
    storage_limit_bytes,
    created_at,
    updated_at,
    disabled_at,
  })
}
