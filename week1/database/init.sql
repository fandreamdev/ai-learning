-- =====================================================
-- ProjectAlpha 数据库初始化脚本
-- 用途: Docker MySQL 容器首次启动自动执行；本地开发可手动运行
-- 来源: specs/week1/0001-spec.md §10.1
-- =====================================================

CREATE DATABASE IF NOT EXISTS project_alpha
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_unicode_ci;

CREATE DATABASE IF NOT EXISTS project_alpha_test
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_unicode_ci;

USE project_alpha;

-- ----------------------------------------------------
-- Ticket 工单表
-- ----------------------------------------------------
CREATE TABLE IF NOT EXISTS tickets (
  id          INT UNSIGNED    NOT NULL AUTO_INCREMENT PRIMARY KEY,
  title       VARCHAR(200)    NOT NULL                COMMENT 'Ticket 标题',
  description TEXT            NULL                    COMMENT '详细描述',
  status      ENUM('open','in_progress','done','closed')
                              NOT NULL DEFAULT 'open' COMMENT '状态',
  priority    ENUM('low','medium','high','urgent')
                              NOT NULL DEFAULT 'medium' COMMENT '优先级',
  assignee    VARCHAR(100)    NULL                    COMMENT '负责人',
  tags        JSON            NULL                    COMMENT '标签列表',
  created_at  DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  updated_at  DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP
                              ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',

  INDEX idx_status     (status),
  INDEX idx_priority   (priority),
  INDEX idx_assignee   (assignee),
  INDEX idx_created_at (created_at),
  INDEX idx_updated_at (updated_at),
  FULLTEXT INDEX ft_title_desc (title, description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Ticket 工单表';
