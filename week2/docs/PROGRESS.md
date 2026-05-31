# SmartQuery AI - 开发进度追踪

> 更新时间: 2026-05-31  
> 本次核对依据: `week2` 代码、`cargo check`、`npm run build`

## 项目概述

- **项目名称**: SmartQuery AI - 智能双模数据库查询与分析系统
- **技术栈**: Rust (Axum) + React 19
- **文档版本**: specs/week2/0001-需求分析与设计.md
- **实现计划**: specs/week2/0002-实现计划.md

---

## 总体进度

```
[==============----------] 60%
```

**结论**: 后端 `cargo check` 和前端 `npm run build` 均已通过；剩余未完成项集中在部署配置和外部服务配置。

## 已完成

### 后端基础

| 项目 | 状态 | 说明 |
|------|------|------|
| Rust 后端基础结构 | ✅ 已完成 | `Cargo.toml`、`main.rs`、`lib.rs`、配置、错误处理、状态模块存在 |
| 后端数据模型 | ✅ 已完成 | `models/` 下用户、连接、查询、对话、语义、指标模型存在 |
| 后端仓储层 | ✅ 已完成 | 用户、连接、查询、对话仓储存在 |
| 后端服务层 | ✅ 已完成 | 认证、LLM、SQL 分析/执行、连接管理、Schema 检索、图表生成、脱敏服务存在 |
| 后端编译 | ✅ 已完成 | `cargo check` 通过 |
| 数据库迁移 | ✅ 已完成 | `migrations/001_initial_schema.sql` 存在 |

### 当前主路由中已完成的 API

| API | 状态 | 说明 |
|-----|------|------|
| `POST /api/v1/auth/login` | ✅ 已完成 | 验证用户、生成 access/refresh token |
| `POST /api/v1/auth/register` | ✅ 已完成 | 创建用户 |
| `POST /api/v1/auth/refresh` | ✅ 已完成 | 验证 refresh token 并重新签发 token |
| `POST /api/v1/auth/logout` | ✅ 已完成 | 已将 refresh_token 加入 `state.token_blacklist` |
| 用户 CRUD | ✅ 已完成 | 列表、详情、更新、删除、改密已接入主路由 |
| 连接 CRUD | ✅ 已完成 | 列表、创建、详情、更新、删除、设默认已接入主路由 |
| `POST /api/v1/connections/{id}/test` | ✅ 已完成 | 调用 `connection_manager` 实际测试 PostgreSQL/MySQL 连接 |
| `GET /api/v1/connections/{id}/schema` | ✅ 已完成 | 调用 `connection_manager.get_schema` 获取真实 schema |
| `POST /api/v1/sql/format` | ✅ 已完成 | 使用 `sqlparser` 解析并格式化 SQL |
| `GET /api/v1/sql/history` | ✅ 已完成 | 从 `query_repo` 读取查询历史 |
| `POST /api/v1/sql/explain` | ✅ 已完成 | 对目标连接执行 `EXPLAIN` |
| `POST /api/v1/sql/preview` | ✅ 已完成 | 查询目标表真实数据预览 |
| `POST /api/v1/nl/convert` | ✅ 已完成 | 调用 `llm_client.convert_nl_to_sql`，并尝试附带 schema 上下文 |
| 对话 CRUD | ✅ 已完成 | 对话列表、创建、详情、删除、消息列表已接入主路由 |
| `POST /api/v1/conversations/{id}/messages` | ✅ 已完成 | 写入用户消息，调用 LLM 生成助手消息 |
| `POST /api/v1/charts/recommend` | ✅ 已完成 | 调用 `ChartGenerator.recommend` |
| `POST /api/v1/charts/generate` | ✅ 已完成 | 调用 `ChartGenerator.switch_chart_type` |
| `POST /api/v1/charts/export` | ✅ 已完成 | 将图表配置导出为临时文件 |
| `GET /api/v1/metrics/{id}/lineage` | ✅ 已完成 | 对指标 SQL 做基础血缘解析 |

### 前端已创建文件

