# ProjectAlpha Frontend

> React 19 + Vite 8 + TypeScript + TailwindCSS v4 + React Router 7。

## 目录结构

```
frontend/
├── public/                   # 静态资源
├── src/
│   ├── api/                  # Axios 封装与业务接口
│   │   ├── request.ts
│   │   ├── tickets.ts
│   │   └── aggregations.ts
│   ├── constants/            # 状态/优先级映射、状态流转表
│   ├── lib/                  # cn / format 等工具
│   ├── mocks/                # MSW handlers (默认关闭)
│   ├── pages/                # 路由页面
│   │   ├── TicketListPage/
│   │   ├── TicketDetailPage/
│   │   └── NotFoundPage/
│   ├── types/                # TS 类型 (api / ticket)
│   ├── App.tsx               # BrowserRouter
│   ├── main.tsx              # 入口 + 条件挂载 MSW
│   ├── index.css             # @import 'tailwindcss'
│   └── vite-env.d.ts
├── vite.config.ts
├── tsconfig*.json
├── eslint.config.js
├── prettier.config.js
├── package.json
└── README.md
```

## 环境要求

- Node.js ≥ 22
- npm ≥ 10
- 后端运行在 `http://localhost:8000`（通过 Vite 代理转发）

## 快速开始

```bash
# 1. 安装依赖
npm install

# 2. 复制本地环境变量
cp .env.example .env

# 3. 启动开发服务
npm run dev
# 访问 http://localhost:5173
```

## 常用命令

| 操作 | 命令 |
|------|------|
| 启动开发服务 | `npm run dev` |
| 类型检查 | `npm run typecheck` |
| Lint | `npm run lint` |
| 自动格式化 | `npm run format` |
| 跑测试 | `npm run test` |
| 测试 + 覆盖率 | `npm run test:coverage` |
| 生产构建 | `npm run build` |
| 预览构建 | `npm run preview` |

## 环境变量

```env
VITE_API_BASE_URL=/api/v1     # 经 Vite 代理转发到后端
VITE_ENABLE_MOCK=false        # 设为 true 启用 MSW Mock
```

## 当前阶段

✅ **阶段 4 - 前端基础框架** 已完成
⏳ **阶段 5 - 列表页** 待开始
