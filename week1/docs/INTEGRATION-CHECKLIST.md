# 阶段 7 联调检查报告

> 日期：2026-05-17  
> 分支：`week1-7`  
> 范围：spec §15.4 联调检查清单（12 项）+ plan §7.4 已知陷阱

## 概览

| 指标 | 结果 |
|------|------|
| 自动化检查项 | **21 / 21 全部通过** |
| 后端 pytest 回归 | 104 / 104 通过 |
| 前端 vitest 回归 | 35 / 35 通过 |
| 发现的问题 | 0 个代码问题；2 个脚本兼容性问题（已修） |
| 修复 commit | week1-7 |

详细脚本输出见本目录下 `integration-check.sh` 的最近一次运行（也可重新执行）。

## 环境

| 组件 | 端口 | 凭据 |
|------|------|------|
| 后端 FastAPI | 8000 | — |
| 前端 Vite Dev | 5173 | — |
| MySQL 8.0 | 3306 | root / 1qa2ws3ed |
| 业务库 | `project_alpha` | — |
| 测试库 | `project_alpha_test` | — |

## 验证步骤

### 1. 准备数据库

```bash
# 注入 30 条种子数据（spec §10.3）
"/c/Program Files/MySQL/MySQL Workbench 8.0/mysql.exe" \
  -h 127.0.0.1 -P 3306 -uroot -p1qa2ws3ed project_alpha \
  < week1/database/seeds/seed.sql
```

种子数据分布：
- **状态**：open 8 条 + in_progress 6 条 + done 8 条 + closed 8 条 = 30 条
- **优先级**：4 种各 ≥ 5 条
- **标签**：15 个不同标签（a11y / backend / bug / chore / design / docs / duplicate / feat / frontend / i18n / observability / perf / refactor / research / security）
- **负责人**：4 个（zhang / li / wang / liu）+ 2 条 NULL
- **时间跨度**：最近 30 天

### 2. 启动服务

```bash
# 终端 1：后端
cd week1/backend && uv run uvicorn app.main:app --port 8000

# 终端 2：前端
cd week1/frontend && npm run dev
```

### 3. 运行联调脚本

```bash
bash week1/tools/integration-check.sh
```

## 检查清单（spec §15.4 + 补充）

### 自动化项（21 项断言全过）

| # | 检查项 | 来源 | 实测结果 |
|---|--------|------|----------|
| 1.1 | `/healthz` 返回统一成功格式 | spec §11 | ✅ `{code:0, data:"ok"}` |
| 1.2 | `/readyz` 携带 DB ping | plan §1 | ✅ `{code:0, data:"ready"}` |
| 2.1 | Vite 代理 `/api → 8000` 透传 | plan §15 | ✅ `GET /api/v1/tickets` 经 5173 返回 200 |
| 2.2 | 前端首页可访问 | plan §4 | ✅ `/` 返回 200 |
| 3.1 | 列表响应字段集 | spec §15.4 #1 | ✅ `code/message/data{items,total,page,page_size}` 完整 |
| 3.2 | 时间格式 ISO 8601 | spec §15.4 #3 | ✅ `created_at` 形如 `2026-05-17T00:00:00` |
| 4 | 多值参数 CSV vs 重复参数等价 | spec §15.4 #5 | ✅ `?status=open,in_progress` 与 `?status=open&status=in_progress` 同 total=14 |
| 5.1 | Status 枚举对齐 | spec §15.4 #2 / §4.2 | ✅ `closed,done,in_progress,open` 与 spec 一致 |
| 5.2 | Priority 枚举对齐 | spec §15.4 #2 / §4.2 | ✅ `high,low,medium,urgent` 与 spec 一致 |
| 6.1 | tags 自动 `lower + trim + 去重` | spec §15.4 #10 | ✅ 输入 `["Bug","bug","BUG","Frontend"]` 存储为 `["bug","frontend"]` |
| 6.2 | 缺省 tags 默认 `[]` 而非 `null` | plan §7.3 | ✅ POST 不带 tags 时返回 `tags: []` |
| 7 | 状态非法流转 → 40002 | spec §15.4 #8 | ✅ `in_progress → open` 返回 HTTP 400 + `code:40002` + `不能从 in_progress 流转到 open` |
| 8 | page_size > 100 → 40001 | plan §3 | ✅ `?page_size=200` 返回 HTTP 400 + `code:40001` |
| 9 | 删除后 GET → 40401 | spec §11 | ✅ DELETE 后 GET 同一 ID 返回 404 + `code:40401` |
| 10 | keyword 命中描述 | spec §15.4 #6 | ✅ `?keyword=captcha` 匹配描述中含该词的 ticket（spec §3.2 F15） |
| 11 | keyword < 2 字符 → 400 | plan §3 | ✅ `?keyword=a` 返回 HTTP 400（FastAPI Query min_length 兜底） |
| 12.1 | `/tags` 去重排序 | spec §5.3.1 | ✅ 字典序、无重复，15 个标签 |
| 12.2 | `/assignees` 去重排序 | spec §5.3.2 | ✅ `[li, liu, wang, zhang]` |
| 13 | 分页边界 | plan §3 | ✅ 第一页满、最后一页有内容、超出页返回空数组 |
| 14 | 创建后 total 增加 | spec §15.4 #7 | ✅ `30 → 31` |
| 15 | 4xx 也走统一 envelope | spec §11 | ✅ 未知路由返回 `{code:40404, message:"Not Found", data:null}` |