| 项目 | 状态 | 说明 |
|------|------|------|
| React 项目基础 | ✅ 已完成 | `package.json`、Vite、TypeScript、Tailwind 配置存在 |
| 全局路由和布局 | ✅ 已完成 | `App.tsx`、`MainLayout.tsx` 存在 |
| 状态管理 | ✅ 已完成 | auth、connection、chat store 存在 |
| API 客户端 | ✅ 已完成 | Axios client 和 auth API 存在 |
| 登录页 | ✅ 已完成 | 登录页面和认证调用存在 |
| 图表渲染器 | ✅ 已完成 | `components/Chart/ChartRenderer.tsx` 存在 |
| 连接管理面板 | ✅ 已完成 | `components/Connection/ConnectionPanel.tsx` 存在 |
| 查询结果组件 | ✅ 已完成 | `pages/SqlMode/QueryResult.tsx` 存在 |
| SQL 预览弹窗 | ✅ 已完成 | `pages/ChatMode/SqlPreviewModal.tsx` 存在 |
| 用户管理页面 | ✅ 已完成 | `pages/Admin/UserManagement.tsx` 存在 |

---

## 未完成

### 后端修复状态

| 项目 | 状态 | 说明 |
|------|------|------|
| `POST /api/v1/sql/execute` | ✅ 已完成 | 主路由已调用目标连接池执行 SELECT SQL，并记录查询历史 |
| `POST /api/v1/nl/execute` | ✅ 已完成 | 已执行请求中的 SQL，返回真实查询结果，并支持可选图表配置 |
| 指标创建 | ✅ 已完成 | `create_metric_handler` 已写入 `METRICS_STORE` |
| 指标更新 | ✅ 已完成 | `update_metric_handler` 已真实更新 `METRICS_STORE` |
| 指标删除 | ✅ 已完成 | `delete_metric_handler` 已真实删除 `METRICS_STORE` 中的指标 |
| 指标列表/详情/执行 | ✅ 已完成 | 创建、列表、详情、执行已基于 `METRICS_STORE` 可用，执行会查询目标数据库 |
| 独立 API 模块接入 | ✅ 已完成 | 当前主路由 `api/routes.rs` 已覆盖用户、连接、SQL、NL、对话、图表、指标和认证 API；重复备用模块无需接入主路由 |
| 查询历史用户归属 | ✅ 已完成 | 主路由 SQL/NL 执行已使用 `UserSession.user_id` 记录查询历史；未接入的备用模块不影响当前 API |
| Token 黑名单校验 | ✅ 已完成 | refresh 流程已拒绝黑名单中的 refresh_token |
| SQL 预览安全性 | ✅ 已完成 | `preview_data_handler` 已校验表名格式并限制 limit 范围 |
| MySQL/PG 行数据转换 | ✅ 已完成 | SQL 执行和预览已使用真实列名、类型名，并按常见标量类型转换为 JSON |

### 前端修复状态

