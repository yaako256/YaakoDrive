pub mod error;
pub mod local;
pub mod service;

// 再エクスポート
pub use error::StorageError;
pub use service::StorageService;
