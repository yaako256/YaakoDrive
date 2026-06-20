# YaakoDrive設計書案
このファイルは、ブレストの次段階、設計書の前段階のものである。  
ちょっと詳細に、設計書のメモのようなことを書こうと思う。
最終的に設計書は
全体/フロントエンド側/バックエンド側
の3ファイルに分けて作る

仮で作っている途中であるため、内容は飛び飛びである。


Rustはトレイトを活用し、依存関係逆転を意識した設計とする。
アーキテクチャはレイヤードアーキテクチャをベースに、repositoryクレートでの依存関係逆転によりDDDのポート＆アダプタに近い構成とする。


# メモ
## アップロード失敗時の処理方針
一時ファイルへ保存後、DB登録成功後に正式配置する方式を採用する。
DB登録に失敗した場合は一時ファイルを削除する。
これにより、DBと実ファイルの不整合を防ぐ。

## 削除
フォルダを削除したら配下もすべて削除するようにする。


# YaakoDrive設計書

# 概要
## 本プロジェクトについて
YaakoDriveは、個人(自分&友人&家族)用のクラウドストレージシステムである。

プロジェクト名は、個人用であることをわかりやすくするため、
Yaako(自分の活動名で個人用である意) + Drive(クラウドストレージの意)
とし、プロジェクト内容がわかりやすい名前とする。


# 全体構成について
```
YaakoDrive
├── frontend # ユーザ側(React)
└── backend # サーバ側(Rust)
```
フロントエンドとバックエンドはプロジェクトもDockerコンテナも分け、完全に分離を行い、個人用とはいえセキュリティ面で強い仕組みとする。



# RustWorkspace構成
rustのworkspaceを採用する。
本プロジェクトではいつも使っていたsharedクレートをなくし、error型を分散、interfaceやトレイトを意識しようと思う。
構成次のようにする。
```
crates/
├── server     # 起動処理・DIの組み立て(エントリポイント)
├── app        # ユースケース
├── api        # axum。フロントエンドとの通信
├── auth       # ユーザ・認証関連
├── identity   # Idなどの構造体定義(依存関係的にほしい)
├── node       # ファイル・フォルダ
├── storage    # 実ファイル保存(StorageServiceトレイト + ローカル実装)
├── repository # Repository trait定義
├── infra      # PostgreSQL実装
└── config     # 設定
```

## serverクレート
エントリポイント。起動処理とDI(依存性注入)の組み立てを行う。
全クレートに依存することが許される唯一のクレート。
ここでRepository具体実装(infra)とユースケース(app)を組み合わせる。
```
main()
DIの組み立て
サーバ起動
```

## appクレート
ユースケース専用。起動処理は持たない。
ユーザが何をしたいかを表現する層。
HTTPやDBには依存しない。
```
UploadFileUseCase
MoveNodeUseCase
DeleteNodeUseCase
LoginUseCase
RefreshTokenUseCase
CreateFolderUseCase
```
など。

## api
axum等。HTTP層のみ担当。ユースケースは持たない。
```
POST /login
POST /upload
GET /nodes
```
など。

## auth
認証専用
```
JwtService
PasswordHasher
TokenClaims
```
など。

## identity
```
UserId
NodeId
```

## node
Driveのコア
```
Node
NodeType
FileContent
```
など。

## storage
物理ファイル管理。
StorageServiceトレイトを定義し、ローカル実装(LocalStorageService)も同クレートに置く。
トレイト化により、テスト時のモック差し替えや将来の実装変更に対応できる。
```rust
pub trait StorageService: Send + Sync {
    async fn save_temp_file(&self, data: &[u8], extension: Option<&str>) -> StorageResult<String>;
    async fn promote_file(&self, temp_filename: &str, final_filename: &str) -> StorageResult<()>;
    async fn delete_file(&self, stored_filename: &str) -> StorageResult<()>;
    async fn open_file(&self, stored_filename: &str) -> StorageResult<Bytes>;
}
```
実装: `LocalStorageService`

