-- 下载任务表添加分类和标签字段
-- 创建时间: 2025-12-04

-- 添加分类字段（JSON 数组）
ALTER TABLE download_task ADD COLUMN categories TEXT;

-- 添加标签字段（JSON 数组）
ALTER TABLE download_task ADD COLUMN tags TEXT;