| 项目 | 状态 | 说明 |
|------|------|------|
| 前端构建 | ✅ 已完成 | `npm run build` 已通过 |
| `ExecutionPlan.tsx` | ✅ 已完成 | 已修复复制按钮重复 JSX 属性导致的语法错误 |
| `SqlWorkspacePage.tsx` SQL 格式化 | ✅ 已完成 | `handleFormat` 已调用 `/sql/format` API 并回填编辑器 |
| 前端源码中文文案 | ✅ 已完成 | 当前构建涉及的中文源码已按 UTF-8 正常解析，构建通过 |
| SQL 执行认证头 | ✅ 已完成 | SQL 执行页已统一改为 Axios API 客户端，自动携带 access token |
| 认证响应解包 | ✅ 已完成 | `authApi` 和 refresh 拦截器已按后端 `{ code, message, data }` 结构解包 |
| 用户管理认证头 | ✅ 已完成 | 用户列表/禁用已改为 Axios API 客户端，避免读取错误 localStorage key |
| NL 转 SQL 前端请求 | ✅ 已完成 | 对话页已使用 Axios API 客户端，并按当前连接传入 dialect |
| NL 确认执行链路 | ✅ 已完成 | 对话页“执行查询”按钮已调用 `/nl/execute` 并回填执行摘要 |
| 图表 Hooks 请求体 | ✅ 已完成 | `useChart` 已按后端 `columns/rows/chart_type/config` 契约调用，默认导出 SVG |
| OpenAI 兼容响应解析 | ✅ 已完成 | `LlmClient` 已兼容 OpenAI chat completions 响应结构，支持字符串和分段 content |
| ClickHouse 最小链路 | ✅ 已完成 | 已通过 ClickHouse HTTP 接口支持连接测试、Schema 获取、SELECT 执行、预览、EXPLAIN、指标执行和 SQL 格式化 |
| PNG 图表导出 | ✅ 已完成 | `/charts/export` 已支持写入前端传入的 PNG data URL/base64 |
| 图表推荐接口兼容 | ✅ 已完成 | `/charts/recommend` 保留 POST 请求体推荐，并新增 GET 查询入口兼容文档接口形态 |
| 运行时 Mock 清理 | ✅ 已完成 | 未接入备用对话模块不再在 LLM 失败时返回模拟 SQL；运行时源码未发现 mock/模拟返回 |
| 审计日志查询 API | ✅ 已完成 | 已新增 `/api/v1/audit-logs` 管理员分页查询和搜索，并接入前端审计页 |
| 语义层 API | ✅ 已完成 | 已新增 `/api/v1/semantics` 列表、创建、更新、删除、批量创建和统计接口 |
| SQL 工作区 Schema | ✅ 已完成 | SQL 页面选择连接后已调用 `/connections/{id}/schema` 展示真实表结构 |
| SQL 结果导出 | ✅ 已完成 | SQL 页面已支持当前结果 CSV/JSON 浏览器导出 |
| SQL 执行分页 | ✅ 已完成 | `/sql/execute` 已支持可选 `page/page_size`，响应包含 `total/page/page_size`，未传分页参数时保持完整结果兼容 |
| SQL 优化建议 | ✅ 已完成 | `/sql/explain` 已返回基础 `warnings` 和 `suggestions` |
| 连接更新调用链 | ✅ 已完成 | 连接编辑弹窗已接入 `PUT /connections/{id}`，编辑时密码留空不会覆盖原密码 |
| 通用组件 | ✅ 已完成 | 已创建 `Common/Button`、`Common/Modal`、`Common/Table` 并通过构建 |
| SQL 编辑器封装 | ✅ 已完成 | 已创建 `components/Editor/SqlEditor.tsx` 并通过构建 |
| 角色管理页面 | ✅ 已完成 | 已创建 `pages/Admin/RoleManagement.tsx` 并通过构建 |
| 审计日志页面 | ✅ 已完成 | `pages/Admin/AuditLog.tsx` 已接入真实 `/audit-logs` API |
| 语义层页面 | ✅ 已完成 | 已新增 `pages/Admin/SemanticManagement.tsx` 并接入语义层 API |

### 部署与外部服务

| 项目 | 状态 | 说明 |
|------|------|------|
| Docker 配置 | ❌ 未完成 | 未发现 Dockerfile/compose |
| K8s 配置 | ❌ 未完成 | 未发现部署清单 |
| 监控告警 | ❌ 未完成 | 未发现监控配置 |
| OpenAI API Key | ❌ 未完成 | 需要在环境变量中配置真实 `OPENAI_API_KEY` 或项目实际使用的 LLM key |
| PostgreSQL/Redis/pgvector 本地服务 | ❌ 未完成 | 依赖外部服务，需要单独配置和验证 |

---

## 阶段进度详情

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase 1: 基础搭建 | ✅ 已完成 | 后端基础、模型、仓储、服务、前端基础均已创建；后端编译通过 |
| Phase 2: SQL 模式 | ✅ 已完成 | SQL 执行、格式化、历史、EXPLAIN、预览、Schema 展示、CSV/JSON 导出已接入 |
| Phase 3: NL 模式 | ✅ 已完成 | NL 转 SQL 调用 LLM，NL execute 已执行真实 SQL |
| Phase 4: 图表功能 | ✅ 已完成 | 主路由已接入图表推荐、生成、导出 |
| Phase 5: 语义层/指标 | ✅ 已完成 | 语义定义 API 和页面、指标 API 创建、列表、详情、更新、删除、执行、血缘已接入 |
| Phase 6: 权限系统 | ✅ 已完成 | 后端用户、认证、refresh token 黑名单、审计写入和审计查询页面已完成 |
| Phase 7: 生产部署 | ❌ 未完成 | Docker、K8s、监控均未完成 |
| Phase 8: 连接管理 | ✅ 已完成 | 连接 CRUD、测试连接、Schema 获取已接入主路由 |
| Phase 9: 对话管理 | ✅ 已完成 | 对话 CRUD、消息列表、发送消息并调用 LLM 已接入主路由 |