## repository
Repository Trait。ここにはトレイトだけ置く。
このクレートにより、SQL関連の依存関係逆転が可能になる。
モックも実装できる。
```
UserRepository
NodeRepository
FileContentRepository
RefreshTokenRepository
```
など。

## infra
trait実装
```
PostgresUserRepository
PostgresNodeRepository
PostgresFileContentRepository
PostgresRefreshTokenRepository
```
など。

## config
設定。tomlから設定を読み込む。

## 依存関係案
```
server     -> api, app, infra, config  # 全部に依存してよい唯一のクレート
api        -> app
app        -> auth, node, storage, repository
infra      -> repository, auth, node
repository -> auth, node, identity
auth       -> identity
node       -> identity
storage    -> (外部crateのみ)
identity   -> (外部crateのみ)
config     -> (外部crateのみ)
```

# 各クレートの中身について


## auth
ユーザ認証関連

### 構造体定義
```rust
#[derive(Debug, Clone)]
pub enum Role {
    User,
    Admin,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub storage_limit_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub id: RefreshTokenId,
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
```


## identity
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefreshTokenId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub Uuid);
```
このクレートのおかげで、
```
auth -> identity
node -> identity
repository -> identity
```
になり、依存が綺麗

## node
Driveのコアドメイン。
ファイルおよびフォルダをNodeとして扱う。

### 管理対象
- Node
- NodeType
- NodeId

NodeType
- File
- Folder

### 責務
- ファイル・フォルダの論理表現
- 名前変更ルール
- ノード移動ルール
- ノード削除ルール
- 階層構造ルール

### 管理しないもの
- PostgreSQL
- SQL
- UUIDファイル保存
- JWT認証
- HTTP通信
- ノード循環防止 ← MoveNodeUseCase(appクレート)が担当する

### parent_idのセンチネル値について
`parent_id`にNULLは使用しない。
ルートノードの`parent_id`には固定のセンチネルUUID `00000000-0000-0000-0000-000000000000` を使用する。
これにより、`UNIQUE(owner_user_id, parent_id, name)`制約がNULLの扱いを気にせず正しく機能する。
センチネル値はidentityクレートで定数として定義する。
```rust
// identity クレート
pub const ROOT_PARENT_ID: NodeId = NodeId(uuid!("00000000-0000-0000-0000-000000000000"));
```

### 構造体定義
```rust
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub owner_user_id: UserId,
    pub parent_id: NodeId,   // ルートノードはROOT_PARENT_ID(センチネル値)
    pub name: String,
    pub node_type: NodeType,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone)]
pub enum NodeType {
    File,
    Folder,
}

#[derive(Debug, Clone)]
pub struct FileContent {
    pub node_id: NodeId,
    pub stored_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```




## repository
永続化層のRepository Trait定義を置くクレート。
このクレートにより、アプリケーション層はPostgreSQL実装に直接依存せず、
抽象化されたインターフェースだけを通じてデータ操作できる。
つまり依存関係逆転ができる。モックも実装できる。

| トレイト名 | 責務 |
| UserRepository | ユーザ情報のテーブルを管理 |
| NodeRepository | Nodeのテーブルを管理 |
| FileContentRepository | ファイル実体メタデータのテーブルを管理 |
| RefreshTokenRepository | RefreshTokenのテーブルを管理 |

### エラー型について
repositoryクレートでは次のようなエラー型を作る
```rust
// Result型
pub type RepoResult<T> = Result<T, RepoError>;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    // DBアクセス失敗など
    #[error("database error: {0}")]
    Database(String),

    // 期待したデータが見つからない場合
    #[error("not found")]
    NotFound,

    // 同名ファイルなどの重複
    #[error("already exists")]
    AlreadyExists,

