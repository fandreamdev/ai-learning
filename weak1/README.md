# ProjectAlpha (weak1)

> 轻量级 Ticket 管理工具 — 实现参见 [specs/weak1/0001-spec.md](../specs/weak1/0001-spec.md) 与 [specs/weak1/0002-implementation-plan.md](../specs/weak1/0002-implementation-plan.md)。

## 目录结构

```
weak1/
├── frontend/          # 前端 (React 19 + Vite 8 + TS)        待实现
├── backend/           # 后端 (Python 3.13 + FastAPI 0.136)    待实现
├── database/          # 数据库脚本
│   ├── init.sql       # 初始化 DDL（含两个数据库）
│   └── seeds/         # 种子数据
├── docs/              # 项目文档（OpenAPI 等）                待补充
├── .editorconfig      # 统一编辑器风格
├── .gitignore         # 忽略规则
├── .pre-commit-config.yaml  # 提交前钩子
├── LICENSE            # MIT
└── README.md
```

## 当前阶段

- **阶段 0 - 项目启动与环境准备**：✅ 已完成
  - 目录骨架
  - 配置文件（LICENSE / .editorconfig / .gitignore / pre-commit）
  - 环境变量模板（`backend/.env.example`、`frontend/.env.example`）
  - 数据库初始化脚本（`database/init.sql`）
- **阶段 1 - 后端基础框架**：⏳ 待开始

## 本地开发：MySQL 准备

本地已运行 MySQL 8.0（默认凭据：`root / 1qa2ws3ed`，端口 3306）。

初始化数据库（仅首次）：

```bash
mysql -h 127.0.0.1 -P 3306 -uroot -p1qa2ws3ed < database/init.sql
```

验证：

```bash
mysql -h 127.0.0.1 -P 3306 -uroot -p1qa2ws3ed -e "SHOW DATABASES LIKE 'project_alpha%';"
```

## 环境变量

```bash
# 后端
cp backend/.env.example backend/.env

# 前端
cp frontend/.env.example frontend/.env
```

## pre-commit（推荐）

```bash
pip install pre-commit
pre-commit install
```

之后每次 `git commit` 会自动执行格式化与静态检查。

## License

[MIT](./LICENSE)
