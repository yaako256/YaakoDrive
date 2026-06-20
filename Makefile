# Makefile
# メモ => PHONY: ファイルではないという指定(ファイルは更新されていないと実行されない): 命令である

# ========================================
# YaakoDrive 開発補助コマンド
# ========================================
# ホストにRust/Node.jsが入っていなくても、Docker経由で操作する。

# ==================================
# 設定・変数定義
# ==================================
.DEFAULT_GOAL := help

# 実行時の引数（未指定時はhelp）
CMD ?= help

# compose
COMPOSE_DEV := docker compose -f compose.yaml -f compose.dev.yaml
COMPOSE_PROD := docker compose -f compose.yaml -f compose.prod.yaml

# service name
BACKEND_SERVICE_NAME := backend
FRONTEND_SERVICE_NAME := frontend
DATABASE_SERVICE_NAME := db


# ==================================
### 実行関連(Execution)
# ==================================
.PHONY: check run-server test

# サーバ起動(開発用)
run-server:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) cargo run -p server

# バックエンドのCargo check
check:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) cargo check

# バックエンドのCargo test
test:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) cargo test

# ==================================
### Database関連
# ==================================
.PHONY: migrate-run migrate-add migrate-psql-l migrate-psql migrate-reset

## migrationの実行
migrate-run:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) \
	  sqlx migrate run --source /workspace/sql/migrations

## SQLファイルの作成
# 使い方: make migrate-add name=create_users
migrate-add:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) \
	  sqlx migrate add --source /workspace/sql/migrations $(name)

## テーブル一覧の確認
# $$DATABASE_URL: Makefileでは$$でエスケープしないとホスト側で展開されてしまう
# また、環境変数関連をいい感じにするため、bash -c '' でいい感じにしている
migrate-psql-l:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) \
	  bash -c 'psql $$DATABASE_URL -c "\l"'	  

## テーブル一覧の確認
# $$DATABASE_URL: Makefileでは$$でエスケープしないとホスト側で展開されてしまう
migrate-psql:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) \
	  bash -c 'psql $$DATABASE_URL -c "\\dt"'

## migrationのリセット（開発時のみ使用）
# volumeを削除してDBを再初期化し、再度migrationを実行する
migrate-reset:
	$(COMPOSE_DEV) down
	docker volume rm yaakodrive-dev_postgres_data_dev || true
	$(COMPOSE_DEV) up -d
	@echo "Waiting for db to be ready..."
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) \
	  sqlx migrate run --source /workspace/sql/migrations






# ==================================
### Docker関連(Docker Management)
# ==================================
# ---- 開発用コンテナ ----
.PHONY: env dev-up dev-stop dev-down dev-build dev-logs dev-ps

## envファイル再読み込み用
# 実態はただのup
env:
	$(COMPOSE_DEV) up -d --force-recreate

## 開発用コンテナを起動(buildでDockerfileの再読み込みもする)
dev-up:
	$(COMPOSE_DEV) up --build

## 開発用コンテナを停止
dev-stop:
	$(COMPOSE_DEV) stop

# 開発用コンテナ・ネットワークを停止・削除
dev-down:
	$(COMPOSE_DEV) down

## 開発用Dockerイメージのビルドチェック
dev-build:
	$(COMPOSE_DEV) build

## 開発用コンテナのログ
dev-logs:
	$(COMPOSE_DEV) logs -f

## 開発用コンテナのログ
dev-ps:
	$(COMPOSE_DEV) ps

.PHONY: backend-shell frontend-shell

## 開発用コンテナのバックエンドに入る
backend-shell:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) bash

## 開発用コンテナのフロントエンドに入る
frontend-shell:
	$(COMPOSE_DEV) exec $(FRONTEND_SERVICE_NAME) bash

.PHONY: backend-check backend-test migrate

backend-check:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) cargo check

backend-test:
	$(COMPOSE_DEV) exec $(BACKEND_SERVICE_NAME) cargo test

# ---- 本番用コンテナ ----
.PHONY: prod-up prod-down deploy prod-ps
## 本番用コンテナ起動
prod-up:
	$(COMPOSE_PROD) up -d --build

## 本番用コンテナを停止
prod-down:
	$(COMPOSE_PROD) down

## 本番デプロイ
deploy:
	$(COMPOSE_PROD) up -d --build --force-recreate

## 本番デプロイ
prod-ps:
	$(COMPOSE_PROD) ps


## 完全本番デプロイ
# - dev停止
# - release build
# - container再作成
#deploy-release:
#	$(COMPOSE_DEV) down
#	$(COMPOSE_DEV) rm
#	$(COMPOSE_PROD) up -d --build --force-recreate


# ==================================
### その他 (Utilities)
# ==================================
.PHONY: chown

## カレントディレクトリ内の全ファイルに権限の付与
chown:
	sudo chown -R $(shell whoami):$(shell whoami) .



