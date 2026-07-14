# Build stage
FROM rust:1.96-bookworm AS builder

WORKDIR /app

# 先复制并编译依赖，利用缓存
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release && rm -rf src

# 复制源码并编译
COPY src ./src
RUN touch src/main.rs \
    && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/oak-website-backend /usr/local/bin/oak-website-backend

ENV APP__DATABASE__URL=postgres://oak:oak@db/oak_website
ENV APP__SERVER__HOST=0.0.0.0
ENV APP__SERVER__PORT=8080
ENV APP__DOCS__HTML_DIR=/data/docs-html

EXPOSE 8080

CMD ["oak-website-backend"]
