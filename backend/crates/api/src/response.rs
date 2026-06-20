/*
backend/crates/api/src/response.rs
レスポンス共通型
app全体で使いまわす
*/

// 外部クレート
// シリアライズ用
use serde::Serialize;

/// 共通レスポンス型
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
  pub data: Option<T>,
  pub error: Option<ApiError>,
}

/// レスポンスエラー型
#[derive(Serialize)]
pub struct ApiError {
  pub code: String,
  pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
  // Okレスポンス
  // 例: return ApiResponse::ok(data)
  pub fn ok(data: T) -> Self {
    Self {
      data: Some(data),
      error: None,
    }
  }
}

// エラーはrustがdata型を知る必要がない
// そのためTなしの別implで定義
impl ApiResponse<()> {
  /// errレスポンス
  /// 例: return ApiResponse::err("code","message")
  pub fn err(code: &str, message: &str) -> Self {
    Self {
      data: None,
      error: Some(ApiError {
        code: code.to_string(),
        message: message.to_string(),
      }),
    }
  }
}
