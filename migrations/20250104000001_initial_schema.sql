-- PicACG 初始数据库结构
-- 创建时间: 2025-01-04

-- 系统表
CREATE TABLE IF NOT EXISTS system (
    version INTEGER PRIMARY KEY
);

-- 插入初始版本号（幂等）
INSERT OR IGNORE INTO system (version) VALUES (1);

-- 漫画表
CREATE TABLE IF NOT EXISTS book (
    id TEXT PRIMARY KEY,              -- 漫画 ID
    title TEXT NOT NULL,              -- 标题
    title2 TEXT,                      -- 繁体标题
    author TEXT,                      -- 作者
    chinese_team TEXT,                -- 汉化组
    description TEXT,                 -- 描述
    eps_count INTEGER DEFAULT 0,      -- 章节数
    pages INTEGER DEFAULT 0,          -- 总页数
    finished INTEGER DEFAULT 0,       -- 是否完本 (0/1)
    categories TEXT,                  -- 分类（逗号分隔）
    tags TEXT,                        -- 标签（逗号分隔）
    likes_count INTEGER DEFAULT 0,    -- 爱心数
    created_at INTEGER DEFAULT 0,     -- 创建时间（Unix 时间戳）
    updated_at INTEGER DEFAULT 0,     -- 更新时间
    path TEXT,                        -- 封面路径
    file_server TEXT,                 -- 文件服务器
    original_name TEXT,               -- 封面原始文件名
    creator TEXT,                     -- 上传者
    total_likes INTEGER DEFAULT 0,    -- 总点赞数
    total_views INTEGER DEFAULT 0     -- 总浏览数
);

CREATE INDEX IF NOT EXISTS idx_book_title ON book(title);
CREATE INDEX IF NOT EXISTS idx_book_updated ON book(updated_at);

-- 分类计数缓存表
CREATE TABLE IF NOT EXISTS category_count (
    category TEXT PRIMARY KEY,
    count INTEGER DEFAULT 0
);

-- 收藏表
CREATE TABLE IF NOT EXISTS favorite (
    book_id TEXT PRIMARY KEY,
    added_at INTEGER DEFAULT 0
);

-- 浏览历史表
CREATE TABLE IF NOT EXISTS history (
    book_id TEXT PRIMARY KEY,
    last_read INTEGER DEFAULT 0,     -- 最后阅读时间
    last_eps INTEGER DEFAULT 0,      -- 最后阅读章节
    last_page INTEGER DEFAULT 0      -- 最后阅读页码
);
