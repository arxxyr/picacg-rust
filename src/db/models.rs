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
