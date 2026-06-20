# backend/Dockerfile
# ========================================
# YaakoDrive Backend Dockerfile
# ========================================
#
# 方針:
# - dev: ホストにRustを入れず、コンテナ内でcargo check/test/watch/migrateする
# - builder: release binaryをビルドする
# - prod: 小さめのruntime imageにserver/cli binaryだけを入れる
#
# compose.dev.yaml は target: dev を使う。
# compose.prod.yaml は target: prod を使う。

# ========================================
# Base Stage (Rust共通)
# ========================================
FROM rust:1.88 AS base

ENV TZ=Asia/Tokyo
WORKDIR /workspace/backend

# sqlxやopenssl系crateで必要になりやすいライブラリを入れる。
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    pkg-config \
    libssl-dev \
    postgresql-client \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# ========================================
# Dev Stage
# ========================================
# 開発用コンテナ。
# cargo-watchとsqlx-cliを入れ、ソースはcomposeでbind mountする。
FROM base AS dev

# -- lockedを付けて、Rust 1.88環境でビルドできるように
# --version を指定して、Rust 1.88環境で確実にビルドできるバージョン（v0.8.3など）を固定
RUN cargo install cargo-watch --locked \
    && cargo install sqlx-cli --version 0.8.3 --no-default-features --features rustls,postgres --locked

CMD ["cargo", "watch", "-x", "check"]

# ========================================
# Planner Stage
# ========================================
# cargo-chefを使うと依存crateのビルドキャッシュが効きやすい。
# まだプロジェクト初期でも、後で効いてくるので最初から入れておく。
FROM base AS chef
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY backend ./
RUN cargo chef prepare --recipe-path recipe.json

# ========================================
# Builder Stage
# ========================================
FROM chef AS builder

COPY --from=planner /workspace/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY backend ./
RUN cargo build --release -p server \
    && cargo build --release -p cli

# ========================================
# Prod Stage
# ========================================
FROM debian:bookworm-slim AS prod

ENV TZ=Asia/Tokyo
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# Rustのrelease binaryだけをコピーする。
COPY --from=builder /workspace/backend/target/release/server /app/yaakodrive-server
COPY --from=builder /workspace/backend/target/release/cli /app/yaakodrive-cli

# configはcompose側で /app/config にmountする想定。
# 実ファイル保存先はcompose側で /data にmountする想定。
EXPOSE 8080

CMD ["/app/yaakodrive-server"]
