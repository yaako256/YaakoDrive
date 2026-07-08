# Makefile
# ============================================================
# YaakoDrive Makefile
# ============================================================
# makeコマンドのエントリポイント
#
# 実際のターゲットは次に定義
#   make/common.mk
#   make/help.mk
#   make/docker.mk
#   make/cli.mk
#   make/database.mk
#   make/utility.mk
#   make/dev.mk
#   make/prod.mk
#
# 一覧表示
#   make help
#		make ENV=prod help
#
# 次のように環境設定できる
# 	ENV=dev (default)
# 	ENV=prod
# ============================================================

# 共通変数・共通マクロ・環境固有コマンドのロード
include make/common.mk

# docker
include make/docker.mk
# アプリのcli
include make/cli.mk
# database
include make/database.mk
# util
include make/utility.mk
# ヘルプコマンド
include make/help.mk


# ==================================================
# Makefileメモ
# ==================================================
# このようにするとコマンドを見れる
# 	make -n up
#
#
# ==================================================
# CLI実行例
# ==================================================
# 実行例:
#
# 管理者ユーザ作成
#		make cli ARGS="create-admin --username yaako"
# 	make cli ENV=prod ARGS="create-admin --username yaako"
#
# migrationの実行
# (実際はパス設定を異ならせてる関係で固有makeがあるが一応)
#		make cli ARGS="create-admin --username yaako"
# 	make cli ENV=prod ARGS="create-admin --username yaako"
#
#		make migrate
#		make migrate ENV=prod
#
#
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