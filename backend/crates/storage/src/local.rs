/*
backend/crates/storage/src/local.rs
StorageServiceの実体定義
ローカルに保存する場合の構造体を作成
*/

//use bytes::Bytes;
//use futures_core::Stream;
//use std::pin::Pin;

// 標準ライブラリ
use std::path::{Path, PathBuf};

// 外部クレート
// ログ
//use tracing::warn;
// UUID
use uuid::Uuid;
// 非同期処理用
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

// 自クレート
// エラー型
use crate::error::{StorageError, StorageResult};
// トレイト型とストリームの型エイリアス
use crate::service::{ByteStream, StorageService};

/// ローカルディスクへの保存実装
pub struct LocalStorageService {
  /// 正式ファイルの保存先
  files_dir: PathBuf,
  /// 一時ファイルの保存先
  temp_dir: PathBuf,
}

impl LocalStorageService {
  // コンストラクタ
  pub fn new(data_dir: &str) -> StorageResult<Self> {
    // パスの作成
    let base = PathBuf::from(data_dir);
    let files_dir = base.join("files");
    let temp_dir = base.join("tmp");

    // ディレクトリを作成する
    std::fs::create_dir_all(&files_dir)?;
    std::fs::create_dir_all(&temp_dir)?;

    Ok(Self {
      files_dir,
      temp_dir,
    })
  }

  // ファイル名をファイルパスにして取得
  fn files_path(&self, filename: &str) -> PathBuf {
    self.files_dir.join(filename)
  }

  // 一時ファイル名を一時ファイルパスにして取得
  fn temp_path(&self, filename: &str) -> PathBuf {
    self.temp_dir.join(filename)
  }
}

#[async_trait]
impl StorageService for LocalStorageService {
  async fn save_temp_stream(
    &self,
    stream: ByteStream,
    original_filename: &str,
  ) -> StorageResult<String> {
    // 拡張子を保持した一時ファイル名を生成
    // ファイル名から拡張子のみを抽出
    // 例: aaa.txt → "txt"
    //     README → ""
    let ext = Path::new(original_filename)
      .extension()
      .and_then(|e| e.to_str())
      .unwrap_or("");

    // 仮ファイル名を作成
    // 拡張子がNoneの時は"."を2重にしない
    let temp_filename = if ext.is_empty() {
      format!("{}.tmp", Uuid::new_v4())
    } else {
      format!("{}.{}.tmp", Uuid::new_v4(), ext)
    };

    // ファイル名を一時ファイル場所のパスにする
    let temp_path = self.temp_path(&temp_filename);

    // 一時ファイルを作成
    let mut file = tokio::fs::File::create(&temp_path).await?;

    // ストリームからバイト列を受け取ってファイルへ書き込む
    // ストリームからデータ（チャンク）を1つずつ取り出してループする
    // 次のデータがある（Some）限り、変数 chunk に代入して中身を実行する
    let mut pinned = stream;
    // ストリームの書き込み処理
    while let Some(chunk) = {
      // ここは条件式(パターンマッチング)の一部
      // ここの戻り値がSome(chunk)ならループ(真)
      pinned.next().await
    } {
      // ループ処理
      // リザルト型の取り外し
      let chunk = chunk?;
      // ファイル書き込み
      file.write_all(&chunk).await?;
    }

    // ファイルの書き込み切る
    // 安全にディスク処理をする
    file.flush().await?;

    // 一時ファイル名を返す
    Ok(temp_filename)
  }

  async fn promote_file(&self, temp_filename: &str, final_filename: &str) -> StorageResult<()> {
    // 一時ファイルのパスを取得
    let temp_path = self.temp_path(temp_filename);
    // 本番ファイルのパスを取得
    let final_path = self.files_path(final_filename);

    // 本番場所に移動・上書きをする(rename)
    // 同一ファイルシステム内ならrenameが原子的に動く
    tokio::fs::rename(&temp_path, &final_path).await?;

    Ok(())
  }

  async fn delete_file(&self, stored_filename: &str) -> StorageResult<()> {
    // ファイルパスを取得
    let path = self.files_path(stored_filename);

    // パスが存在することを確認
    if path.exists() {
      // ファイルを物理削除する
      tokio::fs::remove_file(path).await?;
    }

    Ok(())
  }

  async fn open_file_stream(&self, stored_filename: &str) -> StorageResult<ByteStream> {
    // ファイルパスの取得
    let path = self.files_path(stored_filename);

    // ファイルが存在しているかの確認
    if !path.exists() {
      return Err(StorageError::NotFound(stored_filename.to_string()));
    }

    // ファイルを開く
    let file = tokio::fs::File::open(path).await?;

    // ファイルをストリームで開く
    let stream = ReaderStream::new(file);

    // streamは通常のResult型なので
    // Error型をStorageError型に変換することで
    // StorageResult型にする
    let mapped = stream.map(|r| r.map_err(StorageError::Io));

    // ByteStream型として返す
    Ok(Box::pin(mapped))
  }

  async fn delete_temp_file(&self, temp_filename: &str) -> StorageResult<()> {
    // 一時ファイルパスの取得
    let path = self.temp_path(temp_filename);

    // パスが存在するかの確認
    if path.exists() {
      // 一時ファイルを物理削除
      tokio::fs::remove_file(path).await?;
    }

    Ok(())
  }
}
