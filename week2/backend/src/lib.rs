//! SmartQuery AI - 智能双模数据库查询与分析系统
//!
//! 后端服务入口模块
//!
//! # 模块结构
//!
//! - [`api`] - API 层，处理 HTTP 请求
//! - [`services`] - 服务层，业务逻辑
//! - [`models`] - 数据模型
//! - [`repositories`] - 数据访问层
//! - [`middleware`] - 中间件
//! - [`utils`] - 工具函数
//!
//! # 快速开始
//!
//! ```bash
//! # 开发模式
//! cargo run
//!
//! # 运行测试
//! cargo test
//!
//! # 构建发布版本
//! cargo build --release
//! ```

pub mod api;
pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod services;
pub mod state;
pub mod utils;
