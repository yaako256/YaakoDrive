/*
backend/crates/infra/src/postgres/user_repository.rs
postgresのUserRepository実体を定義
*/

// 外部クレート
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use auth::model::{Role, User};
use identity::UserId;
use repository::{RepoError, RepoResult, UserRepository};

// 自クレート
// エラー型伝搬用
use crate::error::{InfraError, InfraResult};

/// postgreSQLのUserRepository実装
pub struct PgUserRepository {
  /// DBコネクションプール
  pool: PgPool,
}

impl PgUserRepository {
  /// コンストラクタ
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

// エラー型を伝搬させるため、
// 内部実装ブロックで分け、
// トレイト実装はすべて .map_err(RepoError::from) で委譲するだけ
#[async_trait]
impl UserRepository for PgUserRepository {
  /// UserIdからUser型を取得する
  async fn find_by_id(&self, id: &UserId) -> RepoResult<Option<User>> {
    self.find_by_id_impl(id).await.map_err(RepoError::from)
  }

  /// usernameからUser型を取得する
  async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>> {
    self
      .find_by_username_impl(username)
      .await
      .map_err(RepoError::from)
  }

  /// 新規User行を作成する
  async fn create(&self, user: &User) -> RepoResult<()> {
    self.create_impl(user).await.map_err(RepoError::from)
  }

  /// 既存User行を更新する
  async fn update(&self, user: &User) -> RepoResult<()> {
    self.update_impl(user).await.map_err(RepoError::from)
  }

  /// 全ユーザを取得する
  async fn list_all(&self) -> RepoResult<Vec<User>> {
    self.list_all_impl().await.map_err(RepoError::from)
  }
}

impl PgUserRepository {
  /// UserIdからUser型を取得するfind_by_idの内部実装
  async fn find_by_id_impl(&self, id: &UserId) -> InfraResult<Option<User>> {
    let row = sqlx::query!(
      r#"
      SELECT
        id,
        username,
        password_hash,
        role,
        storage_limit_bytes,
        created_at,
        updated_at,
        disabled_at
      FROM users
      WHERE id = $1
      "#,
      id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await?;

    // 匿名構造体をUser型に変換して返す
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

  /// usernameからUser型を取得するfind_by_usernameの内部実装
  async fn find_by_username_impl(&self, username: &str) -> InfraResult<Option<User>> {
    // usernameから対象要素を取得
    let row = sqlx::query!(
      r#"
      SELECT
        id,
        username,
        password_hash,
        role,
        storage_limit_bytes,
        created_at,
        updated_at,
        disabled_at
      FROM
        users
      WHERE
        username = $1
      "#,
      username
    )
    .fetch_optional(&self.pool)
    .await?;

    // 匿名構造体をUser型に変換して返す
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

  /// 新規User行を作成するcreateの内部実装
  async fn create_impl(&self, user: &User) -> InfraResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO users (
        id,
        username,
        password_hash,
        role,
        storage_limit_bytes,
        created_at,
        updated_at,
        disabled_at
        )
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      "#,
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
    .await?;

    // sqlx::query!はPgQueryResult
    // InfraResultとして返す
    Ok(())
  }

  /// 既存User行を更新するupdateの内部実装
  async fn update_impl(&self, user: &User) -> InfraResult<()> {
    // 対象idのUser情報を更新する
    // 確認のために更新件数を取得
    let affected = sqlx::query!(
      r#"
      UPDATE
        users
      SET
        username = $2,
        password_hash = $3,
        role = $4,
        storage_limit_bytes = $5,
        updated_at = $6,
        disabled_at = $7
      WHERE
        id = $1
      "#,
      user.id.as_uuid(),
      user.username,
      user.password_hash,
      user.role.as_str(),
      user.storage_limit_bytes,
      user.updated_at,
      user.disabled_at,
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

  /// 全ユーザを取得するlist_allの内部実装
  async fn list_all_impl(&self) -> InfraResult<Vec<User>> {
    let rows = sqlx::query!(
      r#"
      SELECT
        id,
        username,
        password_hash,
        role,
        storage_limit_bytes,
        created_at,
        updated_at,
        disabled_at
      FROM
        users
      ORDER BY
        created_at ASC
      "#
    )
    .fetch_all(&self.pool)
    .await?;

    // 匿名構造体をUser型にして返す
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

/// 匿名構造体をUser型に変換する内部関数
fn map_user_row(
  id: Uuid,
  username: String,
  password_hash: String,
  role: String,
  storage_limit_bytes: i64,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  disabled_at: Option<DateTime<Utc>>,
) -> InfraResult<User> {
  // 文字列からEnum型へ変換
  // 失敗時はエラー伝搬
  let role = Role::try_from(role.as_str())?;

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

#[cfg(test)]
mod tests {
  use super::*;
  use auth::model::Role;
  use chrono::Utc;
  use identity::UserId;

  // テスト用DBのURLを環境変数から取得するヘルパー
  async fn test_pool() -> PgPool {
    let url =
      std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    sqlx::PgPool::connect(&url)
      .await
      .expect("Failed to connect")
  }

  // #[sqlx::test] はテスト用に一時DBを作成・破棄する。テスト間の干渉はない
  // #[sqlx::test]
  // migrationsが標準の場所にないため、パスを明示的に指定しないとダメ
  #[sqlx::test(migrations = "../../../sql/migrations")]
  async fn test_create_and_find_user(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let now = Utc::now();

    let user = User {
      id: UserId::new(),
      username: "testuser".to_string(),
      password_hash: "dummy_hash".to_string(),
      role: Role::User,
      storage_limit_bytes: 10737418240,
      created_at: now,
      updated_at: now,
      disabled_at: None,
    };

    repo.create(&user).await.unwrap();

    let found = repo.find_by_username("testuser").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().username, "testuser");
  }
}
