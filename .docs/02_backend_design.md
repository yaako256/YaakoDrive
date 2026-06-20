# YaakoDrive バックエンド設計書

## 方針
Rust workspaceを採用する。
レイヤードアーキテクチャをベースに、Repository TraitとStorageService Traitによって依存関係逆転を行う。
`shared` クレートにすべてを集める設計は避け、各クレートの責務を明確にする。

## Workspace構成
```text
backend/
├── Cargo.toml
└── crates/
    ├── server      # HTTPサーバ起動、DI組み立て
    ├── cli         # 管理コマンド
    ├── app         # ユースケース
    ├── api         # axum HTTP層
    ├── auth        # 認証/JWT/PasswordHasher
    ├── identity    # UserId, NodeIdなど
    ├── node        # ファイル・フォルダのドメイン
    ├── storage     # 実ファイル保存
    ├── repository  # Repository Trait / UnitOfWork Trait
    ├── infra       # PostgreSQL/sqlx実装
    └── config      # 設定読み込み
```

## 依存関係
```text
server     -> api, app, infra, config
cli        -> app, infra, config
api        -> app, auth, identity
app        -> auth, node, storage, repository
infra      -> repository, auth, node, identity
repository -> auth, node, identity
node       -> identity
auth       -> identity
storage    -> 外部crateのみ
config     -> 外部crateのみ
identity   -> 外部crateのみ
```

`server` と `cli` はエントリポイントであり、具体実装を組み立てることができる。
`app` はHTTPやPostgreSQLを知らない。
`api` はHTTP層を担当するが、業務ルールは持たない。

## 主要crate候補
```toml
# HTTP / async
axum
tokio
tower
tower-http

# DB
sqlx

# serialize
serde
serde_json

# error
thiserror

# log / tracing
tracing
tracing-subscriber

# auth
jsonwebtoken
argon2
rand

# time / id
chrono
uuid

# async trait / stream / bytes
async-trait
bytes
futures-core
tokio-util

# config
config
dotenvy

# cookie
axum-extra
# または tower-cookies

# mime / validation
mime_guess
validator

# test
tempfile
```

## 各クレートの責務
### server
HTTPサーバの起動とDI組み立てを担当する。
`infra` の具象Repository、`storage` の具象実装、`app` のユースケース、`api` のRouterを組み合わせる。

### cli
管理コマンドを担当する。
HTTPを経由せず、DBメンテナンスや整合性チェックなどを実行する。

### app
ユースケース層。
ログイン、トークン更新、アップロード、移動、削除、復元、検索、Dashboard集計などを実装する。
HTTP、axum、SQL、PostgreSQLには依存しない。

### api
HTTP層。
Routing、Request解析、Response生成、Cookie処理、JWT認証ミドルウェア、HTTP StatusCode変換、Multipart受信を担当する。

JWTの検証自体はauthクレートを利用する。
認証済みユーザIDなどをapp層へ渡す。

### auth
JWT生成・検証、PasswordHasher、TokenClaims、User、RefreshToken、Roleなどを担当する。

### identity
`UserId`、`NodeId`、`RefreshTokenId` などのID型を担当する。

### node
Driveのコアドメイン。
Node、NodeType、FileContent、名前変更ルール、移動ルール、削除ルール、階層構造ルールを担当する。

### storage
実ファイル保存を担当する。
MVPでもアップロード/ダウンロードはストリーミング前提とする。

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

### repository
Repository TraitとUnitOfWork Traitを定義する。
SQLやsqlxには依存しない。

```rust
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type Tx: TransactionContext;

    async fn begin(&self) -> RepoResult<Self::Tx>;
}
```

### infra
Repository TraitとUnitOfWork TraitのPostgreSQL/sqlx実装を担当する。
SQL、sqlx、PostgreSQL固有処理はこのクレートに閉じ込める。

### config
`config/default.toml`、環境別toml、環境変数を読み込み、アプリ設定型へ変換する。

## エラー方針
各クレートに専用Error型とResult型を置く。
`app` クレートでは、各クレートのErrorをまとめる `AppError` / `AppResult` を定義する。

## アップロードユースケース
```text
api::upload_handler
 ↓
UploadFileUseCase
 ↓
storage.save_temp_stream()
 ↓
unit_of_work.begin()
 ↓
node_repository.create(status = pending)
 ↓
file_content_repository.create(status = pending)
 ↓
storage.promote_file()
 ↓
node/file_content status = active
 ↓
transaction.commit()
```

失敗時はrollbackし、一時ファイル/正式配置済みファイルを可能な範囲で削除する。

## 認証方針
Access TokenとRefresh TokenはHttpOnly Cookieで管理する。
Refresh Tokenはローテーション方式を採用する。
User-Agentはセッション一覧表示用に記録するが、認証の必須検証条件にはしない。

## sqlx方針
- migrationは `sqlx migrate` を使う
- DBアクセスはinfraクレートに閉じ込める
- app/repository/node/authはsqlxに依存しない
- SQLファイルは `sql/` またはsqlx標準の `migrations/` 配下に置く方針を実装時に確定する
