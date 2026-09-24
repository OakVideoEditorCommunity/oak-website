# Oak Video Editor 官网

本项目是 [Oak Video Editor](https://github.com/OakVideoEditorCommunity/oak) 的官方网站，包含：

- **前端**：Nuxt 4 + Vue 3，SSG 纯静态站点（nginx 托管），深绿-金色暗色主题，SEO 优化，中英双语。
- **后端**：Rust + Axum + SeaORM + PostgreSQL。
- **文档外链**：文档托管在独立站点 [docs.oakvideoeditor.org](https://docs.oakvideoeditor.org)，官网 `/docs` 路由 301 重定向过去。
- **下载分发**：从 GitHub Releases 拉取二进制，上传到 Cloudflare R2，用户下载时返回 R2 预签名链接。
- **CDN**：静态资源可配置 CDN 域名前缀。

## 目录结构

```
oak-website/
├── frontend/       # Nuxt 4 前端（SSG 静态站点 + nginx）
├── backend/        # Rust + Axum 后端
├── docker-compose.yml      # 生产：拉取 GHCR 预构建镜像 + watchtower
├── docker-compose.dev.yml  # 开发：本地构建镜像
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

生产模式（拉取 GHCR 预构建镜像，watchtower 自动更新）：

```bash
docker compose up -d
```

开发模式（本地构建 backend/frontend 镜像，无 watchtower）：

```bash
docker compose -f docker-compose.dev.yml up --build
```

服务：

- 前端：`http://localhost:3000`（nginx 托管静态站点，`/api/*` 反代到后端）
- 后端 API：`http://localhost:8081`
- PostgreSQL：`localhost:5432`（生产模式未暴露到宿主机，仅在容器网络内访问；开发模式暴露便于调试）

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
- `GET /api/v1/update/latest?platform=&arch=` — 自动更新检查：返回最新稳定版本号与更新说明（`notes`）；尚无稳定版时回退到最新预发布版（响应中 `is_prerelease` 为 `true`）；给出 `platform`（可选 `arch`）且对应资产就绪时附带站内下载地址 `download_url`（302 跳转的真实文件路径）
- `POST /api/v1/bug-reports` — 提交 Bug 反馈（multipart 表单：`title`/`version`/`content` 必填，`email`/`screenshot`/`log` 选填；截图仅限图片、单文件最大 10MB；附件存入 R2，提交成功后邮件通知 `APP__SMTP__REPORT_RECIPIENT`，未配置 SMTP 时只入库）
- `GET /api/v1/bug-reports/{id}/files/{screenshot|log}` — 反馈附件的永久直链（免鉴权，报告 ID 即访问凭证；访问时现场生成预签名 URL 并 302 跳转），用于邮件列表场景；需在 `APP__SERVER__PUBLIC_URL` 配置后端公网地址
- `GET /api/admin/bug-reports` — 查看 Bug 反馈列表（需 admin Token，附件返回 1 小时有效的预签名链接；也可在官网 `/admin` 页面输入 Token 查看）
- `GET /api/v1/docs` — 文档目录（zh/en，默认版本；可用 `?version=<版本>` 指定版本）
- `GET /api/v1/docs/versions` — 所有文档版本及默认（最新）版本
- `GET /api/v1/docs/{lang}/{slug}` — 单篇文档 HTML（默认版本）
- `GET /api/v1/docs/{version}/{lang}/{slug}` — 指定版本的单篇文档 HTML
- `POST /api/admin/releases/sync` — 触发 GitHub → R2 同步

## 文档站

文档已迁移到独立站点 [docs.oakvideoeditor.org](https://docs.oakvideoeditor.org)：

- 官网导航中的「文档」链接直接指向文档站（新标签页打开）。
- 官网 `/docs`、`/zh/docs` 及其子路径由前端 nginx 返回 301 重定向到文档站对应路径（去掉 `/docs` 前缀）；静态页内也保留了 meta refresh 与「已迁移」提示作为兜底。
- 后端的 `/api/v1/docs*` 接口仍然保留，供文档站或其它方作为数据源使用。

## Cloudflare CDN

在前端 `.env` 或 `docker-compose.yml` 中设置：

```bash
NUXT_PUBLIC_CDN_DOMAIN=https://assets.oakvideoeditor.org
```

Nuxt 会将静态资源（JS/CSS/图片）的 URL 前缀替换为该域名（构建时固化）。API 请求不会被 CDN 缓存。

注意：前端是纯静态站点，`NUXT_PUBLIC_*` 在**构建时**固化到产物中——直接 `npm run generate` 时请提前设置环境变量；使用 Docker 时通过 `--build-arg NUXT_PUBLIC_SITE_URL=...`（或 compose 的 `build.args`）注入。

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
4. 构建前端镜像时通过 build arg 设置 `NUXT_PUBLIC_SITE_URL` 与 `NUXT_PUBLIC_CDN_DOMAIN`（默认 `https://www.oakvideoeditor.org` 与空）。
5. 运行 `docker compose up -d`。
6. 调用一次 `/api/admin/releases/sync` 同步历史 release。
7. 配置 Cloudflare DNS 指向运行 `frontend` 的服务器，并在 Cloudflare 控制台开启 CDN。

## 注意事项

- 当前 Oak Video Editor 处于 alpha 阶段，请在下载页显著位置提示用户。
- R2 预签名链接有效期为 5 分钟。
- 大文件同步可能需要较长时间，视网络与 R2 速度而定。
- 文档构建使用 `docs-builder` 服务，会在容器启动时按配置的版本列表（或文档仓库的全部 semver tag）构建一次；更新文档或新增版本 tag 后需重启 `docs-builder` 与 `backend` 服务。
