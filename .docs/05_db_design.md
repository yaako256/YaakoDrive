# YaakoDrive DB設計書

## 方針
PostgreSQL 15以降を利用する。
DBアクセスはsqlxを使い、実装はinfraクレートに閉じ込める。
アプリケーション層はRepository TraitとUnitOfWork Traitだけを利用する。

## migration方針
`sqlx migrate` を利用する。
SQLファイルの配置は、実装開始時に `sql/` と `backend/migrations/` のどちらに寄せるか決める。
sqlx標準に合わせるなら `backend/migrations/` が自然である。

## 主なテーブル
- users
- refresh_tokens
- nodes
- file_contents

MVP後に追加候補:
- favorites
- tags
- node_tags
- shares

## users
ユーザ管理テーブル。

主なカラム:
- id
- username
- password_hash
- role
- storage_limit_bytes
- created_at
- updated_at
- disabled_at

制約:
```sql
UNIQUE(username)
```

## refresh_tokens
Refresh Token管理テーブル。
Refresh Tokenは平文では保存せず、ハッシュ化して保存する。

主なカラム:
- id
- user_id
- token_hash
- user_agent
- expires_at
- created_at
- revoked_at

制約:
```sql
UNIQUE(token_hash)
```

## nodes
ファイル・フォルダの論理構造を管理する。
実ファイル情報は持たない。

主なカラム:
- id
- owner_user_id
- parent_id
- name
- node_type
- status
- deleted_at
- created_at
- updated_at

`parent_id` はルートノードでは `NULL` とする。
それ以外では親フォルダの `id` を参照する。

FK(Foreign Key / 外部キー)により、存在しない親ノードを参照しないようDB側でも保証する。

同一フォルダ内の同名ノードは禁止する。
削除済みノードと同名のノードは作成できるようにするため、Partial Indexを使う。
PostgreSQL 15以降の `NULLS NOT DISTINCT` を利用し、ルート直下でも同名制約が正しく効くようにする。

```sql
CREATE UNIQUE INDEX nodes_active_unique
  ON nodes(owner_user_id, parent_id, name)
  NULLS NOT DISTINCT
  WHERE deleted_at IS NULL;
```

## file_contents
ファイル実体に関する情報を管理する。
フォルダは保持しない。

主なカラム:
- node_id
- stored_filename
- mime_type
- size_bytes
- status
- created_at
- updated_at

制約:
```sql
UNIQUE(node_id)
UNIQUE(stored_filename)
```

## status方針
アップロード中の不整合を減らすため、`nodes` と `file_contents` に状態を持たせる。

```text
pending: DB登録済みだがアップロード完了前
active: 利用可能
```

一定時間以上 `pending` のまま残っているデータは、管理コマンドで検出・掃除する。

## 削除方針
削除は `deleted_at` による論理削除とする。
フォルダをゴミ箱へ移動するときは、配下ノードにも同じ `deleted_at` を設定する。

物理削除時は、DB上の `file_contents` / `nodes` を削除したあと、実ファイルを削除する方針とする。
実ファイル削除に失敗した場合に備え、孤立ファイル検出コマンドを用意する。

## 容量集計
Dashboardでは以下を集計する。

- ユーザごとの使用容量
- 容量上限
- 総ファイル数
- 総フォルダ数
- MIME Typeごとの件数

容量は `file_contents.size_bytes` をもとに集計する。
原則として `active` なファイルを対象にする。
ゴミ箱内ファイルを容量に含めるかどうかは実装前に最終決定する。

## バックアップ整合性
DBバックアップと実ファイルバックアップは取得タイミングが完全には一致しない可能性がある。
復元後または定期メンテナンスで以下を検出する。

- DBには存在するが実ファイルが存在しない
- 実ファイルは存在するがDBに対応する `file_contents` が存在しない
- `pending` 状態のまま一定時間以上経過したアップロード
