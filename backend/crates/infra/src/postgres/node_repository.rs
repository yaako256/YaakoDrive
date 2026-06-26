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
use node::model::{Node, NodeRow};
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

  /// ゴミ箱のルート一覧を取得する。
  async fn list_trash_roots(&self, owner_user_id: &UserId) -> RepoResult<Vec<Node>> {
    self
      .list_trash_roots_impl(owner_user_id)
      .await
      .map_err(RepoError::from)
  }

  /// ゴミ箱内フォルダの子ノード一覧を取得する。
  async fn list_deleted_children(
    &self,
    owner_user_id: &UserId,
    parent_id: &NodeId,
  ) -> RepoResult<Vec<Node>> {
    self
      .list_deleted_children_impl(owner_user_id, parent_id)
      .await
      .map_err(RepoError::from)
  }

  /// 指定した名前の active ノードが同フォルダ内に存在するか確認する。
  /// 復元時の同名衝突チェックに使う。
  /// parent_id が None の場合はルート直下を確認する。
  async fn exists_active_with_name(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
    name: &str,
  ) -> RepoResult<bool> {
    self
      .exists_active_with_name_impl(owner_user_id, parent_id, name)
      .await
      .map_err(RepoError::from)
  }

  /// 指定ノードと配下の deleted_at を NULL に戻す(論理削除解除)。
  async fn restore_with_descendants(
    &self,
    id: &NodeId,
    updated_at: DateTime<Utc>,
  ) -> RepoResult<()> {
    self
      .restore_with_descendants_impl(id, updated_at)
      .await
      .map_err(RepoError::from)
  }

  /// 名前の部分一致検索(active かつ未削除のみ、大文字小文字を無視)。
  async fn search_by_name(&self, owner_user_id: &UserId, query: &str) -> RepoResult<Vec<Node>> {
    self
      .search_by_name_impl(owner_user_id, query)
      .await
      .map_err(RepoError::from)
  }

  /// 指定ノードと配下の NodeId を全件返す(自身を含む)。
  /// 物理削除前に削除対象 ID を収集するために使う。
  async fn collect_descendant_ids(&self, id: &NodeId) -> RepoResult<Vec<NodeId>> {
    self
      .collect_descendant_ids_impl(id)
      .await
      .map_err(RepoError::from)
  }

  /// 複数の NodeId を一括物理削除する。
  /// file_contents は ON DELETE CASCADE で連動削除される。
  async fn hard_delete_many(&self, ids: &[NodeId]) -> RepoResult<()> {
    self
      .hard_delete_many_impl(ids)
      .await
      .map_err(RepoError::from)
  }

  /// active フォルダ数を返す。Dashboard 集計に使う。
  async fn count_active_folders(&self, owner_user_id: &UserId) -> RepoResult<i64> {
    self
      .count_active_folders_impl(owner_user_id)
      .await
      .map_err(RepoError::from)
  }
}

