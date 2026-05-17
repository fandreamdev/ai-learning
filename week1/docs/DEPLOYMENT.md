# 部署指南

> 阶段 9 交付物：使用 Docker Compose 一键启动 ProjectAlpha 全栈。

## 总览

部署架构（spec §8）：

```
┌──────────────────────────────────────────────────────┐
│              Docker Compose Network                   │
│                                                        │
│   ┌──────────────┐   ┌──────────────┐  ┌──────────┐  │
│   │   frontend    │   │   backend     │  │  mysql    │  │
│   │  (Nginx+SPA)  │──→│  (FastAPI)    │─→│  (8.0)    │  │
│   │  :80 ←──host  │   │  :8000 (内部) │  │ :3307→3306│  │
│   └──────┬───────┘   └──────────────┘  └──────────┘  │
│          │                                            │
└──────────┼────────────────────────────────────────────┘
           │
        :80 → curl http://localhost/
```

实测镜像体积：
- `week1-backend` 182 MB
- `week1-frontend` 48.6 MB
- `mysql:8.0` 799 MB（官方）

---

## 环境要求

| 软件 | 版本 |
|------|------|
| Docker | 26+ |
| Docker Compose | v2.27+ |
| 端口 | 80（前端 Nginx）、3307（MySQL，向主机暴露便于调试）|
| 磁盘 | ~ 2 GB（镜像 + 数据卷） |

---

## 文件清单

```
week1/
├── docker-compose.yml              # 三服务编排
├── .env.docker.example             # 环境变量模板（DB 密码、端口）
├── backend/
│   ├── Dockerfile                  # 多阶段：uv 安装 → 运行 alembic + uvicorn
│   ├── docker-entrypoint.sh        # 等 mysql + 自动迁移
│   └── .dockerignore
├── frontend/
│   ├── Dockerfile                  # 多阶段：vite build → nginx 提供 dist
│   ├── nginx.conf                  # 静态 + /api 反代 + 安全头
│   └── .dockerignore
└── database/
    ├── init.sql                    # 在 MySQL initdb 时自动创建两个数据库
    └── seeds/seed.sql              # 30 条演示数据（部署后手动注入）
```

---

## 一键启动

```bash
cd week1

# 1. 准备环境变量
cp .env.docker.example .env.docker
# 按需修改 DB_PASSWORD / WEB_PORT / DB_PORT

# 2. 构建并启动（首次约 5~10 分钟，含 MySQL 镜像下载）
docker compose --env-file .env.docker up -d --build

# 3. 等待 MySQL healthy（10 秒以内）
docker compose --env-file .env.docker ps

# 4. 注入演示数据（可选）
docker compose --env-file .env.docker exec -T mysql \
  mysql -uroot -p"$(grep DB_PASSWORD .env.docker | cut -d= -f2)" project_alpha \
  < database/seeds/seed.sql

# 5. 验证
curl http://localhost/healthz
curl http://localhost/api/v1/tickets?page_size=5

# 6. 浏览器访问
#    http://localhost/         # SPA 列表页
#    http://localhost/docs     # FastAPI Swagger UI（经 Nginx 反代）
```

---

## 端口与网络

| 服务 | 容器内 | 主机端口 | 备注 |
|------|--------|----------|------|
| frontend (Nginx) | 80 | **80** (`WEB_PORT`) | 唯一对外端口 |
| backend (FastAPI) | 8000 | — | 仅内部，不向主机暴露 |
| mysql | 3306 | **3307** (`DB_PORT`) | 主机用 3307 避开本地 3306 冲突 |

`/api/v1/*` 和 `/healthz` `/readyz` `/docs` `/openapi.json` 都被 Nginx 反向代理到 backend，前端代码无需感知后端地址。

---

## 启动顺序与健康检查

启动顺序由 compose 的 `depends_on` 与 healthcheck 控制：

1. **mysql 容器**：`mysqladmin ping` 健康检查（5s/次，最多 12 次 + 10s start_period）
2. **backend 容器**：等 mysql healthy，再走 `docker-entrypoint.sh`
   - 用 `pymysql` 重试 60 次确认连接（双保险）
   - 自动执行 `CREATE DATABASE IF NOT EXISTS` 确保 test 库也存在
   - 跑 `alembic upgrade head` 创建 / 升级表
   - `exec uvicorn app.main:app`
3. **frontend 容器**：等 backend 启动后启动 Nginx

> backend 容器日志中应能看到：`Running upgrade -> 6f9c1a8e0b21, init tickets table` 与 `Application startup complete.`

---

## 镜像构建说明

