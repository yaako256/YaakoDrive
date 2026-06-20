/*
sql/migrations/20260620192420_create_refresh_tokens.sql
RefreshTokensテーブルの定義
*/

-- テーブル定義
CREATE TABLE refresh_tokens (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT        NOT NULL,
    user_agent  TEXT,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at  TIMESTAMPTZ
);

-- 制約条件
CREATE UNIQUE INDEX refresh_tokens_token_hash_unique ON refresh_tokens(token_hash);