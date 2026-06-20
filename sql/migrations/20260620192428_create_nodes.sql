/*
sql/migrations/20260620192428_create_nodes.sql
nodeテーブルの定義
*/

-- テーブル定義
CREATE TABLE nodes (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id       UUID        REFERENCES nodes(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    node_type       TEXT        NOT NULL, -- 'file' | 'folder'
    status          TEXT        NOT NULL DEFAULT 'active', -- 'pending' | 'active'
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 制約
-- 同一フォルダ内の同名禁止（削除済みは除外）
-- NULLS NOT DISTINCTでルート直下(parent_id IS NULL)も正しく動く
CREATE UNIQUE INDEX nodes_active_unique
    ON nodes(owner_user_id, parent_id, name)
    NULLS NOT DISTINCT
    WHERE deleted_at IS NULL;

/*
メモ
NULLS NOT DISTINCT はPostgreSQL 15以降の機能で、parent_id IS NULL（ルート直下）のノードに対しても同名制約が正しく効くようにする。
これがないとルート直下では同名ファイルが複数作れてしまう。
循環参照防止として、parent_id は同じテーブルの id を参照する自己参照外部キーである。
*/