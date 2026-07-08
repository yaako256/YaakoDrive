# ==================================================
# 環境固有定数(prod)
# ==================================================
COMPOSE := docker compose -f compose.yaml -f compose.prod.yaml
DB_NAME := yaakodrive_prod
CLI := /app/yaakodrive-cli
CONTAINER_SHELL := sh
MIGRATIONS_PATH := /app/sql/migrations


# ==================================================
### 環境固有コマンド(prod)
# ==================================================
.PHONY:	setup	deploy

## 本番環境初回セットアップ
#
# 初回のみ実行する。
#
# 作成ディレクトリ
# /srv/yaakodrive/postgres
# /srv/yaakodrive/data/files
# /srv/yaakodrive/data/tmp
#
setup:
	sudo mkdir -p /srv/yaakodrive/postgres
	sudo mkdir -p /srv/yaakodrive/data/files
	sudo mkdir -p /srv/yaakodrive/data/tmp
	@echo "本番用ディレクトリを作成しました"


## 本番デプロイ
# make deploy ENV=prod
deploy:
	$(COMPOSE) up -d --build --force-recreate


# ==================================================
# 初回デプロイ手順
# ==================================================
#
# make setup ENV=prod
# make up ENV=prod
# make migrate ENV=prod
# make create-admin USERNAME=yaako ENV=prod
#
# ==================================================
# 更新時
# ==================================================
#
# make deploy ENV=prod
#