/*
backend/crates/infra/src/postgres/node_repository.rs
postgresのNodeRepository実体を定義
*/

// 外部クレート
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::{Node, NodeStatus, NodeType};
use repository::{NodeRepository, RepoError, RepoResult};

// 自クレート
// エラー型伝搬用
use crate::error::{InfraError, InfraResult};

/// postgreSQLのNodeRepository実装
pub struct PgNodeRepository {
  /// DBコネクションプール
  pool: PgPool,
}

impl PgNodeRepository {
  /// コンストラクタ
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

// エラー型を伝搬させるため、
// 内部実装ブロックで分け、
// トレイト実装はすべて .map_err(RepoError::from) で委譲するだけ
#[async_trait]
impl NodeRepository for PgNodeRepository {
  /// iDからNodeを取得する
  async fn find_by_id(&self, id: &NodeId) -> RepoResult<Option<Node>> {
    self.find_by_id_impl(id).await.map_err(RepoError::from)
  }

  /// 親idから子のリストをVec<Node>で返す
  async fn list_children(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
  ) -> RepoResult<Vec<Node>> {
    self
      .list_children_impl(owner_user_id, parent_id)
      .await
      .map_err(RepoError::from)
  }

  /// 新規ノードを作成する
  async fn create(&self, node: &Node) -> RepoResult<()> {
    self.create_impl(node).await.map_err(RepoError::from)
  }

  /// Node情報の更新をする
  async fn update(&self, node: &Node) -> RepoResult<()> {
    self.update_impl(node).await.map_err(RepoError::from)
  }

  /// 論理削除をする
  async fn soft_delete_with_descendants(
    &self,
    id: &NodeId,
    deleted_at: DateTime<Utc>,
  ) -> RepoResult<()> {
    self
      .soft_delete_with_descendants_impl(id, deleted_at)
      .await
      .map_err(RepoError::from)
  }

  /// 物理削除をする
  async fn hard_delete(&self, id: &NodeId) -> RepoResult<()> {
    self.hard_delete_impl(id).await.map_err(RepoError::from)
  }

  /// NodeIdから祖先のリストをVec<NodeId>で取得する
  async fn find_ancestor_ids(&self, id: &NodeId) -> RepoResult<Vec<NodeId>> {
    self
      .find_ancestor_ids_impl(id)
      .await
      .map_err(RepoError::from)
  }
}

impl PgNodeRepository {
  /// iDからNodeを取得するfind_by_idの内部実装
  async fn find_by_id_impl(&self, id: &NodeId) -> InfraResult<Option<Node>> {
    // sqlx::query! → コンパイル時にSQLを検証し、"匿名構造体（Record）" を生成する
    // Node列からNodeidが一致するものを持ってくる
    let row = sqlx::query!(
      r#"
      SELECT 
        id, 
        owner_user_id, 
        parent_id, 
        name, 
        node_type, 
        status, 
        deleted_at, 
        created_at, 
        updated_at 
      FROM
        nodes 
      WHERE
        id = $1
      "#,
      id.as_uuid()
    )
    // fetch_one: 必ず1件ある前提。0件でエラー
    // fetch_optional: 0件か1件。存在しない場合はNone
    // fetch_all: 0件以上の複数件
    .fetch_optional(&self.pool)
    .await?;

    // 匿名構造体(record)をNode型に変換
    row
      .map(|r| {
        map_node_row(
          r.id,
          r.owner_user_id,
          r.parent_id,
          r.name,
          r.node_type,
          r.status,
          r.deleted_at,
          r.created_at,
          r.updated_at,
        )
      })
      .transpose()
  }

  // 親idから子のリストをVec<Node>で返すlist_children_implの内部実装
  async fn list_children_impl(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
  ) -> InfraResult<Vec<Node>> {
    // 匿名構造体が別構造体と考えられる結果、
    // matchが使えないのでif文でparent_id分岐をする
    if let Some(pid) = parent_id {
      let rows = sqlx::query!(
        r#"
        SELECT 
          id, 
          owner_user_id, 
          parent_id, 
          name, 
          node_type, 
          status, 
          deleted_at, 
          created_at, 
          updated_at
        FROM
          nodes
        WHERE
          owner_user_id = $1 
          AND parent_id = $2
          AND deleted_at IS NULL
          AND status = 'active'
        ORDER BY
          node_type DESC,
          name ASC
        "#,
        owner_user_id.as_uuid(),
        pid.as_uuid()
      )
      .fetch_all(&self.pool)
      .await?;

      // 匿名構造体(record)をNode型に変換
      return rows
        .into_iter()
        .map(|r| {
          map_node_row(
            r.id,
            r.owner_user_id,
            r.parent_id,
            r.name,
            r.node_type,
            r.status,
            r.deleted_at,
            r.created_at,
            r.updated_at,
          )
        })
        .collect();
    }

    let rows = sqlx::query!(
      r#"
      SELECT 
        id, 
        owner_user_id, 
        parent_id, 
        name, 
        node_type, 
        status, 
        deleted_at, 
        created_at, 
        updated_at
      FROM
        nodes
      WHERE
        owner_user_id = $1 
        AND parent_id IS NULL 
        AND deleted_at IS NULL 
        AND status = 'active' 
      ORDER BY
        node_type DESC,
        name ASC
      "#,
      owner_user_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await?;

    // 匿名構造体(record)をNode型に変換
    rows
      .into_iter()
      .map(|r| {
        map_node_row(
          r.id,
          r.owner_user_id,
          r.parent_id,
          r.name,
          r.node_type,
          r.status,
          r.deleted_at,
          r.created_at,
          r.updated_at,
        )
      })
      .collect()
  }

  /// 新規ノードを作成するcreateの内部実装
  async fn create_impl(&self, node: &Node) -> InfraResult<()> {
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
      node.id.as_uuid(),
      node.owner_user_id.as_uuid(),
      node.parent_id.as_ref().map(|id| *id.as_uuid()),
      node.name,
      node.node_type.as_str(),
      node.status.as_str(),
      node.deleted_at,
      node.created_at,
      node.updated_at,
    )
    .execute(&self.pool)
    .await
    .map_err(|e| {
      // UNIQUE制約違反を Conflict に変換する
      if let sqlx::Error::Database(ref db_err) = e {
        if db_err.code().as_deref() == Some("23505") {
          return InfraError::Conflict("name already exists".to_string());
        }
      }
      InfraError::Database(e)
    })?;

    Ok(())
  }

