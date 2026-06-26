/*
backend/crates/infra/src/postgres/refresh_token_row.rs
query_as用のRefreshTokenRowを定義
*/

// 外部クレート
use chrono::{DateTime, Utc};
use uuid::Uuid;

// 内部ライブラリ
// Id型
use identity::{RefreshTokenId, UserId};
// node型
use auth::AuthError;
use auth::model::RefreshToken;

// query_asにする用の構造体
// fromで変換し安全性を高める
#[derive(Debug)]
pub struct RefreshTokenRow {
  pub id: Uuid,
  pub user_id: Uuid,
  pub token_hash: String,
  pub user_agent: Option<String>,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub revoked_at: Option<DateTime<Utc>>,
}

impl TryFrom<RefreshTokenRow> for RefreshToken {
  type Error = AuthError;

  fn try_from(row: RefreshTokenRow) -> Result<Self, Self::Error> {
    Ok(RefreshToken::reconstitute(
      RefreshTokenId::from_uuid(row.id),
      UserId::from_uuid(row.user_id),
      row.token_hash,
      row.user_agent,
      row.expires_at,
      row.created_at,
      row.revoked_at,
    ))
  }
}
