/*
backend/crates/auth/src/model.rs
認証関連の型定義
*/

// 外部クレート
// 時間型用
use chrono::{DateTime, Utc};

// 内部ライブラリ
// Id型用
use identity::{RefreshTokenId, UserId};

// 自クレート
use crate::error::AuthResult;
use crate::password::hash_password;
use crate::validation::{validate_password, validate_username};

/// ロールの列挙型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
  Admin,
  User,
}

impl Role {
  /// Roleの文字列変換
  pub fn as_str(&self) -> &'static str {
    match self {
      Role::Admin => "admin",
      Role::User => "user",
    }
  }
}

impl TryFrom<&str> for Role {
  type Error = String;

  // 文字列からRole型の取得
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    match s {
      "admin" => Ok(Role::Admin),
      "user" => Ok(Role::User),
      other => Err(format!("unknown role: {}", other)),
    }
  }
}

/// ユーザの型定義
#[derive(Debug, Clone)]
pub struct User {
  pub id: UserId,
  pub username: String,
  pub password_hash: String,
  pub role: Role,
  pub storage_limit_bytes: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub disabled_at: Option<DateTime<Utc>>,
}
impl User {
  // --- コンストラクタ系 ---
  /// ユーザを作成する共通関数
  fn new(
    username: String,
    password: String,
    role: Role,
    storage_limit_bytes: i64,
  ) -> AuthResult<Self> {
    // ユーザ名検証
    validate_username(&username)?;
    // パスワード検証
    validate_password(&password)?;

    // パスワードハッシュ化
    let password_hash = hash_password(&password)?;

    Ok(Self {
      id: UserId::new(),
      username,
      password_hash,
      role,
      storage_limit_bytes,
      created_at: Utc::now(),
      updated_at: Utc::now(),
      disabled_at: None,
    })
  }

  /// 通常ユーザを作成する
  pub fn new_user(username: String, password: String) -> AuthResult<Self> {
    Self::new(
      username,
      password,
      Role::User,
      10 * 1024 * 1024 * 1024, // 仮で10MB
    )
  }

  /// 管理者を作成する
  pub fn new_admin(username: String, password: String) -> AuthResult<Self> {
    Self::new(
      username,
      password,
      Role::Admin,
      20 * 1024 * 1024 * 1024, // 仮で20MB → いつかenv等にする
    )
  }

  /// disabledされているか
  pub fn is_disabled(&self) -> bool {
    self.disabled_at.is_some()
  }

  /// 管理者かどうか
  pub fn is_admin(&self) -> bool {
    self.role == Role::Admin
  }
}

/// RefreshToken型
#[derive(Debug, Clone)]
pub struct RefreshToken {
  pub id: RefreshTokenId,
  pub user_id: UserId,
  pub token_hash: String,
  pub user_agent: Option<String>,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
  /// トークンが有効か
  pub fn is_valid(&self) -> bool {
    self.revoked_at.is_none() && self.expires_at > Utc::now()
  }
}
