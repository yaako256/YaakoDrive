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
DBと実ファイルの整合性を保つため、アップロードは以下の流れで行う。

```text
一時ファイルへストリーミング保存
DB transaction開始
nodes / file_contents を pending 状態で作成
一時ファイルを正式配置へ移動
DB上の状態を active に更新
commit
```

途中で失敗した場合は、DB transactionをrollbackし、一時ファイルおよび正式配置済みファイルを可能な範囲で削除する。
これにより、「DBには存在するが実ファイルがない」「実ファイルはあるがDBに存在しない」という不整合をできるだけ防ぐ。

ただし、ファイルシステム操作とDB transactionは完全な一体化ができないため、将来的には整合性チェック用の管理コマンド/バッチを用意する。

## 削除
フォルダを削除したら配下もすべて削除するようにする。
親フォルダをゴミ箱へ移動するときは、配下ノードにも同じ `deleted_at` を設定する。

ゴミ箱でも通常のファイル構造に近い形で表示し、削除済みフォルダの中身やメタデータを確認できるようにする。
復元時に元の親フォルダ内へ同名ファイル・同名フォルダが作成済みだった場合は、ユーザに確認画面を出し、必要に応じて `example (1).jpg` のような自動リネームを行う。

物理削除時は、DB上の `file_contents` / `nodes` を削除したあと、実ファイルを削除する方針とする。
実ファイル削除に失敗した場合に備え、孤立ファイルを検出・掃除する整合性チェック処理を将来的に用意する。
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

MVPでは最大10MB程度の画像・zipファイルを想定するが、クラウドストレージとしての拡張性を考え、アップロード/ダウンロードは最初からストリーミング前提にする。
つまり、ファイル全体を一度 `Bytes` や `Vec<u8>` に載せる設計にはしない。

```rust
use bytes::Bytes;
use futures_core::Stream;
use std::pin::Pin;

pub type StorageByteStream = Pin<Box<dyn Stream<Item = StorageResult<Bytes>> + Send>>;

pub trait StorageService: Send + Sync {
    async fn save_temp_stream(
        &self,
        stream: StorageByteStream,
        extension: Option<&str>,
    ) -> StorageResult<String>;

    async fn promote_file(&self, temp_filename: &str, final_filename: &str) -> StorageResult<()>;
    async fn delete_file(&self, stored_filename: &str) -> StorageResult<()>;

    async fn open_file_stream(&self, stored_filename: &str) -> StorageResult<StorageByteStream>;
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
api        -> app, auth, identity
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

## api と認証責務
apiクレートはHTTP層を担当するため、JWT認証ミドルウェアもapiクレートに置く。
ただし、JWTの発行・検証などの純粋な認証処理はauthクレートが担当する。
apiクレートはCookieやHeaderからトークンを取り出し、authクレートで検証し、認証済みユーザIDなどをappクレートへ渡す。

```text
api: Cookie/Headerの取得、認証ミドルウェア、HTTPエラー変換
auth: JWT生成・検証、PasswordHasher、TokenClaims
app: 認証済みユーザとしてユースケースを実行、認可ルールを判断
```
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

### parent_idとFKについて
ルートノードの `parent_id` は `NULL` とする。
それ以外のノードは、親フォルダの `id` を `parent_id` に入れる。

FK(Foreign Key / 外部キー)とは、あるテーブルの値が別テーブルの実在する行を参照していることをDBに保証させる制約である。
たとえば `nodes.parent_id` にFKを張ると、「存在しない親フォルダIDを持つノード」をDB側で防げる。

当初はルートノード用にセンチネルUUID `00000000-0000-0000-0000-000000000000` を使う案も考えたが、FK制約との相性が悪い。
そのため、PostgreSQL 15以降を想定し、`parent_id IS NULL` をルートとして扱う方針にする。
同一フォルダ内の名前重複制約は、PostgreSQL 15以降の `NULLS NOT DISTINCT` を利用して表現する。

### 構造体定義
```rust
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub owner_user_id: UserId,
    pub parent_id: Option<NodeId>,   // ルートノードは None
    pub name: String,
    pub node_type: NodeType,
    pub deleted_at: Option<DateTime<Utc>>,
    pub status: NodeStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone)]
pub enum NodeType {
    File,
    Folder,
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Pending,
    Active,
}

