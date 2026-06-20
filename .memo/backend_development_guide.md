# YaakoDrive バックエンド開発指導書

## 目的
この文書は、YaakoDriveのバックエンドを実装するときの開発順序、テスト方針、フロントエンドとの関係を整理するための指導書である。

基本方針として、先にバックエンドをかなり完成に近い状態まで作り、その後フロントエンドを本格実装する。
ただし、バックエンド開発中にも動作確認用の最小UIやHTTPクライアントは必要になる可能性がある。

## 結論
バックエンドを先に作る認識でよい。
特にYaakoDriveは、認証、DB、ファイル保存、ゴミ箱、整合性チェックなど、バックエンド側の設計がシステム全体の土台になる。

ただし、「フロントエンドを完全に後回し」にするのではなく、以下のように分ける。

```text
先に作る: バックエンド本体、DB、CLI、API、OpenAPI相当のAPI仕様、HTTPテスト
必要に応じて作る: 動作確認用の最小HTML/React画面
後で本格的に作る: 本番用React UI、PWA、画面設計、UX調整
```

## 開発フェーズ全体像
```text
Phase 0: リポジトリ土台
Phase 1: Rust workspace土台
Phase 2: config / logging / health check
Phase 3: DB / sqlx migration
Phase 4: domain型 / repository trait
Phase 5: infra実装 / UnitOfWork
Phase 6: CLI管理コマンド
Phase 7: 認証API
Phase 8: ノード/フォルダAPI
Phase 9: ファイルアップロード/ダウンロードAPI
Phase 10: ゴミ箱/検索/Dashboard
Phase 11: 統合テスト/運用確認
Phase 12: フロントエンド本格実装へ移行
```

## Phase 0: リポジトリ土台
最初にプロジェクトの外形を作る。

作成対象:
```text
YaakoDrive/
├── compose.yaml
├── compose.dev.yaml
├── compose.prod.yaml
├── Dockerfile
├── Makefile
├── config/
├── docs/
├── frontend/
├── backend/
│   ├── Cargo.toml
│   └── crates/
└── sql/ または backend/migrations/
```

この段階では中身は最小でよい。
目的は、プロジェクト全体の置き場所と責務を固定することである。

## Phase 1: Rust workspace土台
`backend/` 配下にRust workspaceを作る。

作成するcrate:
```text
crates/server
crates/cli
crates/app
crates/api
crates/auth
crates/identity
crates/node
crates/storage
crates/repository
crates/infra
crates/config
```

最初の達成条件:
```bash
cargo check
```

全crateが空に近い状態でもよいので、依存関係が循環せず、ビルドできる状態にする。

## Phase 2: config / logging / health check
最初にアプリ全体の土台を実装する。

実装対象:
- `config/default.toml`
- `config/development.toml`
- `.env` 読み込み
- `APP__...` 環境変数上書き
- `tracing` / `tracing-subscriber`
- axum server起動
- `/api/health`

達成条件:
```bash
cargo run -p server
curl http://localhost:xxxx/api/health
```

レスポンス例:
```json
{ "data": { "status": "ok" }, "error": null }
```

この段階でDocker ComposeからPostgreSQLも起動できるようにしておくとよい。

## Phase 3: DB / sqlx migration
PostgreSQLとsqlx migrationを導入する。

実装対象:
- users
- refresh_tokens
- nodes
- file_contents

方針:
- migrationは `sqlx migrate` を使う
- DB固有処理はinfraクレートに閉じ込める
- app/repository/node/authはsqlxに依存しない

達成条件:
```bash
sqlx migrate run
```

DBに初期テーブルが作成されること。

注意:
`sql/` と `backend/migrations/` のどちらに置くかは実装時に決める。
sqlx標準に寄せるなら `backend/migrations/` が自然である。

## Phase 4: domain型 / repository trait
DB実装に入る前に、ドメイン型とTraitを作る。

実装対象:
- identity
  - UserId
  - NodeId
  - RefreshTokenId
- auth
  - User
  - Role
  - RefreshToken
  - PasswordHasher
  - JwtService
- node
  - Node
  - NodeType
  - NodeStatus
  - FileContent
  - FileContentStatus
- repository
  - UserRepository
  - RefreshTokenRepository
  - NodeRepository
  - FileContentRepository
  - UnitOfWork

達成条件:
```bash
cargo check
cargo test
```

この段階ではDBに接続しなくてもよい。
型とTraitの関係が整理されていることが重要である。

## Phase 5: infra実装 / UnitOfWork
sqlxを使ってRepository TraitのPostgreSQL実装を作る。

実装対象:
- PostgresUserRepository
- PostgresRefreshTokenRepository
- PostgresNodeRepository
- PostgresFileContentRepository
- PostgresUnitOfWork

優先順:
```text
1. UserRepository
2. RefreshTokenRepository
3. NodeRepository
4. FileContentRepository
5. UnitOfWork
```

達成条件:
- テストDBに対するRepositoryテストが通る
- transactionがcommit/rollbackできる
- `nodes` と `file_contents` を同一transactionで扱える

## Phase 6: CLI管理コマンド
招待制/管理者発行制のため、最初の管理者ユーザを作る入口が必要である。
HTTP APIより先にCLIを作ると開発が進めやすい。

最初に作るコマンド:
```bash
yaakodrive-cli create-admin --username yaako
```

その後に作る候補:
```bash
yaakodrive-cli cleanup-expired-tokens
yaakodrive-cli cleanup-trash --older-than-days 30
yaakodrive-cli check-storage-consistency
yaakodrive-cli cleanup-pending-uploads
```

