-- 迁移版本：010
-- 数据库类型：SQLite
-- 作用：为 global_connections 表添加 auth_method 字段，记录数据库认证方式
-- 更新时间：2026-05-22

ALTER TABLE global_connections ADD COLUMN auth_method TEXT;