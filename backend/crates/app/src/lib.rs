/*
backend/crates/app/src/lib.rs
app はHTTPもsqlxも知らない。
UserRepository Traitだけに依存する
*/

mod error;
pub mod usecase;

// 再エクスポート
pub use error::AppError;
