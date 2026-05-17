# ProjectAlpha 实现计划

> 版本: 1.0  
> 日期: 2026-05-17  
> 来源: [0001-spec.md](./0001-spec.md)  
> 目标读者: 开发团队（前端 / 后端 / 测试 / 运维）  
> 状态: 待评审

---

## 目录

1. [总体策略](#1-总体策略)
2. [开发任务分解（WBS）](#2-开发任务分解wbs)
3. [阶段 0 - 项目启动与环境准备](#阶段-0---项目启动与环境准备)
4. [阶段 1 - 后端基础框架](#阶段-1---后端基础框架)
5. [阶段 2 - 后端 Ticket CRUD](#阶段-2---后端-ticket-crud)
6. [阶段 3 - 后端筛选 / 搜索 / 排序 / 分页](#阶段-3---后端筛选--搜索--排序--分页)
7. [阶段 4 - 前端基础框架](#阶段-4---前端基础框架)
8. [阶段 5 - 前端 Ticket 列表页](#阶段-5---前端-ticket-列表页)
9. [阶段 6 - 前端 Ticket 详情与表单](#阶段-6---前端-ticket-详情与表单)
10. [阶段 7 - 前后端联调](#阶段-7---前后端联调)
11. [阶段 8 - 测试与质量保障](#阶段-8---测试与质量保障)
12. [阶段 9 - 部署上线](#阶段-9---部署上线)
13. [依赖关系与关键路径](#13-依赖关系与关键路径)
14. [风险管理](#14-风险管理)
15. [验收清单（DoD）](#15-验收清单dod)
16. [角色分工与协作约定](#16-角色分工与协作约定)

---

## 1. 总体策略

### 1.1 工程原则

| 原则 | 说明 |
|------|------|
| 接口先行 | 先冻结 OpenAPI 文档，前后端基于契约并行开发 |
| 分层解耦 | 后端遵循 `Router → Service → Model` 三层分离；前端遵循 `Page → Component → API` 分层 |
| 增量交付 | 每个阶段都是可运行的中间产物，逐步累加功能 |
| 测试驱动 | 后端 Service 层、关键中间件先写测试再写实现 |
| 类型安全 | 前端 TypeScript strict；后端全量类型注解 + mypy |

### 1.2 总工期与排期

预计总工期 **14 个工作日**，对应 spec §17 的 8 个里程碑：

```
天数:   1   2   3   4   5   6   7   8   9   10  11  12  13  14
       |---|---|---|---|---|---|---|---|---|---|---|---|---|---|
M1 启动 ▓
M2 后端CRUD ▓▓▓
M3 后端筛选 ▓
M4 前端列表  ▓▓
M5 前端详情     ▓▓
M6 联调            ▓▓
M7 测试               ▓▓
M8 部署                  ▓▓
                                                              ↑
                                                            发布
```

### 1.3 并行策略

- **后端**（M2~M3）与 **前端 Mock 阶段**（M4~M5）并行
- 联调阶段（M6）作为汇合点
- 测试编写贯穿全程，集中修复在 M7

---

## 2. 开发任务分解（WBS）

```
ProjectAlpha
├── 阶段 0 项目启动 (Day 1)
│   ├── 仓库初始化
│   ├── 目录骨架搭建
│   ├── 开发环境约定
│   └── 数据库本地启动
├── 阶段 1 后端基础框架 (Day 1~2)
│   ├── FastAPI 应用骨架
│   ├── 配置加载 (Pydantic Settings)
│   ├── 数据库连接 (SQLAlchemy)
│   ├── Alembic 迁移配置
│   └── 全局异常 / 响应封装
├── 阶段 2 后端 CRUD (Day 2~3)
│   ├── Ticket Model
│   ├── Pydantic Schema
│   ├── ticket_service
│   ├── /tickets 路由
│   └── 单元测试 + 集成测试
├── 阶段 3 后端筛选与搜索 (Day 4)
│   ├── 列表查询拼装
│   ├── 多状态/多优先级筛选
│   ├── 关键字模糊搜索
│   ├── 排序 + 分页
│   ├── /tags 与 /assignees 聚合接口
│   └── 状态流转校验
├── 阶段 4 前端基础框架 (Day 5)
│   ├── Vite + React + TS 脚手架
│   ├── TailwindCSS / Shadcn UI
│   ├── React Router 配置
│   ├── Axios 实例封装
│   ├── 全局类型 / 常量 / 工具
│   └── MSW Mock 接口
├── 阶段 5 前端 Ticket 列表页 (Day 5~6)
│   ├── 整体布局 (Header + Sidebar + Content)
│   ├── Header 组件 (Logo / SearchBar / NewTicketButton)
│   ├── Sidebar 组件 (状态/优先级/负责人/标签筛选)
│   ├── TicketTable 组件
│   ├── 分页 / 排序
│   ├── 空状态与骨架屏
│   └── URL 同步筛选条件
├── 阶段 6 前端详情与表单 (Day 7~8)
│   ├── TicketForm 弹窗 (创建/编辑)
│   ├── TicketDetailPage
│   ├── 状态切换组件
│   ├── 删除确认弹窗
│   └── 错误 Toast
├── 阶段 7 联调 (Day 9~10)
│   ├── 关闭 MSW 切换真实 API
│   ├── 跨域代理验证
│   ├── 字段 / 枚举 / 时间格式校对
│   ├── 错误场景验证
│   └── Bug 修复
├── 阶段 8 测试与质量 (Day 11~12)
│   ├── 后端覆盖率达标
│   ├── 前端组件测试
│   ├── E2E 关键路径
│   ├── 性能验证 (NF01/NF02)
│   └── Lint / 类型检查
└── 阶段 9 部署 (Day 13~14)
    ├── Dockerfile (后端 / 前端)
    ├── docker-compose
    ├── Nginx 配置
    ├── 环境变量与 secrets
    ├── 数据库迁移与种子
    └── 冒烟测试
```

---

## 阶段 0 - 项目启动与环境准备

**周期：** Day 1（半天）  
**对应 spec：** §9 项目目录结构、§13 开发规范

### 0.1 任务清单

| 序号 | 任务 | 负责方 | 产出 |
|------|------|--------|------|
| T0.1 | 创建 Git 仓库，建立 `main` / `dev` 分支 | 所有人 | 仓库地址 |
| T0.2 | 按 spec §9 创建顶层目录骨架 | 全栈 | `frontend/` `backend/` `database/` `docs/` 等 |
| T0.3 | 编写 `.gitignore`、`README.md`、`LICENSE` | 全栈 | 基础文档 |
| T0.4 | 配置 `.editorconfig` + pre-commit 钩子 | 全栈 | 统一格式化规则 |
| T0.5 | 本地启动 MySQL 8.0（Docker 单容器） | 后端 | 可访问 3306 |
| T0.6 | 创建 `project_alpha` 与 `project_alpha_test` 数据库 | 后端 | 两个数据库 |
| T0.7 | 编写 `.env.example`（前 + 后） | 全栈 | 模板文件 |

### 0.2 启动命令

```bash
# 启动 MySQL（仅本地开发用）
docker run -d \
  --name pa-mysql-dev \
  -e MYSQL_ROOT_PASSWORD=root \
  -e MYSQL_DATABASE=project_alpha \
  -p 3306:3306 \
  -v pa-mysql-data:/var/lib/mysql \
  mysql:8.0

# 创建测试数据库
docker exec -i pa-mysql-dev mysql -uroot -proot \
  -e "CREATE DATABASE project_alpha_test CHARACTER SET utf8mb4;"
```

### 0.3 完成标准

- [ ] 仓库可被克隆，目录结构与 spec §9 一致
- [ ] MySQL 容器运行正常，可用 `mysql -h 127.0.0.1 -uroot -p` 登录
- [ ] `.env.example` 文件齐全

---

## 阶段 1 - 后端基础框架

**周期：** Day 1 下午 ~ Day 2 上午  
**对应 spec：** §8、§9（backend）、§10、§11

### 1.1 任务清单

| 序号 | 任务 | 产出文件 |
|------|------|----------|
| T1.1 | 初始化 Poetry 项目，安装核心依赖 | `pyproject.toml` `poetry.lock` |
| T1.2 | 编写 `app/core/config.py`（Pydantic Settings 加载 .env） | `app/core/config.py` |
| T1.3 | 编写 `app/core/database.py`（SQLAlchemy engine + Session） | `app/core/database.py` |
| T1.4 | 编写 `app/api/deps.py`（DB session 依赖注入） | `app/api/deps.py` |
| T1.5 | 编写 `app/main.py`（FastAPI 实例 + 路由挂载 + 中间件） | `app/main.py` |
| T1.6 | 编写 `app/middlewares/cors.py` | `app/middlewares/cors.py` |
| T1.7 | 编写 `app/core/exceptions.py`（自定义异常类） | `app/core/exceptions.py` |
| T1.8 | 编写 `app/middlewares/error_handler.py`（全局异常处理） | `app/middlewares/error_handler.py` |
| T1.9 | 编写 `app/schemas/common.py`（统一响应结构） | `app/schemas/common.py` |
| T1.10 | 初始化 Alembic | `alembic/` `alembic.ini` |
| T1.11 | 编写 `database/init.sql`（spec §10.1 DDL） | `database/init.sql` |

### 1.2 关键代码契约

**统一响应结构（spec §5.1）：**

```python
# app/schemas/common.py
from typing import Generic, TypeVar, Optional
from pydantic import BaseModel

T = TypeVar("T")

class ApiResponse(BaseModel, Generic[T]):
    code: int = 0
    message: str = "success"
    data: Optional[T] = None

class PageData(BaseModel, Generic[T]):
    items: list[T]
    total: int
    page: int
    page_size: int
```

**自定义异常基类：**

```python
# app/core/exceptions.py
class BusinessException(Exception):
    code: int = 50001
    message: str = "internal error"
    http_status: int = 500

class TicketNotFound(BusinessException):
    code = 40401
    message = "Ticket 不存在"
    http_status = 404

class InvalidStatusTransition(BusinessException):
    code = 40002
    message = "状态流转不合法"
    http_status = 400
```

### 1.3 完成标准

- [ ] `poetry run uvicorn app.main:app --reload` 可启动并访问 `/docs`
- [ ] 访问 `/healthz` 返回 `{"code":0,"message":"success","data":"ok"}`
- [ ] Alembic `alembic upgrade head` 成功创建 `alembic_version` 表
- [ ] 故意触发异常，能返回统一错误格式

---

## 阶段 2 - 后端 Ticket CRUD

**周期：** Day 2 下午 ~ Day 3  
**对应 spec：** §4、§5.2.1~§5.2.6、§10

### 2.1 任务清单

| 序号 | 任务 | 产出 |
|------|------|------|
| T2.1 | 编写 SQLAlchemy 模型 `Ticket` | `app/models/ticket.py` |
| T2.2 | 生成首次 Alembic 迁移 | `alembic/versions/xxxx_init.py` |
| T2.3 | 编写 Pydantic Schema（Create/Update/Read/StatusUpdate） | `app/schemas/ticket.py` |
| T2.4 | 编写 `ticket_service` 的 `create/get/update/delete` | `app/services/ticket_service.py` |
| T2.5 | 编写 `tickets.py` 路由（GET 详情 / POST / PUT / DELETE） | `app/api/v1/tickets.py` |
| T2.6 | 单元测试（service 层） | `tests/unit/test_ticket_service.py` |
| T2.7 | 集成测试（API 层） | `tests/integration/test_ticket_api.py` |

### 2.2 Ticket 模型字段（对照 spec §4.1）

```python
# app/models/ticket.py
from sqlalchemy import String, Text, Enum, JSON, DateTime, func
from sqlalchemy.orm import Mapped, mapped_column
from datetime import datetime
from app.core.database import Base

class Ticket(Base):
    __tablename__ = "tickets"

    id: Mapped[int] = mapped_column(primary_key=True, autoincrement=True)
    title: Mapped[str] = mapped_column(String(200), nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    status: Mapped[str] = mapped_column(
        Enum("open", "in_progress", "done", "closed", name="ticket_status"),
        nullable=False, default="open",
    )
    priority: Mapped[str] = mapped_column(
        Enum("low", "medium", "high", "urgent", name="ticket_priority"),
        nullable=False, default="medium",
    )
    assignee: Mapped[str | None] = mapped_column(String(100), nullable=True)
    tags: Mapped[list[str]] = mapped_column(JSON, nullable=False, default=list)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now(),
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False,
        server_default=func.now(),
        onupdate=func.now(),
    )
```

### 2.3 Pydantic Schema 校验规则（对照 spec §5.2.3）

| 字段 | 校验规则 |
|------|----------|
| title | `min_length=1, max_length=200`，必填 |
| description | 可空 |
| priority | 必须为枚举 `low/medium/high/urgent`，默认 `medium` |
| assignee | `max_length=100`，可空 |
| tags | `max_items=10`，每项 `min_length=1, max_length=20` |
| status（PATCH） | 必须为合法枚举 |

### 2.4 单元测试用例清单

| 用例 ID | 场景 | 预期 |
|---------|------|------|
| UT-T2-01 | 创建合法 Ticket | 返回带 id 的对象，时间字段非空 |
| UT-T2-02 | 标题为空 | 抛出 `ValidationError` |
| UT-T2-03 | 标签超过 10 项 | 抛出 `ValidationError` |
| UT-T2-04 | 标签长度 > 20 | 抛出 `ValidationError` |
| UT-T2-05 | priority 非法枚举 | 抛出 `ValidationError` |
| UT-T2-06 | 更新存在的 Ticket（仅传部分字段） | 仅修改传入字段，其他不变 |
| UT-T2-07 | 更新不存在的 Ticket | 抛出 `TicketNotFound` |
| UT-T2-08 | 删除存在的 Ticket | 返回成功，再次查询 404 |
| UT-T2-09 | 删除不存在的 Ticket | 抛出 `TicketNotFound` |

### 2.5 完成标准

- [ ] Postman / Swagger UI 测试所有 5 个 CRUD 接口通过
- [ ] 数据库表结构与 spec §10.1 完全一致
- [ ] 单元测试通过率 100%，覆盖率 ≥ 80%

---

## 阶段 3 - 后端筛选 / 搜索 / 排序 / 分页

**周期：** Day 4  
**对应 spec：** §3.2、§3.3、§5.2.1、§5.3

### 3.1 任务清单

| 序号 | 任务 | 产出 |
|------|------|------|
| T3.1 | 实现列表查询条件拼装（动态 where） | `ticket_service.list_tickets` |
| T3.2 | 实现关键字模糊搜索（先 LIKE，量大可切 FULLTEXT） | 同上 |
| T3.3 | 实现排序 + 分页 | 同上 |
| T3.4 | 实现 `GET /tickets` 路由 + 查询参数校验 | `app/api/v1/tickets.py` |
| T3.5 | 实现 `PATCH /tickets/:id/status` + 状态流转校验 | 同上 |
| T3.6 | 实现 `GET /tags` 聚合（`SELECT DISTINCT` + JSON 展开） | `app/api/v1/tags.py` |
| T3.7 | 实现 `GET /assignees` 聚合 | 同上 |
| T3.8 | 补充集成测试 | `tests/integration/test_ticket_list.py` |

### 3.2 列表查询参数解析约定

```python
# 多值参数（如 status, priority）通过逗号分隔
# 示例：?status=open,in_progress&priority=high,urgent
def parse_multi(value: str | None) -> list[str] | None:
    if not value:
        return None
    return [v.strip() for v in value.split(",") if v.strip()]
```

### 3.3 状态流转校验逻辑（spec §3.3）

```python
STATUS_TRANSITIONS = {
    "open":        {"in_progress", "closed"},
    "in_progress": {"done", "closed"},
    "done":        {"closed"},
    "closed":      {"open"},
}

def validate_transition(current: str, target: str) -> None:
    if target not in STATUS_TRANSITIONS.get(current, set()):
        raise InvalidStatusTransition(
            f"不能从 {current} 流转到 {target}"
        )
```

### 3.4 集成测试用例清单

| 用例 ID | 场景 | 预期 |
|---------|------|------|
| IT-T3-01 | 无筛选获取列表 | 返回所有 Ticket，按 created_at 降序 |
| IT-T3-02 | `?status=open,in_progress` | 仅返回这两种状态 |
| IT-T3-03 | `?priority=urgent&keyword=登录` | 同时满足两个条件 |
| IT-T3-04 | `?keyword=ab`（< 2 字符） | 后端容错（前端已限制），返回所有 |
| IT-T3-05 | `?page=1&page_size=20` | total 正确，items.length ≤ 20 |
| IT-T3-06 | `?page_size=200` | 自动截断为 100（最大值） |
| IT-T3-07 | `?sort_by=updated_at&sort_order=asc` | 按 updated_at 升序 |
| IT-T3-08 | 状态合法流转 `open → in_progress` | 200 |
| IT-T3-09 | 状态非法流转 `done → in_progress` | 400, code=40002 |

### 3.5 完成标准

- [ ] `GET /tickets` 支持所有 9 个查询参数（spec §5.2.1）
- [ ] 状态流转规则与 spec §3.3 完全一致
- [ ] 集成测试通过率 100%
- [ ] 1000 条数据下列表接口响应 < 500ms（NF01）

---

## 阶段 4 - 前端基础框架

**周期：** Day 5 上午  
**对应 spec：** §8、§9（frontend）、§15

### 4.1 任务清单

| 序号 | 任务 | 产出 |
|------|------|------|
| T4.1 | `npm create vite@latest frontend -- --template react-ts` | 项目骨架 |
| T4.2 | 安装并配置 TailwindCSS + Shadcn UI | `tailwind.config.ts` |
| T4.3 | 安装 `react-router-dom`、`axios`、`@tanstack/react-query`（可选） | 依赖更新 |
| T4.4 | 编写 `src/api/request.ts`（Axios 实例 + 拦截器） | 同 |
| T4.5 | 编写 `src/api/tickets.ts` + `tags.ts` | 同 |
| T4.6 | 编写 `src/constants/enums.ts`（状态/优先级/颜色） | 同 |
| T4.7 | 编写 `src/types/ticket.ts` | 同 |
| T4.8 | 编写 `src/App.tsx` 路由 + 基础布局 | 同 |
| T4.9 | 配置 Vite 代理（`/api → http://localhost:8000`） | `vite.config.ts` |
| T4.10 | 安装 MSW 并编写 mock handlers | `src/mocks/handlers.ts` |

### 4.2 关键代码契约

**Axios 拦截器（统一解包响应、统一错误处理）：**

```typescript
// src/api/request.ts
import axios from 'axios'
import { toast } from '@/components/ui/use-toast'

export const request = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 10_000,
})

request.interceptors.response.use(
  (resp) => {
    const { code, message, data } = resp.data
    if (code === 0) return data
    toast({ variant: 'destructive', title: message })
    return Promise.reject(new ApiError(code, message))
  },
  (err) => {
    if (err.code === 'ECONNABORTED') {
      toast({ variant: 'destructive', title: '请求超时' })
    } else if (err.response?.status === 404) {
      toast({ variant: 'destructive', title: 'Ticket 不存在' })
    } else {
      toast({ variant: 'destructive', title: '服务器错误' })
    }
    return Promise.reject(err)
  },
)
```

**枚举与颜色（spec §4.2）：**

```typescript
// src/constants/enums.ts
export const STATUS_LABEL = {
  open: '待处理',
  in_progress: '处理中',
  done: '已完成',
  closed: '已关闭',
} as const

export const PRIORITY_LABEL = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
} as const

export const PRIORITY_COLOR = {
  low: '#999999',
  medium: '#1890FF',
  high: '#FA8C16',
  urgent: '#F5222D',
} as const
```

### 4.3 完成标准

- [ ] `npm run dev` 可启动，访问 5173 端口显示空白布局
- [ ] MSW 在开发环境启用，访问 `/api/v1/tickets` 返回 Mock 数据
- [ ] 类型与枚举与后端 100% 对齐

---

## 阶段 5 - 前端 Ticket 列表页

**周期：** Day 5 下午 ~ Day 6  
**对应 spec：** §6.1、§6.2、§6.6

### 5.1 任务清单

| 序号 | 任务 | 产出 |
|------|------|------|
| T5.1 | `Layout` 组件（顶部 Header + 下方左右分栏） | `components/Layout/` |
| T5.2 | `Header` 组件（Logo + SearchBar + NewTicketButton） | `components/Header/` |
| T5.3 | `SearchBar`（300ms 防抖 + 最少 2 字符触发） | `components/SearchBar/` |
| T5.4 | `Sidebar`（状态/优先级 复选框，负责人/标签 列表） | `components/Sidebar/` |
| T5.5 | `TicketTable`（列：ID/标题/状态/优先级/负责人/更新时间） | `components/TicketTable/` |
| T5.6 | `Pagination` 组件 | `components/Pagination/` |
| T5.7 | `EmptyState` 组件 | `components/EmptyState/` |
| T5.8 | `Skeleton` 加载态 | `components/Skeleton/` |
| T5.9 | `TicketListPage` 页面（聚合所有组件 + 查询状态管理） | `pages/TicketListPage/` |
| T5.10 | 查询条件与 URL 同步（`useSearchParams`） | 同上 |

### 5.2 组件树

```
TicketListPage
├── Layout
│   ├── Header
│   │   ├── Logo
│   │   ├── SearchBar         (受控 + 防抖)
│   │   └── NewTicketButton   (打开 TicketForm 弹窗)
│   └── Body
│       ├── Sidebar
│       │   ├── StatusFilter      (Checkbox group)
│       │   ├── PriorityFilter    (Checkbox group)
│       │   ├── AssigneeFilter    (Select)
│       │   └── TagFilter         (Checkbox group, 来自 /api/tags)
│       └── Content
│           ├── SortBar            (排序下拉)
│           ├── TicketTable | Skeleton | EmptyState
│           └── Pagination
```

### 5.3 URL 同步规则

| 查询条件 | URL 参数 | 默认值 |
|----------|----------|--------|
| 关键字 | `?q=` | - |
| 状态 | `?status=open,in_progress` | - |
| 优先级 | `?priority=high` | - |
| 负责人 | `?assignee=张三` | - |
| 标签 | `?tag=bug` | - |
| 排序 | `?sort=created_at:desc` | `created_at:desc` |
| 页码 | `?page=1` | 1 |

> 刷新页面或分享 URL 应能完整还原当前筛选状态。

### 5.4 完成标准（基于 Mock 数据）

- [ ] 整体布局与 spec §6.1 ASCII 图一致
- [ ] 任意筛选条件变更后列表刷新
- [ ] 加载中显示骨架屏，空数据显示空状态
- [ ] 浏览器后退/前进能正确恢复筛选状态

---

## 阶段 6 - 前端 Ticket 详情与表单

**周期：** Day 7 ~ Day 8  
**对应 spec：** §6.3、§6.4、§6.5

### 6.1 任务清单

| 序号 | 任务 | 产出 |
|------|------|------|
| T6.1 | `TicketForm` 弹窗组件（新建/编辑两用） | `components/TicketForm/` |
| T6.2 | `TagInput` 组件（输入新标签 + 选择已有标签） | `components/TagInput/` |
| T6.3 | `PriorityRadio` 组件 | `components/PriorityRadio/` |
| T6.4 | `AssigneeInput`（带历史负责人下拉） | `components/AssigneeInput/` |
| T6.5 | `StatusSelect`（受流转规则约束） | `components/StatusSelect/` |
| T6.6 | `ConfirmDialog` 通用确认弹窗 | `components/ConfirmDialog/` |
| T6.7 | `TicketDetailPage` 页面 | `pages/TicketDetailPage/` |
| T6.8 | 路由 `/tickets/:id`，404 跳回列表 | `App.tsx` |
| T6.9 | 表单校验（title 必填、tags 长度、tag 数量） | `TicketForm` |
| T6.10 | "放弃修改"二次确认 | `TicketForm` |

### 6.2 表单校验规则（与后端 Pydantic 对齐）

| 字段 | 前端校验 |
|------|----------|
| title | 不可为空，长度 ≤ 200 |
| description | 长度无限制 |
| priority | radio 必选 |
| assignee | 长度 ≤ 100 |
| tags | 单个 1~20 字符，最多 10 个，自动转小写、去重 |

### 6.3 状态切换规则（前端限制）

下拉只展示当前状态允许流转到的目标（spec §3.3），其余置灰：

| 当前 | 可选 |
|------|------|
| open | 处理中、已关闭 |
| in_progress | 已完成、已关闭 |
| done | 已关闭 |
| closed | 待处理 |

### 6.4 完成标准

- [ ] 新建/编辑表单 UI 与 spec §6.3 一致
- [ ] 详情页 UI 与 spec §6.4 一致
- [ ] 删除前必弹确认（spec §6.5）
- [ ] 编辑时若未保存关闭，弹"放弃修改？"
- [ ] 状态切换按 spec §3.3 限制可选项

---

## 阶段 7 - 前后端联调

**周期：** Day 9 ~ Day 10  
**对应 spec：** §15

### 7.1 联调准备

| 序号 | 任务 |
|------|------|
| T7.1 | 后端启动到 8000 端口，前端关闭 MSW |
| T7.2 | 验证 Vite 代理转发到后端 |
| T7.3 | 后端导入种子数据（≥ 30 条） |
| T7.4 | 前后端各派 1 人对照 Swagger UI 走查所有接口 |

### 7.2 联调检查清单（spec §15.4 + 补充）

| # | 检查项 | 检查方式 |
|---|--------|----------|
| 1 | 列表接口字段名 / 嵌套结构 | 对比 Swagger 与前端 type |
| 2 | 状态/优先级枚举值 | 后端返回 `open`，前端展示"待处理" |
| 3 | 时间格式 ISO 8601 | 前端格式化为 `YYYY-MM-DD HH:mm:ss` |
| 4 | 分页：page 从 1 开始 | 检查首末页边界 |
| 5 | 多状态筛选逗号分隔 | `?status=open,in_progress` |
| 6 | 关键字模糊：标题 + 描述均命中 | 输入命中描述的关键字 |
| 7 | 创建后列表自动刷新 | 验证 UX |
| 8 | 状态非法流转：弹错误 Toast | 触发 done → in_progress |
| 9 | 删除二次确认 | 取消不删，确认才删 |
| 10 | tags 大小写、去重 | 输入 `Bug` 应存为 `bug` |
| 11 | 网络断开 | 拔网线测试 Toast |
| 12 | 500 错误 | 后端故意 raise 测试 |

### 7.3 已知联调陷阱与防范

| 陷阱 | 防范 |
|------|------|
| 后端默认 JSON 时间用 ISO，前端误传字符串 | 在 API 函数中统一转换 |
| 后端 tags 默认 `[]`，前端误判为 `null` | 类型上写 `string[]`（非 optional） |
| 大小写不敏感的 tag 实际写入大写 | 后端 service 内部 `.lower()` |
| page_size 上限 100，前端误传 200 | 后端兜底截断 + 前端选项限制 |
| CORS 报错 | 开发期前端走 Vite 代理，避免直连 |

### 7.4 完成标准

- [ ] 12 项检查全部通过
- [ ] 所有验收标准（spec §3 F01~F16）在真实接口下回归通过
- [ ] 关闭 MSW 后系统功能完整可用

---

## 阶段 8 - 测试与质量保障

**周期：** Day 11 ~ Day 12  
**对应 spec：** §14、§12

### 8.1 测试覆盖目标

| 层级 | 行覆盖率 | 分支覆盖率 |
|------|----------|-----------|
| 后端 Service | ≥ 80% | ≥ 70% |
| 后端 API | ≥ 70% | ≥ 60% |
| 前端核心组件 | ≥ 70% | ≥ 60% |

### 8.2 后端测试计划

| 类型 | 用例数 | 工具 |
|------|--------|------|
| Schema 校验单元测试 | 10+ | pytest |
| Service 单元测试 | 25+ | pytest |
| API 集成测试 | 20+ | pytest + httpx.AsyncClient |
| 状态流转专项 | 8+ | pytest |
| 性能测试（1000 条 + 列表查询） | 1 | pytest-benchmark |

**测试隔离方案：**

```python
# tests/conftest.py 关键 fixture
@pytest.fixture
def db_session():
    connection = engine.connect()
    transaction = connection.begin()
    session = Session(bind=connection)
    yield session
    session.close()
    transaction.rollback()   # 每个用例结束回滚
    connection.close()
```

### 8.3 前端测试计划

| 组件 | 测试用例 |
|------|----------|
| TicketForm | 必填校验、标签数量限制、放弃修改弹窗 |
| TicketTable | 列渲染、空状态、点击行跳转 |
| Sidebar | 多选/取消、清除筛选、URL 同步 |
| SearchBar | 防抖 300ms、最少 2 字符 |
| StatusSelect | 仅显示合法流转目标 |
| ConfirmDialog | 确认/取消回调 |

**E2E 关键路径（可选 Playwright）：**

1. 打开列表页 → 看到种子数据
2. 点击 "+ 新建 Ticket" → 填表 → 保存 → 列表新增一条
3. 在 Sidebar 勾选状态 → 列表过滤
4. 在 Header 搜索关键字 → 列表过滤
5. 点击列表行 → 进入详情页 → 修改状态 → 返回列表看到状态变化
6. 详情页点 "删除" → 二次确认 → 列表减少一条

### 8.4 性能与非功能验证

| 编号 | 指标 | 验证方法 |
|------|------|----------|
| NF01 | 列表接口 < 500ms | locust 或简易脚本压测，1000 条数据 |
| NF02 | 前端首屏 < 2s | Chrome DevTools Lighthouse |
| NF08 | 浏览器兼容 | Chrome / Firefox / Edge 最新两版 |
| NF09 | 最小宽度 1280px | DevTools 切换分辨率 |

### 8.5 质量门禁

| 检查项 | 命令 | 通过条件 |
|--------|------|----------|
| 后端 lint | `poetry run ruff check .` | 无 error |
| 后端 type check | `poetry run mypy app` | 无 error |
| 后端测试 + 覆盖率 | `poetry run pytest --cov=app` | 覆盖率达标 |
| 前端 lint | `npm run lint` | 无 error |
| 前端 type check | `npm run typecheck` | 无 error |
| 前端测试 | `npm test` | 全部通过 |
| 前端构建 | `npm run build` | 成功 |

### 8.6 完成标准

- [ ] 所有质量门禁通过
- [ ] 覆盖率达标
- [ ] 性能指标达标
- [ ] 6 条 E2E 关键路径通过

---

## 阶段 9 - 部署上线

**周期：** Day 13 ~ Day 14  
**对应 spec：** §16

### 9.1 任务清单

| 序号 | 任务 | 产出 |
|------|------|------|
| T9.1 | 编写后端 `Dockerfile`（多阶段构建可选） | `backend/Dockerfile` |
| T9.2 | 编写前端 `Dockerfile`（Nginx 静态托管） | `frontend/Dockerfile` |
| T9.3 | 编写 `docker-compose.yml`（spec §16.2） | 根目录 |
| T9.4 | 编写 `nginx.conf`（spec §16.3） | `frontend/nginx.conf` |
| T9.5 | 配置生产 `.env`（含强密码） | 服务器本地 |
| T9.6 | 编写 `database/init.sql` 与 seed 脚本 | `database/` |
| T9.7 | 一键启动 + 冒烟测试 | 跑通 |
| T9.8 | 编写部署手册（README 中） | `README.md` |

### 9.2 前端 Dockerfile（参考）

```dockerfile
# 构建阶段
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# 运行阶段
FROM nginx:1.27-alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
```

### 9.3 部署冒烟测试用例

| # | 用例 | 预期 |
|---|------|------|
| 1 | `docker-compose up -d --build` | 三个容器 healthy |
| 2 | `docker exec pa-mysql mysql -uroot -p -e "SHOW DATABASES"` | 包含 `project_alpha` |
| 3 | `curl http://localhost/api/v1/tickets` | 200 + JSON |
| 4 | 浏览器访问 `http://localhost` | 看到列表页 |
| 5 | 浏览器访问 `http://localhost/docs` | 看到 Swagger UI |
| 6 | 创建一条 Ticket 后重启容器，数据仍在 | 数据卷生效 |

### 9.4 回滚预案

| 场景 | 处理方式 |
|------|----------|
| 部署后 API 报错 | `docker-compose down && git checkout <last-stable> && docker-compose up -d --build` |
| 数据库迁移失败 | `alembic downgrade -1` |
| 镜像构建失败 | 保留上一版镜像，先恢复服务再排查 |
| 数据被破坏 | 从 MySQL 卷快照恢复（生产建议每天备份） |

### 9.5 完成标准

- [ ] 6 条冒烟测试全部通过
- [ ] README 部署章节可被新人一次跑通
- [ ] 容器重启 / 服务器重启后数据不丢
- [ ] 部署手册评审通过

---

## 13. 依赖关系与关键路径

```
阶段0 -> 阶段1 -> 阶段2 -> 阶段3 ----------+
                              \             \
                               +-> 阶段7 -> 阶段8 -> 阶段9
                              /             /
阶段0 -> 阶段4 -> 阶段5 -> 阶段6 ----------+
```

**关键路径：** 阶段 0 → 1 → 2 → 3 → 7 → 8 → 9（共 ~14 天）

**可并行：**
- 阶段 4~6（前端）与阶段 2~3（后端）通过 MSW Mock 解耦并行
- 阶段 8 测试编写可贯穿阶段 2~6

---

## 14. 风险管理

| 编号 | 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|------|----------|
| R01 | OpenAPI 契约前后端理解不一致 | 中 | 高 | 阶段 1 末尾共同评审 Swagger，冻结契约 |
| R02 | tags 用 JSON 字段难以索引 | 中 | 中 | 本期数据量小可接受；超 1 万条时引入 `ticket_tags` 关联表 |
| R03 | 关键字搜索 LIKE 慢 | 低 | 中 | FULLTEXT 索引已建；必要时引入 ES（后期） |
| R04 | 联调期间字段语义偏差（如 status 用中文 vs 英文） | 中 | 中 | spec §4.2 已固定枚举映射；联调检查清单第 2 项必查 |
| R05 | Alembic 自动生成迁移漏字段 | 中 | 中 | 每次生成后人工 diff `alembic upgrade head --sql` |
| R06 | Docker 镜像体积过大 | 低 | 低 | 后端用 `python:3.11-slim`；前端多阶段构建只保留 dist |
| R07 | 时区不一致（开发本地 vs 容器 UTC） | 高 | 中 | 后端统一存 UTC，返回 ISO；前端按浏览器时区展示 |
| R08 | 14 天工期偏紧 | 中 | 高 | 阶段 5、6 可适当简化交互（不影响验收），优先保功能完整 |

---

## 15. 验收清单（DoD）

### 15.1 功能验收

对照 spec §3，逐项打勾：

- [ ] F01 创建 Ticket
- [ ] F02 查看列表（含分页）
- [ ] F03 查看详情
- [ ] F04 编辑
- [ ] F05 删除（含二次确认）
- [ ] F06 修改状态（含流转校验）
- [ ] F07 设置优先级（含颜色）
- [ ] F08 添加/移除标签
- [ ] F09 指定负责人
- [ ] F10 时间记录
- [ ] F11~F14 四类筛选
- [ ] F15 关键字搜索（防抖、最少 2 字符）
- [ ] F16 排序

### 15.2 非功能验收（spec §12）

- [ ] NF01 列表 < 500ms
- [ ] NF02 首屏 < 2s
- [ ] NF03 分页 10/20/50/100 切换
- [ ] NF04 前后端校验完整
- [ ] NF05 统一错误格式
- [ ] NF06 三层架构清晰
- [ ] NF07 类型化代码 + mypy / tsc 通过
- [ ] NF08 三大浏览器最新两版可用
- [ ] NF09 1280px 宽度正常

### 15.3 工程验收

- [ ] CI 跑通所有质量门禁
- [ ] README 完整（运行 / 构建 / 部署 / 测试章节）
- [ ] Swagger UI 文档完整
- [ ] 生产环境一键部署成功
- [ ] 数据持久化验证通过

---

## 16. 角色分工与协作约定

### 16.1 角色

| 角色 | 主要职责 | 阶段参与 |
|------|----------|----------|
| 后端开发 | 阶段 1~3、阶段 8（后端测试）、阶段 9（后端镜像） | 1, 2, 3, 7, 8, 9 |
| 前端开发 | 阶段 4~6、阶段 8（前端测试）、阶段 9（前端镜像） | 4, 5, 6, 7, 8, 9 |
| 测试 / QA | 阶段 7~8 主导测试用例编写与执行 | 7, 8 |
| 运维 / 部署 | 阶段 9 主导部署，提供基础设施 | 9 |
| 项目负责人 | 阶段 0 启动；各阶段评审与风险跟踪 | 全程 |

### 16.2 协作约定

| 项 | 约定 |
|----|------|
| 每日同步 | 站会 15 分钟，过昨日 / 今日 / 阻塞 |
| 接口变更 | 必须先改 Swagger，再通知前端 |
| 分支策略 | `feature/*` → `dev` 通过 PR，至少 1 人 review |
| 提交信息 | Conventional Commits（feat / fix / docs / refactor / test / chore） |
| 评审节点 | 阶段 1 末（契约）、阶段 6 末（UI）、阶段 8 末（质量）、阶段 9 末（上线） |
| 文档维护 | spec 是契约源，所有偏离需评审通过后回写 spec |

---

## 附录 A：里程碑对应表

| spec §17 里程碑 | 本计划阶段 |
|------------------|-------------|
| M1 项目初始化 | 阶段 0 + 阶段 1 |
| M2 后端 CRUD | 阶段 2 |
| M3 后端筛选排序 | 阶段 3 |
| M4 前端列表页 | 阶段 4 + 阶段 5 |
| M5 前端详情与表单 | 阶段 6 |
| M6 前后端联调 | 阶段 7 |
| M7 测试与修复 | 阶段 8 |
| M8 部署上线 | 阶段 9 |

## 附录 B：交付物清单

| 类型 | 文件 / 内容 |
|------|-------------|
| 源代码 | `frontend/` `backend/` 完整目录 |
| 数据库 | `database/init.sql`、`backend/alembic/versions/*` |
| 文档 | `0001-spec.md`、`0002-implementation-plan.md`、`README.md`、Swagger UI |
| 部署 | `docker-compose.yml`、两个 `Dockerfile`、`nginx.conf`、`.env.example` |
| 测试 | `backend/tests/`、`frontend/src/**/*.test.tsx` |
| 报告 | 覆盖率报告（HTML）、性能测试报告 |
