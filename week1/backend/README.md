# ProjectAlpha Backend

> FastAPI + SQLAlchemy + MySQL，使用 `uv` 管理依赖与虚拟环境。

## 目录结构

```
backend/
├── app/
│   ├── api/                    # 路由层
│   │   ├── v1/                 # /api/v1 业务路由
│   │   ├── deps.py             # 依赖注入（DB session 等）
│   │   └── health.py           # /healthz /readyz
│   ├── core/                   # 基础设施
│   │   ├── config.py           # Pydantic Settings 配置加载
│   │   ├── database.py         # SQLAlchemy engine + Session + Base
│   │   ├── exceptions.py       # 自定义业务异常
│   │   └── logging.py          # 日志配置
│   ├── middlewares/            # 中间件
│   │   ├── cors.py             # CORS 配置
│   │   └── error_handler.py    # 全局异常处理
│   ├── schemas/                # Pydantic 请求/响应模型
│   │   └── common.py           # ApiResponse / PageData 通用结构
│   ├── models/                 # SQLAlchemy 模型（阶段 2 起填充）
│   ├── utils/                  # 工具函数
│   └── main.py                 # FastAPI 应用入口
├── alembic/                    # 数据库迁移
│   ├── env.py
│   └── versions/
├── alembic.ini
├── tests/                      # pytest 测试
├── pyproject.toml              # uv / PEP 621 配置
├── .python-version
├── .env.example
└── README.md
```

## 环境要求

- Python ≥ 3.13
- uv ≥ 0.9
- 本地可访问的 MySQL 8.0（已验证：`root / 1qa2ws3ed @ localhost:3306`）

## 快速开始

```bash
# 1. 创建本地环境变量
cp .env.example .env

# 2. 安装依赖（uv 自动创建 .venv，生成 uv.lock）
uv sync

# 3. 运行数据库迁移
uv run alembic upgrade head

# 4. 启动开发服务
uv run uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

# 访问：
#   - http://localhost:8000/docs   Swagger UI
#   - http://localhost:8000/healthz   健康检查
#   - http://localhost:8000/readyz    就绪检查（含 DB ping）
```

## 常用命令

| 操作 | 命令 |
|------|------|
| 安装/同步依赖 | `uv sync` |
| 启动开发服务 | `uv run uvicorn app.main:app --reload` |
| 跑测试 | `uv run pytest` |
| 跑测试 + 覆盖率 | `uv run pytest --cov=app --cov-report=term-missing` |
| Lint | `uv run ruff check .` |
| 自动修复 | `uv run ruff check . --fix` |
| 格式化 | `uv run ruff format .` |
| 类型检查 | `uv run mypy app` |
| 新建迁移 | `uv run alembic revision --autogenerate -m "<msg>"` |
| 应用迁移 | `uv run alembic upgrade head` |
| 回滚一步 | `uv run alembic downgrade -1` |

## 当前阶段

✅ **阶段 1 - 后端基础框架** 已完成（参见 [实现计划](../../specs/week1/0002-implementation-plan.md#阶段-1---后端基础框架)）

⏳ **阶段 2 - Ticket CRUD** 待开始
