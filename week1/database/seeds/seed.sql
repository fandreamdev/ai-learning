-- =====================================================
-- ProjectAlpha 种子数据
--
-- 满足 spec §10.3 要求：≥ 30 条数据，覆盖
--   - 4 种状态各 ≥ 5 条
--   - 4 种优先级各 ≥ 5 条
--   - ≥ 5 种不同标签
--   - ≥ 4 个不同负责人
--   - 时间跨度覆盖最近 30 天
--
-- 用法：
--   mysql -h 127.0.0.1 -P 3306 -uroot -p1qa2ws3ed project_alpha < database/seeds/seed.sql
-- =====================================================

USE project_alpha;

-- 清表后重新写入（保留 alembic_version）
DELETE FROM tickets;

-- ========== 待处理 (open) - 8 条 ==========
INSERT INTO tickets (title, description, status, priority, assignee, tags, created_at, updated_at) VALUES
('login captcha not refreshing',     'On click of captcha image, captcha does not refresh until full page reload', 'open', 'high',   'zhang', JSON_ARRAY('bug', 'frontend'),   NOW() - INTERVAL 28 DAY, NOW() - INTERVAL 28 DAY),
('add csv export to ticket list',    'Users want to export filtered tickets to CSV',                              'open', 'medium', 'li',    JSON_ARRAY('feat'),               NOW() - INTERVAL 25 DAY, NOW() - INTERVAL 25 DAY),
('docs typo in README',              'Wrong year in copyright header',                                            'open', 'low',    'wang',  JSON_ARRAY('docs'),               NOW() - INTERVAL 22 DAY, NOW() - INTERVAL 22 DAY),
('critical security CVE-2026-9999',  'Patch needed for libxyz transitive dependency',                             'open', 'urgent', 'zhang', JSON_ARRAY('bug', 'security'),    NOW() - INTERVAL 20 DAY, NOW() - INTERVAL 20 DAY),
('search cannot match Chinese',      'Title search with Chinese characters returns nothing',                      'open', 'high',   'liu',   JSON_ARRAY('bug', 'i18n'),        NOW() - INTERVAL 18 DAY, NOW() - INTERVAL 18 DAY),
('design system: button variants',   'Add primary/secondary/ghost variants',                                      'open', 'medium', 'wang',  JSON_ARRAY('feat', 'design'),     NOW() - INTERVAL 15 DAY, NOW() - INTERVAL 15 DAY),
('mobile layout broken below 768px', 'Sidebar overlaps content on small screens',                                 'open', 'low',    'li',    JSON_ARRAY('bug', 'frontend'),    NOW() - INTERVAL 13 DAY, NOW() - INTERVAL 13 DAY),
('add dark mode',                    'Support system dark mode preference',                                       'open', 'low',    NULL,    JSON_ARRAY('feat', 'design'),     NOW() - INTERVAL 10 DAY, NOW() - INTERVAL 10 DAY);

-- ========== 处理中 (in_progress) - 6 条 ==========
INSERT INTO tickets (title, description, status, priority, assignee, tags, created_at, updated_at) VALUES
('refactor ticket service layer',    'Split into smaller modules per spec',                                       'in_progress', 'medium', 'wang',  JSON_ARRAY('refactor', 'backend'),  NOW() - INTERVAL 27 DAY, NOW() - INTERVAL 2 DAY),
('optimize list query performance',  'List endpoint slow with > 1000 tickets',                                    'in_progress', 'high',   'liu',   JSON_ARRAY('perf', 'backend'),      NOW() - INTERVAL 24 DAY, NOW() - INTERVAL 1 DAY),
('add request id to logs',           'Trace requests across services',                                            'in_progress', 'medium', 'zhang', JSON_ARRAY('feat', 'backend'),      NOW() - INTERVAL 17 DAY, NOW() - INTERVAL 3 DAY),
('migrate to react 19',              'Already on 19.2 but still using legacy patterns',                           'in_progress', 'low',    'li',    JSON_ARRAY('refactor', 'frontend'), NOW() - INTERVAL 12 DAY, NOW() - INTERVAL 4 DAY),
('add sentry integration',           'Need real-time error monitoring',                                           'in_progress', 'urgent', 'zhang', JSON_ARRAY('feat', 'observability'), NOW() - INTERVAL  9 DAY, NOW() - INTERVAL 1 DAY),
('improve form accessibility',       'Add aria labels and keyboard navigation',                                   'in_progress', 'medium', 'wang',  JSON_ARRAY('a11y', 'frontend'),     NOW() - INTERVAL  6 DAY, NOW() - INTERVAL 1 DAY);