impl PgNodeRepository {
  /// iDからNodeを取得するfind_by_idの内部実装
  async fn find_by_id_impl(&self, id: &NodeId) -> InfraResult<Option<Node>> {
    // sqlx::query! → コンパイル時にSQLを検証し、"匿名構造体（Record）" を生成する
    // Node列からNodeidが一致するものを持ってくる
    let row = sqlx::query_as!(
      NodeRow,
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

    // query_asで得たNodeRow型ををNode型に変換
    // ここで NodeError → InfraError に変換される
    let node = row.map(Node::try_from).transpose()?;

    // InfraResultで返す
    Ok(node)
  }

  // 親idから子のリストをVec<Node>で返すlist_children_implの内部実装
  async fn list_children_impl(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
  ) -> InfraResult<Vec<Node>> {
    // 匿名構造体が別構造体と考えられる結果、
    // matchが使えないのでif文でparent_id分岐をする
    // → query_as! により両方とも Vec<NodeRow> を返すため、matchで分岐できるようになった
    // PostgreSQLなら、SQL構文でSQLを1本にもできるらしい。
    let rows = match parent_id {
      Some(pid) => {
        sqlx::query_as!(
          NodeRow,
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
        .await?
      }
      None => {
        sqlx::query_as!(
          NodeRow,
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
        .await?
      }
    };

    // query_asで得たNodeRow型ををNode型に変換
    let node = rows
      .into_iter()
      .map(Node::try_from)
      .collect::<Result<Vec<_>, _>>()?;

    // InfraResultで返す
    Ok(node)
  }

  /// 新規ノードを作成するcreateの内部実装 $$$
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

  async fn list_trash_roots_impl(&self, owner_user_id: &UserId) -> InfraResult<Vec<Node>> {
    // 「直接ゴミ箱に入れたノード」を取得する。
    // 削除されているが、親が active（deleted_at IS NULL）か、
    // 親がいない（parent_id IS NULL）ノードが対象。
    let rows = sqlx::query_as!(
      NodeRow,
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
      FROM nodes
      WHERE
        owner_user_id = $1
        AND deleted_at IS NOT NULL
        AND (
        parent_id IS NULL
          OR EXISTS (
            SELECT 1 FROM nodes p
            WHERE p.id = nodes.parent_id
              AND p.deleted_at IS NULL
        )
      )
      ORDER BY deleted_at DESC
      "#,
      owner_user_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await?;

    // query_asで得たNodeRow型ををNode型に変換
    let node = rows
      .into_iter()
      .map(Node::try_from)
      .collect::<Result<Vec<_>, _>>()?;

    // InfraResultで返す
    Ok(node)
  }

  async fn list_deleted_children_impl(
    &self,
    owner_user_id: &UserId,
    parent_id: &NodeId,
  ) -> InfraResult<Vec<Node>> {
    let rows = sqlx::query_as!(
      NodeRow,
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
      FROM nodes
      WHERE
        owner_user_id = $1
        AND parent_id = $2
        AND deleted_at IS NOT NULL
      ORDER BY node_type DESC, name ASC
      "#,
      owner_user_id.as_uuid(),
      parent_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await?;

    // query_asで得たNodeRow型ををNode型に変換
    let node = rows
      .into_iter()
      .map(Node::try_from)
      .collect::<Result<Vec<_>, _>>()?;

    // InfraResultで返す
    Ok(node)
  }

  async fn exists_active_with_name_impl(
    &self,
    owner_user_id: &UserId,
    parent_id: Option<&NodeId>,
    name: &str,
  ) -> InfraResult<bool> {
    // IS NOT DISTINCT FROM を使うと NULL 同士も等値として扱える。
    // これで parent_id = NULL（ルート直下）でも正しく動く。
    let row = sqlx::query!(
      r#"
      SELECT EXISTS(
        SELECT 1 FROM nodes
        WHERE owner_user_id = $1
          AND parent_id IS NOT DISTINCT FROM $2
          AND name = $3
          AND deleted_at IS NULL
          AND status = 'active'
        ) as "exists!"
      "#,
      owner_user_id.as_uuid(),
      parent_id.map(|id| *id.as_uuid()),
      name
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(row.exists)
  }

  async fn restore_with_descendants_impl(
    &self,
    id: &NodeId,
    updated_at: DateTime<Utc>,
  ) -> InfraResult<()> {
    sqlx::query!(
      r#"
      WITH RECURSIVE descendants AS (
        SELECT id
        FROM nodes
        WHERE id = $1
        UNION ALL
        SELECT n.id FROM nodes n
        INNER JOIN descendants d ON n.parent_id = d.id
        )
      UPDATE nodes
      SET
        deleted_at = NULL,
        updated_at = $2
      WHERE id IN (SELECT id FROM descendants)
      "#,
      id.as_uuid(),
      updated_at,
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn search_by_name_impl(
    &self,
    owner_user_id: &UserId,
    query: &str,
  ) -> InfraResult<Vec<Node>> {
    // ILIKE で大文字小文字を無視した部分一致検索。
    // 結果は最大 100 件に制限する。
    let pattern = format!("%{}%", query);

    let rows = sqlx::query_as!(
      NodeRow,
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
      FROM nodes
      WHERE
        owner_user_id = $1
          AND deleted_at IS NULL
          AND status = 'active'
          AND name ILIKE $2
      ORDER BY node_type DESC, name ASC
      LIMIT 100
      "#,
      owner_user_id.as_uuid(),
      pattern
    )
    .fetch_all(&self.pool)
    .await?;

    // query_asで得たNodeRow型ををNode型に変換
    let node = rows
      .into_iter()
      .map(Node::try_from)
      .collect::<Result<Vec<_>, _>>()?;

    // InfraResultで返す
    Ok(node)
  }

  async fn collect_descendant_ids_impl(&self, id: &NodeId) -> InfraResult<Vec<NodeId>> {
    // 自身を含む全子孫の NodeId を収集する。
    let rows = sqlx::query!(
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
      SELECT id as "id!"
      FROM descendants
      "#,
      id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(|r| NodeId::from_uuid(r.id)).collect())
  }

  async fn hard_delete_many_impl(&self, ids: &[NodeId]) -> InfraResult<()> {
    if ids.is_empty() {
      return Ok(());
    }
    // NodeId を Uuid に変換して配列として渡す。
    // file_contents は ON DELETE CASCADE で自動削除される。
    let uuids: Vec<Uuid> = ids.iter().map(|id| *id.as_uuid()).collect();
    sqlx::query!(
      r#"
      DELETE
      FROM nodes
      WHERE id = ANY($1)
      "#,
      &uuids[..] as &[Uuid]
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn count_active_folders_impl(&self, owner_user_id: &UserId) -> InfraResult<i64> {
    let row = sqlx::query!(
      r#"
      SELECT COUNT(*) as "count!"
      FROM nodes
      WHERE owner_user_id = $1
        AND deleted_at IS NULL
        AND status = 'active'
        AND node_type = 'folder'
      "#,
      owner_user_id.as_uuid()
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(row.count)
  }
}
