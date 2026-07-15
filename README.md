# Oak Video Editor 官网

本项目是 [Oak Video Editor](https://github.com/OakVideoEditorCommunity/oak) 的官方网站，包含：

- **前端**：Nuxt 3 + Vue 3，SSR，SEO 优化，中英双语。
- **后端**：Rust + Axum + SeaORM + PostgreSQL。
- **文档托管**：自动构建 `/home/mikesolar/Projects/oak-docs` 中的 RST 文档（Sphinx）。
- **下载分发**：从 GitHub Releases 拉取二进制，上传到 Cloudflare R2，用户下载时返回 R2 预签名链接。
- **CDN**：Nuxt Nitro 支持 Cloudflare CDN 域名配置。

## 目录结构

```
oak-website/
├── frontend/       # Nuxt 3 前端
├── backend/        # Rust + Axum 后端
├── docs-builder/   # Sphinx 文档构建镜像
├── docker-compose.yml
└── .env.example
```

## 快速开始

### 1. 环境要求

- Docker + Docker Compose
- （可选）Node.js 20+ 与 npm，用于本地前端开发
- （可选）Rust 1.96+，用于本地后端开发

### 2. 配置

```bash
cp .env.example .env
# 编辑 .env，填写 Cloudflare R2 凭据与管理员 Token
```

关键配置项：

| 变量 | 说明 |
|------|------|
| `APP__GITHUB__TOKEN` | GitHub Personal Access Token（无 Token 也可访问公开 releases，但 rate limit 较低） |
| `APP__R2__ENDPOINT_URL` | Cloudflare R2 endpoint，如 `https://<account_id>.r2.cloudflarestorage.com` |
| `APP__R2__ACCESS_KEY_ID` | R2 Access Key ID |
| `APP__R2__SECRET_ACCESS_KEY` | R2 Secret Access Key |
| `APP__R2__BUCKET_NAME` | R2 bucket 名称 |
| `APP__ADMIN__TOKEN` | 管理接口 Bearer Token |

### 3. 启动

```bash
docker compose up -d
```

服务：

- 前端：`http://localhost:3000`
- 后端 API：`http://localhost:8081`
- PostgreSQL：`localhost:5432`（未暴露到宿主机，仅在容器网络内访问）

> 如果本地 8080 已被占用，`docker-compose.yml` 默认将后端映射到 `8081:8080`。

### 4. 同步 GitHub Releases 到 R2

```bash
curl -X POST http://localhost:8081/api/admin/releases/sync \
  -H "Authorization: Bearer <APP__ADMIN__TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{}'
```

后端会自动：

1. 拉取 `OakVideoEditorCommunity/oak` 的 releases。
2. 解析每个 asset 的平台与架构。
3. 下载 asset 并上传到 Cloudflare R2。
4. 更新数据库中的 `sync_status` 为 `ready`。

用户点击下载按钮时，前端调用 `/api/v1/releases/{id}/download?platform=...`，后端返回 302 重定向到 R2 预签名链接。

## 主要 API

- `GET /api/v1/health` — 健康检查
- `GET /api/v1/releases` — 所有 releases
- `GET /api/v1/releases/latest` — 最新 release
- `GET /api/v1/releases/{id}/download?platform=&arch=` — 302 到 R2 预签名链接
- `GET /api/v1/docs` — 文档目录（zh/en）
- `GET /api/v1/docs/{lang}/{slug}` — 单篇文档 HTML
- `POST /api/admin/releases/sync` — 触发 GitHub → R2 同步

## Cloudflare CDN

在前端 `.env` 或 `docker-compose.yml` 中设置：

```bash
NUXT_PUBLIC_CDN_DOMAIN=https://assets.oakvideoeditor.org
```

Nuxt Nitro 会将静态资源（JS/CSS/图片）的 URL 前缀替换为该域名。API 路由不会被 CDN 缓存；文档页面可配置 ISR 缓存。

## 测试

### 后端

```bash
cd backend
cargo test
```

生成覆盖率报告（需要 [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)）：

```bash
cd backend
cargo tarpaulin
```

### 前端

```bash
cd frontend
npm run test
```

生成覆盖率报告：

```bash
cd frontend
npm run test:coverage
```

## 本地开发

### 后端

```bash
cd backend
cargo run
```

需要本地 PostgreSQL，并设置环境变量 `APP__DATABASE__URL`。

### 前端

```bash
cd frontend
npm install
npm run dev
```

浏览器端默认使用同源相对路径请求 `/api/...`。本地开发时后端不在同源，请在 `frontend/.env` 中设置 `NUXT_PUBLIC_API_BASE_URL=http://localhost:8080`。

## 部署到生产

1. 准备 PostgreSQL 数据库。
2. 创建 Cloudflare R2 bucket 并生成 API token。
3. 填写 `.env` 中所有 R2 与管理员配置。
4. 设置 `NUXT_PUBLIC_SITE_URL` 与 `NUXT_PUBLIC_CDN_DOMAIN`。
5. 运行 `docker compose up -d`。
6. 调用一次 `/api/admin/releases/sync` 同步历史 release。
7. 配置 Cloudflare DNS 指向运行 `frontend` 的服务器，并在 Cloudflare 控制台开启 CDN。

## 注意事项

- 当前 Oak Video Editor 处于 alpha 阶段，请在下载页显著位置提示用户。
- R2 预签名链接有效期为 5 分钟。
- 大文件同步可能需要较长时间，视网络与 R2 速度而定。
- 文档构建使用 `docs-builder` 服务，会在容器启动时运行一次；更新文档后需重启 `docs-builder` 与 `backend` 服务。