国内网络下默认使用：
- 基础镜像：`docker.m.daocloud.io/library/{python,node,nginx,mysql}:*`（DaoCloud 镜像加速）
- pip 源：`https://pypi.tuna.tsinghua.edu.cn/simple`（清华源）
- npm 源：`https://registry.npmmirror.com`

**国际网络环境**可改回 Docker Hub 官方源：去掉 `Dockerfile` 与 `docker-compose.yml` 中所有 `docker.m.daocloud.io/library/` 前缀。

---

## 常用运维命令

### 查看状态

```bash
docker compose --env-file .env.docker ps          # 状态
docker compose --env-file .env.docker logs -f      # 滚动日志（全部服务）
docker logs pa-backend --tail 50                   # 单服务日志
```

### 进入容器

```bash
docker compose --env-file .env.docker exec backend sh
docker compose --env-file .env.docker exec mysql \
  mysql -uroot -p"$DB_PASSWORD" project_alpha
```

### 重启 / 停止

```bash
# 软停（保留数据卷）
docker compose --env-file .env.docker stop
docker compose --env-file .env.docker start

# 删容器（保留数据卷，下次启动数据仍在）
docker compose --env-file .env.docker down

# 删容器并清空数据
docker compose --env-file .env.docker down -v
```

### 重新构建（代码改动后）

```bash
docker compose --env-file .env.docker up -d --build backend  # 仅重建后端
docker compose --env-file .env.docker up -d --build          # 全部
```

### 数据库迁移

```bash
# 容器内手动跑迁移（一般不需要，启动时已自动跑）
docker compose --env-file .env.docker exec backend alembic upgrade head

# 创建新迁移（开发期）
docker compose --env-file .env.docker exec backend \
  alembic revision --autogenerate -m "add foo column"
```

---

## 部署冒烟（DoD 验证）

阶段 9 已实测通过：

| # | 检查 | 实测 |
|---|------|------|
| 1 | 三容器健康启动 | ✅ all `Up` / mysql `(healthy)` |
| 2 | mysql 数据卷持久化 | ✅ `down` 后 `up` 数据保留（卷未删） |
| 3 | backend 自动迁移 | ✅ 容器日志含 `Running upgrade -> 6f9c1a8e0b21, init tickets table` |
| 4 | 前端 :80 访问 | ✅ `curl http://localhost/` HTTP 200 |
| 5 | API 经 Nginx 反代 | ✅ `curl http://localhost/api/v1/healthz` 返回 `{"code":0,"data":"ok"}` |
| 6 | integration-check 全过 | ✅ **21/21 PASS**（spec §15.4） |
| 7 | seed 数据可见 | ✅ 注入后 total=30，浏览器列表渲染 |
| 8 | 镜像体积合理 | ✅ backend 182 MB / frontend 48.6 MB |

```bash
# 一行复现联调脚本：
BACKEND_URL=http://localhost FRONTEND_URL=http://localhost \
  bash week1/tools/integration-check.sh
```

---

## 故障排查

| 现象 | 可能原因 | 处理 |
|------|----------|------|
| `:80 already in use` | 主机 80 被其他服务占用 | 改 `.env.docker` 的 `WEB_PORT=8080` 后重新 up |
| `mysql not ready after 60 attempts` | mysql 启动慢或密码不一致 | 检查 `.env.docker` 的 `DB_PASSWORD`；首次启动等 30s 再检查 |
| Nginx 502 Bad Gateway | backend 还没起来或挂了 | `docker logs pa-backend` 看 alembic 错误；mysql 没起来时常发生 |
| `failed to authorize: ...auth.docker.io` | 网络拉镜像失败 | 已默认走 `docker.m.daocloud.io`；若仍失败，配置 Docker Daemon 的 `registry-mirrors` |
| 前端样式丢失 | Nginx 配置未加载或 build 缺 dist | `docker exec pa-frontend ls /usr/share/nginx/html` 应有 `index.html` 与 `assets/` |
| 浏览器看不到数据 | seed 还没注入 | 跑步骤 4 的 mysql exec 命令注入 `seed.sql` |
| 数据消失 | 跑了 `down -v` | 数据卷被删；重新走 1~4 步骤 |

---

## 不在本阶段范围内

- **HTTPS / TLS**：生产前请加 Let's Encrypt（certbot）或反代到带 TLS 的负载均衡器
- **多副本 / 健康打散 / Kubernetes**
- **监控**：Prometheus + Grafana
- **日志聚合**：Loki / ELK
- **数据库主从 / 自动备份**
- **CI/CD**（用户决策不写 GitHub Actions）

后续若需 CI，可参考 plan §阶段 8 的 9 道质量门禁，组合成单一 workflow 文件即可。
