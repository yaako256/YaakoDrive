/*
backend/crates/api/src/download_token.rs
ダウンロードトークン管理
MVPではサーバメモリ上で管理
*/
use identity::NodeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// ダウンロードトークンの有効期限
const TOKEN_TTL_SECS: u64 = 60;

#[derive(Clone)]
struct TokenEntry {
  node_id: NodeId,
  expires_at: Instant,
}

/// メモリ上のダウンロードトークンストア
/// サーバ再起動で失効する。MVP向け。
#[derive(Clone, Default)]
pub struct DownloadTokenStore {
  inner: Arc<Mutex<HashMap<String, TokenEntry>>>,
}

impl DownloadTokenStore {
  pub fn new() -> Self {
    Self::default()
  }

  /// トークンを発行してNodeIdに紐付ける
  pub fn issue(&self, node_id: NodeId) -> String {
    let token = Uuid::new_v4().to_string();
    let entry = TokenEntry {
      node_id,
      expires_at: Instant::now() + Duration::from_secs(TOKEN_TTL_SECS),
    };
    self.inner.lock().unwrap().insert(token.clone(), entry);
    token
  }

  /// トークンを検証してNodeIdを返す
  /// 検証後はトークンを削除する（使い捨て）
  pub fn consume(&self, token: &str) -> Option<NodeId> {
    let mut map = self.inner.lock().unwrap();
    let entry = map.remove(token)?;
    if entry.expires_at > Instant::now() {
      Some(entry.node_id)
    } else {
      None
    }
  }
}
