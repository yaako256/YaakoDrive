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

/// ストリームの型エイリアス
/// Pin: メモリ上の位置を固定
/// Box: ヒープメモリに迷子にならないように格納
/// Stream: 順番に流す(ストリーム)
/// StorageResult: エラーかバイト列か
/// Send: マルチスレッド間を安全に行き来できる
pub type ByteStream = Pin<Box<dyn Stream<Item = StorageResult<Bytes>> + Send>>;

#[async_trait]
pub trait StorageService: Send + Sync {
  /// ストリームを一時ファイルに保存する
  /// 戻り値は一時ファイル名
  async fn save_temp_stream(
    &self,
    stream: ByteStream,
    original_filename: &str,
  ) -> StorageResult<String>;

  /// 一時ファイルを正式な場所へ移動する
  async fn promote_file(&self, temp_filename: &str, final_filename: &str) -> StorageResult<()>;

  /// ファイルを削除する
  async fn delete_file(&self, stored_filename: &str) -> StorageResult<()>;

  /// ファイルをストリームとして開く
  async fn open_file_stream(&self, stored_filename: &str) -> StorageResult<ByteStream>;

  /// 一時ファイルを削除する（クリーンアップ用）
  async fn delete_temp_file(&self, temp_filename: &str) -> StorageResult<()>;
}