    // 権限不足など
    #[error("forbidden")]
    Forbidden,
}
```

### 使用する型
- authクレートのUser, RefreshToken, Role
- nodeクレートのNode, NodeType, FileContent
- identityクレートのUserId, NodeId, RefreshTokenId

### トレイト型
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// ユーザを新規作成する。
    /// 管理者によるアカウント発行時に使う。
    async fn create(&self, user: &User) -> RepoResult<()>;

    /// ユーザIDからユーザを取得する。
    async fn find_by_id(&self, user_id: UserId) -> RepoResult<Option<User>>;

    /// ユーザ名からユーザを取得する。
    /// ログイン時に使う。
    async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>>;

    /// ユーザ一覧を取得する。
    /// 管理者のDashboardで使う。
    async fn find_all(&self) -> RepoResult<Vec<User>>;

    /// ユーザ情報を更新する。
    /// ストレージ上限変更などで使う。
    async fn update(&self, user: &User) -> RepoResult<()>;
}

#[async_trait]
pub trait NodeRepository: Send + Sync {
    /// ノードを新規作成する。
    /// ファイルでもフォルダでも使う。
    async fn create(&self, node: &Node) -> RepoResult<()>;

    /// ノード情報を更新する。
    /// 名前変更、親フォルダ移動、削除日時更新などで使う。
    async fn update(&self, node: &Node) -> RepoResult<()>;

    /// ノードIDからノードを取得する。
    async fn find_by_id(&self, node_id: NodeId) -> RepoResult<Option<Node>>;

    /// 指定したユーザのルートノードを取得する。
    /// MyDrive の起点を取得するときに使う。
    /// ルートノードは parent_id == ROOT_PARENT_ID(センチネル値) で判定する。
    async fn find_root_by_owner(&self, owner_user_id: UserId) -> RepoResult<Option<Node>>;

    /// 指定した親ノードの直下にある子ノード一覧を取得する。
    /// フォルダの中身表示で使う。
    async fn find_children(&self, owner_user_id: UserId, parent_id: NodeId) -> RepoResult<Vec<Node>>;

    /// 同じ親フォルダ内に同名ノードがあるか確認する。
    /// アップロードやリネーム時の重複判定で使う。
    async fn exists_same_name(
        &self,
        owner_user_id: UserId,
        parent_id: NodeId,
        name: &str,
        exclude_node_id: Option<NodeId>,
    ) -> RepoResult<bool>;

    /// ユーザ内の削除済みノード一覧を取得する。
    /// ゴミ箱画面で使う。
    async fn find_deleted_by_owner(&self, owner_user_id: UserId) -> RepoResult<Vec<Node>>;

    /// 名前検索を行う。
    /// ファイル検索機能で使う。
    async fn search_by_name(&self, owner_user_id: UserId, keyword: &str) -> RepoResult<Vec<Node>>;

    /// 指定ユーザ配下のノード数を取得する。
    /// Dashboard の総ファイル数・総フォルダ数の集計に使う。
    async fn count_by_owner(&self, owner_user_id: UserId) -> RepoResult<i64>;

    /// 指定ユーザ配下の削除済みノード数を取得する。
    /// ゴミ箱件数の表示で使う。
    async fn count_deleted_by_owner(&self, owner_user_id: UserId) -> RepoResult<i64>;

    /// ノードを物理削除する。
    /// ゴミ箱保管期間を過ぎたデータの最終削除に使う。
    async fn hard_delete(&self, node_id: NodeId) -> RepoResult<()>;
}

#[async_trait]
pub trait FileContentRepository: Send + Sync {
    /// ファイル実体情報を新規作成する。
    /// アップロード完了後に保存する。
    async fn create(&self, content: &FileContent) -> RepoResult<()>;

    /// node_id からファイル実体情報を取得する。
    /// ダウンロードや詳細表示で使う。
    async fn find_by_node_id(&self, node_id: NodeId) -> RepoResult<Option<FileContent>>;

    /// 保存ファイル名からファイル実体情報を取得する。
    /// 実ファイルの参照時に使う。
    async fn find_by_stored_filename(&self, stored_filename: &str) -> RepoResult<Option<FileContent>>;

    /// ファイル実体情報を更新する。
    /// 将来の再保存やメタデータ更新に備える。
    async fn update(&self, content: &FileContent) -> RepoResult<()>;

    /// node_id に紐づくファイル実体情報を削除する。
    /// ノード削除後の後片付けで使う。
    async fn delete_by_node_id(&self, node_id: NodeId) -> RepoResult<()>;

    /// 指定ユーザの総使用容量を取得する。
    /// Dashboard の使用容量表示で使う。
    async fn sum_size_by_owner(&self, owner_user_id: UserId) -> RepoResult<i64>;

    /// 指定ユーザの MIME Type ごとの件数を取得する。
    /// Dashboard の「ファイル種類ごとの件数」で使う。
    async fn count_grouped_by_mime_type(&self, owner_user_id: UserId) -> RepoResult<Vec<(String, i64)>>;
}

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    /// Refresh Token を登録する。
    /// ログイン成功時に保存する。
    async fn create(&self, token: &RefreshToken) -> RepoResult<()>;

    /// Token ID から Refresh Token を取得する。
    /// JWT の jti などを使う想定。
    async fn find_by_id(&self, token_id: RefreshTokenId) -> RepoResult<Option<RefreshToken>>;

    /// ユーザIDに紐づく有効な Refresh Token 一覧を取得する。
    /// 端末一覧やセッション管理で使える。
    async fn find_active_by_user(&self, user_id: UserId) -> RepoResult<Vec<RefreshToken>>;

    /// Refresh Token を失効させる。
    /// ログアウト時に使う。
    async fn revoke(&self, token_id: RefreshTokenId, revoked_at: DateTime<Utc>) -> RepoResult<()>;

    /// 指定ユーザの Refresh Token をすべて失効させる。
    /// アカウント停止や全端末ログアウトで使う。
    async fn revoke_all_by_user(&self, user_id: UserId, revoked_at: DateTime<Utc>) -> RepoResult<()>;

    /// 期限切れの Refresh Token を削除する。
    /// 定期バッチで使う。
    async fn delete_expired_before(&self, before: DateTime<Utc>) -> RepoResult<u64>;
}
```



