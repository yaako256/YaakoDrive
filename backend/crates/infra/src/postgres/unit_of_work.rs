/*
backend/crates/infra/src/unit_of_work.rs
postgresのTransactionContext実体を定義
*/

// 外部クレート
// 非同期トレイト
use async_trait::async_trait;
// sqlx
use sqlx::{PgPool, Postgres, Transaction};

// 内部ライブラリ
// Node型とファイル型
use node::model::{FileContent, Node};
// トレイト型
use repository::{RepoError, RepoResult, TransactionContext, UnitOfWork};

// 自クレート
use crate::postgres::UNIQUE_VIOLATION;

/// PostgreSQL トランザクションのコンテキスト。
/// tx を所有したまま、各操作を直接 sqlx で実行する。
pub struct PgTransactionContext {
  tx: Transaction<'static, Postgres>,
}

#[async_trait]
impl TransactionContext for PgTransactionContext {
  async fn insert_node(&mut self, node: &Node) -> RepoResult<()> {
    // 新規Node行を作成
    sqlx::query!(
      r#"
      INSERT INTO nodes (
        id,
        owner_user_id,
        parent_id,
        name,
        node_type,
        status,
        deleted_at,
        created_at,
        updated_at
      )
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      "#,
      node.id().as_uuid(),
      node.owner_user_id().as_uuid(),
      node.parent_id().as_ref().map(|id| *id.as_uuid()),
      node.name(),
      node.node_type().as_str(),
      node.status().as_str(),
      node.deleted_at(),
      node.created_at(),
      node.updated_at(),
    )
    .execute(&mut *self.tx)
    .await
    .map_err(|e| {
      // UNIQUE制約違反を Conflict に変換する
      if let sqlx::Error::Database(ref db_err) = e {
        if db_err.code().as_deref() == Some(UNIQUE_VIOLATION) {
          return RepoError::Conflict("name already exists".to_string());
        }
      }
      RepoError::Database(e.to_string())
    })?;

    Ok(())
  }

  async fn update_node(&mut self, node: &Node) -> RepoResult<()> {
    // 対象idのNode情報を更新する
    // 確認のために更新件数を取得
    let affected = sqlx::query!(
      r#"
      UPDATE nodes SET
        parent_id  = $2,
        name       = $3,
        status     = $4,
        deleted_at = $5,
        updated_at = $6
      WHERE
        id = $1
      "#,
      node.id().as_uuid(),
      node.parent_id().as_ref().map(|id| *id.as_uuid()),
      node.name(),
      node.status().as_str(),
      node.deleted_at(),
      node.updated_at(),
    )
    .execute(&mut *self.tx)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?
    .rows_affected();

    // 取得失敗したらNotFoundエラー
    if affected == 0 {
      return Err(RepoError::NotFound);
    }

    Ok(())
  }

  async fn insert_file_content(&mut self, content: &FileContent) -> RepoResult<()> {
    // FileContentの新規行の作成
    sqlx::query!(
      r#"
      INSERT INTO file_contents (
        node_id,
        stored_filename,
        mime_type,
        size_bytes,
        status,
        created_at,
        updated_at
      )
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      "#,
      content.node_id().as_uuid(),
      content.stored_filename(),
      content.mime_type(),
      content.size_bytes(),
      content.status().as_str(),
      content.created_at(),
      content.updated_at(),
    )
    .execute(&mut *self.tx)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    // sqlx::query!はPgQueryResult
    // RepoResultとして返す
    Ok(())
  }

  async fn update_file_content(&mut self, content: &FileContent) -> RepoResult<()> {
    // 対象NodeIdのFileContentを更新
    // 確認のために更新件数を取得
    let affected = sqlx::query!(
      r#"
      UPDATE file_contents SET
        stored_filename = $2,
        mime_type       = $3,
        size_bytes      = $4,
        status          = $5,
        updated_at      = $6
      WHERE
        node_id = $1
      "#,
      content.node_id().as_uuid(),
      content.stored_filename(),
      content.mime_type(),
      content.size_bytes(),
      content.status().as_str(),
      content.updated_at(),
    )
    .execute(&mut *self.tx)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?
    .rows_affected();

    // 取得失敗したらNotFoundエラー
    if affected == 0 {
      return Err(RepoError::NotFound);
    }
    Ok(())
  }

  async fn commit(self: Box<Self>) -> RepoResult<()> {
    self
      .tx
      .commit()
      .await
      .map_err(|e| RepoError::Database(e.to_string()))
  }

  async fn rollback(self: Box<Self>) -> RepoResult<()> {
    self
      .tx
      .rollback()
      .await
      .map_err(|e| RepoError::Database(e.to_string()))
  }
}

pub struct PgUnitOfWork {
  pool: PgPool,
}

impl PgUnitOfWork {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
  async fn begin(&self) -> RepoResult<Box<dyn TransactionContext>> {
    let tx = self
      .pool
      .begin()
      .await
      .map_err(|e| RepoError::Database(e.to_string()))?;
    Ok(Box::new(PgTransactionContext { tx }))
  }
}
