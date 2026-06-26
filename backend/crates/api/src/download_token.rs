/*
backend/crates/api/src/download_token.rs
ダウンロードトークン管理
MVPではサーバメモリ上で管理
*/
// api/src/download_token.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use identity::NodeId;
use uuid::Uuid;

const TTL_SECS: u64 = 60;

struct Entry {
  node_id: NodeId,
  expires_at: Instant,
}

/// メモリ上のダウンロードトークンストア。
/// MVP では単一インスタンス前提のためサーバ再起動で失効する。
#[derive(Clone, Default)]
pub struct DownloadTokenStore {
  inner: Arc<Mutex<HashMap<String, Entry>>>,
}

impl DownloadTokenStore {
  pub fn new() -> Self {
    Self::default()
  }

  /// NodeId に紐付いたトークンを発行する
  pub fn issue(&self, node_id: NodeId) -> String {
    let token = Uuid::new_v4().to_string();
    self.inner.lock().unwrap().insert(
      token.clone(),
      Entry {
        node_id,
        expires_at: Instant::now() + Duration::from_secs(TTL_SECS),
      },
    );
    token
  }

  /// トークンを検証して NodeId を返す（使い捨て）
  pub fn consume(&self, token: &str) -> Option<NodeId> {
    let entry = self.inner.lock().unwrap().remove(token)?;
    (entry.expires_at > Instant::now()).then_some(entry.node_id)
  }
}