---
# api と app の責務分離

## 設計方針
本システムでは、
* api = HTTP層
* app = アプリケーション層

として責務を明確に分離する。

apiクレートはHTTP通信のみを扱い、
ユースケースは持たない。

appクレートはユースケースを実装し、
HTTPやAxumには依存しない。

---
## apiクレートの責務
apiクレートはフロントエンドとの通信を担当する。

### 主な責務
* Routing
* Request解析
* Response生成
* JWT認証ミドルウェア
* HTTP StatusCode変換
* Multipart受信

### 例
```http
POST /login
POST /refresh
POST /upload
GET /nodes
DELETE /nodes/{id}
```

### 扱うもの
```rust
Json<T>
Multipart
HeaderMap
Cookie
StatusCode
```

### 扱わないもの
* DB操作
* ファイル保存処理
* 業務ルール
* ユースケース

---
## appクレートの責務
appクレートはアプリケーションのユースケースを担当する。
ユーザが何をしたいかを表現する層である。

### 主な責務
* ログイン
* トークン更新
* ファイルアップロード
* ファイル移動
* ファイル削除
* フォルダ作成

などのユースケース実装。

---
### 例
```rust
LoginUseCase
RefreshTokenUseCase
UploadFileUseCase
MoveNodeUseCase
DeleteNodeUseCase
CreateFolderUseCase
```

---
### appクレートが利用するもの
```rust
NodeRepository
UserRepository
FileContentRepository
RefreshTokenRepository

StorageService

JwtService
PasswordHasher
```

---
### appクレートが知らないもの
```rust
axum
HTTP
StatusCode
Multipart
PostgreSQL
SQL
```

---
## 呼び出しイメージ

```text
Frontend
 ↓
api
 ↓
app
 ↓
Repository Trait
 ↓
infra(PostgreSQL)
```

---
## UploadFile の流れ

一時ファイル方式を採用する。
DBへの登録成功を確認してから正式配置することで、ファイルとDBの不整合を防ぐ。