達成条件:
- CLIから管理者ユーザを作成できる
- 作成したユーザでログインAPIのテストができる

## Phase 7: 認証API
認証まわりを実装する。

実装対象:
- POST `/api/auth/login`
- POST `/api/auth/refresh`
- POST `/api/auth/logout`
- JWT生成/検証
- Refresh Tokenローテーション
- HttpOnly Cookie設定
- 開発/本番Cookie設定差分

達成条件:
- CLIで作った管理者ユーザでログインできる
- Access Token切れ時にRefreshできる
- LogoutでRefresh Tokenがrevokeされる
- Cookieが期待通りに設定/削除される

テスト方法:
- Rust integration test
- curl
- HTTPクライアント
- 必要なら最小HTMLページ

## Phase 8: ノード/フォルダAPI
ファイルアップロードより前に、フォルダ構造を扱えるようにする。

実装対象:
- GET `/api/nodes`
- GET `/api/nodes/{id}/children`
- POST `/api/nodes/{id}/folders`
- PATCH `/api/nodes/{id}/rename`
- PATCH `/api/nodes/{id}/move`
- DELETE `/api/nodes/{id}`

重要な確認:
- ルートは `parent_id IS NULL`
- 同一フォルダ内の同名禁止
- 移動時に循環を防ぐ
- 削除時に配下ノードへ `deleted_at` を設定する

達成条件:
- フォルダ作成、一覧、リネーム、移動、削除がAPIで確認できる

## Phase 9: ファイルアップロード/ダウンロードAPI
ストレージとして重要な部分を実装する。

実装対象:
- POST `/api/nodes/{id}/upload`
- GET `/api/nodes/{id}/download-url`
- GET `/api/files/download/{token}`
- StorageService
- LocalStorageService
- pending/active制御
- UnitOfWorkを使ったアップロードtransaction

アップロードの流れ:
```text
multipart受信
 ↓
一時ファイルへストリーミング保存
 ↓
DB transaction開始
 ↓
nodes / file_contents を pending 作成
 ↓
正式配置
 ↓
statusをactiveへ更新
 ↓
commit
```

達成条件:
- 10MB程度のファイルをアップロードできる
- ダウンロードできる
- DBと実ファイルが対応している
- 失敗時にrollbackと掃除が行われる

このPhaseでは、テスト用の最小フロントエンドがあると便利である。
ただし本番UIを作り込む必要はない。

## Phase 10: ゴミ箱/検索/Dashboard
MVPとして必要な周辺機能を実装する。

実装対象:
- GET `/api/trash`
- POST `/api/trash/{id}/restore`
- DELETE `/api/trash/{id}`
- GET `/api/search?q=...`
- GET `/api/dashboard`

確認すること:
- ゴミ箱内でもメタデータが見える
- 復元時に同名衝突を処理できる
- 物理削除後に実ファイルも削除される
- Dashboardで使用容量などが集計できる

## Phase 11: 統合テスト/運用確認
バックエンドをフロントエンドへ渡せる状態にするため、統合確認を行う。

確認項目:
- clean環境でDocker Compose起動できる
- migrationが通る
- CLIで管理者ユーザ作成できる
- ログインできる
- フォルダ作成できる
- ファイルアップロード/ダウンロードできる
- ゴミ箱/復元/物理削除できる
- 検索できる
- Dashboardを取得できる
- 整合性チェックCLIが動く

## バックエンド開発中のフロントエンドの扱い
バックエンドを先に作る方針でよいが、動作確認のために以下のいずれかを使う。

### 1. Rust integration test
最も再現性が高い。
ユースケース、Repository、APIのテストに使う。

### 2. curl / HTTPクライアント
認証Cookieやアップロード確認に使う。
手動確認に便利。

### 3. 最小HTMLページ
Cookie付きログイン、ファイルアップロード、ダウンロード確認に使える。
React本実装ではなく、検証用として作る。

### 4. 最小React検証画面
アップロード進捗など、ブラウザ挙動を確認したい場合のみ作る。
本番フロントエンドとは分けて考える。

## フロントエンドへ移るタイミング
以下を満たしたら、フロントエンド本格実装へ移ってよい。

- APIレスポンス形式が固まっている
- 認証Cookieの挙動が確認済み
- 主要APIがMVP分そろっている
- DB migrationが安定している
- Docker Composeでバックエンド/DBが起動できる
- CLIで初期管理者を作れる
- ファイルアップロード/ダウンロードが通る
- ゴミ箱/検索/Dashboardが最低限動く

逆に、以下が未確定ならフロントエンドへ本格移行しない方がよい。

- APIパスが頻繁に変わる
- レスポンスJSON構造が頻繁に変わる
- 認証Cookieの仕様が決まっていない
- ファイルアップロード仕様が変わり続けている

## 実装順の推奨まとめ
```text
1. workspace作成
2. config/logging/health check
3. DB migration
4. domain型/repository trait
5. infra/sqlx実装
6. CLI create-admin
7. auth API
8. folder/node API
9. file upload/download API
10. trash/search/dashboard API
11. integration test / Docker確認
12. frontend本格実装
```

## 開発時の心構え
最初から全機能を作ろうとしない。
まずは「DBにつながる」「起動する」「管理者を作れる」「ログインできる」「フォルダを作れる」「ファイルを保存できる」の順に、縦に細く通す。

バックエンドが安定すると、フロントエンド開発はAPI仕様に乗るだけになる。
そのため、YaakoDriveではバックエンドを先にしっかり作る方針が合っている。
