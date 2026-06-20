# YaakoDrive 全体設計書

## 目的
YaakoDriveは、個人・友人・家族向けのクラウドストレージシステムである。
本番運用はTailscale内のみを想定するが、練習も兼ねて、将来的にインターネット公開しても耐えられる構成・認証・保守設計を目指す。

## 技術スタック
- フロントエンド: React
- バックエンド: Rust
- HTTP API: axum
- DB: PostgreSQL 15以降
- DBアクセス: sqlx
- コンテナ: Docker / Docker Compose
- 認証: JWT + HttpOnly Cookie + Refresh Tokenローテーション
- パスワードハッシュ: argon2

## MVP範囲
MVPでは、個人用クラウドストレージとして最低限使える範囲を実装する。

- ログイン/ログアウト
- 管理者によるユーザ作成
- ファイルアップロード
  - 複数ファイル選択
  - 順次アップロード
  - 最大10MB程度
  - ストリーミング保存
- ファイルダウンロード
- 一覧表示
- フォルダ作成
- リネーム
- 移動
- 削除(ゴミ箱)
- ゴミ箱から復元
- 物理削除
- 検索
- ダッシュボード(使用容量、総ファイル数など)

## MVP後に回すもの
- チャンクアップロード
- フォルダごとアップロード
- 通信速度制限
- 共有機能
- タグ
- お気に入り
- プレビュー
- 複数インスタンス対応

## 想定フォルダ構造
```text
YaakoDrive/
├── compose.yaml
├── compose.dev.yaml
├── compose.prod.yaml
├── Dockerfile
├── Makefile
├── config/              # config.tomlなどの設定ファイル
├── docs/                # 設計書
├── frontend/            # React
├── backend/
│   ├── Cargo.toml
│   └── crates/          # Rust workspace crates
└── sql/                 # migrationや補助SQL
```

## コンテナ方針
フロントエンドとバックエンドはコンテナを分ける。
開発用コンテナも用意し、フロントエンド・バックエンド・設定・SQLなど、アプリ全体を見渡せるようにする。

```text
frontend container: React開発/配信
backend container: Rust API server
postgres container: PostgreSQL
dev container: 開発補助用。リポジトリ全体をマウントして作業できる
```

## 設定方針
アプリ設定は `config.toml` 系で管理し、秘密情報や環境差分は環境変数で上書きできるようにする。
`Cargo.toml` はRust依存関係・ビルド設定用であり、アプリ設定には使わない。

例:
```text
config/default.toml
config/development.toml
config/production.toml
.env
```

環境変数は `APP__...` 形式で階層的に上書きできるようにする。

```text
APP__DATABASE__URL=postgres://...
APP__JWT__SECRET=...
APP__COOKIE__SECURE=false
APP__STORAGE__DATA_DIR=./dev-data
APP__UPLOAD__MAX_SIZE_BYTES=10485760
```

## ストレージ方針
実ファイルはディスク上にフラットに保存し、ユーザに見せるフォルダ構造はDBで管理する。
ファイル移動・リネームはDB更新だけで完結させる。

```text
yaakodrive-data/
└── files/
    ├── 550e8400-e29b.dat
    └── 550e8400-e29c.dat
```

## アップロード整合性方針
DBと実ファイルの不整合を減らすため、アップロードは以下の流れで行う。

```text
一時ファイルへストリーミング保存
DB transaction開始
nodes / file_contents を pending 状態で作成
一時ファイルを正式配置へ移動
DB上の状態を active に更新
commit
```

失敗時はtransactionをrollbackし、一時ファイルおよび正式配置済みファイルを可能な範囲で削除する。
ファイルシステムとDB transactionは完全には一体化できないため、整合性チェック用の管理コマンドを用意する。

## セキュリティ方針
本番アクセスはTailscale内のみを想定する。
ただし、認証・Cookie・入力検証・ファイル保存パスなどは、将来的にインターネット公開しても耐えられる設計を目指す。

- Access Token / Refresh TokenはHttpOnly Cookieで保持
- Refresh Tokenは専用Pathに限定
- Refresh Tokenはローテーション方式
- 本番CookieはSecure=true
- 開発環境ではSecure=falseを許可
- ユーザ入力を保存パスに使わない
- 保存ファイル名はサーバ生成UUID等を使う

## バックアップ方針
バックアップ対象はDBと実ファイルである。

- DB: `pg_dump` などで定期バックアップ
- 実ファイル: restic等でバックアップ
- 復元試験を年数回行う
- 復元後にDBと実ファイルの整合性チェックを行う

## 設計書一覧
- [`01_overall_design.md`](./01_overall_design.md): 全体設計
- [`02_backend_design.md`](./02_backend_design): Rustバックエンド設計
- [`03_frontend_design.md`](./03_frontend_design.md): Reactフロントエンド設計
- [`04_api_design.md`](./04_api_design.md): API設計
- [`05_db_design.md`](./05_db_design.md): DB設計
- [`06_operations_design.md`](./06_operations_design.md): CLI/運用設計
