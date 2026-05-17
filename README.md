# ProjectAlpha

> 一个轻量级的 Ticket 管理工具 — 用于团队任务、问题、需求的统一跟踪与协作。

[![Status](https://img.shields.io/badge/status-design--phase-yellow)]()
[![Backend](https://img.shields.io/badge/backend-Python%203.11%2B%20%7C%20FastAPI-009688)]()
[![Frontend](https://img.shields.io/badge/frontend-React%2019%20%7C%20Vite%20%7C%20TS-61DAFB)]()
[![Database](https://img.shields.io/badge/database-MySQL%208.0-4479A1)]()
[![License](https://img.shields.io/badge/license-TBD-lightgrey)]()

---

## 📋 目录

- [项目简介](#-项目简介)
- [当前状态](#-当前状态)
- [核心功能](#-核心功能)
- [技术栈](#-技术栈)
- [系统架构](#-系统架构)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [开发计划](#-开发计划)
- [文档导航](#-文档导航)
- [开发规范](#-开发规范)
- [贡献指南](#-贡献指南)

---

## 📖 项目简介

**ProjectAlpha** 是一个面向中小型团队的轻量级项目管理工具，核心能力是对 **Ticket**（任务 / 问题 / 需求 / 待办事项）进行：

- ✅ 创建、查看、编辑、删除（CRUD）
- 🏷️ 通过标签灵活分类
- 📊 通过状态跟踪处理进度（待处理 → 处理中 → 已完成 → 已关闭）
- 🚦 通过优先级（低 / 中 / 高 / 紧急）区分紧急程度
- 👤 通过负责人明确责任归属
- 🔍 多维筛选、模糊搜索、排序

设计目标是 **结构清晰、易于扩展**，为后续权限、评论、附件、通知等模块预留空间。

---

## 🚧 当前状态

> **本项目目前处于「设计阶段」**，仅完成需求分析与技术方案设计。代码实现尚未开始。

| 阶段 | 状态 |
|------|------|
| 📝 需求与设计文档 | ✅ 已完成 |
| 🗺️ 实现计划 | ✅ 已完成 |
| 💻 后端实现 | ⏳ 待开始 |
| 🎨 前端实现 | ⏳ 待开始 |
| 🧪 测试 | ⏳ 待开始 |
| 🚀 部署 | ⏳ 待开始 |

预计总工期 **14 个工作日**，详见 [实现计划](specs/weak1/0002-implementation-plan.md)。

---

## ✨ 核心功能

### Ticket 管理
- 创建、查看、编辑、删除 Ticket
- 设置 / 修改状态、优先级、负责人、标签
- 自动记录创建与更新时间

### 分类与筛选
- 按 **状态**（多选）筛选
- 按 **优先级**（多选）筛选
- 按 **负责人** 筛选
- 按 **标签** 筛选
- 按 **关键字** 模糊搜索标题与描述（300ms 防抖 / 最少 2 字符）
- 按 **创建时间 / 更新时间** 升序或降序排列

### 状态流转

```
待处理 ──► 处理中 ──► 已完成 ──► 已关闭
   │          │                     ▲
   └──────────┴─────────────────────┘
        (任意状态可直接关闭)

已关闭 ──► 待处理   (重新打开)
```

详细的功能验收标准见 [需求与设计文档 §3](specs/weak1/0001-spec.md#3-功能需求)。

---

## 🛠️ 技术栈

### 前端

| 类别 | 技术 | 用途 |
|------|------|------|
| 框架 | **React 19** | 组件化开发 |
| 构建 | **Vite** | 快速热更新 |
| 语言 | **TypeScript** (strict) | 类型安全 |
| UI | **TailwindCSS + Shadcn UI** | 样式与组件 |
| 路由 | **React Router v6** | SPA 路由 |
| HTTP | **Axios** | API 请求 |
| Mock | **MSW** | 联调前的接口模拟 |

### 后端

| 类别 | 技术 | 用途 |
|------|------|------|
| 语言 | **Python 3.11+** | 全量类型注解 |
| 框架 | **FastAPI** | 异步高性能 Web 框架 |
| ASGI | **Uvicorn** | ASGI 服务器 |
| ORM | **SQLAlchemy 2.x** | 数据库 ORM |
| 迁移 | **Alembic** | 数据库版本管理 |
| 校验 | **Pydantic v2** | 类型驱动的数据校验 |
| 驱动 | **PyMySQL** | MySQL 连接驱动 |
| 依赖 | **Poetry** | 依赖与虚拟环境管理 |

### 数据库 & 基础设施

| 类别 | 技术 |
|------|------|
| 数据库 | **MySQL 8.0** |
| 容器 | **Docker + Docker Compose** |
| 反向代理 | **Nginx** |
| API 文档 | **Swagger UI** (FastAPI 自动生成) |

### 质量保障

| 类别 | 工具 |
|------|------|
| 后端测试 | pytest + pytest-asyncio + httpx |
| 前端测试 | Vitest + React Testing Library |
| 后端 Lint | Ruff + Black + isort + mypy |
| 前端 Lint | ESLint + Prettier + tsc |
| Git 规范 | Conventional Commits + pre-commit |

---

## 🏗️ 系统架构

```
┌────────────────────┐      ┌────────────────────┐      ┌────────────────────┐
│                    │      │                    │      │                    │
│    前端 (SPA)      │ HTTP │    后端 (API)      │  SQL │   MySQL 数据库      │
│  React + Vite      │◄────►│ Python + FastAPI   │◄────►│                    │
│  端口: 5173        │ REST │   端口: 8000       │      │   端口: 3306       │
│                    │      │                    │      │                    │
└────────────────────┘      └────────────────────┘      └────────────────────┘
         │                            │
         ▼                            ▼
   浏览器 (Chrome)           Swagger UI (自动生成)
                              http://localhost:8000/docs
```

部署架构（生产）：

```
┌──────────────────────────────────────────────────────────┐
│                      服务器 / 云主机                       │
│                                                            │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐    │
│  │  Nginx   │    │   Uvicorn    │    │    MySQL     │    │
│  │ :80/:443 │───►│  + FastAPI   │───►│    :3306     │    │
│  │          │    │    :8000     │    │              │    │
│  │ 静态资源 │    │   API 服务   │    │   数据存储   │    │
│  │ +反向代理│    │   (Python)   │    │              │    │
│  └──────────┘    └──────────────┘    └──────────────┘    │
│                                                            │
└──────────────────────────────────────────────────────────┘
```

详见 [需求与设计文档 §8](specs/weak1/0001-spec.md#8-技术架构)。

---

## 📁 项目结构

```
project-alpha/
├── frontend/               # 前端 (React + Vite + TS) — 待创建
│   └── ...
├── backend/                # 后端 (Python + FastAPI) — 待创建
│   ├── app/
│   │   ├── api/            # 路由层
│   │   ├── services/       # 业务逻辑层
│   │   ├── models/         # SQLAlchemy 模型
│   │   ├── schemas/        # Pydantic 模型
│   │   ├── core/           # 配置 / 数据库 / 异常
│   │   ├── middlewares/    # 中间件
│   │   └── main.py         # 应用入口
│   ├── tests/              # 测试
│   ├── alembic/            # 数据库迁移
│   └── pyproject.toml
├── database/               # 数据库脚本 — 待创建
│   ├── init.sql            # 初始化 DDL
│   └── seeds/              # 种子数据
├── docs/                   # 项目文档 — 待创建
├── specs/                  # 规范文档 ✅
│   └── weak1/
│       ├── instructions.md          # 原始需求
│       ├── 0001-spec.md             # 需求与设计文档
│       └── 0002-implementation-plan.md  # 实现计划
├── docker-compose.yml      # 容器编排 — 待创建
├── .gitignore              ✅
└── README.md               ✅
```

完整目录设计见 [需求与设计文档 §9](specs/weak1/0001-spec.md#9-项目目录结构)。

---

## 🚀 快速开始

> ⚠️ 以下命令为项目实现后的目标使用方式。当前代码尚未实现，命令仅供参考。

### 环境要求

| 软件 | 版本 |
|------|------|
| Python | 3.11+ |
| Node.js | 20+ |
| MySQL | 8.0+ |
| Docker | 24+ |
| Docker Compose | 2.x+ |

### 方式一：Docker 一键启动（推荐）

```bash
# 1. 克隆项目
git clone <repo-url> project-alpha
cd project-alpha

# 2. 配置环境变量
cp .env.example .env
# 编辑 .env 设置数据库密码等

# 3. 启动所有服务（前端 + 后端 + MySQL）
docker-compose up -d --build

# 4. 验证服务
curl http://localhost/api/v1/tickets
# 访问 Swagger UI: http://localhost/docs
# 访问 Web 页面: http://localhost
```

### 方式二：本地开发

**1. 启动 MySQL**

```bash
docker run -d \
  --name pa-mysql-dev \
  -e MYSQL_ROOT_PASSWORD=root \
  -e MYSQL_DATABASE=project_alpha \
  -p 3306:3306 \
  -v pa-mysql-data:/var/lib/mysql \
  mysql:8.0
```

**2. 启动后端**

```bash
cd backend
poetry install
cp .env.example .env

# 数据库迁移
poetry run alembic upgrade head

# 启动开发服务器
poetry run uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

# 访问 http://localhost:8000/docs 查看 Swagger UI
```

**3. 启动前端**

```bash
cd frontend
npm install
cp .env.example .env

npm run dev

# 访问 http://localhost:5173
```

### 常用命令

**后端：**

```bash
poetry run pytest                    # 跑所有测试
poetry run pytest --cov=app          # 含覆盖率
poetry run ruff check .              # Lint
poetry run mypy app                  # 类型检查
poetry run alembic revision -m "..." # 新增迁移
poetry run alembic upgrade head      # 应用迁移
```

**前端：**

```bash
npm run dev                          # 开发服务器
npm run build                        # 生产构建
npm run lint                         # Lint
npm run typecheck                    # 类型检查
npm test                             # 测试
npm run test:coverage                # 含覆盖率
```

---

## 🗺️ 开发计划

预计 **14 个工作日**，分为 10 个阶段：

| 阶段 | 主题 | 周期 |
|------|------|------|
| 0 | 项目启动与环境准备 | Day 1 |
| 1 | 后端基础框架 | Day 1~2 |
| 2 | 后端 Ticket CRUD | Day 2~3 |
| 3 | 后端筛选 / 搜索 / 排序 / 分页 | Day 4 |
| 4 | 前端基础框架 | Day 5 上午 |
| 5 | 前端 Ticket 列表页 | Day 5~6 |
| 6 | 前端详情与表单 | Day 7~8 |
| 7 | 前后端联调 | Day 9~10 |
| 8 | 测试与质量保障 | Day 11~12 |
| 9 | 部署上线 | Day 13~14 |

详细任务分解、关键路径、风险登记见 [实现计划](specs/weak1/0002-implementation-plan.md)。

---

## 📚 文档导航

| 文档 | 说明 |
|------|------|
| [原始需求](specs/weak1/instructions.md) | 项目最初的需求描述 |
| [需求与设计文档](specs/weak1/0001-spec.md) | 完整的功能需求、数据模型、API 设计、UI 原型、部署方案 |
| [实现计划](specs/weak1/0002-implementation-plan.md) | 14 天分阶段任务、WBS、风险登记、DoD 验收清单 |
| Swagger UI | 后端启动后访问 `/docs`（待实现） |

---

## 📐 开发规范

### 代码规范

| 项目 | 规范 |
|------|------|
| 前端语言 | TypeScript（strict 模式） |
| 后端语言 | Python 3.11+（全量类型注解） |
| 前端格式化 | ESLint + Prettier |
| 后端格式化 | Ruff + Black + isort |
| 静态检查 | tsc --noEmit / mypy strict |
| 命名约定 | 前端 camelCase / PascalCase；后端 snake_case / PascalCase；常量 UPPER_SNAKE_CASE |

### Git 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/)：

```
feat:     新增功能
fix:      修复 Bug
docs:     文档变更
style:    代码格式变化（不影响功能）
refactor: 重构
test:     测试相关
chore:    构建 / 配置 / 依赖等
```

示例：

```
feat: 添加 Ticket 创建接口
fix: 修复状态筛选不生效的问题
docs: 更新 API 接口文档
test: 补充 ticket_service 单元测试
```

### 分支策略

```
main      ← 生产分支（受保护）
  └── dev ← 开发分支
        └── feature/<name> ← 功能分支
```

### 测试覆盖率要求

| 层级 | 行覆盖率 | 分支覆盖率 |
|------|----------|-----------|
| 后端 Service 层 | ≥ 80% | ≥ 70% |
| 后端 API 层 | ≥ 70% | ≥ 60% |
| 前端核心组件 | ≥ 70% | ≥ 60% |

---

## 🤝 贡献指南

### 提交 PR 流程

1. Fork 仓库，从 `dev` 分支创建 `feature/<your-feature>`
2. 在本地开发并补充测试
3. 确保所有质量门禁通过：
   - 后端：`pytest`、`ruff check`、`mypy`
   - 前端：`npm test`、`npm run lint`、`npm run typecheck`、`npm run build`
4. 按 Conventional Commits 规范提交
5. 提 PR 到 `dev` 分支，至少 1 人 Review
6. 通过后合并

### 接口变更

⚠️ 接口契约变更必须 **先改 Swagger 文档**，再通知前端，并同步更新 [需求与设计文档](specs/weak1/0001-spec.md)。

---

## 🔭 后续扩展方向

不在本期范围内，但已预留扩展空间：

- 🔐 用户认证与权限管理（JWT + 角色）
- 💬 Ticket 评论功能（支持 @提及）
- 📎 文件附件上传
- 📜 操作日志 / 变更历史
- 🔔 通知系统（邮件 / 站内信）
- 📊 仪表盘与统计报表
- 🗂️ 批量操作
- 📤 导入 / 导出（CSV / Excel）

---

## 📄 License

License: TBD（待定）
