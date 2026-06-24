# makefile

# ==================================
# 設定・変数定義のロード
# ==================================
-include Makefile.common.mk

# ==========================================
### メイン / CLI処理用の設定
# ==========================================
.PHONY: create-admin

## 管理者ユーザ作成
# 使い方: make create-admin USERNAME=yaako
create-admin:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) \
		cargo run -p cli -- create-admin --username $(USERNAME)

# ==========================================
### メイン / 開発用DB確認
# ==========================================
.PHONY: user user-x
## ユーザのテーブル(一部)を表示
user:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -c "SELECT id, username, role, password_hash FROM users;"

## ユーザのテーブル(すべて)を縦に表示
user-x:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -x -c "SELECT * FROM users;"


.PHONY: token token-x

## RefreshTokensのテーブル(一部)を表示
token:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -c "SELECT id, user_id, expires_at, created_at, revoked_at FROM refresh_tokens;"


## RefreshTokensのテーブル(すべて)を縦に表示
token-x:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -x -c "SELECT * FROM refresh_tokens;"


.PHONY: node node-x

## Nodeのテーブル(一部)を表示
node:
#	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -c "SELECT id, owner_user_id, parent_id, name, node_type, created_at, updated_at, deleted_at FROM nodes;"
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -c "SELECT id, owner_user_id, parent_id, name, node_type, updated_at, deleted_at FROM nodes;"


## Nodeのテーブル(すべて)を縦に表示
node-x:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -x -c "SELECT * FROM nodes;"


.PHONY: file file-x

## file_contentsのテーブル(一部)を表示
file:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -c "SELECT node_id, stored_filename, mime_type, size_bytes, status FROM file_contents;"

## file_contentsのテーブル(すべて)を縦に表示
file-x:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -x -c "SELECT * FROM file_contents;"

# ------------------------------------------
# 開発用コマンドの読み込み
# (ファイルがなければ無視する -include)
# ------------------------------------------
-include Makefile.dev.mk
