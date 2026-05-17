# AI 训练营实战项目

> 本仓库是 **AI 训练营** 的实战项目集合。  
> 每周会创建一个独立的子项目（`week1`、`week2`、…），通过实战项目巩固一周所学。

---

## 📂 项目列表

| 周次 | 项目 | 主题 | 状态 |
|------|------|------|------|
| [week1](./week1) | **ProjectAlpha** | 轻量级 Ticket 管理工具（React + FastAPI + MySQL） | 🛠️ 进行中 |

> 后续周次的项目将陆续添加到此处。

---

## 🗂️ 仓库结构

```
ai-learning/
├── specs/                 # 各周项目的需求与设计文档
│   ├── week1/             #   ├── 原始需求、需求与设计文档、实现计划
│   └── week2/             #   └── ...（后续添加）
├── week1/                 # 第 1 周项目代码（ProjectAlpha）
├── week2/                 # 第 2 周项目代码（后续添加）
├── .gitignore
├── README.md              # 当前文件 — 训练营总览
└── ...
```

每个 `weekN/` 子目录是一个独立的子项目，包含自己的 `README.md`、`LICENSE`、依赖配置与代码。

每个 `specs/weekN/` 目录存放对应周项目的需求与设计文档。

---

## 🧭 快速导航

| 想做的事 | 去哪里 |
|----------|--------|
| 查看当前周的项目 | [`week1/README.md`](./week1/README.md) |
| 查看当前周的需求文档 | [`specs/week1/`](./specs/week1/) |
| 提交代码 | 切到对应的 `weekN/` 子目录开发 |

---

## 📐 通用约定

虽然每周项目独立，但遵循统一的工程约定：

- **分支策略**：`main`（稳定）→ `dev`（开发）→ `feature/*`（功能分支）
- **提交规范**：[Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/)
- **每周项目独立**：每个 `weekN/` 拥有自己的 `LICENSE`、`README`、依赖配置；互不依赖

---

## 📄 License

各子项目的 License 见各自目录下的 `LICENSE` 文件。