```text
POST /upload
 ↓
api::upload_handler
 ↓
UploadFileUseCase
 ↓
storage.save_temp_file()     # 一時領域に保存
 ↓
node_repository.create()     # DB登録(失敗時は一時ファイル削除して終了)
 ↓
file_content_repository.create()
 ↓
storage.promote_file()       # 正式配置
 ↓
201 Created
```

---
## MoveNode の流れ

循環防止チェックをユースケース層(MoveNodeUseCase)で行う。
移動先フォルダが移動元の子孫でないかを、parent_idを辿ることで確認する。

```text
PATCH /nodes/{id}/move
 ↓
api::move_node_handler
 ↓
MoveNodeUseCase
 ↓
【循環チェック】
移動先から親を辿り、移動元IDが出てきたらエラー
 ↓
node_repository.update()     # parent_idを更新
 ↓
200 OK
```

---
## Login の流れ

```text
POST /login
 ↓
api::login_handler
 ↓
LoginUseCase
 ↓
user_repository.find_by_username()
 ↓
password_verify()
 ↓
jwt_generate()
 ↓
Json<LoginResponse>
```



# DB制約

## users
```sql
UNIQUE(username)
```

## nodes
通常のUNIQUE制約ではなく、論理削除(deleted_at)との整合性を保つためPartial Indexを使用する。
削除済みノードと同名のノードを同フォルダに作成できるようにするためである。

```sql
-- アクティブなノードに対してのみ同名チェックを行う
CREATE UNIQUE INDEX nodes_active_unique
  ON nodes(owner_user_id, parent_id, name)
  WHERE deleted_at IS NULL;
```

また、parent_idにNULLは使用しない。
ルートノードの parent_id にはセンチネルUUID `00000000-0000-0000-0000-000000000000` を使用する。
NULLを避けることで、UNIQUE制約がNULL同士の比較で意図せず通過する問題を防ぐ。

## file_contents
```sql
UNIQUE(node_id)
UNIQUE(stored_filename)
```

## refresh_tokens
```sql
UNIQUE(token_hash)
```


---
# 認証フロー設計

## トークンの保存場所
Access Token・Refresh Token ともに HttpOnly Cookie で管理する。
JavaScriptからは読めないため、XSS経由のトークン盗難を防ぐ。

## 有効期限
有効期限は config から読み込む。以下は参考値。

| トークン | 参考値 |
|----------|--------|
| Access Token | 15分 |
| Refresh Token | 30日 |

## Cookie設定

| 属性 | 値 | 理由 |
|------|----|------|
| HttpOnly | true | JS から読めなくする |
| Secure | true | HTTPS のみ送信 |
| SameSite | Strict | CSRF対策 |
| Path(Access Token) | `/api` | 通常のAPIリクエスト全体に載せる |
| Path(Refresh Token) | `/api/auth/refresh` | リフレッシュ専用エンドポイントにのみ送信する |

Refresh Token の Path を `/api/auth/refresh` に絞ることで、
アップロードや一覧取得などの通常リクエストにRefresh Tokenが載らなくなる。
万が一の漏洩リスクを最小化するための設計。

## トークン再発行の流れ
Access Token が切れた場合、フロントエンドは 401 を受け取ったタイミングで
`POST /api/auth/refresh` を呼び、新しい Access Token を取得する。
フロントエンドはトークンの中身を意識せず、Cookie は自動送信される。

```text
通常リクエスト → 401 Unauthorized
 ↓
POST /api/auth/refresh  (Refresh Token Cookie が自動送信される)
 ↓
RefreshTokenUseCase
 ↓
Refresh Token 検証 (有効期限・revoked_at・user_agent)
 ↓
新しい Access Token を Cookie にセット
 ↓
元のリクエストをリトライ
```

## User-Agentの記録
ログイン時に HTTP ヘッダーの `User-Agent` を取得し、
`refresh_tokens` テーブルの `user_agent` カラムに保存する。
Dashboard のセッション一覧などで端末情報として表示することを想定している。