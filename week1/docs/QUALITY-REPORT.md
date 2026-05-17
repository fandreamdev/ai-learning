# 阶段 8 质量保障报告

> 日期：2026-05-17  
> 分支：`week1-8`  
> 范围：spec §14 测试策略 + plan §阶段 8 质量门禁

## 总览

**9 道质量门禁全部通过。**

| # | 门禁 | 阈值 | 实际 | 结果 |
|---|------|------|------|------|
| 1 | 后端 lint (ruff) | 0 错误 | 0 错误 | ✅ |
| 2 | 后端 type (mypy strict) | 0 错误 | 0 错误，28 文件 | ✅ |
| 3 | 后端测试 + 覆盖率 | line ≥ 80% (service) | **96%** (496 行) | ✅ |
| 4 | 前端 lint (eslint) | 0 错误 | 0 错误 | ✅ |
| 5 | 前端 type (tsc) | 0 错误 | 0 错误 | ✅ |
| 6 | 前端测试 + 覆盖率 | line ≥ 60% (核心 ≥ 70%) | **64.12%** total，核心组件多在 85%+ | ✅ |
| 7 | 前端 build | 成功 | dist 337 KB / 108 KB gzipped | ✅ |
| 8 | E2E (Playwright) | 6 / 6 | **6 / 6** in 5.3s | ✅ |
| 9 | 性能 (NF01) | p50 < 500 ms | **p50 = 4.3 ms** | ✅ |

---

## 1. 后端测试详情

### 测试规模
- **104 用例**全过 / 1.06 s
- 单元 (service / schema): 44 用例
- 集成 (API + 真实 MySQL + SAVEPOINT 回滚): 60 用例
- 覆盖：5 个 CRUD 端点 + 列表筛选 + 聚合 + 状态流转 6 合法 + 6 非法

### 覆盖率（96% 行）

| 模块 | line% | branch% | 备注 |
|------|------|--------|------|
| `app/api/v1/tickets.py` | 100 | 100 | 完全覆盖 |
| `app/api/v1/aggregations.py` | 100 | 100 | 完全覆盖 |
| `app/api/v1/_query_utils.py` | 100 | 100 | 完全覆盖 |
| `app/services/ticket_service.py` | 93 | — | 极高 |
| `app/services/aggregation_service.py` | 91 | — | 极高 |
| `app/schemas/ticket.py` | 97 | — | Pydantic 校验 |
| `app/middlewares/error_handler.py` | 94 | — | 仅 4xx 兜底未触达 |
| `app/main.py` | 100 | 100 | — |
| **总计** | **96** | — | 远超 spec 要求 |

未覆盖的 20 行主要是 `database.py` 的工厂初始化与几个边界异常分支。

---

## 2. 前端测试详情

### 测试规模
- **97 用例**全过 / 6.62 s
- 23 个测试文件
- 覆盖：API 客户端 / 业务 hook / 通用组件 / 表单组件 / 表格 / Toast / 弹窗

### 覆盖率（64.12% 行）

| 区域 | line% | 评级 |
|------|------|------|
| 通用组件 (Modal / ConfirmDialog / Toast / Pagination / SortBar / TicketTable) | **88~100** | ⭐⭐⭐ 核心组件 |
| Hooks (useTickets / useTicket / useTicketListUrlState / useTags / useDebouncedValue) | **78~100** | ⭐⭐⭐ |
| Schema 工具 (queryString / cn) | 92~98 | ⭐⭐⭐ |
| TagInput / SearchBar | 86 | ⭐⭐ |
| API 客户端 (request / tickets / aggregations) | 通过 mock 测拼装 | 间接覆盖 |
| TicketListPage / TicketDetailPage | 0 (vitest) | 通过 E2E 覆盖 |
| TicketForm / Sidebar wrapper / Header / Layout / AssigneeFilter / TagFilter | 较低 (0~80) | E2E + 间接覆盖 |

> 核心组件覆盖率（spec §14.5 ≥ 70%）已达标；页面级覆盖率交给 E2E。