  /// Node情報の更新をするupdateの内部実装
  async fn update_impl(&self, node: &Node) -> InfraResult<()> {
    // 対象idのNode情報を更新する
    // 確認のために更新件数を取得
    let affected = sqlx::query!(
      r#"
      UPDATE nodes SET 
        parent_id = $2, 
        name = $3, 
        status = $4,
        deleted_at = $5,
        updated_at = $6
      WHERE
        id = $1
      "#,
      node.id.as_uuid(),
      node.parent_id.as_ref().map(|id| *id.as_uuid()),
      node.name,
      node.status.as_str(),
      node.deleted_at,
      node.updated_at,
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

  /// 論理削除をするsoft_delete_with_descendantsの内部実装
  async fn soft_delete_with_descendants_impl(
    &self,
    id: &NodeId,
    deleted_at: DateTime<Utc>,
  ) -> InfraResult<()> {
    // CTEで再帰的に子孫を取得してまとめてdeleted_atを設定する
    sqlx::query!(
      r#"
      WITH RECURSIVE descendants AS (
        SELECT id
        FROM nodes
        WHERE id = $1
        UNION ALL
        SELECT n.id
        FROM nodes n
        INNER JOIN descendants d ON n.parent_id = d.id
      )
      UPDATE nodes SET 
        deleted_at = $2, 
        updated_at = $2
      WHERE
        id IN (SELECT id FROM descendants)
        AND deleted_at IS NULL
      "#,
      id.as_uuid(),
      deleted_at,
    )
    .execute(&self.pool)
    .await?;

    // sqlx::query!はPgQueryResult
    // InfraResultとして返す
    Ok(())
  }

  /// 物理削除をするhard_deleteの内部実装
  async fn hard_delete_impl(&self, id: &NodeId) -> InfraResult<()> {
    // 対象idの物理削除をする
    sqlx::query!(
      r#"
      DELETE FROM
        nodes
      WHERE
        id = $1
      "#,
      id.as_uuid()
    )
    .execute(&self.pool)
    .await?;

    // sqlx::query!はPgQueryResult
    // InfraResultとして返す
    Ok(())
  }

  /// NodeIdから祖先のリストをVec<NodeId>で取得するfind_ancestor_idsの内部実装
  async fn find_ancestor_ids_impl(&self, id: &NodeId) -> InfraResult<Vec<NodeId>> {
    // 祖先リストを取得
    let rows = sqlx::query!(
      r#"
      WITH RECURSIVE ancestors AS (
        SELECT parent_id
        FROM nodes
        WHERE id = $1
        UNION ALL
        SELECT n.parent_id
        FROM nodes n
        INNER JOIN ancestors a ON n.id = a.parent_id
      )
      SELECT
        parent_id
      FROM
        ancestors 
      WHERE
        parent_id IS NOT NULL
      "#,
      id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(
      // 匿名構造体をNodeIdに直して返す
      rows
        .into_iter()
        .filter_map(|r| r.parent_id)
        .map(NodeId::from_uuid)
        .collect(),
    )
  }
}

/// 匿名構造体をNode型に変換する内部関数
fn map_node_row(
  id: Uuid,
  owner_user_id: Uuid,
  parent_id: Option<Uuid>,
  name: String,
  node_type: String,
  status: String,
  deleted_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
) -> InfraResult<Node> {
  // 文字列からEnum型へ変換
  // 失敗時はエラー伝搬
  let node_type = NodeType::try_from(node_type.as_str())?;
  let status = NodeStatus::try_from(status.as_str())?;

  Ok(Node {
    id: NodeId::from_uuid(id),
    owner_user_id: UserId::from_uuid(owner_user_id),
    parent_id: parent_id.map(NodeId::from_uuid),
    name,
    node_type,
    status,
    deleted_at,
    created_at,
    updated_at,
  })
}
