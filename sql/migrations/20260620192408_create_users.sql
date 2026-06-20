/*
sql/migrations/20260620192408_create_users.sql
userテーブルの定義
*/

-- テーブル定義
CREATE TABLE users (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    username        TEXT        NOT NULL,
    password_hash   TEXT        NOT NULL,
    role            TEXT        NOT NULL DEFAULT 'user',
    storage_limit_bytes BIGINT  NOT NULL DEFAULT 10737418240, -- 10GB
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    disabled_at     TIMESTAMPTZ
);

-- 制約条件
CREATE UNIQUE INDEX users_username_unique ON users(username);