/*
backend/crates/storage/src/local.rs
StorageServiceの実体定義
ローカルに保存する場合の構造体を作成
*/

// 標準ライブラリ
use std::path::PathBuf;

// 外部クレート
// ログ
use tracing::warn;
// UUID
use uuid::Uuid;
// バイトを効率的に処理する
use bytes::Bytes;
// 非同期処理用
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio_util::io::ReaderStream;

// 自クレート
// エラー型
use crate::error::{StorageError, StorageResult};
// トレイト型とストリームの型エイリアス
use crate::service::{ByteStream, StorageService, TempFile};

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

  fn temp_filename(original: &str) -> String {
    // 拡張子を保持した一時ファイル名を生成
    // ファイル名から拡張子のみを抽出
    // 例: aaa.txt → "txt"
    //     README → ""
    let ext = std::path::Path::new(original)
      .extension()
      .and_then(|e| e.to_str())
      .unwrap_or("");

    // 仮ファイル名を作成
    // 拡張子がNoneの時は"."を2重にしない
    if ext.is_empty() {
      format!("{}.tmp", Uuid::new_v4())
    } else {
      format!("{}.{}.tmp", Uuid::new_v4(), ext)
    }
  }
}

#[async_trait]
impl StorageService for LocalStorageService {
  async fn save_temp(&self, data: Bytes, original_filename: &str) -> StorageResult<TempFile> {
    let filename = Self::temp_filename(original_filename);
    let path = self.temp_dir.join(&filename);
    let size_bytes = data.len() as u64;
    tokio::fs::write(&path, &data).await?;
    Ok(TempFile {
      filename,
      size_bytes,
    })
  }

  async fn promote(&self, temp_filename: &str, final_filename: &str) -> StorageResult<()> {
    let src = self.temp_dir.join(temp_filename);
    let dst = self.files_dir.join(final_filename);
    // 同一ファイルシステム内なら rename は原子的に動く
    tokio::fs::rename(&src, &dst).await?;
    Ok(())
  }

  async fn delete(&self, filename: &str) -> StorageResult<()> {
    let path = self.files_dir.join(filename);
    match tokio::fs::remove_file(&path).await {
      Ok(()) => Ok(()),
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
        warn!("delete: ファイルが見つからない: {}", filename);
        Ok(())
      }
      Err(e) => Err(StorageError::Io(e)),
    }
  }

  async fn delete_temp(&self, temp_filename: &str) -> StorageResult<()> {
    let path = self.temp_dir.join(temp_filename);
    match tokio::fs::remove_file(&path).await {
      Ok(()) => Ok(()),
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
      Err(e) => Err(StorageError::Io(e)),
    }
  }

  async fn open_stream(&self, filename: &str) -> StorageResult<ByteStream> {
    let path = self.files_dir.join(filename);
    let file = tokio::fs::File::open(&path).await.map_err(|e| {
      if e.kind() == std::io::ErrorKind::NotFound {
        StorageError::NotFound(filename.to_string())
      } else {
        StorageError::Io(e)
      }
    })?;
    // ReaderStream を StorageError へ変換してピン留め
    let stream = ReaderStream::new(file).map(|r| r.map_err(StorageError::Io));
    Ok(Box::pin(stream))
  }
}
