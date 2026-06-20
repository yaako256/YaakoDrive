/*
sql/migrations/20260620192437_create_file_contents.sql
ファイルのメタデータの定義
*/


CREATE TABLE file_contents (
    node_id         UUID        PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    stored_filename TEXT        NOT NULL,
    mime_type       TEXT        NOT NULL,
    size_bytes      BIGINT      NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'active', -- 'pending' | 'active'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 制約
CREATE UNIQUE INDEX file_contents_stored_filename_unique ON file_contents(stored_filename);