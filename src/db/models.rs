use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 漫画数据库实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbBook {
    pub id: String,
    pub title: String,
    pub title2: Option<String>,
    pub author: Option<String>,
    pub chinese_team: Option<String>,
    pub description: Option<String>,
    pub eps_count: i64,
    pub pages: i64,
    pub finished: i64, // 0 = false, 1 = true
    pub categories: Option<String>,
    pub tags: Option<String>,
    pub likes_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub path: Option<String>,
    pub file_server: Option<String>,
    pub original_name: Option<String>,
    pub creator: Option<String>,
    pub total_likes: i64,
    pub total_views: i64,
}

impl DbBook {
    pub fn new(id: String, title: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id,
            title,
            title2: None,
            author: None,
            chinese_team: None,
            description: None,
            eps_count: 0,
            pages: 0,
            finished: 0,
            categories: None,
            tags: None,
            likes_count: 0,
            created_at: now,
            updated_at: now,
            path: None,
            file_server: None,
            original_name: None,
            creator: None,
            total_likes: 0,
            total_views: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished != 0
    }

    pub fn set_finished(&mut self, finished: bool) {
        self.finished = if finished { 1 } else { 0 };
    }

    pub fn get_categories(&self) -> Vec<String> {
        self.categories
            .as_ref()
            .map(|s| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn set_categories(&mut self, categories: Vec<String>) {
        self.categories = if categories.is_empty() {
            None
        } else {
            Some(categories.join(","))
        };
    }

    pub fn get_tags(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .map(|s| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = if tags.is_empty() {
            None
        } else {
            Some(tags.join(","))
        };
    }
}

// 分类计数实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbCategoryCount {
    pub category: String,
    pub count: i64,
}

// 收藏实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbFavorite {
    pub book_id: String,
    pub added_at: i64,
}

impl DbFavorite {
    pub fn new(book_id: String) -> Self {
        Self {
            book_id,
            added_at: Utc::now().timestamp(),
        }
    }
}

// 浏览历史实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbHistory {
    pub book_id: String,
    pub last_read: i64,
    pub last_eps: i64,
    pub last_page: i64,
}

impl DbHistory {
    pub fn new(book_id: String) -> Self {
        Self {
            book_id,
            last_read: Utc::now().timestamp(),
            last_eps: 0,
            last_page: 0,
        }
    }

    pub fn update_progress(&mut self, eps: i64, page: i64) {
        self.last_read = Utc::now().timestamp();
        self.last_eps = eps;
        self.last_page = page;
    }
}

// 下载任务数据库实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbDownloadTask {
    pub comic_id: String,
    pub comic_title: String,
    pub total_episodes: i64,
    pub episode_orders: String, // JSON 数组
    pub save_path: String,
    pub state: String,              // Queued/Downloading/Paused/Completed/Failed
    pub state_data: Option<String>, // JSON: { current_episode, current_page, error }
    pub created_at: i64,
    pub updated_at: i64,
}

/// 下载状态附加数据（序列化为 JSON 存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStateData {
    #[serde(default)]
    pub current_episode: i32,
    #[serde(default)]
    pub current_page: i32,
    #[serde(default)]
    pub error: Option<String>,
}

impl Default for DownloadStateData {
    fn default() -> Self {
        Self {
            current_episode: 0,
            current_page: 0,
            error: None,
        }
    }
}

impl DbDownloadTask {
    /// 创建新的下载任务
    pub fn new(
        comic_id: String,
        comic_title: String,
        episode_orders: Vec<i32>,
        save_path: String,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            comic_id,
            comic_title,
            total_episodes: episode_orders.len() as i64,
            episode_orders: serde_json::to_string(&episode_orders).unwrap_or_default(),
            save_path,
            state: "Queued".to_string(),
            state_data: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 获取章节顺序列表
    pub fn get_episode_orders(&self) -> Vec<i32> {
        serde_json::from_str(&self.episode_orders).unwrap_or_default()
    }

    /// 获取状态数据
    pub fn get_state_data(&self) -> DownloadStateData {
        self.state_data
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// 设置状态数据
    pub fn set_state_data(&mut self, data: &DownloadStateData) {
        self.state_data = serde_json::to_string(data).ok();
        self.updated_at = Utc::now().timestamp();
    }

    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        self.state == "Completed"
    }

    /// 是否正在下载
    pub fn is_downloading(&self) -> bool {
        self.state == "Downloading"
    }

    /// 是否可以暂停
    pub fn can_pause(&self) -> bool {
        self.state == "Queued" || self.state == "Downloading"
    }

    /// 是否可以恢复
    pub fn can_resume(&self) -> bool {
        self.state == "Paused" || self.state == "Failed"
    }
}