### 手工/UI 验证项（spec §15.4 剩余）

| # | 检查项 | 验证结果 |
|---|--------|----------|
| spec §15.4 #4 | 分页：page=1 起始 | ✅ 自动化项 13 已覆盖 |
| spec §15.4 #9 | 删除二次确认 | ✅ 阶段 6 ConfirmDialog（destructive 红色按钮 + loading 防重复） |
| spec §15.4 #11 | 网络断开 | ✅ 阶段 6 ApiError 拦截器抛 `0/网络错误`，前端 ErrorToast 显示重试按钮 |
| spec §15.4 #12 | 500 错误格式 | ✅ 全局 `_unhandled_exc_handler` 返回 `{code:50001, message:"服务器内部错误"}`（已在 `app/middlewares/error_handler.py` 验证；脚本中通过 4xx 间接验证 envelope 一致性） |

## 已知陷阱与防范（plan §7.3）

| 陷阱 | 验证 | 状态 |
|------|------|------|
| 时间用 ISO，前端误传字符串 | 自动化项 3.2 验证 ISO 格式；前端 `formatDateTime` 仅消费 ISO | ✅ |
| tags 默认 `[]` 前端误判 `null` | 自动化项 6.2；前端 `Ticket.tags: string[]`（非可选） | ✅ |
| tags 大小写/去重 | 自动化项 6.1；前后端均做 `lower+trim` | ✅ |
| page_size 上限 | 自动化项 8 | ✅ |
| CORS 报错 | 通过 Vite 代理避开；后端 `CORSMiddleware` 已配置 5173 | ✅ |
| 空 description 用 null | 前端 `description.trim() \|\| null` 处理；脚本未直接断言但通过手工冒烟验证 | ✅ |

## 联调中发现的问题

| # | 问题 | 影响 | 处理 |
|---|------|------|------|
| 1 | 脚本中 `/tmp/_resp.json` 路径在 Windows Git Bash 下不可写 | 仅影响脚本运行，不涉及业务代码 | 脚本改为 `${TMPDIR:-.}/_integration_resp_$$.json` + `trap` 自动清理 |
| 2 | bash 字符串比较 Python list 字面量时引号转义复杂 | 同上 | 改为在 Python 里直接比较，输出 `ok:` / `bad:` 前缀给 bash 判断 |

**业务代码：0 处修改**。前后端跨阶段实现保持高度一致，未发现需要回填的对接缺陷。

## 后续

- 阶段 8（测试与质量保障）：再做一次性能 / E2E 与覆盖率检查
- 阶段 9（部署）：把 `seed.sql` 与 `integration-check.sh` 集成到部署冒烟流程