---

## 3. E2E（Playwright）

### 配置
- 仅 chromium，单 worker
- baseURL `http://localhost:5173`
- `beforeEach` 通过前端代理 API 清空 + 注入 3 条 fixture（避免依赖 mysql cli）

### 6 条关键路径

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | 列表加载 | 标题渲染、3 条 seed 全部可见 |
| 2 | 优先级过滤 | 勾选 "高" 后只剩 high 的 ticket，取消恢复 |
| 3 | 关键字搜索 | `csv` 命中描述，列表收窄 |
| 4 | 新建 Ticket | 弹窗 → 输入 → 保存 → 列表新增 |
| 5 | 状态切换 | 详情页下拉 `→ 处理中`，后端 PATCH 落库 |
| 6 | 删除（带确认） | ConfirmDialog → 确认 → 列表减少 |

**结果：6 / 6 passed in 5.3s**

---

## 4. 性能（NF01）

### 实施
- 通过 `backend/scripts/perf_benchmark.py` 自动化：
  1. 清空 `tickets`，批量插入 1000 条
  2. 5 次预热 + 50 次 GET `/api/v1/tickets?page_size=20`
  3. 计算 p50 / p95 / p99 / max
  4. 自动清理

### 结果

| 指标 | 阈值 | 实测 |
|------|------|------|
| p50 | < 500 ms | **4.3 ms** |
| p95 | < 1000 ms | 5.0 ms |
| p99 | — | 5.3 ms |
| avg | — | 4.3 ms |
| max | — | 5.5 ms |

> 性能远超阈值（约 1% 余量已使用），1000 条数据规模下完全无瓶颈。

---

## 5. 工具与产物

| 文件 | 作用 |
|------|------|
| `backend/scripts/perf_benchmark.py` | 性能基准脚本 |
| `frontend/playwright.config.ts` | Playwright 配置 |
| `frontend/e2e/fixtures.ts` | 通用 fixture（清空 + 注入） |
| `frontend/e2e/list.spec.ts` | E2E #1~3 |
| `frontend/e2e/crud.spec.ts` | E2E #4~6 |
| `frontend/tsconfig.e2e.json` | E2E 独立 ts 配置 |
| `tools/integration-check.sh` | 阶段 7 联调脚本（保留） |
| `database/seeds/seed.sql` | 种子数据（30 条） |

---

## 6. 一键运行命令

```bash
# 后端：质量门禁
cd week1/backend
uv run ruff check .
uv run mypy app
uv run pytest --cov=app

# 后端：性能基准（需要后端先启动）
uv run uvicorn app.main:app --port 8000 &
uv run python -m scripts.perf_benchmark

# 前端：质量门禁
cd ../frontend
npm run typecheck
npm run lint
npm run test:coverage
npm run build

# 前端：E2E（需要前端 + 后端启动）
npm run dev &
cd ../backend && uv run uvicorn app.main:app --port 8000 &
cd ../frontend && npm run e2e
```

---

## 7. 阶段 8 调整记录

| # | 问题 | 处理 |
|---|------|------|
| 1 | Vite 代理 `localhost` 在 Windows 解析为 IPv6，导致后端连接失败 | 改用 `127.0.0.1` 作为代理 target |
| 2 | E2E fixture 用 `page_size=200` 超过后端 100 上限 | 改为 100（fixture 数据量很小） |
| 3 | `getByLabel` 不能匹配 wrap-style label | 改用 dialog 内 first input 选择器 |
| 4 | 多处 "高" 文本（侧栏 + 表格） | E2E 用 `getByRole('complementary')` 限定 sidebar |
| 5 | 性能脚本在 print 中误用 f-string | ruff 自动建议移除 `f` 前缀 |

**所有调整都为脚本/测试代码层面，业务实现 0 处修改**。

---

## 8. 后续

- 阶段 9（部署）：将质量门禁集成到 CI（GitHub Actions），并打 Docker 镜像
- 后续若引入认证/评论模块，复用现有覆盖率框架与 E2E 模板