#[derive(Debug, Clone)]
pub struct FileContent {
    pub node_id: NodeId,
    pub stored_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub status: FileContentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum FileContentStatus {
    Pending,
    Active,
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
| UnitOfWork | 複数Repository操作を同一DB transactionで扱う |

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

### UnitOfWorkについて
アップロードなどでは、`nodes` と `file_contents` を同一DB transactionで更新する必要がある。
Repository Traitを分けるだけではtransaction境界を表現しにくいため、UnitOfWorkを導入する。

UnitOfWorkは「この一連のRepository操作を同じtransaction内で行う」ための境界を表す。
appクレートはPostgreSQLの具体実装を知らずに、UnitOfWorkを通じて安全に複数Repository操作をまとめる。

```rust
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type Tx: TransactionContext;

    async fn begin(&self) -> RepoResult<Self::Tx>;
}

#[async_trait]
pub trait TransactionContext: Send + Sync {
    type NodeRepo: NodeRepository;
    type FileContentRepo: FileContentRepository;

    fn nodes(&self) -> &Self::NodeRepo;
    fn file_contents(&self) -> &Self::FileContentRepo;

    async fn commit(self) -> RepoResult<()>;
    async fn rollback(self) -> RepoResult<()>;
}
```

具体的な型設計は実装時に調整するが、設計方針としては「UploadFileUseCaseなど、複数テーブル更新が必要なユースケースはUnitOfWorkを使う」とする。

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
    /// ルートノードは parent_id IS NULL で判定する。
    async fn find_root_by_owner(&self, owner_user_id: UserId) -> RepoResult<Option<Node>>;

    /// 指定した親ノードの直下にある子ノード一覧を取得する。
    /// フォルダの中身表示で使う。
    async fn find_children(&self, owner_user_id: UserId, parent_id: Option<NodeId>) -> RepoResult<Vec<Node>>;

    /// 同じ親フォルダ内に同名ノードがあるか確認する。
    /// アップロードやリネーム時の重複判定で使う。
    async fn exists_same_name(
        &self,
        owner_user_id: UserId,
        parent_id: Option<NodeId>,
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
storage.save_temp_stream()   # 一時領域にストリーミング保存
 ↓
unit_of_work.begin()         # DB transaction開始
 ↓
node_repository.create()     # status = pending
 ↓
file_content_repository.create() # status = pending
 ↓
storage.promote_file()       # 正式配置
 ↓
node/file_content statusをactiveへ更新
 ↓
transaction.commit()
 ↓
201 Created

失敗時:
transaction.rollback()
一時ファイル/正式配置済みファイルを可能な範囲で削除
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

ルートノードの `parent_id` は `NULL` とする。
PostgreSQL 15以降を想定し、`NULLS NOT DISTINCT` を使うことで、`parent_id IS NULL` のルート直下でも同名制約が正しく効くようにする。

```sql
-- アクティブなノードに対してのみ同名チェックを行う
CREATE UNIQUE INDEX nodes_active_unique
  ON nodes(owner_user_id, parent_id, name)
  NULLS NOT DISTINCT
  WHERE deleted_at IS NULL;
```

`parent_id` にはFK制約を設定し、存在しない親ノードを参照することをDB側でも防ぐ。

## file_contents
```sql
UNIQUE(node_id)
UNIQUE(stored_filename)
```

# バックアップと整合性チェック
`pg_dump` によるDBバックアップと、restic等による実ファイルバックアップは取得タイミングが完全には一致しない可能性がある。
そのため、復元後または定期メンテナンス用に、以下を検出する管理コマンド/バッチを用意する方針とする。

- DBには存在するが実ファイルが存在しない
- 実ファイルは存在するがDBに対応する `file_contents` が存在しない
- `pending` 状態のまま一定時間以上経過したアップロード

MVP時点ではメモ書きに留めるが、ファイルストレージとして重要な保守機能である。

## refresh_tokens
```sql
UNIQUE(token_hash)
```


---
# ファイル名・MIME Type・パス安全性ルール
詳細は後で詰めるが、設計項目として以下を用意する。

- `stored_filename` はUUID等のサーバ生成値のみを使い、ユーザ入力をパスに使わない
- ユーザ表示用ファイル名はDBで管理し、保存パスとは分離する
- ファイル名のUnicode正規化、禁止文字、最大長を決める
- MIME Typeはアップロード時の申告値を信用しすぎず、可能ならサーバ側でも推定する
- ダウンロード時の `Content-Type` / `Content-Disposition` を適切に設定する

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

## Cookieの環境差分
本番では `Secure = true` とし、HTTPSでのみCookieを送信する。
開発中はローカルHTTPで動作確認する可能性があるため、開発環境では `Secure = false` を許可する。

```text
production: Secure=true, HTTPS必須
development: Secure=false または mkcert/Tailscale HTTPSを利用
```

MVPの本番運用はTailscale内のみとする。
ただし、練習も兼ねて、将来的にインターネット公開しても耐えられるセキュリティ設計を意識する。

## トークン再発行の流れ
Access Token が切れた場合、フロントエンドは 401 を受け取ったタイミングで
`POST /api/auth/refresh` を呼び、新しい Access Token を取得する。
フロントエンドはトークンの中身を意識せず、Cookie は自動送信される。

Refresh Tokenはローテーション方式を採用する。
Refresh Tokenを使ってAccess Tokenを再発行したら、使用済みRefresh Tokenは失効させ、新しいRefresh Tokenを発行する。
これにより、万が一古いRefresh Tokenが漏洩しても、再利用を検知・拒否しやすくなる。

```text
通常リクエスト → 401 Unauthorized
 ↓
POST /api/auth/refresh  (Refresh Token Cookie が自動送信される)
 ↓
RefreshTokenUseCase
 ↓
Refresh Token 検証 (有効期限・revoked_at)
 ↓
新しい Access Token を Cookie にセット
 ↓
元のリクエストをリトライ
```

## User-Agentの記録
ログイン時に HTTP ヘッダーの `User-Agent` を取得し、
`refresh_tokens` テーブルの `user_agent` カラムに保存する。
Dashboard のセッション一覧などで端末情報として表示することを想定している。

User-Agentは記録用であり、認証の必須検証条件にはしない。
ブラウザ更新やPWA環境によって値が変わる可能性があるため、検証条件にすると誤ログアウトの原因になりうる。


---
# APIレスポンス設計

## エンベロープ形式
全レスポンスを `data` / `error` でラップする形式を採用する。
フロントエンドは `response.error` を見るだけで成功・失敗を判定できる。

```json
// 成功時
{ "data": { ... }, "error": null }

// 失敗時
{ "data": null, "error": { "code": "not_found", "message": "指定されたノードが存在しません" } }
```

Rust側の型イメージ:
```rust
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}
```

## エラーコード一覧
| code | HTTPステータス | 意味 |
|------|---------------|------|
| `unauthorized` | 401 | 未認証 |
| `forbidden` | 403 | 権限不足 |
| `not_found` | 404 | リソースが存在しない |
| `already_exists` | 409 | 同名ファイル・フォルダの重複 |
| `storage_limit_exceeded` | 409 | ストレージ上限超過 |
| `invalid_request` | 422 | リクエスト内容が不正 |
| `internal_error` | 500 | サーバ内部エラー |

## エンドポイント一覧

### 認証
| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/auth/login` | ログイン。Access Token・Refresh Token を Cookie にセット |
| POST | `/api/auth/refresh` | Access Token 再発行 |
| POST | `/api/auth/logout` | ログアウト。Cookie を削除し Refresh Token を revoke |

### ノード(ファイル・フォルダ共通)
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/nodes` | ルート直下の一覧取得 |
| GET | `/api/nodes/{id}/children` | 指定フォルダの直下一覧取得 |
| GET | `/api/nodes/{id}` | ノード情報取得 |
| PATCH | `/api/nodes/{id}/rename` | リネーム |
| PATCH | `/api/nodes/{id}/move` | 移動 |
| DELETE | `/api/nodes/{id}` | ゴミ箱へ移動(論理削除) |

### ファイル
| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/nodes/{id}/upload` | 指定フォルダへファイルアップロード |
| GET | `/api/nodes/{id}/download-url` | 一時ダウンロードURL発行 |
| GET | `/api/files/download/{token}` | 実ファイル取得(JWT不要・トークン認証) |

### フォルダ
| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/nodes/{id}/folders` | 指定フォルダ配下にフォルダ作成 |

### ゴミ箱
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/trash` | ゴミ箱一覧取得 |
| POST | `/api/trash/{id}/restore` | ゴミ箱から復元 |
| DELETE | `/api/trash/{id}` | 物理削除 |

### 検索
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/search?q={keyword}` | ファイル・フォルダ名検索 |

### Dashboard
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/dashboard` | 使用容量・ファイル数などの統計情報取得 |

### ユーザ管理(管理者のみ)
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/admin/users` | ユーザ一覧取得 |
| POST | `/api/admin/users` | ユーザ作成 |
| PATCH | `/api/admin/users/{id}` | ユーザ情報変更(ストレージ上限・停止など) |

## レスポンス例

### GET /api/nodes/{id}/children
フォルダとファイルを混在して返す。フロントが `node_type` で振り分ける。
フォルダを上に表示するようなソートもフロント側で行う。

```json
{
  "data": {
    "nodes": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "写真",
        "node_type": "folder",
        "parent_id": "550e8400-e29b-41d4-a716-446655440001",
        "deleted_at": null,
        "created_at": "2025-01-01T00:00:00Z",
        "updated_at": "2025-01-01T00:00:00Z"
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "name": "memo.txt",
        "node_type": "file",
        "parent_id": "550e8400-e29b-41d4-a716-446655440001",
        "deleted_at": null,
        "created_at": "2025-01-01T00:00:00Z",
        "updated_at": "2025-01-01T00:00:00Z"
      }
    ]
  },
  "error": null
}
```

### GET /api/nodes/{id}/download-url
一時ダウンロードURLを発行する。トークンはサーバのメモリ(HashMap)で管理する。
有効期限は短命(参考値: 5分)とし、1回使い切りとする。

MVPでは単一インスタンス前提とする。
そのため、サーバ再起動で未使用の一時ダウンロードURLは失効する。
複数インスタンス構成には対応しない。
将来的にはDBやRedisでの管理に拡張することを検討する。

```json
{
  "data": {
    "url": "/api/files/download/a3f8c2d1b9e1f4a2"
  },
  "error": null
}
```

### POST /api/auth/login
レスポンスボディには最低限の情報のみ返す。
トークンはCookieにセットするためボディには含めない。

```json
{
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "yaako",
    "role": "admin"
  },
  "error": null
}
```
