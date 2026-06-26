/*
backend/crates/infra/src/postgres/file_content_repository.rs
postgresのFileContentRepository実体を定義
*/

// 外部クレート
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::{FileContent, FileContentStatus};
use repository::{FileContentRepository, MimeStat, RepoError, RepoResult, UsageStats};

// 自クレート
// エラー型伝搬用
use crate::error::{InfraError, InfraResult};

/// postgreSQLのFileContentRepository実装
pub struct PgFileContentRepository {
  /// DBコネクションプール
  pool: PgPool,
}

impl PgFileContentRepository {
  /// コンストラクタ
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

// エラー型を伝搬させるため、
// 内部実装ブロックで分け、
// トレイト実装はすべて .map_err(RepoError::from) で委譲するだけ
#[async_trait]
impl FileContentRepository for PgFileContentRepository {
  /// NodeIdからファイルの中身を取得する
  async fn find_by_node_id(&self, node_id: &NodeId) -> RepoResult<Option<FileContent>> {
    self
      .find_by_node_id_impl(node_id)
      .await
      .map_err(RepoError::from)
  }

  /// 新規FileContentを作成する
  async fn create(&self, content: &FileContent) -> RepoResult<()> {
    self.create_impl(content).await.map_err(RepoError::from)
  }

  /// 既存FileContentを更新する
  async fn update(&self, content: &FileContent) -> RepoResult<()> {
    self.update_impl(content).await.map_err(RepoError::from)
  }

  /// 完全削除をする
  async fn hard_delete(&self, node_id: &NodeId) -> RepoResult<()> {
    self
      .hard_delete_impl(node_id)
      .await
      .map_err(RepoError::from)
  }

  async fn find_stored_filenames_by_node_ids(
    &self,
    node_ids: &[NodeId],
  ) -> RepoResult<Vec<String>> {
    self
      .find_stored_filenames_by_node_ids_impl(node_ids)
      .await
      .map_err(RepoError::from)
  }

  async fn get_usage_stats(&self, owner_user_id: &UserId) -> RepoResult<UsageStats> {
    self
      .get_usage_stats_impl(owner_user_id)
      .await
      .map_err(RepoError::from)
  }
}

impl PgFileContentRepository {
  /// NodeIdからファイルの中身を取得するfind_by_node_idの内部実装
  async fn find_by_node_id_impl(&self, node_id: &NodeId) -> InfraResult<Option<FileContent>> {
    // NodeIdが一致するFileContentの取得
    let row = sqlx::query!(
      r#"
      SELECT
        node_id,
        stored_filename,
        mime_type,
        size_bytes,
        status,
        created_at,
        updated_at
      FROM
        file_contents
      WHERE
        node_id = $1
      "#,
      node_id.as_uuid()
    )
    .fetch_optional(&self.pool)
    .await?;

    // 匿名構造体をFileContent型にして返す
    Ok(row.map(|r| FileContent {
      node_id: NodeId::from_uuid(r.node_id),
      stored_filename: r.stored_filename,
      mime_type: r.mime_type,
      size_bytes: r.size_bytes,
      status: if r.status == "active" {
        FileContentStatus::Active
      } else {
        FileContentStatus::Pending
      },
      created_at: r.created_at,
      updated_at: r.updated_at,
    }))
  }

  /// 新規FileContent行を作成するcreateの内部実装
  async fn create_impl(&self, content: &FileContent) -> InfraResult<()> {
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
      content.node_id.as_uuid(),
      content.stored_filename,
      content.mime_type,
      content.size_bytes,
      content.status.as_str(),
      content.created_at,
      content.updated_at,
    )
    .execute(&self.pool)
    .await?;

    // sqlx::query!はPgQueryResult
    // InfraResultとして返す
    Ok(())
  }

  /// 既存FileContentを更新するupdateの内部実装
  async fn update_impl(&self, content: &FileContent) -> InfraResult<()> {
    // 対象NodeIdのFileContentを更新
    // 確認のために更新件数を取得
    let affected = sqlx::query!(
      r#"
      UPDATE file_contents SET
        stored_filename = $2,
        mime_type = $3,
        size_bytes = $4,
        status = $5,
        updated_at = $6
      WHERE
        node_id = $1
      "#,
      content.node_id.as_uuid(),
      content.stored_filename,
      content.mime_type,
      content.size_bytes,
      content.status.as_str(),
      content.updated_at,
    )
    .execute(&self.pool)
    .await?
    .rows_affected();

    // 取得失敗したらNotFoundエラー
    if affected == 0 {
      return Err(InfraError::NotFound);
    }

    Ok(())
  }

  /// 完全削除をするhard_deleteの内部実装
  async fn hard_delete_impl(&self, node_id: &NodeId) -> InfraResult<()> {
    // 対象NodeIdのFileContentを削除
    sqlx::query!(
      r#"
      DELETE FROM
        file_contents
      WHERE
        node_id = $1
      "#,
      node_id.as_uuid()
    )
    .execute(&self.pool)
    .await?;

    // sqlx::query!はPgQueryResult
    // InfraResultとして返す
    Ok(())
  }

  async fn find_stored_filenames_by_node_ids_impl(
    &self,
    node_ids: &[NodeId],
  ) -> InfraResult<Vec<String>> {
    if node_ids.is_empty() {
      return Ok(vec![]);
    }
    let uuids: Vec<Uuid> = node_ids.iter().map(|id| *id.as_uuid()).collect();
    let rows = sqlx::query!(
      r#"
        SELECT stored_filename
        FROM file_contents
        WHERE node_id = ANY($1)
        "#,
      &uuids[..] as &[Uuid]
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.stored_filename).collect())
  }

  async fn get_usage_stats_impl(&self, owner_user_id: &UserId) -> InfraResult<UsageStats> {
    // 使用容量とファイル数を取得する。
    let totals = sqlx::query!(
      r#"
      SELECT
        COALESCE(SUM(fc.size_bytes), 0) as "total_bytes!: i64",
        COUNT(fc.node_id)::BIGINT       as "file_count!: i64"
      FROM nodes n
      JOIN file_contents fc ON fc.node_id = n.id
      WHERE n.owner_user_id = $1
        AND n.deleted_at IS NULL
        AND n.status = 'active'
        AND fc.status = 'active'
      "#,
      owner_user_id.as_uuid()
    )
    .fetch_one(&self.pool)
    .await?;

    // MIME Type ごとのファイル数を取得する。
    let mime_rows = sqlx::query!(
      r#"
      SELECT
        fc.mime_type as "mime_type!",
        COUNT(*)::BIGINT as "count!: i64"
      FROM nodes n
      JOIN file_contents fc ON fc.node_id = n.id
      WHERE n.owner_user_id = $1
        AND n.deleted_at IS NULL
        AND n.status = 'active'
        AND fc.status = 'active'
      GROUP BY fc.mime_type
      ORDER BY COUNT(*) DESC
      "#,
      owner_user_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(UsageStats {
      total_bytes: totals.total_bytes,
      file_count: totals.file_count,
      mime_stats: mime_rows
        .into_iter()
        .map(|r| MimeStat {
          mime_type: r.mime_type,
          count: r.count,
        })
        .collect(),
    })
  }
}
