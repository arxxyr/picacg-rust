-- 下载任务表
-- 创建时间: 2025-01-04

-- 下载任务表
CREATE TABLE IF NOT EXISTS download_task (
    comic_id TEXT PRIMARY KEY,           -- 漫画 ID
    comic_title TEXT NOT NULL,           -- 漫画标题
    total_episodes INTEGER DEFAULT 0,    -- 总章节数
    episode_orders TEXT NOT NULL,        -- 要下载的章节顺序列表（JSON 数组）
    save_path TEXT NOT NULL,             -- 保存路径
    state TEXT NOT NULL DEFAULT 'Queued', -- 当前状态（Queued/Downloading/Paused/Completed/Failed）
    state_data TEXT,                     -- 状态附加数据（JSON，如 current_episode, current_page, error）
    created_at INTEGER DEFAULT 0,        -- 创建时间（Unix 时间戳）
    updated_at INTEGER DEFAULT 0         -- 更新时间
);

CREATE INDEX IF NOT EXISTS idx_download_task_state ON download_task(state);
CREATE INDEX IF NOT EXISTS idx_download_task_updated ON download_task(updated_at);
