# Makefile.common.mk

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