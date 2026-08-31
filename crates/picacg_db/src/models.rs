//! 数据库模型

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 浏览历史实体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DbHistory {
    pub book_id: String,
    pub last_read: i64,
    pub last_eps: i64,
    pub last_page: i64,
    /// 漫画标题（冗余存储，用于历史列表展示）
    #[sqlx(default)]
    pub comic_title: Option<String>,
    /// 封面缩略图 URL（冗余存储，用于历史列表展示）
    #[sqlx(default)]
    pub thumb_url: Option<String>,
    /// 最后阅读的章节标题
    #[sqlx(default)]
    pub last_eps_title: Option<String>,
}

impl DbHistory {
    /// 创建包含完整信息的历史记录
    pub fn with_info(
        book_id: String,
        comic_title: String,
        thumb_url: String,
        last_eps: i64,
        last_eps_title: String,
        last_page: i64,
    ) -> Self {
        Self {
            book_id,
            last_read: Utc::now().timestamp(),
            last_eps,
            last_page,
            comic_title: Some(comic_title),
            thumb_url: Some(thumb_url),
            last_eps_title: Some(last_eps_title),
        }
    }
}

// 点赞记录实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbLikeRecord {
    /// 漫画 ID
    pub comic_id: String,
    /// 漫画标题
    pub comic_title: String,
    /// 封面缩略图 URL
    #[sqlx(default)]
    pub thumb_url: Option<String>,
    /// 点赞时间（时间戳）
    pub liked_at: i64,
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
    #[sqlx(default)]
    pub categories: Option<String>, // JSON 数组
    #[sqlx(default)]
    pub tags: Option<String>, // JSON 数组
    /// JSON 数组：已完整下载的章节号
    ///
    /// **唯一写者是 `add_completed_episode_async`**（读-改-写）。全量 upsert
    /// 对该列用 COALESCE 只增不清——FSM 的 `to_db_task()` 不携带该字段，
    /// 裸覆盖会让每次进度保存都把标记冲掉（历史 bug，有回归锁单测）。
    #[sqlx(default)]
    pub completed_episodes: Option<String>,
    /// 独立下载路径（None 时使用全局设置）
    #[sqlx(default)]
    pub custom_download_path: Option<String>,
    /// 独立 CBZ 打包开关（None 时使用全局设置）
    #[sqlx(default)]
    pub custom_auto_pack_cbz: Option<i64>, // 0=false, 1=true, NULL=使用全局
    /// 下载/更新当时服务端 `epsCount` 的快照（None = 老记录，未知）
    ///
    /// **更新检测的基准**。不能拿 `epsCount` 直接跟本地章节数比——实测该字段
    /// 与 `/comics/{id}/eps` 的真实条数长期对不上，且两个方向都会偏
    /// （48↔49、46↔48、12↔15、55↔53）。它是个漂移的冗余计数，但对同一本漫画
    /// **随时间自增**，所以「今天的 epsCount > 下载当时的
    /// epsCount」才是可靠信号： 同一字段自比，系统偏差相消。
    #[sqlx(default)]
    pub remote_eps_count: Option<i64>,
}

/// 下载状态附加数据（序列化为 JSON 存储）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadStateData {
    #[serde(default)]
    pub current_episode: i32,
    #[serde(default)]
    pub current_page: i32,
    #[serde(default)]
    pub error: Option<String>,
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
            categories: None,
            tags: None,
            completed_episodes: None,
            custom_download_path: None,
            custom_auto_pack_cbz: None,
            remote_eps_count: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 获取独立 CBZ 打包开关
    pub fn get_custom_auto_pack_cbz(&self) -> Option<bool> {
        self.custom_auto_pack_cbz.map(|v| v != 0)
    }

    /// 设置独立 CBZ 打包开关
    pub fn set_custom_auto_pack_cbz(&mut self, value: Option<bool>) {
        self.custom_auto_pack_cbz = value.map(|v| if v { 1 } else { 0 });
    }

    /// 获取分类列表
    pub fn get_categories(&self) -> Vec<String> {
        self.categories
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// 设置分类列表
    pub fn set_categories(&mut self, categories: &[String]) {
        self.categories = serde_json::to_string(categories).ok();
    }

    /// 获取标签列表
    pub fn get_tags(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// 设置标签列表
    pub fn set_tags(&mut self, tags: &[String]) {
        self.tags = serde_json::to_string(tags).ok();
    }

    /// 获取已完成的章节列表
    pub fn get_completed_episodes(&self) -> Vec<i32> {
        self.completed_episodes
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// 设置已完成的章节列表
    pub fn set_completed_episodes(&mut self, episodes: &[i32]) {
        self.completed_episodes = serde_json::to_string(episodes).ok();
        self.updated_at = Utc::now().timestamp();
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
}
