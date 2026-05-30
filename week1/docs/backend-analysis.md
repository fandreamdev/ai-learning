# ProjectAlpha Backend 代码分析文档

> 本文档对 `week1/backend` 项目进行全方位深度分析，可作为从零开发具有相同功能项目的完整参考指南。

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术栈](#2-技术栈)
3. [目录结构详解](#3-目录结构详解)
4. [整体架构设计](#4-整体架构设计)
5. [核心模块分析](#5-核心模块分析)
6. [API 接口文档](#6-api-接口文档)
7. [数据库设计](#7-数据库设计)
8. [请求处理流程](#8-请求处理流程)
9. [关键设计模式与理念](#9-关键设计模式与理念)
10. [依赖管理](#10-依赖管理)
11. [测试架构](#11-测试架构)
12. [Docker 部署](#12-docker-部署)
13. [从零开发指南](#13-从零开发指南)
14. [开发规范](#14-开发规范)

---

## 1. 项目概述

### 1.1 项目定位

ProjectAlpha Backend 是一个**轻量级 Ticket（工单）管理工具的后端 API 服务**，采用现代 Python Web 技术栈构建。

### 1.2 核心功能

| 功能模块 | 描述 |
|---------|------|
| Ticket CRUD | 工单的创建、读取、更新、删除 |
| 状态管理 | 4 种状态（open/in_progress/done/closed）及合法流转控制 |
| 优先级管理 | 4 种优先级（low/medium/high/urgent） |
| 标签系统 | 支持多标签（小写、去重、最多 10 项） |
| 分页查询 | 支持多条件筛选、关键词搜索、排序分页 |
| 聚合查询 | 获取所有已使用标签、负责人列表 |
| 健康检查 | 存活探针（/healthz）、就绪探针（/readyz） |

---

## 2. 技术栈

### 2.1 核心技术选型

| 层级 | 技术选型 | 版本要求 | 说明 |
|------|---------|---------|------|
| **Web 框架** | FastAPI | >=0.136.1 | 异步高性能 API 框架，自动 OpenAPI 文档 |
| **ASGI 服务器** | Uvicorn | >=0.47.0 | ASGI 规范实现，支持热重载 |
| **ORM** | SQLAlchemy | >=2.0.49 | 2.0 版本新特性：mapped_column, Mapped 类型注解 |
| **数据库** | MySQL 8.0 | - | 使用 PyMySQL 驱动 |
| **迁移工具** | Alembic | >=1.18.4 | 数据库版本化管理 |
| **数据验证** | Pydantic | >=2.13.4 | 2.x 版本，支持泛型 |
| **配置管理** | pydantic-settings | >=2.14.1 | 基于 Pydantic 的配置加载 |
| **依赖管理** | uv | - | 现代 Python 包管理工具 |

### 2.2 开发工具链

| 工具 | 用途 |
|-----|------|
| pytest | 测试框架 |
| pytest-asyncio | 异步测试支持 |
| pytest-cov | 测试覆盖率 |
| httpx | HTTP 客户端（用于测试） |
| ruff | Lint + 格式化（集成工具） |
| black | 代码格式化 |
| mypy | 静态类型检查 |

### 2.3 Python 版本要求

```
requires-python = ">=3.13"
```

使用 Python 3.13 及以上版本，充分利用最新语法特性（`from __future__ import annotations`）。

---

## 3. 目录结构详解

### 3.1 完整目录树

```
week1/backend/
├── app/                              # 应用核心代码
│   ├── __init__.py
│   ├── main.py                       # FastAPI 应用入口，create_app() 工厂函数
│   │
│   ├── api/                          # 路由层（HTTP 关注点）
│   │   ├── __init__.py
│   │   ├── deps.py                   # 依赖注入（DB Session）
│   │   ├── health.py                 # 健康检查路由
│   │   └── v1/                      # API v1 版本
│   │       ├── __init__.py           # 路由聚合（api_v1.router）
│   │       ├── tickets.py            # Ticket CRUD 路由
│   │       ├── aggregations.py       # 聚合查询路由
│   │       └── _query_utils.py       # 查询参数辅助函数
│   │
│   ├── core/                         # 基础设施层
│   │   ├── __init__.py
│   │   ├── config.py                # Pydantic Settings 配置管理
│   │   ├── database.py              # SQLAlchemy 引擎与会话
│   │   ├── exceptions.py            # 自定义业务异常体系
│   │   ├── logging.py               # 日志配置
│   │   └── constants.py             # 业务常量（状态流转表）
│   │
│   ├── middlewares/                 # 中间件
│   │   ├── __init__.py
│   │   ├── cors.py                  # CORS 配置
│   │   └── error_handler.py          # 全局异常处理
│   │
│   ├── models/                      # ORM 模型层
│   │   ├── __init__.py
│   │   └── ticket.py                # Ticket ORM 模型
│   │
│   ├── schemas/                     # Pydantic Schema 层
│   │   ├── __init__.py
│   │   ├── ticket.py                # Ticket 相关 Schema（入参/出参）
│   │   └── common.py                 # 通用响应结构
│   │
│   ├── services/                    # 业务逻辑层
│   │   ├── __init__.py
│   │   ├── ticket_service.py        # Ticket 业务逻辑
│   │   └── aggregation_service.py    # 聚合查询业务逻辑
│   │
│   └── utils/                       # 工具函数
│       ├── __init__.py
│       └── responses.py             # 响应辅助函数
│
├── alembic/                          # 数据库迁移
│   ├── env.py                       # Alembic 配置
│   ├── script.py.mako               # 迁移脚本模板
│   └── versions/                    # 迁移版本文件
│       └── 2026_05_17_1730-6f9c1a8e0b21_init_tickets.py
│
├── alembic.ini                      # Alembic 配置文件
├── pyproject.toml                   # 项目配置（PEP 621 标准）
├── .env.example                     # 环境变量示例
├── Dockerfile                       # 多阶段构建 Dockerfile
├── docker-entrypoint.sh             # Docker 启动脚本
├── .dockerignore                    # Docker 忽略文件
├── uv.lock                          # 依赖锁定文件
├── README.md                        # 项目说明
│
├── scripts/                         # 辅助脚本
│   ├── __init__.py
│   └── perf_benchmark.py            # 性能基准测试脚本
│
└── tests/                           # 测试代码
    ├── __init__.py
    ├── conftest.py                  # pytest 配置与共用 fixture
    ├── test_health.py               # 健康检查测试
    │
    ├── unit/                       # 单元测试
    │   ├── __init__.py
    │   ├── test_ticket_service.py   # Service 层单元测试
    │   ├── test_ticket_schema.py     # Schema 验证测试
    │   ├── test_list_query.py       # 列表查询测试
    │   └── test_ticket_service_list.py
    │
    └── integration/                 # 集成测试
        ├── __init__.py
        ├── test_aggregations_api.py # 聚合 API 测试
        ├── test_ticket_api.py       # Ticket API 测试
        └── test_ticket_list_api.py  # 列表 API 测试
```

### 3.2 目录结构设计理念

项目采用**分层架构（Layered Architecture）**，各层职责清晰分离：

```
┌─────────────────────────────────────────┐
│         API Routes (路由层)              │  ← HTTP 请求入口
├─────────────────────────────────────────┤
│         Services (业务逻辑层)             │  ← 核心业务逻辑
├─────────────────────────────────────────┤
│         Models/Schemas (数据层)           │  ← ORM + 数据验证
├─────────────────────────────────────────┤
│         Core (基础设施层)                 │  ← 配置/数据库/异常
└─────────────────────────────────────────┘
```

**设计原则**：
- **路由层**：只处理 HTTP 关注点（解析请求、调用服务、返回响应）
- **服务层**：纯业务逻辑，不依赖 FastAPI（可独立测试）
- **数据层**：SQLAlchemy ORM + Pydantic Schema 分离

---

## 4. 整体架构设计

### 4.1 应用启动流程

```python
# app/main.py

def create_app() -> FastAPI:
    """构建并返回 FastAPI 实例"""
    setup_logging()                          # 1. 初始化日志
    
    app = FastAPI(                           # 2. 创建 FastAPI 实例
        title="ProjectAlpha API",
        version="0.1.0",
        description="轻量级 Ticket 管理工具 API",
        docs_url="/docs",                    # Swagger UI
        redoc_url="/redoc",                  # ReDoc 文档
        openapi_url="/openapi.json",
        debug=settings.app_debug,
    )
    
    add_cors_middleware(app)                 # 3. 注册 CORS 中间件
    register_exception_handlers(app)         # 4. 注册全局异常处理
    
    app.include_router(health_router)        # 5. 注册健康检查路由
    app.include_router(api_v1.router)        # 6. 注册 API 路由
    
    return app

app: FastAPI = create_app()                  # 模块级应用实例
```

### 4.2 请求生命周期

```
HTTP 请求
    │
    ▼
┌─────────────────┐
│   CORS Middleware │  检查 Origin，允许跨域请求
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Exception Handler │  捕获并统一处理异常
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Route Matching │  匹配路由（如 /api/v1/tickets）
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Dependency Injection │  注入 DB Session
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Route Handler │  调用 Service 层
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Service Layer │  执行业务逻辑
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   ORM / Database │  数据库操作
└─────────────────┘
    │
    ▼
响应数据
```

---

## 5. 核心模块分析

### 5.1 配置管理 (`app/core/config.py`)

#### 5.1.1 为什么使用 Pydantic Settings

```python
class Settings(BaseSettings):
    """应用全局配置"""
    
    model_config = SettingsConfigDict(
        env_file=".env",              # 读取 .env 文件
        env_file_encoding="utf-8",
        case_sensitive=False,         # 环境变量不区分大小写
        extra="ignore",               # 忽略额外字段
    )
```

**设计优势**：
1. **类型安全**：配置值有类型注解，IDE 自动补全
2. **自动验证**：Pydantic 自动校验配置值合法性
3. **环境变量优先**：可被环境变量覆盖，适合容器化部署
4. **单例模式**：使用 `@lru_cache` 确保配置只加载一次

#### 5.1.2 配置字段说明

| 字段 | 类型 | 默认值 | 说明 |
|-----|------|-------|------|
| `app_env` | str | "development" | 运行环境 |
| `app_host` | str | "0.0.0.0" | 监听地址 |
| `app_port` | int | 8000 | 监听端口 |
| `app_debug` | bool | True | 调试模式 |
| `db_host` | str | "localhost" | 数据库地址 |
| `db_port` | int | 3306 | 数据库端口 |
| `db_user` | str | "root" | 数据库用户 |
| `db_password` | str | "" | 数据库密码 |
| `db_name` | str | "project_alpha" | 数据库名 |
| `db_name_test` | str | "project_alpha_test" | 测试数据库名 |
| `cors_allow_origins` | str | 多个 localhost 地址 | CORS 允许的源 |
| `log_level` | str | "INFO" | 日志级别 |

#### 5.1.3 计算属性

```python
@property
def sqlalchemy_url(self) -> str:
    """生成 SQLAlchemy 连接串"""
    if self.database_url:
        return self.database_url
    return (
        f"mysql+pymysql://{self.db_user}:{self.db_password}"
        f"@{self.db_host}:{self.db_port}/{self.db_name}?charset=utf8mb4"
    )

@property
def cors_origins_list(self) -> list[str]:
    """解析 CORS 允许列表"""
    return [o.strip() for o in self.cors_allow_origins.split(",") if o.strip()]
```

---

### 5.2 数据库层 (`app/core/database.py`)

#### 5.2.1 Declarative Base

```python
class Base(DeclarativeBase):
    """所有 ORM 模型的基类"""
    pass
```

所有 ORM 模型继承 `Base`，使用 SQLAlchemy 2.0 的新写法。

#### 5.2.2 引擎配置

```python
def _build_engine() -> Engine:
    """构建 SQLAlchemy Engine"""
    return create_engine(
        settings.sqlalchemy_url,
        pool_pre_ping=True,      # 连接前 ping，检查连通性
        pool_recycle=3600,       # 1小时回收连接
        future=True,             # 使用 2.0 特性
        echo=False,              # 不打印 SQL
    )
```

**配置说明**：
- `pool_pre_ping=True`：每次使用连接前先 ping 数据库，确保连接有效
- `pool_recycle=3600`：防止 MySQL 的 8 小时空闲超时
- `echo=False`：生产环境关闭 SQL 日志

#### 5.2.3 Session 工厂

```python
SessionLocal = sessionmaker(
    bind=engine,
    autoflush=False,        # 不自动 flush，手动控制
    autocommit=False,       # 不自动提交，手动控制
    expire_on_commit=False, # 提交后不失效对象
)
```

#### 5.2.4 依赖注入

```python
def get_db() -> Generator[Session]:
    """请求级数据库 Session 生成器"""
    session: Session = SessionLocal()
    try:
        yield session
    finally:
        session.close()
```

使用 FastAPI 的 `Depends()` 实现请求级 Session 管理。

---

### 5.3 Ticket ORM 模型 (`app/models/ticket.py`)

#### 5.3.1 模型定义

```python
class Ticket(Base):
    __tablename__ = "tickets"
    
    id: Mapped[int] = mapped_column(INTEGER(unsigned=True), primary_key=True)
    title: Mapped[str] = mapped_column(String(200), nullable=False)
    description: Mapped[str | None] = mapped_column(Text)
    status: Mapped[str] = mapped_column(Enum(...), default="open", index=True)
    priority: Mapped[str] = mapped_column(Enum(...), default="medium", index=True)
    assignee: Mapped[str | None] = mapped_column(String(100), index=True)
    tags: Mapped[list[str] | None] = mapped_column(JSON)
    created_at: Mapped[datetime] = mapped_column(DateTime, server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime, server_default=func.now(), onupdate=func.now())
```

#### 5.3.2 为什么使用 Mapped 类型注解

SQLAlchemy 2.0 引入的 `Mapped` 类型注解让模型定义更清晰：
- 明确声明 Python 类型
- 可读性强
- 与类型检查工具兼容

#### 5.3.3 字段详细说明

| 字段 | 类型 | 默认值 | 索引 | 说明 |
|-----|------|-------|-----|------|
| `id` | INTEGER | 自增 | PK | 主键 |
| `title` | VARCHAR(200) | - | - | 标题，必填 |
| `description` | TEXT | NULL | - | 描述，可为空 |
| `status` | ENUM | "open" | ✓ | 状态，有索引 |
| `priority` | ENUM | "medium" | ✓ | 优先级，有索引 |
| `assignee` | VARCHAR(100) | NULL | ✓ | 负责人，有索引 |
| `tags` | JSON | NULL | - | 标签列表 |
| `created_at` | DATETIME | NOW() | ✓ | 创建时间 |
| `updated_at` | DATETIME | NOW() | ✓ | 更新时间 |

#### 5.3.4 索引设计

```python
__table_args__ = (
    Index("ft_title_desc", "title", "description", mysql_prefix="FULLTEXT"),  # 全文索引
    {"mysql_engine": "InnoDB", "mysql_charset": "utf8mb4"},                  # 表级配置
)
```

- **普通索引**：status, priority, assignee, created_at, updated_at
- **全文索引**：title + description 组合全文搜索

---

### 5.4 业务异常体系 (`app/core/exceptions.py`)

#### 5.4.1 异常类层次

```
BusinessException (基类)
├── TicketNotFound (40401)
├── InvalidStatusTransition (40002)
└── ValidationError (40001)
```

#### 5.4.2 为什么设计业务异常体系

1. **分离关注点**：HTTP 异常 vs 业务异常分离
2. **统一响应格式**：所有异常统一转换为 `{code, message, data}` 格式
3. **可扩展**：新增业务异常只需继承 `BusinessException`

#### 5.4.3 异常设计细节

```python
class BusinessException(Exception):
    code: int = 50001           # 业务错误码
    message: str = "internal error"  # 错误信息
    http_status: int = 500      # HTTP 状态码
    
    def __init__(self, message: str | None = None) -> None:
        if message is not None:
            self.message = message
        super().__init__(self.message)
```

**设计要点**：
- 类属性定义默认值，实例可覆盖
- 支持自定义错误信息
- 错误码设计：前3位 HTTP 状态，后2位业务细分

---

### 5.5 状态流转控制 (`app/core/constants.py`)

#### 5.5.1 状态流转规则

```python
STATUS_TRANSITIONS: dict[TicketStatus, frozenset[TicketStatus]] = {
    TicketStatus.open:        frozenset({TicketStatus.in_progress, TicketStatus.closed}),
    TicketStatus.in_progress:  frozenset({TicketStatus.done, TicketStatus.closed}),
    TicketStatus.done:         frozenset({TicketStatus.closed}),
    TicketStatus.closed:       frozenset({TicketStatus.open}),
}
```

#### 5.5.2 流转图示

```
open ──────────────► in_progress ──────────────► done
  │                        │                         │
  │                        │                         │
  ▼                        ▼                         ▼
closed ◄────────────── closed ◄────────────────── closed
  │
  │
  └────────────────────────────────────────────────► open (重新打开)
```

#### 5.5.3 为什么使用 frozenset

- **不可变**：状态流转规则不应该被修改
- **查询高效**：O(1) 复杂度检查目标状态是否允许
- **类型安全**：静态类型检查器可以验证

---

### 5.6 Pydantic Schema (`app/schemas/`)

#### 5.6.1 统一响应结构 (`common.py`)

```python
class ApiResponse[T](BaseModel):
    """统一 API 响应信封"""
    code: int = 0                              # 0 表示成功
    message: str = "success"                   # 描述信息
    data: T | None = None                      # 业务数据

class PageData[T](BaseModel):
    """分页响应数据"""
    items: list[T] = Field(default_factory=list)
    total: int = 0
    page: int = 1
    page_size: int = 20
```

**设计要点**：使用泛型 `T` 支持不同数据类型。

#### 5.6.2 Ticket Schema (`ticket.py`)

**枚举定义**：

```python
class TicketStatus(StrEnum):
    open = "open"
    in_progress = "in_progress"
    done = "done"
    closed = "closed"

class TicketPriority(StrEnum):
    low = "low"
    medium = "medium"
    high = "high"
    urgent = "urgent"
```

**为什么使用 StrEnum**：自动转换为字符串，便于 JSON 序列化。

**标签规范化**：

```python
def _normalize_tag_list(value: object) -> list[str] | None:
    """对 tags 列表做规范化"""
    if value is None:
        return None
    if not isinstance(value, list):
        raise ValueError("tags 必须为字符串数组")
    
    seen: set[str] = set()
    result: list[str] = []
    for raw in value:
        item = raw.strip().lower()           # strip + 小写
        if item not in seen:                 # 去重
            seen.add(item)
            result.append(item)
    return result
```

**设计要点**：
1. 自动 `strip()` 去除首尾空格
2. 自动 `lower()` 转小写
3. 自动去重
4. 长度校验（1-20字符）
5. 数量限制（最多10项）

---

### 5.7 服务层 (`app/services/`)

#### 5.7.1 Ticket Service (`ticket_service.py`)

**设计原则**：
- 函数式风格，不依赖请求上下文
- 接受 `Session` + 领域入参
- 抛 `BusinessException` 子类
- 不做参数二次校验（已由 Schema 保证）

**核心函数**：

```python
def create_ticket(db: Session, data: TicketCreate) -> Ticket
def get_ticket(db: Session, ticket_id: int) -> Ticket
def update_ticket(db: Session, ticket_id: int, data: TicketUpdate) -> Ticket
def update_status(db: Session, ticket_id: int, target: TicketStatus) -> Ticket
def delete_ticket(db: Session, ticket_id: int) -> None
def list_tickets(db: Session, q: TicketListQuery) -> tuple[list[Ticket], int]
```

**状态流转校验**：

```python
def _validate_transition(current: str, target: TicketStatus) -> None:
    """校验状态流转是否合法"""
    current_enum = TicketStatus(current)
    if current_enum == target:
        return  # 同状态幂等
    
    allowed = STATUS_TRANSITIONS.get(current_enum, frozenset())
    if target not in allowed:
        raise InvalidStatusTransition(...)
```

#### 5.7.2 聚合服务 (`aggregation_service.py`)

```python
def list_tags(db: Session) -> list[str]:
    """聚合所有 ticket.tags 的并集，去重 + 字典序排序"""
    rows = db.scalars(select(Ticket.tags)).all()
    # Python 层聚合（简单实现）
    seen: set[str] = set()
    for tags in rows:
        if tags:
            for tag in tags:
                if tag not in seen:
                    seen.add(tag)
                    result.append(tag)
    return sorted(result)

def list_assignees(db: Session) -> list[str]:
    """聚合所有非空负责人，去重 + 字典序排序"""
    rows = db.scalars(
        select(Ticket.assignee)
        .where(Ticket.assignee.is_not(None))
        .distinct()
    ).all()
    return sorted(name for name in rows if name)
```

---

### 5.8 中间件 (`app/middlewares/`)

#### 5.8.1 CORS 中间件

```python
def add_cors_middleware(app: FastAPI) -> None:
    app.add_middleware(
        CORSMiddleware,
        allow_origins=settings.cors_origins_list,
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
```

**为什么配置 CORS**：
- 前端开发服务器（localhost:5173）需要跨域访问 API
- `allow_credentials=True` 允许携带 Cookie
- `allow_origins` 明确白名单，不使用 `*`

#### 5.8.2 全局异常处理

```python
def register_exception_handlers(app: FastAPI) -> None:
    @app.exception_handler(BusinessException)
    async def _business_exc_handler(...)  # 业务异常 → 统一格式
    
    @app.exception_handler(RequestValidationError)
    async def _validation_exc_handler(...)  # Pydantic 校验失败 → 40001
    
    @app.exception_handler(StarletteHTTPException)
    async def _http_exc_handler(...)  # HTTP 异常 → 统一格式
    
    @app.exception_handler(Exception)
    async def _unhandled_exc_handler(...)  # 未处理异常 → 50001 + 日志
```

---

## 6. API 接口文档

### 6.1 基础信息

| 项目 | 值 |
|-----|---|
| Base URL | `http://localhost:8000` |
| API 版本 | v1 |
| API 前缀 | `/api/v1` |
| 文档地址 | `/docs` (Swagger UI), `/redoc` (ReDoc) |

### 6.2 统一响应格式

**成功响应**：
```json
{
  "code": 0,
  "message": "success",
  "data": { ... }
}
```

**分页响应**：
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "items": [...],
    "total": 100,
    "page": 1,
    "page_size": 20
  }
}
```

**错误响应**：
```json
{
  "code": 40401,
  "message": "Ticket 不存在",
  "data": null
}
```

### 6.3 健康检查接口

#### GET /healthz - 存活探针

**说明**：检查应用是否运行。

**响应示例**：
```json
{
  "code": 0,
  "message": "success",
  "data": "ok"
}
```

#### GET /readyz - 就绪探针

**说明**：检查应用及数据库是否就绪。

**响应示例**：
```json
{
  "code": 0,
  "message": "success",
  "data": "ready"
}
```

### 6.4 Ticket 接口

#### GET /api/v1/tickets - 分页查询列表

**请求参数**（Query Parameters）：

| 参数 | 类型 | 默认值 | 必填 | 说明 |
|-----|------|-------|-----|------|
| `status` | string[] | - | 否 | 按状态筛选，支持逗号分隔或重复参数 |
| `priority` | string[] | - | 否 | 按优先级筛选 |
| `assignee` | string | - | 否 | 按负责人精确筛选 |
| `tag` | string | - | 否 | 按单个标签筛选 |
| `keyword` | string | - | 否 | 模糊搜索标题/描述（最少2字符） |
| `sort_by` | string | `created_at` | 否 | 排序字段：`created_at` 或 `updated_at` |
| `sort_order` | string | `desc` | 否 | 排序方向：`asc` 或 `desc` |
| `page` | int | 1 | 否 | 页码，从1开始 |
| `page_size` | int | 20 | 否 | 每页条数，1~100 |

**请求示例**：
```
GET /api/v1/tickets?status=open&priority=high&page=1&page_size=10
GET /api/v1/tickets?keyword=bug&sort_by=created_at&sort_order=desc
```

**响应示例**：
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "items": [
      {
        "id": 1,
        "title": "修复登录bug",
        "description": "用户无法登录",
        "status": "open",
        "priority": "high",
        "assignee": "张三",
        "tags": ["bug", "urgent"],
        "created_at": "2026-05-17T10:30:00",
        "updated_at": "2026-05-17T10:30:00"
      }
    ],
    "total": 1,
    "page": 1,
    "page_size": 10
  }
}
```

---

#### POST /api/v1/tickets - 创建 Ticket

**请求体**（JSON）：

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|-----|------|-----|-------|------|
| `title` | string | **是** | - | 标题，1-200字符 |
| `description` | string | 否 | null | 描述 |
| `priority` | string | 否 | `medium` | 优先级：low/medium/high/urgent |
| `assignee` | string | 否 | null | 负责人 |
| `tags` | string[] | 否 | [] | 标签，最多10项，每项1-20字符 |

**请求示例**：
```json
{
  "title": "新功能开发",
  "description": "开发用户管理模块",
  "priority": "medium",
  "assignee": "李四",
  "tags": ["feature", "backend"]
}
```

**响应示例**（201 Created）：
```json
{
  "code": 0,
  "message": "created",
  "data": {
    "id": 1,
    "title": "新功能开发",
    "description": "开发用户管理模块",
    "status": "open",
    "priority": "medium",
    "assignee": "李四",
    "tags": ["feature", "backend"],
    "created_at": "2026-05-17T10:30:00",
    "updated_at": "2026-05-17T10:30:00"
  }
}
```

---

#### GET /api/v1/tickets/{id} - 获取详情

**路径参数**：

| 参数 | 类型 | 说明 |
|-----|------|------|
| `id` | int | Ticket ID |

**响应示例**：
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "id": 1,
    "title": "新功能开发",
    "description": "开发用户管理模块",
    "status": "open",
    "priority": "medium",
    "assignee": "李四",
    "tags": ["feature", "backend"],
    "created_at": "2026-05-17T10:30:00",
    "updated_at": "2026-05-17T10:30:00"
  }
}
```

**错误响应**（404）：
```json
{
  "code": 40401,
  "message": "Ticket 不存在",
  "data": null
}
```

---

#### PUT /api/v1/tickets/{id} - 更新 Ticket（部分字段）

**路径参数**：

| 参数 | 类型 | 说明 |
|-----|------|------|
| `id` | int | Ticket ID |

**请求体**（所有字段可选）：

| 字段 | 类型 | 说明 |
|-----|------|------|
| `title` | string | 标题，1-200字符 |
| `description` | string | 描述 |
| `priority` | string | 优先级 |
| `assignee` | string | 负责人 |
| `tags` | string[] | 标签 |
| `status` | string | 状态（会触发状态流转校验） |

**请求示例**：
```json
{
  "title": "更新标题",
  "priority": "high"
}
```

**响应示例**：
```json
{
  "code": 0,
  "message": "success",
  "data": { ... }
}
```

---

#### PATCH /api/v1/tickets/{id}/status - 切换状态

**路径参数**：

| 参数 | 类型 | 说明 |
|-----|------|------|
| `id` | int | Ticket ID |

**请求体**：
```json
{
  "status": "in_progress"
}
```

**有效状态流转**：
- `open` → `in_progress`, `closed`
- `in_progress` → `done`, `closed`
- `done` → `closed`
- `closed` → `open`

**错误响应**（400）：
```json
{
  "code": 40002,
  "message": "不能从 open 流转到 done",
  "data": null
}
```

---

#### DELETE /api/v1/tickets/{id} - 删除 Ticket

**路径参数**：

| 参数 | 类型 | 说明 |
|-----|------|------|
| `id` | int | Ticket ID |

**响应**：204 No Content

---

### 6.5 聚合查询接口

#### GET /api/v1/tags - 获取所有已使用标签

**响应示例**：
```json
{
  "code": 0,
  "message": "success",
  "data": ["backend", "bug", "feature", "urgent"]
}
```

---

#### GET /api/v1/assignees - 获取所有负责人

**响应示例**：
```json
{
  "code": 0,
  "message": "success",
  "data": ["张三", "李四", "王五"]
}
```

---

### 6.6 错误码对照表

| 错误码 | HTTP 状态 | 说明 |
|-------|----------|------|
| 0 | 2xx | 成功 |
| 40001 | 400 | 请求参数校验失败 |
| 40002 | 400 | 状态流转不合法 |
| 40401 | 404 | Ticket 不存在 |
| 40404 | 404 | 资源不存在 |
| 50001 | 500 | 服务器内部错误 |

---

## 7. 数据库设计

### 7.1 表结构

```sql
CREATE TABLE tickets (
    id INTEGER UNSIGNED AUTO_INCREMENT PRIMARY KEY COMMENT '主键',
    title VARCHAR(200) NOT NULL COMMENT 'Ticket 标题',
    description TEXT COMMENT '详细描述',
    status ENUM('open', 'in_progress', 'done', 'closed') 
        NOT NULL DEFAULT 'open' COMMENT '状态',
    priority ENUM('low', 'medium', 'high', 'urgent') 
        NOT NULL DEFAULT 'medium' COMMENT '优先级',
    assignee VARCHAR(100) COMMENT '负责人',
    tags JSON COMMENT '标签列表',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP 
        ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_status (status),
    INDEX idx_priority (priority),
    INDEX idx_assignee (assignee),
    INDEX idx_created_at (created_at),
    INDEX idx_updated_at (updated_at),
    FULLTEXT INDEX ft_title_desc (title, description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Ticket 工单表';
```

### 7.2 索引设计分析

| 索引名 | 类型 | 字段 | 用途 |
|-------|------|-----|------|
| idx_status | B-Tree | status | 按状态筛选 |
| idx_priority | B-Tree | priority | 按优先级筛选 |
| idx_assignee | B-Tree | assignee | 按负责人筛选 |
| idx_created_at | B-Tree | created_at | 按创建时间排序/筛选 |
| idx_updated_at | B-Tree | updated_at | 按更新时间排序/筛选 |
| ft_title_desc | Fulltext | title, description | 全文搜索 |

### 7.3 为什么这样设计

1. **无主键索引**：主键索引自动创建
2. **JSON 类型 tags**：避免多对多关联表，简化查询
3. **server_default**：数据库层面设置默认值，减少应用代码
4. **ON UPDATE CURRENT_TIMESTAMP**：数据库自动更新 `updated_at`
5. **FULLTEXT 索引**：支持标题/描述的全文搜索
6. **utf8mb4**：完整 UTF-8 支持，包括 emoji

---

## 8. 请求处理流程

### 8.1 创建 Ticket 流程

```
POST /api/v1/tickets
    │
    ▼
┌─────────────────────────┐
│ 路由层 (tickets.py)       │
│ 1. 解析请求体 (TicketCreate)
│ 2. Pydantic 自动校验参数
│ 3. 注入 DB Session
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 服务层 (ticket_service.py)│
│ 1. 创建 Ticket ORM 对象
│ 2. db.add() 添加到 session
│ 3. db.commit() 提交事务
│ 4. db.refresh() 刷新获取 ID
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ ORM 层 (ticket.py)        │
│ 转换为 SQL INSERT 语句
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 数据库                   │
│ 执行 INSERT，返回 ID
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 响应辅助函数              │
│ success(TicketRead)
└─────────────────────────┘
    │
    ▼
HTTP 201 Created
```

### 8.2 列表查询流程

```
GET /api/v1/tickets?status=open&page=1&page_size=20
    │
    ▼
┌─────────────────────────┐
│ 路由层                   │
│ 1. 解析 Query Parameters
│ 2. 转换为 TicketListQuery
│ 3. 调用 ticket_service.list_tickets
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 服务层 - 构建查询条件      │
│ _build_list_filters():
│ - status IN (...)
│ - priority IN (...)
│ - assignee = ...
│ - JSON_CONTAINS(tags, ...)
│ - title LIKE %...% OR description LIKE %...%
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 服务层 - 执行查询         │
│ 1. SELECT COUNT(*) FROM tickets WHERE ...
│ 2. SELECT * FROM tickets WHERE ... 
│    ORDER BY created_at DESC
│    LIMIT 20 OFFSET 0
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 响应辅助函数              │
│ paginated(items, total, page, page_size)
└─────────────────────────┘
    │
    ▼
HTTP 200 OK
```

### 8.3 状态流转校验流程

```
PATCH /api/v1/tickets/1/status
{"status": "done"}
    │
    ▼
┌─────────────────────────┐
│ 路由层                   │
│ 调用 ticket_service.update_status
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 服务层                   │
│ 1. 获取当前 Ticket
│ 2. 调用 _validate_transition
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ 校验逻辑                 │
│ current = "in_progress"  │
│ target = "done"         │
│ allowed = {done, closed} │
│ done ∈ allowed? ✓ 放行   │
└─────────────────────────┘
    │
    ▼
如果校验失败：
┌─────────────────────────┐
│ 抛出 InvalidStatusTransition
│ 异常 → 全局异常处理器
│ → HTTP 400 + 错误信息
└─────────────────────────┘
```

---

## 9. 关键设计模式与理念

### 9.1 分层架构

```
Routes → Services → Models/Schemas → Database
```

**为什么分层**：
- **可测试性**：Service 层可独立于 HTTP 测试
- **可维护性**：职责清晰，修改一层不影响其他层
- **可复用性**：Service 层可在其他场景复用

### 9.2 依赖注入

```python
# app/api/deps.py
def get_db_session() -> Generator[Session]:
    yield from get_db()

# 使用
@router.get("/items")
def list_items(db: Session = Depends(get_db_session)) -> ...: ...
```

**为什么使用依赖注入**：
- **解耦**：路由层不直接依赖数据库实现
- **可测试**：可 mock 依赖进行测试
- **生命周期管理**：FastAPI 自动管理资源

### 9.3 工厂模式

```python
# main.py
def create_app() -> FastAPI:
    ...
    return app

app = create_app()
```

**为什么使用工厂函数**：
- **测试友好**：可创建多个 app 实例用于不同测试场景
- **配置灵活**：可根据环境创建不同配置的 app

### 9.4 异常统一处理

```python
# 所有 BusinessException 由全局处理器统一转换
```

**为什么统一处理**：
- **一致性**：所有错误响应格式统一
- **可追踪**：错误码统一管理
- **解耦**：业务层不关心 HTTP 细节

### 9.5 泛型响应

```python
class ApiResponse[T](BaseModel):
    data: T | None = None
```

**为什么使用泛型**：
- **类型安全**：不同接口有不同的 data 类型
- **IDE 支持**：自动补全 data 的字段

---

## 10. 依赖管理

### 10.1 pyproject.toml 结构

```toml
[project]
name = "project-alpha-backend"
version = "0.1.0"
requires-python = ">=3.13"
dependencies = [...]

[dependency-groups]
dev = [...]

[tool.ruff]
line-length = 100
target-version = "py313"

[tool.black]
line-length = 100

[tool.mypy]
python_version = "3.13"
strict = true

[tool.pytest.ini_options]
testpaths = ["tests"]
```

### 10.2 依赖说明

| 依赖 | 用途 | 为什么选择 |
|-----|------|----------|
| fastapi | Web 框架 | 异步、高性能、自动 OpenAPI |
| uvicorn | ASGI 服务器 | FastAPI 标准服务器 |
| sqlalchemy | ORM | 功能强大、2.0 版本现代化 |
| pydantic | 数据验证 | 类型安全、自动文档 |
| alembic | 数据库迁移 | 与 SQLAlchemy 集成 |
| pymysql | MySQL 驱动 | 纯 Python 实现 |

### 10.3 开发工具

| 工具 | 用途 |
|-----|------|
| ruff | Lint + Format（集成工具，替代 flake8/isort/black） |
| black | 代码格式化（可选） |
| mypy | 静态类型检查 |
| pytest | 测试框架 |
| pytest-cov | 覆盖率 |

---

## 11. 测试架构

### 11.1 测试设计原则

1. **事务隔离**：每个测试在独立事务中执行，自动回滚
2. **依赖注入**：通过 `dependency_overrides` 注入测试 Session
3. **分层测试**：单元测试覆盖 Service 层，集成测试覆盖 API 层

### 11.2 测试配置 (`conftest.py`)

```python
@pytest.fixture
def db_session(test_engine: Engine) -> Generator[Session]:
    """事务回滚式 DB Session"""
    connection = test_engine.connect()
    transaction = connection.begin()
    session = Session(
        bind=connection,
        join_transaction_mode="create_savepoint",  # 关键：使用 SAVEPOINT
    )
    yield session
    session.close()
    transaction.rollback()  # 自动回滚
    connection.close()
```

**为什么使用 SAVEPOINT**：
- Service 层的 `db.commit()` 只提交内层 SAVEPOINT
- 外层事务始终可回滚
- 测试之间完全隔离

### 11.3 测试覆盖

| 测试类型 | 覆盖范围 |
|---------|---------|
| test_health.py | /healthz, /readyz |
| test_ticket_service.py | CRUD + 状态流转 |
| test_ticket_schema.py | Pydantic 字段验证 |
| test_ticket_api.py | CRUD API 端点 |
| test_ticket_list_api.py | 列表查询 API |
| test_aggregations_api.py | tags/assignees API |

---

## 12. Docker 部署

### 12.1 Dockerfile 分析

```dockerfile
# 多阶段构建
FROM python:3.13-slim AS builder
# 安装依赖到 /app/.venv

FROM python:3.13-slim
# 只复制 .venv + 必要文件
```

**为什么多阶段构建**：
- 减小最终镜像体积（不含构建工具）
- 构建缓存优化（依赖不变时不重建）
- 分离构建和运行环境

### 12.2 Docker 入口脚本

```bash
#!/bin/sh
# 执行数据库迁移
alembic upgrade head
# 启动服务
exec uvicorn app.main:app --host 0.0.0.0 --port 8000
```

### 12.3 启动命令

```bash
# 构建镜像
docker build -t pa-backend .

# 运行容器
docker run -p 8000:8000 --env-file .env pa-backend
```

---

## 13. 从零开发指南

### 13.1 项目初始化

```bash
# 1. 创建项目目录
mkdir week1 && cd week1
mkdir backend && cd backend

# 2. 创建 pyproject.toml
cat > pyproject.toml << 'EOF'
[project]
name = "project-alpha-backend"
version = "0.1.0"
requires-python = ">=3.13"
dependencies = [
    "fastapi>=0.136.1",
    "uvicorn[standard]>=0.47.0",
    "sqlalchemy>=2.0.49",
    "alembic>=1.18.4",
    "pymysql>=1.1.3",
    "cryptography>=44.0.0",
    "pydantic>=2.13.4",
    "pydantic-settings>=2.14.1",
    "python-dotenv>=1.2.2",
]

[dependency-groups]
dev = [
    "pytest>=9.0.3",
    "pytest-asyncio>=1.3.0",
    "pytest-cov>=7.1.0",
    "httpx>=0.28.1",
    "ruff>=0.15.13",
    "black>=26.5.0",
    "mypy>=2.1.0",
]
EOF

# 3. 安装依赖
uv sync

# 4. 初始化项目结构
mkdir -p app/{api/v1,core,middlewares,models,schemas,services,utils}
mkdir -p alembic/versions
mkdir -p tests/{unit,integration}
```

### 13.2 配置管理实现

**app/core/config.py**：
```python
from __future__ import annotations

from functools import lru_cache
from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict

class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    app_env: str = "development"
    app_host: str = "0.0.0.0"
    app_port: int = 8000
    app_debug: bool = True

    db_host: str = "localhost"
    db_port: int = 3306
    db_user: str = "root"
    db_password: str = ""
    db_name: str = "project_alpha"

    @property
    def sqlalchemy_url(self) -> str:
        return (
            f"mysql+pymysql://{self.db_user}:{self.db_password}"
            f"@{self.db_host}:{self.db_port}/{self.db_name}?charset=utf8mb4"
        )

@lru_cache(maxsize=1)
def get_settings() -> Settings:
    return Settings()

settings = get_settings()
```

### 13.3 数据库层实现

**app/core/database.py**：
```python
from __future__ import annotations

from collections.abc import Generator
from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.orm import DeclarativeBase, Session, sessionmaker
from app.core.config import settings

class Base(DeclarativeBase):
    pass

engine = create_engine(
    settings.sqlalchemy_url,
    pool_pre_ping=True,
    pool_recycle=3600,
    future=True,
)

SessionLocal = sessionmaker(
    bind=engine,
    autoflush=False,
    autocommit=False,
    expire_on_commit=False,
)

def get_db() -> Generator[Session]:
    session = SessionLocal()
    try:
        yield session
    finally:
        session.close()
```

### 13.4 ORM 模型实现

**app/models/ticket.py**：
```python
from __future__ import annotations

from datetime import datetime
from sqlalchemy import DateTime, Enum, String, Text, func
from sqlalchemy.dialects.mysql import INTEGER, JSON
from sqlalchemy.orm import Mapped, mapped_column
from app.core.database import Base

class Ticket(Base):
    __tablename__ = "tickets"

    id: Mapped[int] = mapped_column(INTEGER(unsigned=True), primary_key=True)
    title: Mapped[str] = mapped_column(String(200), nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    status: Mapped[str] = mapped_column(
        Enum("open", "in_progress", "done", "closed", name="ticket_status"),
        nullable=False, default="open", index=True
    )
    priority: Mapped[str] = mapped_column(
        Enum("low", "medium", "high", "urgent", name="ticket_priority"),
        nullable=False, default="medium", index=True
    )
    assignee: Mapped[str | None] = mapped_column(String(100), nullable=True, index=True)
    tags: Mapped[list[str] | None] = mapped_column(JSON, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime, nullable=False, server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now(), onupdate=func.now()
    )
```

### 13.5 Schema 实现

**app/schemas/ticket.py**：
```python
from __future__ import annotations

from datetime import datetime
from enum import StrEnum
from pydantic import BaseModel, ConfigDict, Field, field_validator

class TicketStatus(StrEnum):
    open = "open"
    in_progress = "in_progress"
    done = "done"
    closed = "closed"

class TicketPriority(StrEnum):
    low = "low"
    medium = "medium"
    high = "high"
    urgent = "urgent"

class TicketCreate(BaseModel):
    title: str = Field(min_length=1, max_length=200)
    description: str | None = None
    priority: TicketPriority = TicketPriority.medium
    assignee: str | None = None
    tags: list[str] = Field(default_factory=list, max_length=10)

class TicketRead(BaseModel):
    id: int
    title: str
    description: str | None
    status: TicketStatus
    priority: TicketPriority
    assignee: str | None
    tags: list[str]
    created_at: datetime
    updated_at: datetime
    model_config = ConfigDict(from_attributes=True)
```

### 13.6 异常体系实现

**app/core/exceptions.py**：
```python
from __future__ import annotations

class BusinessException(Exception):
    code: int = 50001
    message: str = "internal error"
    http_status: int = 500

    def __init__(self, message: str | None = None) -> None:
        if message is not None:
            self.message = message
        super().__init__(self.message)

class TicketNotFound(BusinessException):
    code = 40401
    message = "Ticket 不存在"
    http_status = 404

class InvalidStatusTransition(BusinessException):
    code = 40002
    message = "状态流转不合法"
    http_status = 400
```

### 13.7 Service 层实现

**app/services/ticket_service.py**：
```python
from __future__ import annotations

from sqlalchemy.orm import Session
from app.core.exceptions import TicketNotFound
from app.models.ticket import Ticket
from app.schemas.ticket import TicketCreate

def create_ticket(db: Session, data: TicketCreate) -> Ticket:
    ticket = Ticket(
        title=data.title,
        description=data.description,
        priority=data.priority.value,
        assignee=data.assignee,
        tags=list(data.tags),
    )
    db.add(ticket)
    db.commit()
    db.refresh(ticket)
    return ticket

def get_ticket(db: Session, ticket_id: int) -> Ticket:
    ticket = db.get(Ticket, ticket_id)
    if ticket is None:
        raise TicketNotFound(f"Ticket #{ticket_id} 不存在")
    return ticket
```

### 13.8 路由实现

**app/api/v1/tickets.py**：
```python
from __future__ import annotations

from fastapi import APIRouter, Depends, status
from sqlalchemy.orm import Session
from app.api.deps import get_db_session
from app.schemas.ticket import TicketCreate, TicketRead
from app.services import ticket_service
from app.utils.responses import success

router = APIRouter(prefix="/tickets", tags=["tickets"])

@router.post("", response_model=TicketRead, status_code=status.HTTP_201_CREATED)
def create_ticket(
    data: TicketCreate,
    db: Session = Depends(get_db_session),
) -> TicketRead:
    obj = ticket_service.create_ticket(db, data)
    return TicketRead.model_validate(obj)

@router.get("/{ticket_id}", response_model=TicketRead)
def get_ticket(
    ticket_id: int,
    db: Session = Depends(get_db_session),
) -> TicketRead:
    obj = ticket_service.get_ticket(db, ticket_id)
    return TicketRead.model_validate(obj)
```

### 13.9 主应用实现

**app/main.py**：
```python
from __future__ import annotations

from fastapi import FastAPI
from app.api.v1.tickets import router as tickets_router

def create_app() -> FastAPI:
    app = FastAPI(
        title="ProjectAlpha API",
        version="0.1.0",
    )
    app.include_router(tickets_router)
    return app

app = create_app()
```

### 13.10 数据库迁移

```bash
# 初始化 Alembic
alembic init alembic

# 配置 alembic.ini（修改 sqlalchemy_url）

# 创建迁移
alembic revision --autogenerate -m "init tickets"

# 执行迁移
alembic upgrade head
```

---

## 14. 开发规范

### 14.1 代码风格

- **Python 版本**：3.13+
- **类型注解**：全量类型注解，`from __future__ import annotations`
- **命名规范**：
  - 函数/变量：`snake_case`
  - 类名：`PascalCase`
  - 常量：`UPPER_SNAKE_CASE`
- **行长度**：100 字符

### 14.2 工具链

```bash
# 格式化
uv run ruff format .

# Lint
uv run ruff check .

# 类型检查
uv run mypy app

# 测试
uv run pytest

# 带覆盖率测试
uv run pytest --cov=app
```

### 14.3 Git 提交规范

```
feat: 新功能
fix: 修复 bug
docs: 文档更新
style: 代码格式（不影响功能）
refactor: 重构
test: 测试相关
chore: 构建/工具相关
```

### 14.4 环境变量配置

```bash
# .env.example
APP_ENV=development
APP_PORT=8000
APP_HOST=0.0.0.0
APP_DEBUG=true

DB_HOST=localhost
DB_PORT=3306
DB_USER=root
DB_PASSWORD=your_password
DB_NAME=project_alpha

CORS_ALLOW_ORIGINS=http://localhost:5173

LOG_LEVEL=INFO
```

---

## 附录

### A. 快速启动命令

```bash
# 1. 复制环境变量
cp .env.example .env

# 2. 修改 .env 中的数据库配置

# 3. 安装依赖
uv sync

# 4. 运行迁移
uv run alembic upgrade head

# 5. 启动服务
uv run uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

# 6. 访问文档
# http://localhost:8000/docs
```

### B. API 文档地址

| 地址 | 说明 |
|-----|------|
| `/docs` | Swagger UI |
| `/redoc` | ReDoc |
| `/openapi.json` | OpenAPI JSON |
| `/healthz` | 存活探针 |
| `/readyz` | 就绪探针 |

### C. 文件清单

本项目共有 **53 个文件**，核心业务代码约 1500 行。

---

> 文档版本：1.0.0  
> 生成时间：2026-05-30  
> 项目仓库：d:\AiLearning\week1\backend
