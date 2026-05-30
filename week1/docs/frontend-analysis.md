# ProjectAlpha Frontend 代码分析文档

> 本文档对 `week1/frontend` 项目进行全方位深度分析，可作为从零开发具有相同功能项目的完整参考指南。

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术栈](#2-技术栈)
3. [目录结构详解](#3-目录结构详解)
4. [整体架构设计](#4-整体架构设计)
5. [核心模块分析](#5-核心模块分析)
6. [页面与组件设计](#6-页面与组件设计)
7. [API 层设计](#7-api-层设计)
8. [状态管理](#8-状态管理)
9. [Hooks 设计](#9-hooks-设计)
10. [UI 组件库](#10-ui-组件库)
11. [测试架构](#11-测试架构)
12. [Docker 部署](#12-docker-部署)
13. [从零开发指南](#13-从零开发指南)
14. [开发规范](#14-开发规范)

---

## 1. 项目概述

### 1.1 项目定位

ProjectAlpha Frontend 是一个**轻量级 Ticket（工单）管理工具的前端应用**，采用 React 19 + TypeScript 构建，提供 Ticket 的列表浏览、详情查看、新建、编辑、删除等功能。

### 1.2 核心功能

| 功能模块 | 描述 |
|---------|------|
| Ticket 列表 | 分页展示、筛选（状态/优先级/负责人/标签）、关键词搜索、排序 |
| Ticket 详情 | 展示完整信息、状态切换、编辑、删除 |
| 新建/编辑表单 | 标题、描述、优先级、负责人、标签 |
| 状态流转 | 可视化状态选择，限制合法流转 |
| URL 状态同步 | 筛选条件保存在 URL，支持分享和浏览器前进/后退 |
| 错误处理 | 全局 Toast 通知、加载状态、骨架屏 |

### 1.3 页面结构

```
┌─────────────────────────────────────────────────────────┐
│                      Header                              │
│  [Logo]  [Search Bar........................] [+ 新建]  │
├──────────────┬──────────────────────────────────────────┤
│   Sidebar    │                 Main                      │
│              │                                           │
│  筛选条件     │   Ticket 列表 / 详情页                    │
│  ├ 状态      │                                           │
│  ├ 优先级     │                                           │
│  ├ 负责人     │                                           │
│  └ 标签      │                                           │
│              │                                           │
└──────────────┴──────────────────────────────────────────┘
```

---

## 2. 技术栈

### 2.1 核心技术选型

| 层级 | 技术选型 | 版本要求 | 说明 |
|------|---------|---------|------|
| **UI 框架** | React | ^19.2.6 | 最新稳定版 |
| **类型系统** | TypeScript | ^6.0.3 | 严格模式 |
| **构建工具** | Vite | ^8.0.13 | 快速 HMR |
| **路由** | React Router | ^7.15.1 | 声明式路由 |
| **HTTP 客户端** | Axios | ^1.16.1 | 请求拦截器 |
| **CSS 框架** | TailwindCSS | ^4.3.0 | 原子化 CSS |
| **样式工具** | clsx + tailwind-merge | - | 条件类名 |

### 2.2 开发工具链

| 工具 | 用途 |
|-----|------|
| ESLint | 代码检查 |
| TypeScript ESLint | TypeScript 代码检查 |
| Prettier | 代码格式化 |
| Vitest | 单元测试 |
| Playwright | E2E 测试 |
| MSW | API Mock |

### 2.3 环境要求

```
Node.js >= 22
```

---

## 3. 目录结构详解

### 3.1 完整目录树

```
week1/frontend/
├── public/                          # 静态资源（空）
│
├── e2e/                            # Playwright E2E 测试
│   ├── fixtures.ts                # 测试数据准备
│   ├── list.spec.ts               # 列表页测试
│   └── crud.spec.ts               # CRUD 测试
│
├── src/
│   ├── __tests__/                # Vitest 冒烟测试
│   │   └── smoke.test.tsx
│   │
│   ├── api/                      # API 层
│   │   ├── request.ts            # Axios 实例封装
│   │   ├── tickets.ts           # Ticket CRUD 接口
│   │   ├── aggregations.ts      # 聚合接口 (tags/assignees)
│   │   └── api.test.ts          # API 测试
│   │
│   ├── components/              # UI 组件库
│   │   ├── ConfirmDialog/       # 确认弹窗
│   │   │   └── ConfirmDialog.tsx
│   │   ├── EmptyState/          # 空状态
│   │   │   └── EmptyState.tsx
│   │   ├── Header/              # 顶部导航栏
│   │   │   └── Header.tsx
│   │   ├── Layout/              # 三栏布局
│   │   │   └── Layout.tsx
│   │   ├── Modal/               # 通用弹窗
│   │   │   └── Modal.tsx
│   │   ├── SearchBar/           # 搜索框
│   │   │   └── SearchBar.tsx
│   │   ├── Sidebar/             # 侧边筛选栏
│   │   │   ├── Sidebar.tsx
│   │   │   ├── StatusFilter.tsx
│   │   │   ├── PriorityFilter.tsx
│   │   │   ├── AssigneeFilter.tsx
│   │   │   └── TagFilter.tsx
│   │   ├── Skeleton/            # 骨架屏
│   │   │   ├── TableSkeleton.tsx
│   │   │   └── DetailSkeleton.tsx
│   │   ├── TicketDetail/         # 详情页组件
│   │   │   └── StatusSelect.tsx
│   │   ├── TicketForm/          # 表单组件
│   │   │   ├── TicketForm.tsx
│   │   │   ├── PriorityRadio.tsx
│   │   │   └── TagInput.tsx
│   │   ├── TicketTable/         # 表格组件
│   │   │   ├── TicketTable.tsx
│   │   │   ├── SortBar.tsx
│   │   │   ├── Pagination.tsx
│   │   │   ├── StatusBadge.tsx
│   │   │   └── PriorityBadge.tsx
│   │   └── Toast/               # 消息通知
│   │       ├── ToastProvider.tsx
│   │       ├── Toast.tsx
│   │       ├── ErrorToast.tsx
│   │       ├── ToastContext.ts
│   │       └── useToast.ts
│   │
│   ├── constants/               # 常量定义
│   │   └── enums.ts            # 状态/优先级映射、颜色、流转规则
│   │
│   ├── hooks/                   # 自定义 Hooks
│   │   ├── useTicket.ts        # 加载单个 Ticket
│   │   ├── useTickets.ts       # 加载 Ticket 列表
│   │   ├── useAssignees.ts     # 加载负责人列表
│   │   ├── useTags.ts          # 加载标签列表
│   │   ├── useDebouncedValue.ts # 防抖 Hook
│   │   └── useTicketListUrlState.ts # URL 状态管理
│   │
│   ├── lib/                    # 工具函数
│   │   ├── cn.ts               # Tailwind 类名合并
│   │   ├── format.ts           # 日期格式化
│   │   └── queryString.ts      # URL 参数序列化
│   │
│   ├── mocks/                  # MSW Mock
│   │   ├── browser.ts
│   │   └── handlers.ts
│   │
│   ├── pages/                  # 页面组件
│   │   ├── TicketListPage/
│   │   │   └── index.tsx       # 列表页
│   │   ├── TicketDetailPage/
│   │   │   └── index.tsx       # 详情页
│   │   └── NotFoundPage/
│   │       └── index.tsx       # 404 页面
│   │
│   ├── types/                  # TypeScript 类型
│   │   ├── api.ts              # API 响应类型、ApiError
│   │   └── ticket.ts           # Ticket 领域类型
│   │
│   ├── App.tsx                # 路由配置
│   ├── main.tsx               # 入口文件
│   ├── index.css              # 全局样式
│   └── vite-env.d.ts          # Vite 类型声明
│
├── .dockerignore
├── .env.example               # 环境变量示例
├── Dockerfile                 # Docker 构建
├── eslint.config.js           # ESLint 配置
├── index.html                 # HTML 入口
├── nginx.conf                # Nginx 配置
├── package.json
├── playwright.config.ts      # Playwright 配置
├── prettier.config.js        # Prettier 配置
├── tsconfig.json
├── tsconfig.app.json
├── tsconfig.e2e.json
├── tsconfig.node.json
├── vitest.setup.ts           # Vitest 配置
└── README.md
```

### 3.2 目录结构设计理念

```
src/
├── api/           # API 层：与后端通信（axios 封装）
├── components/    # 展示组件：纯 UI，可复用
├── constants/     # 常量：枚举、映射表
├── hooks/         # 自定义 Hooks：逻辑复用
├── lib/           # 工具函数：纯函数
├── pages/         # 页面组件：组合组件，路由入口
├── types/         # 类型定义：TypeScript 类型
└── mocks/         # Mock：开发/测试用
```

**设计原则**：
- **API 层分离**：所有 HTTP 请求集中在 `api/` 目录
- **组件原子化**：小而专注的组件，可组合
- **类型驱动**：TypeScript 类型定义完整
- **URL 即状态**：筛选条件保存在 URL

---

## 4. 整体架构设计

### 4.1 应用入口

**main.tsx**：
```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import ToastProvider from '@/components/Toast/ToastProvider'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ToastProvider>
      <App />
    </ToastProvider>
  </StrictMode>,
)
```

**设计要点**：
- `ToastProvider` 在最外层，提供全局 Toast 能力
- `StrictMode` 帮助发现副作用问题

### 4.2 路由配置

**App.tsx**：
```tsx
export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<TicketListPage />} />
        <Route path="/tickets/:id" element={<TicketDetailPage />} />
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </BrowserRouter>
  )
}
```

**路由设计**：
- `/` - Ticket 列表页
- `/tickets/:id` - Ticket 详情页（动态路由）
- `*` - 404 页面

---

## 5. 核心模块分析

### 5.1 类型系统 (`src/types/`)

#### 5.1.1 Ticket 领域类型 (`ticket.ts`)

```typescript
export type TicketStatus = 'open' | 'in_progress' | 'done' | 'closed'
export type TicketPriority = 'low' | 'medium' | 'high' | 'urgent'

export interface Ticket {
  id: number
  title: string
  description: string | null
  status: TicketStatus
  priority: TicketPriority
  assignee: string | null
  tags: string[]
  created_at: string   // ISO 8601
  updated_at: string   // ISO 8601
}

export interface TicketCreateInput {
  title: string
  description?: string | null
  priority?: TicketPriority
  assignee?: string | null
  tags?: string[]
}

export interface TicketUpdateInput {
  title?: string
  description?: string | null
  priority?: TicketPriority
  assignee?: string | null
  tags?: string[]
  status?: TicketStatus   // 状态可单独更新
}
```

**为什么这样设计**：
- 与后端 Pydantic Schema 1:1 对齐
- `null` vs `undefined` 区分（后端处理）
- 输入类型与输出类型分离

#### 5.1.2 API 响应类型 (`api.ts`)

```typescript
// 统一响应信封
export interface ApiResponse<T> {
  code: number      // 0 表示成功
  message: string
  data: T | null
}

// 分页数据
export interface PageData<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}

// 业务错误
export class ApiError extends Error {
  public readonly code: number
  constructor(code: number, message: string) {
    super(message)
    this.code = code
  }
}
```

### 5.2 Axios 封装 (`src/api/request.ts`)

#### 5.2.1 请求配置

```typescript
const baseURL = import.meta.env.VITE_API_BASE_URL || '/api/v1'

export const request: AxiosInstance = axios.create({
  baseURL,
  timeout: 10_000,
  headers: { 'Content-Type': 'application/json' },
})
```

#### 5.2.2 响应拦截器

```typescript
request.interceptors.response.use(
  (resp) => {
    // 204 No Content 等无 body
    if (resp.status === 204 || resp.data == null) {
      return undefined
    }
    const body = resp.data as ApiResponse<unknown>
    // 非标准响应（不应发生）
    if (typeof body !== 'object' || !('code' in body)) {
      return resp.data
    }
    // 业务错误
    if (body.code !== 0) {
      throw new ApiError(body.code, body.message ?? 'unknown error')
    }
    return body.data
  },
  (err: AxiosError<ApiResponse<unknown>>) => {
    // 错误处理
    if (err.response?.data) {
      throw new ApiError(
        err.response.data.code ?? err.response.status,
        err.response.data.message ?? err.message ?? 'request failed',
      )
    }
    if (err.code === 'ECONNABORTED') {
      throw new ApiError(0, '请求超时，请稍后重试')
    }
    throw new ApiError(0, err.message || '网络错误')
  },
)
```

**设计要点**：
- 统一拆封 `{code, message, data}` 结构
- 业务错误转为 `ApiError`
- 网络/超时错误码为 0
- 默认超时 10 秒

### 5.3 Ticket 接口 (`src/api/tickets.ts`)

```typescript
// 查询参数序列化
function serializeQuery(q: TicketListQuery): Record<string, string | number> {
  const params: Record<string, string | number> = {}
  if (q.status?.length) params.status = q.status.join(',')  // 逗号分隔
  if (q.priority?.length) params.priority = q.priority.join(',')
  if (q.assignee) params.assignee = q.assignee
  if (q.tag) params.tag = q.tag
  if (q.keyword) params.keyword = q.keyword
  if (q.sort_by) params.sort_by = q.sort_by
  if (q.sort_order) params.sort_order = q.sort_order
  if (q.page) params.page = q.page
  if (q.page_size) params.page_size = q.page_size
  return params
}

export function listTickets(query: TicketListQuery, config?: AxiosRequestConfig)
export function getTicket(id: number)
export function createTicket(input: TicketCreateInput)
export function updateTicket(id: number, input: TicketUpdateInput)
export function updateTicketStatus(id: number, status: TicketStatus)
export function deleteTicket(id: number)
```

**接口对照表**：

| 函数 | 方法 | 路径 | 说明 |
|-----|------|------|------|
| `listTickets` | GET | `/tickets` | 分页列表 |
| `getTicket` | GET | `/tickets/:id` | 详情 |
| `createTicket` | POST | `/tickets` | 创建 |
| `updateTicket` | PUT | `/tickets/:id` | 更新 |
| `updateTicketStatus` | PATCH | `/tickets/:id/status` | 状态切换 |
| `deleteTicket` | DELETE | `/tickets/:id` | 删除 |

---

## 6. 页面与组件设计

### 6.1 列表页 (`TicketListPage`)

```tsx
export default function TicketListPage() {
  // URL 状态管理
  const { query, toggleStatus, togglePriority, setAssignee, ... } = useTicketListUrlState()
  
  // 数据加载
  const { data, loading, error, reload } = useTickets(query)
  
  // 新建弹窗
  const [createOpen, setCreateOpen] = useState(false)

  return (
    <Layout header={...} sidebar={...}>
      {/* 标题 + 排序 */}
      <h1>Ticket 列表</h1>
      <SortBar ... />
      
      {/* 错误提示 */}
      {error && <ErrorToast message={error.message} onRetry={reload} />}
      
      {/* 表格 */}
      <TicketTable items={items} loading={loading} />
      
      {/* 分页 */}
      <Pagination ... />
    </Layout>
    
    {/* 新建弹窗 */}
    <TicketForm open={createOpen} onClose={() => setCreateOpen(false)} onSubmitted={reload} />
  )
}
```

**设计要点**：
- URL 状态驱动：`useTicketListUrlState()` 同步 URL
- 错误处理：`ErrorToast` 显示 + 重试按钮
- 三态分离：加载中/空数据/有数据

### 6.2 详情页 (`TicketDetailPage`)

```tsx
export default function TicketDetailPage() {
  const params = useParams<{ id: string }>()
  const navigate = useNavigate()
  const toast = useToast()
  
  const id = params.id ? Number.parseInt(params.id, 10) : undefined
  const { data: ticket, loading, error, setData } = useTicket(id)
  
  // 加载失败 → toast → 跳转
  useEffect(() => {
    if (!error) return
    const message = error.code === 40401 ? 'Ticket 不存在' : error.message
    toast.error(message)
    navigate('/', { replace: true })
  }, [error, toast, navigate])

  // 删除处理
  const handleDelete = async () => {
    try {
      await deleteTicket(ticket!.id)
      toast.success(`已删除 Ticket #${ticket!.id}`)
      navigate('/', { replace: true })
    } catch (err) {
      toast.error(...)
    }
  }

  return (
    <main>
      {/* 返回按钮 */}
      <button onClick={() => navigate(-1)}>← 返回列表</button>
      
      {/* 信息展示 */}
      <header>
        <p>Ticket #{ticket.id}</p>
        <h1>{ticket.title}</h1>
      </header>
      
      <dl>
        <dt>状态</dt>
        <dd><StatusSelect ticket={ticket} onChanged={setData} /></dd>
        <dt>优先级</dt>
        <dd><PriorityBadge priority={ticket.priority} /></dd>
        {/* ... */}
      </dl>
      
      {/* 操作按钮 */}
      <button onClick={() => setEditOpen(true)}>编辑</button>
      <button onClick={() => setConfirmDeleteOpen(true)}>删除</button>
    </main>
  )
}
```

### 6.3 组件树结构

```
App
├── ToastProvider
│   └── Toast (浮层)
└── BrowserRouter
    └── Routes
        ├── TicketListPage
        │   └── Layout
        │       ├── Header
        │       │   ├── Logo
        │       │   ├── SearchBar
        │       │   └── 新建按钮
        │       ├── Sidebar
        │       │   ├── StatusFilter
        │       │   ├── PriorityFilter
        │       │   ├── AssigneeFilter
        │       │   └── TagFilter
        │       └── Main
        │           ├── SortBar
        │           ├── ErrorToast
        │           ├── TicketTable
        │           │   ├── StatusBadge
        │           │   └── PriorityBadge
        │           ├── Pagination
        │           └── TicketForm (Modal)
        │
        ├── TicketDetailPage
        │   ├── 返回按钮
        │   ├── 详情卡片
        │   │   ├── StatusSelect
        │   │   └── PriorityBadge
        │   ├── 编辑按钮
        │   ├── 删除按钮
        │   ├── TicketForm (Modal)
        │   └── ConfirmDialog (Modal)
        │
        └── NotFoundPage
```

---

## 7. API 层设计

### 7.1 请求流程

```
组件调用 API 函数
    ↓
api/tickets.ts (封装请求参数)
    ↓
api/request.ts (axios 实例)
    ↓
HTTP 请求 → 后端
    ↓
响应拦截器 (拆封 {code, data, message})
    ↓
业务错误 → ApiError
    ↓
组件获取数据 / 处理错误
```

### 7.2 请求取消机制

```typescript
// useTickets.ts
useEffect(() => {
  const ctrl = new AbortController()  // 创建控制器
  setState((s) => ({ ...s, loading: true }))
  listTickets(query, { signal: ctrl.signal })
    .then((data) => {
      if (ctrl.signal.aborted) return  // 已取消则忽略
      setState({ data, loading: false })
    })
    .catch((err) => {
      if (ctrl.signal.aborted) return  // 已取消则忽略
      // 处理错误...
    })
  return () => ctrl.abort()  // 组件卸载时取消
}, [queryKey])
```

**为什么需要请求取消**：
- 快速切换筛选条件时，旧请求会返回
- 不取消会导致竞态：旧请求覆盖新请求结果

### 7.3 错误处理模式

```typescript
// 方式1：try-catch
try {
  const ticket = await updateTicket(id, payload)
  toast.success('已保存')
  onSubmitted(ticket)
} catch (err) {
  const message = err instanceof ApiError ? err.message : '保存失败'
  toast.error(message)
}

// 方式2：useTickets 的错误状态
const { data, loading, error, reload } = useTickets(query)
if (error) {
  return <ErrorToast message={error.message} onRetry={reload} />
}
```

---

## 8. 状态管理

### 8.1 URL 即状态

核心思想：**所有筛选条件保存在 URL**

```typescript
// URL: /?status=open,in_progress&priority=high&page=2

// useTicketListUrlState Hook
export function useTicketListUrlState() {
  const [params, setParams] = useSearchParams()
  
  // URL → Query
  const query = useMemo(() => parseQueryFromSearch(params), [params])
  
  // Query → URL
  const setQuery = useCallback((next) => {
    setParams(buildSearchFromQuery(next), { replace: false })
  }, [setParams])
  
  return { query, setQuery, ... }
}
```

**为什么这样做**：
1. **可分享**：URL 可分享、收藏
2. **可回退**：浏览器前进/后退正常工作
3. **SSR 友好**：刷新页面不丢失状态
4. **简单**：无需 Redux/Zustand

### 8.2 URL 参数序列化

**URL → Query**：
```typescript
// ?status=open,in_progress&priority=high&page=2
parseQueryFromSearch(params) => {
  status: ['open', 'in_progress'],
  priority: ['high'],
  page: 2,
  // ...
}
```

**Query → URL**：
```typescript
// 跳过默认值
buildSearchFromQuery({
  status: ['open'],
  page: 1,  // 默认值，不写入 URL
}) => "?status=open"
```

### 8.3 状态流转规则

```typescript
// constants/enums.ts
export const STATUS_TRANSITIONS: Record<TicketStatus, TicketStatus[]> = {
  open: ['in_progress', 'closed'],
  in_progress: ['done', 'closed'],
  done: ['closed'],
  closed: ['open'],
}

// 状态选择器使用
const allowed = STATUS_TRANSITIONS[currentStatus]
// 限制可选择的目标状态
```

---

## 9. Hooks 设计

### 9.1 useTickets - 列表加载

```typescript
interface State {
  data: PageData<Ticket> | null
  loading: boolean
  error: ApiError | null
}

export function useTickets(query: TicketListQuery): State & {
  reloadKey: number
  reload: () => void
} {
  const [state, setState] = useState<State>(INITIAL)
  const [reloadKey, setReloadKey] = useState(0)
  
  useEffect(() => {
    const ctrl = new AbortController()
    listTickets(query, { signal: ctrl.signal })
      .then((data) => {
        if (ctrl.signal.aborted) return
        setState({ data, loading: false, error: null })
      })
      .catch((err) => {
        if (ctrl.signal.aborted) return
        const apiErr = err instanceof ApiError ? err : new ApiError(0, String(err))
        setState({ data: null, loading: false, error: apiErr })
      })
    return () => ctrl.abort()
  }, [queryKey, reloadKey])
  
  return { ...state, reloadKey, reload: () => setReloadKey((n) => n + 1) }
}
```

### 9.2 useTicket - 详情加载

```typescript
export function useTicket(id: number | undefined): State & {
  setData: (next: Ticket | null) => void
  reload: () => void
} {
  const [state, setState] = useState<State>(INITIAL)
  const [reloadKey, setReloadKey] = useState(0)
  
  useEffect(() => {
    if (id === undefined) {
      setState({ data: null, loading: false, error: new ApiError(40400, '非法 Ticket ID') })
      return
    }
    const ctrl = new AbortController()
    getTicket(id)
      .then((data) => {
        if (ctrl.signal.aborted) return
        setState({ data, loading: false, error: null })
      })
      .catch((err) => { ... })
    return () => ctrl.abort()
  }, [id, reloadKey])
  
  // setData: 允许父级更新数据，无需重新请求
  return { ...state, setData: (next) => setState((s) => ({ ...s, data: next })), reload }
}
```

### 9.3 useTicketListUrlState - URL 双向绑定

```typescript
export function useTicketListUrlState() {
  const [params, setParams] = useSearchParams()
  const query = useMemo(() => parseQueryFromSearch(params), [params])
  
  const setQuery = useCallback((next) => {
    setParams(buildSearchFromQuery(next), { replace: false })
  }, [setParams])
  
  // 切换单个状态
  const toggleStatus = useCallback((s: TicketStatus) => {
    const current = query.status ?? []
    const next = current.includes(s) ? current.filter(x => x !== s) : [...current, s]
    updateFilter({ status: next.length ? next : undefined })
  }, [query.status, updateFilter])
  
  // 重置分页
  const updateFilter = useCallback((patch) => {
    setQuery({ ...query, ...patch, page: 1 })
  }, [query, setQuery])
  
  return { query, setQuery, toggleStatus, togglePriority, setAssignee, setTag, setKeyword, setSort, setPage, setPageSize, clearAll }
}
```

### 9.4 useToast - 全局 Toast

```typescript
// ToastContext.ts
interface ToastContextValue {
  show: (kind: ToastKind, message: string) => void
  success: (message: string) => void
  error: (message: string) => void
  info: (message: string) => void
}

// useToast.ts
export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext)
  if (!ctx) throw new Error('useToast 必须在 <ToastProvider> 内部使用')
  return ctx
}

// 使用
const toast = useToast()
toast.success('保存成功')
toast.error('保存失败')
```

---

## 10. UI 组件库

### 10.1 组件清单

| 组件 | 路径 | 说明 |
|-----|------|------|
| `Layout` | `components/Layout/` | 三栏布局容器 |
| `Header` | `components/Header/` | 顶部导航栏 |
| `Sidebar` | `components/Sidebar/` | 侧边筛选栏 |
| `SearchBar` | `components/SearchBar/` | 搜索输入框 |
| `TicketTable` | `components/TicketTable/` | 数据表格 |
| `TicketForm` | `components/TicketForm/` | 新建/编辑表单 |
| `Modal` | `components/Modal/` | 通用弹窗 |
| `ConfirmDialog` | `components/ConfirmDialog/` | 确认对话框 |
| `Pagination` | `components/TicketTable/` | 分页控件 |
| `StatusBadge` | `components/TicketTable/` | 状态徽章 |
| `PriorityBadge` | `components/TicketTable/` | 优先级徽章 |
| `StatusSelect` | `components/TicketDetail/` | 状态选择器 |
| `ToastProvider` | `components/Toast/` | Toast 容器 |
| `Toast` | `components/Toast/` | 单条 Toast |
| `EmptyState` | `components/EmptyState/` | 空状态 |
| `Skeleton` | `components/Skeleton/` | 骨架屏 |

### 10.2 设计系统

**TailwindCSS v4 CSS-first 配置** (`index.css`)：

```css
@import 'tailwindcss';

@theme {
  /* 颜色 */
  --color-primary: #0066ff;
  --color-brand-amber: #ffc107;
  --color-brand-green: #4caf50;
  --color-gray-900: #1a1f26;
  --color-gray-600: #6c757d;
  --color-gray-50: #f8f9fa;
  
  /* 字体 */
  --font-sans: 'Inter', -apple-system, ...;
  
  /* 圆角 */
  --radius-sm: 0.25rem;
  --radius-md: 0.5rem;
  --radius-lg: 0.75rem;
}
```

### 10.3 状态颜色

```typescript
// StatusBadge
const STATUS_BG: Record<TicketStatus, string> = {
  open: 'bg-gray-100 text-gray-700',
  in_progress: 'bg-blue-50 text-primary',
  done: 'bg-emerald-50 text-emerald-700',
  closed: 'bg-gray-200 text-gray-500',
}

// PriorityBadge
export const PRIORITY_COLOR: Record<TicketPriority, string> = {
  low: '#6C757D',      // gray: 低
  medium: '#0066FF',   // blue: 中
  high: '#FA8C16',     // orange: 高
  urgent: '#FF4D4F',   // red: 紧急
}
```

### 10.4 通用弹窗 (Modal)

```typescript
interface ModalProps {
  open: boolean
  onClose: () => void
  title?: string
  size?: 'sm' | 'md' | 'lg'
  closeOnOverlayClick?: boolean
  closeOnEsc?: boolean
  children: ReactNode
  footer?: ReactNode
}
```

**功能**：
- ESC 键关闭
- 点击蒙层关闭（可配置）
- 打开时锁住 body 滚动
- 清理时还原滚动状态

### 10.5 表单组件 (TicketForm)

```typescript
interface TicketFormProps {
  open: boolean
  initial?: Ticket        // 提供则为编辑模式
  onClose: () => void
  onSubmitted: (saved: Ticket) => void
}
```

**特性**：
- 新建/编辑共用
- 脏值检测：未保存关闭时提示确认
- 表单验证：标题非空、最长200字符
- 负责人输入带历史下拉提示

---

## 11. 测试架构

### 11.1 测试类型

| 类型 | 框架 | 位置 | 覆盖范围 |
|-----|------|------|---------|
| 单元测试 | Vitest | `src/__tests__/` | 冒烟测试 |
| API 测试 | Vitest | `src/api/` | 接口封装 |
| 组件测试 | Vitest + RTL | `src/components/` | 组件渲染 |
| E2E 测试 | Playwright | `e2e/` | 完整流程 |

### 11.2 Vitest 配置

```typescript
// vite.config.ts
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./vitest.setup.ts'],
  },
})
```

### 11.3 E2E 测试用例

**crud.spec.ts**：
- 新建 Ticket
- 编辑 Ticket
- 切换状态
- 删除 Ticket

**list.spec.ts**：
- 列表加载
- 状态筛选
- 优先级筛选
- 关键字搜索
- 分页

---

## 12. Docker 部署

### 12.1 多阶段构建

```dockerfile
# 构建阶段
FROM node:22-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build:docker

# 运行阶段
FROM nginx:1.27-alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### 12.2 Nginx 配置

```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    
    # 静态资源缓存
    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    # API 反向代理
    location /api/ {
        proxy_pass http://backend:8000/api/;
    }
    
    # 健康检查
    location /healthz { proxy_pass http://backend:8000/healthz; }
    location /readyz { proxy_pass http://backend:8000/readyz; }
}
```

---

## 13. 从零开发指南

### 13.1 项目初始化

```bash
# 1. 创建项目
mkdir frontend && cd frontend
npm create vite@latest . -- --template react-ts

# 2. 安装依赖
npm install react-router-dom axios clsx tailwind-merge
npm install -D tailwindcss @tailwindcss/vite @types/node

# 3. 配置 TailwindCSS v4 (vite.config.ts)
import tailwindcss from '@tailwindcss/vite'
export default defineConfig({
  plugins: [react(), tailwindcss()],
})

# 4. 创建目录结构
mkdir -p src/{api,components/{Header,Layout,Sidebar,Modal,TicketForm,TicketTable,TicketDetail,Toast,EmptyState,Skeleton,ConfirmDialog,SearchBar},constants,hooks,lib,mocks,pages/{TicketListPage,TicketDetailPage,NotFoundPage},types}
```

### 13.2 类型定义

**src/types/ticket.ts**：
```typescript
export type TicketStatus = 'open' | 'in_progress' | 'done' | 'closed'
export type TicketPriority = 'low' | 'medium' | 'high' | 'urgent'

export interface Ticket {
  id: number
  title: string
  description: string | null
  status: TicketStatus
  priority: TicketPriority
  assignee: string | null
  tags: string[]
  created_at: string
  updated_at: string
}

export interface TicketCreateInput {
  title: string
  description?: string | null
  priority?: TicketPriority
  assignee?: string | null
  tags?: string[]
}

export interface TicketUpdateInput extends Partial<TicketCreateInput> {
  status?: TicketStatus
}
```

### 13.3 API 封装

**src/api/request.ts**：
```typescript
import axios, { AxiosInstance, AxiosError } from 'axios'

const baseURL = import.meta.env.VITE_API_BASE_URL || '/api/v1'

export const request: AxiosInstance = axios.create({
  baseURL,
  timeout: 10_000,
  headers: { 'Content-Type': 'application/json' },
})

export class ApiError extends Error {
  constructor(public code: number, message: string) {
    super(message)
  }
}

request.interceptors.response.use(
  (resp) => {
    if (resp.status === 204 || resp.data == null) return undefined
    const body = resp.data
    if (body.code !== 0) throw new ApiError(body.code, body.message)
    return body.data
  },
  (err: AxiosError) => {
    const data = err.response?.data as any
    if (data?.code) throw new ApiError(data.code, data.message)
    if (err.code === 'ECONNABORTED') throw new ApiError(0, '请求超时')
    throw new ApiError(0, err.message || '网络错误')
  },
)
```

**src/api/tickets.ts**：
```typescript
import { request } from './request'
import type { Ticket, TicketCreateInput, TicketUpdateInput, TicketStatus } from '@/types/ticket'
import type { PageData } from '@/types/api'

export function listTickets(query: Record<string, any> = {}) {
  return request.get<PageData<Ticket>>('/tickets', { params: query })
}

export function getTicket(id: number) {
  return request.get<Ticket>(`/tickets/${id}`)
}

export function createTicket(data: TicketCreateInput) {
  return request.post<Ticket>('/tickets', data)
}

export function updateTicket(id: number, data: TicketUpdateInput) {
  return request.put<Ticket>(`/tickets/${id}`, data)
}

export function updateTicketStatus(id: number, status: TicketStatus) {
  return request.patch<Ticket>(`/tickets/${id}/status`, { status })
}

export function deleteTicket(id: number) {
  return request.delete(`/tickets/${id}`)
}
```

### 13.4 Hooks 实现

**src/hooks/useTickets.ts**：
```typescript
import { useEffect, useState } from 'react'
import { listTickets } from '@/api/tickets'
import { ApiError } from '@/types/api'
import type { Ticket } from '@/types/ticket'
import type { PageData } from '@/types/api'

interface State {
  data: PageData<Ticket> | null
  loading: boolean
  error: ApiError | null
}

export function useTickets(query: Record<string, any>) {
  const [state, setState] = useState<State>({ data: null, loading: true, error: null })
  const [reloadKey, setReloadKey] = useState(0)

  useEffect(() => {
    const ctrl = new AbortController()
    setState(s => ({ ...s, loading: true }))
    listTickets(query, { signal: ctrl.signal })
      .then(data => {
        if (ctrl.signal.aborted) return
        setState({ data, loading: false, error: null })
      })
      .catch(err => {
        if (ctrl.signal.aborted) return
        const apiErr = err instanceof ApiError ? err : new ApiError(0, String(err))
        setState({ data: null, loading: false, error: apiErr })
      })
    return () => ctrl.abort()
  }, [JSON.stringify(query), reloadKey])

  return { ...state, reload: () => setReloadKey(k => k + 1) }
}
```

### 13.5 Toast 实现

**src/components/Toast/ToastContext.ts**：
```typescript
import { createContext } from 'react'

export type ToastKind = 'success' | 'error' | 'info'

export interface ToastItem {
  id: string
  kind: ToastKind
  message: string
}

export interface ToastContextValue {
  show: (kind: ToastKind, message: string) => void
  success: (message: string) => void
  error: (message: string) => void
  info: (message: string) => void
}

export const ToastContext = createContext<ToastContextValue | null>(null)
```

**src/components/Toast/ToastProvider.tsx**：
```typescript
import { useState, useCallback, useMemo } from 'react'
import { ToastContext, type ToastContextValue } from './ToastContext'
import Toast from './Toast'

export default function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([])

  const show = useCallback((kind: ToastKind, message: string) => {
    const id = crypto.randomUUID()
    setToasts(prev => [...prev, { id, kind, message }])
  }, [])

  const dismiss = useCallback((id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id))
  }, [])

  const value = useMemo<ToastContextValue>(() => ({
    show,
    success: m => show('success', m),
    error: m => show('error', m),
    info: m => show('info', m),
  }), [show])

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div className="fixed right-4 top-4 z-[100] flex flex-col gap-2">
        {toasts.map(t => <Toast key={t.id} toast={t} onDismiss={dismiss} />)}
      </div>
    </ToastContext.Provider>
  )
}
```

### 13.6 页面实现

**src/pages/TicketListPage/index.tsx**：
```typescript
import { useState } from 'react'
import Layout from '@/components/Layout/Layout'
import Header from '@/components/Header/Header'
import Sidebar from '@/components/Sidebar/Sidebar'
import TicketTable from '@/components/TicketTable/TicketTable'
import Pagination from '@/components/TicketTable/Pagination'
import { useTickets } from '@/hooks/useTickets'
import { useTicketListUrlState } from '@/hooks/useTicketListUrlState'

export default function TicketListPage() {
  const { query, toggleStatus, togglePriority, setAssignee, setTag, setKeyword, setPage, setPageSize } = useTicketListUrlState()
  const { data, loading, error, reload } = useTickets(query)
  const [createOpen, setCreateOpen] = useState(false)

  return (
    <Layout
      header={<Header keyword={query.keyword} onKeywordChange={setKeyword} onNewTicket={() => setCreateOpen(true)} />}
      sidebar={<Sidebar query={query} onToggleStatus={toggleStatus} onTogglePriority={togglePriority} onChangeAssignee={setAssignee} onChangeTag={setTag} />}
    >
      <TicketTable items={data?.items ?? []} loading={loading} />
      {data && <Pagination page={data.page} pageSize={data.page_size} total={data.total} onPageChange={setPage} onPageSizeChange={setPageSize} />}
    </Layout>
  )
}
```

### 13.7 路由配置

**src/App.tsx**：
```typescript
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import TicketListPage from '@/pages/TicketListPage'
import TicketDetailPage from '@/pages/TicketDetailPage'
import NotFoundPage from '@/pages/NotFoundPage'
import ToastProvider from '@/components/Toast/ToastProvider'

export default function App() {
  return (
    <ToastProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<TicketListPage />} />
          <Route path="/tickets/:id" element={<TicketDetailPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </BrowserRouter>
    </ToastProvider>
  )
}
```

### 13.8 全局样式

**src/index.css**：
```css
@import 'tailwindcss';

@theme {
  --color-primary: #0066ff;
  --color-gray-900: #1a1f26;
  --color-gray-600: #6c757d;
  --color-gray-50: #f8f9fa;
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

html, body, #root {
  height: 100%;
  margin: 0;
}

body {
  font-family: var(--font-sans);
  background-color: var(--color-gray-50);
  color: var(--color-gray-900);
}
```

### 13.9 启动命令

```bash
# 开发
npm run dev

# 构建
npm run build

# 测试
npm run test

# E2E
npm run e2e
```

---

## 14. 开发规范

### 14.1 代码风格

- **TypeScript**：严格模式，全量类型注解
- **命名规范**：
  - 组件：`PascalCase`
  - Hooks：`camelCase`，以 `use` 开头
  - 类型：`PascalCase`
  - 工具函数：`camelCase`
- **文件组织**：
  - 每个组件一个文件夹
  - `index.tsx` 作为入口

### 14.2 ESLint 配置

```javascript
// eslint.config.js
export default tseslint.config(
  { ignores: ['dist', 'node_modules'] },
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    rules: {
      'react-hooks/recommended': 'error',
      'react-refresh/only-export-components': 'warn',
    },
  },
)
```

### 14.3 Prettier 配置

```javascript
// prettier.config.js
export default {
  semi: false,
  singleQuote: true,
  trailingComma: 'all',
  printWidth: 100,
  tabWidth: 2,
  arrowParens: 'always',
}
```

### 14.4 环境变量

```env
VITE_API_BASE_URL=/api/v1
VITE_ENABLE_MOCK=false
```

### 14.5 Git 提交规范

```
feat: 新功能
fix: 修复 bug
docs: 文档更新
style: 代码格式
refactor: 重构
test: 测试
chore: 构建/工具
```

---

## 附录

### A. 快速启动命令

```bash
# 安装依赖
npm install

# 开发服务器
npm run dev

# 类型检查
npm run typecheck

# Lint
npm run lint

# 格式化
npm run format

# 测试
npm run test

# 构建
npm run build
```

### B. 浏览器兼容

| 浏览器 | 版本 |
|-------|------|
| Chrome | >= 90 |
| Firefox | >= 88 |
| Safari | >= 14 |
| Edge | >= 90 |

### C. 文件清单

本项目共有约 **70+ 个文件**，核心业务代码约 2000 行。

---

> 文档版本：1.0.0  
> 生成时间：2026-05-30  
> 项目仓库：d:\AiLearning\week1\frontend