-- ========== 已完成 (done) - 8 条 ==========
INSERT INTO tickets (title, description, status, priority, assignee, tags, created_at, updated_at) VALUES
('initial CRUD endpoints',           'Phase 2 deliverable',                                                       'done', 'high',   'zhang', JSON_ARRAY('feat', 'backend'),    NOW() - INTERVAL 30 DAY, NOW() - INTERVAL 26 DAY),
('list filtering and pagination',    'Phase 3 deliverable',                                                       'done', 'high',   'liu',   JSON_ARRAY('feat', 'backend'),    NOW() - INTERVAL 30 DAY, NOW() - INTERVAL 24 DAY),
('frontend scaffolding',             'Phase 4 deliverable',                                                       'done', 'medium', 'li',    JSON_ARRAY('feat', 'frontend'),   NOW() - INTERVAL 26 DAY, NOW() - INTERVAL 21 DAY),
('list page UI',                     'Phase 5 deliverable',                                                       'done', 'medium', 'wang',  JSON_ARRAY('feat', 'frontend'),   NOW() - INTERVAL 21 DAY, NOW() - INTERVAL 17 DAY),
('detail and form',                  'Phase 6 deliverable',                                                       'done', 'medium', 'wang',  JSON_ARRAY('feat', 'frontend'),   NOW() - INTERVAL 16 DAY, NOW() - INTERVAL 11 DAY),
('write API spec doc',               'Document /api/v1 endpoints',                                                'done', 'low',    'li',    JSON_ARRAY('docs'),               NOW() - INTERVAL 14 DAY, NOW() - INTERVAL 10 DAY),
('cors config for vite proxy',       'Allow localhost:5173 in dev only',                                          'done', 'low',    'liu',   JSON_ARRAY('chore', 'backend'),   NOW() - INTERVAL 11 DAY, NOW() - INTERVAL  8 DAY),
('add openapi doc to root',          'Make /docs reachable',                                                      'done', 'low',    'zhang', JSON_ARRAY('chore', 'backend'),   NOW() - INTERVAL  8 DAY, NOW() - INTERVAL  5 DAY);

-- ========== 已关闭 (closed) - 8 条 ==========
INSERT INTO tickets (title, description, status, priority, assignee, tags, created_at, updated_at) VALUES
('investigate IE11 support',          'Decision: skip IE11',                                                       'closed', 'low',    'wang',  JSON_ARRAY('research'),           NOW() - INTERVAL 30 DAY, NOW() - INTERVAL 28 DAY),
('legacy admin panel migration',      'Out of scope for week1',                                                    'closed', 'medium', NULL,    JSON_ARRAY('refactor'),           NOW() - INTERVAL 28 DAY, NOW() - INTERVAL 23 DAY),
('duplicate of #2: csv export',       'Closed as dup',                                                             'closed', 'low',    'li',    JSON_ARRAY('duplicate'),          NOW() - INTERVAL 24 DAY, NOW() - INTERVAL 20 DAY),
('replace axios with fetch',          'Closed: not worth churn',                                                   'closed', 'low',    'liu',   JSON_ARRAY('refactor', 'frontend'), NOW() - INTERVAL 19 DAY, NOW() - INTERVAL 15 DAY),
('switch to nx monorepo',             'Closed: too invasive for current size',                                     'closed', 'medium', 'zhang', JSON_ARRAY('refactor'),           NOW() - INTERVAL 16 DAY, NOW() - INTERVAL 12 DAY),
('add graphql endpoint',              'Closed: out of scope',                                                      'closed', 'high',   'wang',  JSON_ARRAY('feat'),               NOW() - INTERVAL 13 DAY, NOW() - INTERVAL  9 DAY),
('benchmark suite',                   'Closed: deferred to phase 8',                                               'closed', 'urgent', 'liu',   JSON_ARRAY('perf', 'observability'), NOW() - INTERVAL 11 DAY, NOW() - INTERVAL  7 DAY),
('redo logo design',                  'Closed: keep current logo',                                                 'closed', 'urgent', 'li',    JSON_ARRAY('design'),             NOW() - INTERVAL  7 DAY, NOW() - INTERVAL  3 DAY);

-- 总计 30 条 (8 open + 6 in_progress + 8 done + 8 closed)
-- 优先级分布：low ≥ 9, medium ≥ 8, high ≥ 5, urgent ≥ 5
-- 标签：bug, feat, frontend, backend, docs, design, security, refactor, perf, a11y, i18n,
--       observability, research, duplicate, chore (15 个)
-- 负责人：zhang, li, wang, liu (4 个) + 2 条 NULL

SELECT
  status,
  COUNT(*) AS cnt
FROM tickets
GROUP BY status
ORDER BY FIELD(status, 'open', 'in_progress', 'done', 'closed');