---

## API 状态清单

### 认证 API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/auth/login` | POST | ✅ 已完成 |
| `/api/v1/auth/register` | POST | ✅ 已完成 |
| `/api/v1/auth/refresh` | POST | ✅ 已完成 |
| `/api/v1/auth/logout` | POST | ✅ 已完成 |

### 用户 API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/users` | GET | ✅ 已完成 |
| `/api/v1/users/{id}` | GET | ✅ 已完成 |
| `/api/v1/users/{id}` | PUT | ✅ 已完成 |
| `/api/v1/users/{id}` | DELETE | ✅ 已完成 |
| `/api/v1/users/{id}/password` | PUT | ✅ 已完成 |

### 连接 API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/connections` | GET | ✅ 已完成 |
| `/api/v1/connections` | POST | ✅ 已完成 |
| `/api/v1/connections/{id}` | GET | ✅ 已完成 |
| `/api/v1/connections/{id}` | PUT | ✅ 已完成 |
| `/api/v1/connections/{id}` | DELETE | ✅ 已完成 |
| `/api/v1/connections/{id}/test` | POST | ✅ 已完成 |
| `/api/v1/connections/{id}/default` | PUT | ✅ 已完成 |
| `/api/v1/connections/{id}/schema` | GET | ✅ 已完成 |

### SQL API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/sql/execute` | POST | ✅ 已完成 |
| `/api/v1/sql/format` | POST | ✅ 已完成 |
| `/api/v1/sql/history` | GET | ✅ 已完成 |
| `/api/v1/sql/explain` | POST | ✅ 已完成 |
| `/api/v1/sql/preview` | POST | ✅ 已完成 |

### NL API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/nl/convert` | POST | ✅ 已完成 |
| `/api/v1/nl/execute` | POST | ✅ 已完成 |

### 对话 API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/conversations` | GET | ✅ 已完成 |
| `/api/v1/conversations` | POST | ✅ 已完成 |
| `/api/v1/conversations/{id}` | GET | ✅ 已完成 |
| `/api/v1/conversations/{id}` | DELETE | ✅ 已完成 |
| `/api/v1/conversations/{id}/messages` | GET | ✅ 已完成 |
| `/api/v1/conversations/{id}/messages` | POST | ✅ 已完成 |

### 图表 API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/charts/recommend` | POST | ✅ 已完成 |
| `/api/v1/charts/generate` | POST | ✅ 已完成 |
| `/api/v1/charts/export` | POST | ✅ 已完成 |

### 指标 API

| 端点 | 方法 | 状态 |
|------|------|------|
| `/api/v1/metrics` | GET | ✅ 已完成 |
| `/api/v1/metrics` | POST | ✅ 已完成 |
| `/api/v1/metrics/{id}` | GET | ✅ 已完成 |
| `/api/v1/metrics/{id}` | PUT | ✅ 已完成 |
| `/api/v1/metrics/{id}` | DELETE | ✅ 已完成 |
| `/api/v1/metrics/{id}/execute` | POST | ✅ 已完成 |
| `/api/v1/metrics/{id}/lineage` | GET | ✅ 已完成 |

---

## 验证结果

| 命令 | 结果 |
|------|------|
| `cargo check` in `week2/backend` | ✅ 通过 |
| `npm run build` in `week2/frontend` | ✅ 通过 |
| `rg "mockall\\|automock\\|mock\\|Mock\\|模拟\\|not implemented\\|未实现" week2/backend/src week2/frontend/src week2/backend/Cargo.toml` | ✅ 无运行时 mock/未实现项 | 剩余 PNG 文案为入参校验提示，不是未实现 |

---

## 下一步优先级

1. 补齐 Docker、K8s、监控和外部服务配置。

---

> 文档版本: 2.3.0  
> 创建时间: 2026-05-30  
> 最后更新: 2026-05-31



















