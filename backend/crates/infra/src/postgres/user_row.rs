/*
backend/crates/infra/src/postgres/user_row.rs
query_as用のUserRowを定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use uuid::Uuid;

// 内部ライブラリ
// Id型
use identity::UserId;
// node型
use auth::AuthError;
use auth::model::{Role, User};

// query_asにする用の構造体
// fromで変換し安全性を高める
#[derive(Debug)]
pub struct UserRow {
  pub id: Uuid,
  pub username: String,
  pub password_hash: String,
  pub role: String,
  pub storage_limit_bytes: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub disabled_at: Option<DateTime<Utc>>,
}

impl TryFrom<UserRow> for User {
  type Error = AuthError;

  fn try_from(row: UserRow) -> Result<Self, Self::Error> {
    Ok(User::reconstitute(
      UserId::from_uuid(row.id),
      row.username,
      row.password_hash,
      Role::try_from(row.role.as_str())?,
      row.storage_limit_bytes,
      row.created_at,
      row.updated_at,
      row.disabled_at,
    ))
  }
}
