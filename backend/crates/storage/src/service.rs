/*
backend/crates/storage/src/server.rs
StorageServiceのトレイトを定義
*/

// 標準ライブラリ
// アドレスを固定するための構造体
use std::pin::Pin;

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;
// バイト列処理用
use bytes::Bytes;
// ストリーム処理用
use futures_core::Stream;

// 自クレート
use crate::error::StorageResult;

/// ダウンロード時のファイルストリーム型
/// Pin: メモリ上の位置を固定
/// Box: ヒープメモリに迷子にならないように格納
/// Stream: 順番に流す(ストリーム)
/// StorageResult: エラーかバイト列か
/// Send: マルチスレッド間を安全に行き来できる
pub type ByteStream = Pin<Box<dyn Stream<Item = StorageResult<Bytes>> + Send>>;

/// save_temp の戻り値。ファイル名とサイズをまとめて返す
pub struct TempFile {
  /// tmp ディレクトリ内のファイル名
  pub filename: String,
  /// 保存されたバイト数
  pub size_bytes: u64,
}

#[async_trait]
pub trait StorageService: Send + Sync {
  /// バイト列を一時ファイルに保存する。TempFile を返す
  async fn save_temp(&self, data: Bytes, original_filename: &str) -> StorageResult<TempFile>;

  /// 一時ファイルを正式な場所へ移動する
  async fn promote(&self, temp_filename: &str, final_filename: &str) -> StorageResult<()>;

  /// 正式ファイルを削除する
  async fn delete(&self, filename: &str) -> StorageResult<()>;

  /// 一時ファイルを削除する（失敗は呼び出し元が無視してよい）
  async fn delete_temp(&self, temp_filename: &str) -> StorageResult<()>;

  /// ファイルを ByteStream として開く
  async fn open_stream(&self, filename: &str) -> StorageResult<ByteStream>;
}
