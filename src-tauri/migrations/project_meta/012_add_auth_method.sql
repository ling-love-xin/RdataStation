-- 迁移版本：012
-- 数据库类型：SQLite
-- 作用：为 connections 表添加 auth_method 字段，记录数据库认证方式
-- 更新时间：2026-05-22

ALTER TABLE connections ADD COLUMN auth_method TEXT;