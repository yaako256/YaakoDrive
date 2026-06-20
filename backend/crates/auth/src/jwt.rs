/*
backend/crates/auth/src/jwt.rs
Jwt認証サービス構造体の定義
*/

// 外部クレート
// 時間型
use chrono::{Duration, Utc};
// jwt
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
// シリアライズ
use serde::{Deserialize, Serialize};

// 内部ライブラリ
// Id型
use identity::UserId;

// 自クレート
// エラー型
use crate::error::{AuthError, AuthResult};

/// AccessToken型
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
  pub sub: String, // UserId
  pub role: String,
  pub exp: i64,
  pub iat: i64,
}

/// JWTサービスの型
pub struct JwtService {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  access_token_expires_secs: u64,
}

impl JwtService {
  // インスタンス
  pub fn new(secret: &str, access_token_expires_secs: u64) -> Self {
    Self {
      encoding_key: EncodingKey::from_secret(secret.as_bytes()),
      decoding_key: DecodingKey::from_secret(secret.as_bytes()),
      access_token_expires_secs,
    }
  }

  // AccessTokenの生成
  pub fn generate_access_token(&self, user_id: &UserId, role: &str) -> AuthResult<String> {
    // 現在時刻取得
    let now = Utc::now();

    // 期限の計算(設定)
    let exp = now + Duration::seconds(self.access_token_expires_secs as i64);

    // AccessTokenClaimsの作成
    let claims = AccessTokenClaims {
      sub: user_id.to_string(),
      role: role.to_string(),
      exp: exp.timestamp(),
      iat: now.timestamp(),
    };

    // エンコードしてStringにする
    encode(&Header::default(), &claims, &self.encoding_key).map_err(|_| AuthError::InvalidToken)
  }

  // AccessTokenが有効かどうかの確認
  pub fn verify_access_token(&self, token: &str) -> AuthResult<AccessTokenClaims> {
    /// AccessTokenClaimsにでコードする
    decode::<AccessTokenClaims>(token, &self.decoding_key, &Validation::default())
      .map(|data| data.claims)
      .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::InvalidToken,
      })
  }
}
