/*
backend/crates/app/src/usecase/node/create_folder.rs
新規フォルダを作成するユースケース
*/

// 外部クレート
use chrono::Utc;

// 内部ライブラリ
use identity::{NodeId, UserId};
use node::model::{Node, NodeStatus, NodeType};
use repository::NodeRepository;

// 自クレート
use crate::error::{AppError, AppResult};
use crate::usecase::node::validate_name;

pub struct CreateFolderInput {
  pub owner_user_id: UserId,
  /// None はルート直下
  pub parent_id: Option<NodeId>,
  pub name: String,
}

/// 新規フォルダ作成のユースケース構造体
pub struct CreateFolderUseCase<'a> {
  node_repo: &'a dyn NodeRepository,
}

impl<'a> CreateFolderUseCase<'a> {
  pub fn new(node_repo: &'a dyn NodeRepository) -> Self {
    Self { node_repo }
  }

  /// フォルダ作成の実行
  pub async fn execute(&self, input: CreateFolderInput) -> AppResult<Node> {
    // 名前の検証
    validate_name(&input.name)?;

    // 親フォルダの存在確認（指定がある場合）
    if let Some(ref parent_id) = input.parent_id {
      let parent = self
        .node_repo
        .find_by_id(parent_id)
        .await?
        .ok_or(AppError::NotFound("parent folder not found".to_string()))?;

      // 他ユーザのフォルダ配下には作れない
      if parent.owner_user_id != input.owner_user_id {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }

      // 削除済みフォルダ配下には作れない
      if parent.is_deleted() {
        return Err(AppError::NotFound("parent folder not found".to_string()));
      }

      // フォルダでないと配下に作れない
      if !parent.is_folder() {
        return Err(AppError::InvalidInput("parent is not a folder".to_string()));
      }
    }

    // 現在時刻の取得
    let now = Utc::now();

    // Node型を作成
    let node = Node {
      id: NodeId::new(),
      owner_user_id: input.owner_user_id,
      parent_id: input.parent_id,
      name: input.name,
      node_type: NodeType::Folder,
      status: NodeStatus::Active,
      deleted_at: None,
      created_at: now,
      updated_at: now,
    };

    // 実際に作成
    self.node_repo.create(&node).await.map_err(|e| {
      // UNIQUE制約違反 → 同名フォルダが存在する
      match e {
        repository::RepoError::Conflict(_) => {
          AppError::AlreadyExists("same name already exists".to_string())
        }
        other => AppError::from(other),
      }
    })?;

    Ok(node)
  }
}
