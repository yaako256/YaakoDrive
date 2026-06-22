# makefile

# ==================================
# 設定・変数定義のロード
# ==================================
-include Makefile.common.mk

# ==========================================
### メイン / CLI処理用の設定
# ==========================================
.PHONY: create-admin user user-x

## 管理者ユーザ作成
# 使い方: make create-admin USERNAME=yaako
create-admin:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) \
		cargo run -p cli -- create-admin --username $(USERNAME)

## ユーザのテーブル(一部)を表示
user:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -c "SELECT id, username, role, password_hash FROM users;"

## ユーザのテーブル(すべて)を縦に表示
user-x:
	$(COMPOSE_DEV) exec $(DATABASE_SERVICE_NAME) \
    psql -U yaakodrive -d yaakodrive_dev -x -c "SELECT * FROM users;"

# ------------------------------------------
# 開発用コマンドの読み込み
# (ファイルがなければ無視する -include)
# ------------------------------------------
-include Makefile.dev.mk
