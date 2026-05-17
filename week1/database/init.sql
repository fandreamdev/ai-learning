-- =====================================================
-- ProjectAlpha 数据库初始化脚本
--
-- 用途: 仅负责创建两个数据库（业务库 + 测试库）。
--       表结构由 Alembic 管理（参见 backend/alembic/versions/）。
--
-- 使用:
--   - Docker 首次启动时自动执行（挂载到 /docker-entrypoint-initdb.d）
--   - 本地新环境手动运行：
--       mysql -h 127.0.0.1 -P 3306 -uroot -p < database/init.sql
--
-- 创建数据库后，应用 Alembic 迁移以建表：
--   cd backend && uv run alembic upgrade head
--
-- 来源约定: specs/week1/0001-spec.md §10
-- =====================================================

CREATE DATABASE IF NOT EXISTS project_alpha
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_unicode_ci;

CREATE DATABASE IF NOT EXISTS project_alpha_test
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_unicode_ci;
