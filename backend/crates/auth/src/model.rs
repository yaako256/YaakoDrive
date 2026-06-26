/*
backend/crates/auth/src/model.rs
認証関連の型定義
*/

// 外部クレート
// 時間型用
use chrono::{DateTime, Duration, Utc};

// 内部ライブラリ
// Id型用
use identity::{RefreshTokenId, UserId};

use crate::AuthError;
// 自クレート
use crate::error::AuthResult;
use crate::password::hash_password;
use crate::token::{generate_refresh_token, hash_token};
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
  type Error = AuthError;

  // 文字列からRole型の取得
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    match s {
      "admin" => Ok(Role::Admin),
      "user" => Ok(Role::User),
      other => Err(AuthError::InvalidRole(other.to_string())),
    }
  }
}

/// ユーザの型定義
#[derive(Debug, Clone)]
pub struct User {
  id: UserId,
  username: String,
  password_hash: String,
  role: Role,
  storage_limit_bytes: i64,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  disabled_at: Option<DateTime<Utc>>,
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

  /// 匿名構造体等を復元するときとかに使う
  pub fn reconstitute(
    id: UserId,
    username: String,
    password_hash: String,
    role: Role,
    storage_limit_bytes: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    disabled_at: Option<DateTime<Utc>>,
  ) -> Self {
    Self {
      id,
      username,
      password_hash,
      role,
      storage_limit_bytes,
      created_at,
      updated_at,
      disabled_at,
    }
  }

  // ---- ゲッター関数 ----
  /// idのゲッター関数
  pub fn id(&self) -> &UserId {
    &self.id
  }
  /// usernameのゲッター関数
  pub fn username(&self) -> &String {
    &self.username
  }
  /// password_hashのゲッター関数
  pub fn password_hash(&self) -> &String {
    &self.password_hash
  }
  /// roleのゲッター関数
  pub fn role(&self) -> &Role {
    &self.role
  }
  /// storage_limit_bytesのゲッター関数
  pub fn storage_limit_bytes(&self) -> &i64 {
    &self.storage_limit_bytes
  }
  /// created_atのゲッター関数
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }
  /// updated_atのゲッター関数
  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
  /// disabled_atのゲッター関数
  pub fn disabled_at(&self) -> &Option<DateTime<Utc>> {
    &self.disabled_at
  }

  // ---- その他関数 ----
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
  id: RefreshTokenId,
  user_id: UserId,
  token_hash: String,
  user_agent: Option<String>,
  expires_at: DateTime<Utc>,
  created_at: DateTime<Utc>,
  revoked_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
  // --- コンストラクタ系 ---
  // 新規トークン
  pub fn new(user_id: UserId, user_agent: Option<String>, expires_secs: u64) -> (String, Self) {
    // 新規トークンの発行
    let raw_token = generate_refresh_token();

    // 現在時刻取得
    let now = Utc::now();
    // 失効時間を設定
    let expires_at = now + Duration::seconds(expires_secs as i64);

    // トークンのハッシュ
    let token_hash = hash_token(&raw_token);

    (
      raw_token,
      Self {
        id: RefreshTokenId::new(),
        user_id: user_id,
        token_hash,
        user_agent: user_agent,
        expires_at,
        created_at: now,
        revoked_at: None,
      },
    )
  }

  /// 匿名構造体等を復元するときとかに使う
  pub fn reconstitute(
    id: RefreshTokenId,
    user_id: UserId,
    token_hash: String,
    user_agent: Option<String>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
  ) -> Self {
    Self {
      id,
      user_id,
      token_hash,
      user_agent,
      expires_at,
      created_at,
      revoked_at,
    }
  }

  // ---- ゲッター関数 ----
  /// idのゲッター関数
  pub fn id(&self) -> &RefreshTokenId {
    &self.id
  }
  /// user_idのゲッター関数
  pub fn user_id(&self) -> &UserId {
    &self.user_id
  }
  /// token_hashのゲッター関数
  pub fn token_hash(&self) -> &String {
    &self.token_hash
  }
  /// user_agentのゲッター関数
  pub fn user_agent(&self) -> &Option<String> {
    &self.user_agent
  }
  /// expires_atのゲッター関数
  pub fn expires_at(&self) -> &DateTime<Utc> {
    &self.expires_at
  }
  /// created_atのゲッター関数
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }
  /// revoked_atのゲッター関数
  pub fn revoked_at(&self) -> &Option<DateTime<Utc>> {
    &self.revoked_at
  }

  // ---- その他関数 ----
  /// トークンが有効か
  pub fn is_valid(&self) -> bool {
    self.revoked_at.is_none() && self.expires_at > Utc::now()
  }
}
