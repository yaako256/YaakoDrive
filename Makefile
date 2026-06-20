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

# 後で書く

# ==================================
### Docker関連(Docker Management)
# ==================================
# ---- 開発用コンテナ ----
.PHONY: dev-up dev-stop dev-down dev-build dev-logs dev-ps

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

migrate:
	$(COMPOSE_DEV) exec backend sqlx migrate run


# ---- 本番用コンテナ ----
## 本番用コンテナ起動
#prod-up:
#	$(COMPOSE_PROD) up -d --build

## 本番用コンテナを停止
#prod-down:
#	$(COMPOSE_PROD) down

## 本番デプロイ
#deploy:
#	$(COMPOSE_PROD) up -d --build --force-recreate


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