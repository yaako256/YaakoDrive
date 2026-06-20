# frontend/Dockerfile
# ========================================
# YaakoDrive Frontend Dockerfile
# ========================================
#
# 方針:
# - dev: Node.jsコンテナでVite dev serverを起動する
# - builder: 静的ファイルをbuildする
# - prod: nginxで静的ファイルを配信する
#
# バックエンド優先開発のため、現段階のfrontendは最小構成でもよい。

# ========================================
# Base Stage (Node.js共通)
# ========================================
FROM node:22-bookworm AS base

WORKDIR /workspace/frontend

# ========================================
# Dev Stage
# ========================================
FROM base AS dev

CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]

# ========================================
# Builder Stage
# ========================================
FROM base AS builder

COPY frontend/package*.json ./
RUN npm install

COPY frontend ./
RUN npm run build

# ========================================
# Prod Stage
# ========================================
FROM nginx:1.27-alpine AS prod

COPY --from=builder /workspace/frontend/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
