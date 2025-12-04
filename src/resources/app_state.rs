//! 应用状态资源
//!
//! 定义应用的全局状态

use bevy::prelude::*;

use crate::{
    api::{
        endpoints::RankTimeType,
        models::{Category, Comic, Episode, Picture},
    },
    config::settings::{AppSettings, ProxyType},
};

/// 应用路由状态
#[derive(Debug, Clone, PartialEq, Eq, Default, States, Hash)]
pub enum AppRoute {
    /// 登录页面
    #[default]
    Login,
    /// 代理设置
    ProxySettings,
    /// 主页
    Home,
    /// 分类浏览
    Categories,
    /// 漫画列表
    ComicsList,
    /// 漫画详情
    ComicDetail,
    /// 阅读界面
    ReadView,
    /// 搜索
    Search,
    /// 排行榜
    Rankings,
    /// 收藏
    Favorites,
    /// 下载管理
    Downloads,
    /// 设置
    Settings,
}

/// 认证状态
#[derive(Resource, Default)]
pub struct AuthState {
    /// 认证 token
    pub token: Option<String>,
    /// 是否已登录
    pub is_logged_in: bool,
}

/// 登录表单状态
#[derive(Resource)]
pub struct LoginFormState {
    /// 用户名/邮箱
    pub email: String,
    /// 密码
    pub password: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 是否保存密码
    pub save_password: bool,
    /// 是否自动登录
    pub auto_login: bool,
    /// 是否自动打卡
    pub auto_punch_in: bool,
}

impl Default for LoginFormState {
    fn default() -> Self {
        // 从配置加载保存的设置
        let settings = AppSettings::global().read();
        Self {
            email: settings.login.saved_email.clone(),
            password: if settings.login.save_password {
                settings.login.saved_password.clone()
            } else {
                String::new()
            },
            is_loading: false,
            error: None,
            save_password: settings.login.save_password,
            auto_login: settings.login.auto_login,
            auto_punch_in: settings.login.auto_punch_in,
        }
    }
}

/// 分类列表状态
#[derive(Resource, Default)]
pub struct CategoriesState {
    /// 分类列表
    pub categories: Vec<Category>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 漫画列表状态
#[derive(Resource)]
pub struct ComicsListState {
    /// 当前分类
    pub category: String,
    /// 漫画列表
    pub comics: Vec<Comic>,
    /// 当前页码
    pub page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 排序方式
    pub sort: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl Default for ComicsListState {
    fn default() -> Self {
        Self {
            category: String::new(),
            comics: Vec::new(),
            page: 1,
            total_pages: 1,
            sort: "dd".to_string(),
            is_loading: false,
            error: None,
        }
    }
}

/// 搜索状态
#[derive(Resource)]
pub struct SearchState {
    /// 搜索关键词
    pub keyword: String,
    /// 搜索结果
    pub results: Vec<Comic>,
    /// 当前页码
    pub page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 排序方式
    pub sort: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 是否已执行过搜索
    pub has_searched: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            keyword: String::new(),
            results: Vec::new(),
            page: 1,
            total_pages: 1,
            sort: "dd".to_string(),
            is_loading: false,
            has_searched: false,
            error: None,
        }
    }
}

/// 排行榜状态
#[derive(Resource)]
pub struct RankingsState {
    /// 当前选中的时间类型
    pub current_type: RankTimeType,
    /// 日榜数据
    pub h24_comics: Vec<Comic>,
    /// 周榜数据
    pub d7_comics: Vec<Comic>,
    /// 月榜数据
    pub d30_comics: Vec<Comic>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl Default for RankingsState {
    fn default() -> Self {
        Self {
            current_type: RankTimeType::H24,
            h24_comics: Vec::new(),
            d7_comics: Vec::new(),
            d30_comics: Vec::new(),
            is_loading: false,
            error: None,
        }
    }
}

impl RankingsState {
    /// 获取当前类型的漫画列表
    pub fn current_comics(&self) -> &[Comic] {
        match self.current_type {
            RankTimeType::H24 => &self.h24_comics,
            RankTimeType::D7 => &self.d7_comics,
            RankTimeType::D30 => &self.d30_comics,
        }
    }

    /// 设置指定类型的漫画列表
    pub fn set_comics(&mut self, time_type: RankTimeType, comics: Vec<Comic>) {
        match time_type {
            RankTimeType::H24 => self.h24_comics = comics,
            RankTimeType::D7 => self.d7_comics = comics,
            RankTimeType::D30 => self.d30_comics = comics,
        }
    }

    /// 检查指定类型是否已加载
    pub fn is_loaded(&self, time_type: RankTimeType) -> bool {
        match time_type {
            RankTimeType::H24 => !self.h24_comics.is_empty(),
            RankTimeType::D7 => !self.d7_comics.is_empty(),
            RankTimeType::D30 => !self.d30_comics.is_empty(),
        }
    }
}

/// 收藏列表状态
#[derive(Resource)]
pub struct FavoritesState {
    /// 收藏的漫画列表
    pub comics: Vec<Comic>,
    /// 当前页码
    pub page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 排序方式 (dd: 新到旧, da: 旧到新)
    pub sort: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl Default for FavoritesState {
    fn default() -> Self {
        Self {
            comics: Vec::new(),
            page: 1,
            total_pages: 1,
            sort: "dd".to_string(),
            is_loading: false,
            error: None,
        }
    }
}

/// 首页状态
#[derive(Resource, Default)]
pub struct HomeState {
    /// 推荐漫画列表
    pub recommendations: Vec<Comic>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 漫画详情状态
#[derive(Resource, Default)]
pub struct ComicDetailState {
    /// 漫画 ID
    pub comic_id: String,
    /// 漫画信息
    pub comic: Option<Comic>,
    /// 章节列表
    pub episodes: Vec<Episode>,
    /// 章节页码
    pub episodes_page: i32,
    /// 章节总页数
    pub episodes_total_pages: i32,
    /// 是否正在加载
    pub is_loading: bool,
    /// 是否正在加载章节
    pub is_loading_episodes: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 是否已收藏
    pub is_favorite: bool,
    /// 是否已点赞
    pub is_liked: bool,
}

/// 阅读模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadMode {
    /// 单页模式
    #[default]
    SinglePage,
    /// 双页模式
    DoublePage,
    /// 滚动模式
    Scroll,
}

/// 阅读器状态
#[derive(Resource)]
pub struct ReaderState {
    /// 漫画 ID
    pub comic_id: String,
    /// 章节顺序
    pub episode_order: i32,
    /// 当前页码
    pub current_page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 图片列表
    pub pictures: Vec<Picture>,
    /// 缩放比例
    pub scale: f32,
    /// 阅读模式
    pub read_mode: ReadMode,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl Default for ReaderState {
    fn default() -> Self {
        Self {
            comic_id: String::new(),
            episode_order: 1,
            current_page: 1,
            total_pages: 0,
            pictures: Vec::new(),
            scale: 1.0,
            read_mode: ReadMode::SinglePage,
            is_loading: false,
            error: None,
        }
    }
}

/// 代理设置状态
#[derive(Resource)]
pub struct ProxySettingsState {
    /// 是否启用
    pub enabled: bool,
    /// 代理类型
    pub proxy_type: ProxyType,
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: String,
    /// 是否使用认证
    pub use_auth: bool,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否正在测试
    pub is_testing: bool,
    /// 测试消息
    pub test_message: Option<String>,
}

impl Default for ProxySettingsState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            enabled: settings.proxy.enabled,
            proxy_type: settings.proxy.proxy_type,
            host: settings.proxy.host.clone(),
            port: settings.proxy.port.to_string(),
            use_auth: settings.proxy.use_auth,
            username: settings.proxy.username.clone(),
            password: settings.proxy.password.clone(),
            is_testing: false,
            test_message: None,
        }
    }
}

/// 全局消息状态（用于显示提示）
#[derive(Resource, Default)]
pub struct GlobalMessageState {
    /// 错误消息
    pub error: Option<String>,
    /// 成功消息
    pub success: Option<String>,
}

/// 下载任务状态（用于 UI 显示）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComicDownloadStatus {
    /// 等待中
    Waiting,
    /// 下载中
    Downloading,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed(String),
}

// ==================== FSM 下载系统 ====================

/// 下载状态（FSM 状态机）
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DownloadState {
    /// 排队中（等待下载）
    Queued,
    /// 下载中
    Downloading {
        current_episode: i32,
        current_page: i32,
    },
    /// 已暂停
    Paused {
        current_episode: i32,
        current_page: i32,
    },
    /// 已完成
    Completed,
    /// 失败
    Failed(String),
}

impl Default for DownloadState {
    fn default() -> Self {
        Self::Queued
    }
}

impl DownloadState {
    /// 转换为 UI 显示状态
    pub fn to_ui_status(&self) -> ComicDownloadStatus {
        match self {
            DownloadState::Queued => ComicDownloadStatus::Waiting,
            DownloadState::Downloading { .. } => ComicDownloadStatus::Downloading,
            DownloadState::Paused { .. } => ComicDownloadStatus::Paused,
            DownloadState::Completed => ComicDownloadStatus::Completed,
            DownloadState::Failed(err) => ComicDownloadStatus::Failed(err.clone()),
        }
    }

    /// 是否可以暂停
    pub fn can_pause(&self) -> bool {
        matches!(
            self,
            DownloadState::Queued | DownloadState::Downloading { .. }
        )
    }

    /// 是否可以恢复
    pub fn can_resume(&self) -> bool {
        matches!(
            self,
            DownloadState::Paused { .. } | DownloadState::Failed(_)
        )
    }

    /// 是否正在下载
    pub fn is_downloading(&self) -> bool {
        matches!(self, DownloadState::Downloading { .. })
    }

    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(self, DownloadState::Completed)
    }

    /// 获取当前章节
    pub fn current_episode(&self) -> i32 {
        match self {
            DownloadState::Downloading {
                current_episode, ..
            } => *current_episode,
            DownloadState::Paused {
                current_episode, ..
            } => *current_episode,
            _ => 0,
        }
    }

    /// 获取当前页码
    pub fn current_page(&self) -> i32 {
        match self {
            DownloadState::Downloading { current_page, .. } => *current_page,
            DownloadState::Paused { current_page, .. } => *current_page,
            _ => 0,
        }
    }
}

/// 下载任务元数据（持久化到数据库）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadTaskMeta {
    /// 漫画 ID
    pub comic_id: String,
    /// 漫画标题
    pub comic_title: String,
    /// 总章节数
    pub total_episodes: i32,
    /// 要下载的章节顺序列表
    pub episode_orders: Vec<i32>,
    /// 保存路径
    pub save_path: String,
    /// 当前状态
    pub state: DownloadState,
    /// 创建时间（时间戳）
    pub created_at: i64,
    /// 更新时间（时间戳）
    pub updated_at: i64,
    /// 分类列表
    #[serde(default)]
    pub categories: Vec<String>,
    /// 标签列表
    #[serde(default)]
    pub tags: Vec<String>,
}

impl DownloadTaskMeta {
    /// 创建新的下载任务元数据
    pub fn new(
        comic_id: String,
        comic_title: String,
        episode_orders: Vec<i32>,
        save_path: String,
        categories: Vec<String>,
        tags: Vec<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            comic_id,
            comic_title,
            total_episodes: episode_orders.len() as i32,
            episode_orders,
            save_path,
            state: DownloadState::Queued,
            created_at: now,
            updated_at: now,
            categories,
            tags,
        }
    }

    /// 转换为数据库实体
    pub fn to_db_task(&self) -> crate::db::models::DbDownloadTask {
        use crate::db::models::{DbDownloadTask, DownloadStateData};

        let mut db_task = DbDownloadTask::new(
            self.comic_id.clone(),
            self.comic_title.clone(),
            self.episode_orders.clone(),
            self.save_path.clone(),
        );

        // 设置状态
        let (state_str, state_data) = match &self.state {
            DownloadState::Queued => ("Queued".to_string(), DownloadStateData::default()),
            DownloadState::Downloading {
                current_episode,
                current_page,
            } => (
                "Downloading".to_string(),
                DownloadStateData {
                    current_episode: *current_episode,
                    current_page: *current_page,
                    error: None,
                },
            ),
            DownloadState::Paused {
                current_episode,
                current_page,
            } => (
                "Paused".to_string(),
                DownloadStateData {
                    current_episode: *current_episode,
                    current_page: *current_page,
                    error: None,
                },
            ),
            DownloadState::Completed => ("Completed".to_string(), DownloadStateData::default()),
            DownloadState::Failed(err) => (
                "Failed".to_string(),
                DownloadStateData {
                    current_episode: 0,
                    current_page: 0,
                    error: Some(err.clone()),
                },
            ),
        };

        db_task.state = state_str;
        db_task.set_state_data(&state_data);
        db_task.created_at = self.created_at;
        db_task.updated_at = self.updated_at;
        db_task.set_categories(&self.categories);
        db_task.set_tags(&self.tags);

        db_task
    }

    /// 从数据库实体创建
    pub fn from_db_task(db_task: &crate::db::models::DbDownloadTask) -> Self {
        let state_data = db_task.get_state_data();

        let state = match db_task.state.as_str() {
            "Queued" => DownloadState::Queued,
            "Downloading" => DownloadState::Downloading {
                current_episode: state_data.current_episode,
                current_page: state_data.current_page,
            },
            "Paused" => DownloadState::Paused {
                current_episode: state_data.current_episode,
                current_page: state_data.current_page,
            },
            "Completed" => DownloadState::Completed,
            "Failed" => DownloadState::Failed(state_data.error.unwrap_or_default()),
            _ => DownloadState::Queued,
        };

        Self {
            comic_id: db_task.comic_id.clone(),
            comic_title: db_task.comic_title.clone(),
            total_episodes: db_task.total_episodes as i32,
            episode_orders: db_task.get_episode_orders(),
            save_path: db_task.save_path.clone(),
            state,
            created_at: db_task.created_at,
            updated_at: db_task.updated_at,
            categories: db_task.get_categories(),
            tags: db_task.get_tags(),
        }
    }

    /// 保存元数据到数据库
    pub fn save(&self) -> Result<(), String> {
        use crate::db::database::{Database, run_db_operation};

        let db_task = self.to_db_task();

        // 使用 run_db_operation 自动处理运行时上下文
        run_db_operation(async move {
            let db = Database::global().read();
            db.upsert_download_task(&db_task)
                .await
                .map_err(|e| format!("保存到数据库失败: {}", e))
        })
    }

    /// 从数据库加载元数据
    pub fn load(save_path: &str) -> Result<Self, String> {
        use crate::db::database::{Database, run_db_operation};

        let save_path_owned = save_path.to_string();

        // 从数据库加载（通过 save_path 查找）
        run_db_operation(async move {
            let db = Database::global().read();
            let tasks = db
                .get_all_download_tasks()
                .await
                .map_err(|e| format!("数据库查询失败: {}", e))?;

            // 通过 save_path 查找
            for task in tasks {
                if task.save_path == save_path_owned {
                    return Ok(Self::from_db_task(&task));
                }
            }
            Err("下载任务不存在".to_string())
        })
    }

    /// 从数据库加载元数据（通过 comic_id）
    pub fn load_by_comic_id(comic_id: &str) -> Result<Self, String> {
        use crate::db::database::{Database, run_db_operation};

        let comic_id_owned = comic_id.to_string();

        run_db_operation(async move {
            let db = Database::global().read();
            let task = db
                .get_download_task(&comic_id_owned)
                .await
                .map_err(|e| format!("数据库查询失败: {}", e))?
                .ok_or_else(|| "下载任务不存在".to_string())?;

            Ok(Self::from_db_task(&task))
        })
    }

    /// 更新状态并保存
    pub fn update_state(&mut self, state: DownloadState) -> Result<(), String> {
        self.state = state;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.save()
    }

    /// 删除下载任务
    pub fn delete(&self) -> Result<(), String> {
        use crate::db::database::{Database, run_db_operation};

        let comic_id = self.comic_id.clone();

        // 从数据库删除
        run_db_operation(async move {
            let db = Database::global().read();
            db.delete_download_task(&comic_id)
                .await
                .map_err(|e| format!("删除下载任务失败: {}", e))
        })
    }
}

/// 共享任务控制（主线程和后台任务之间通信）
#[derive(Debug)]
pub struct SharedTaskControl {
    /// 暂停请求标志
    pub pause_requested: std::sync::atomic::AtomicBool,
    /// 取消请求标志
    pub cancel_requested: std::sync::atomic::AtomicBool,
}

impl SharedTaskControl {
    pub fn new() -> Self {
        Self {
            pause_requested: std::sync::atomic::AtomicBool::new(false),
            cancel_requested: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn request_pause(&self) {
        self.pause_requested
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn request_cancel(&self) {
        self.cancel_requested
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_pause_requested(&self) -> bool {
        self.pause_requested
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn is_cancel_requested(&self) -> bool {
        self.cancel_requested
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.pause_requested
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.cancel_requested
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for SharedTaskControl {
    fn default() -> Self {
        Self::new()
    }
}

/// 下载任务 FSM
#[derive(Debug)]
pub struct DownloadTaskFSM {
    /// 任务元数据
    pub meta: DownloadTaskMeta,
    /// 共享控制（用于暂停/取消）
    pub control: std::sync::Arc<SharedTaskControl>,
    /// 当前章节总页数（运行时数据）
    pub current_episode_total_pages: i32,
}

impl DownloadTaskFSM {
    /// 创建新任务
    pub fn new(meta: DownloadTaskMeta) -> Self {
        Self {
            meta,
            control: std::sync::Arc::new(SharedTaskControl::new()),
            current_episode_total_pages: 0,
        }
    }

    /// 从元数据文件加载任务
    pub fn load(save_path: &str) -> Result<Self, String> {
        let meta = DownloadTaskMeta::load(save_path)?;
        Ok(Self::new(meta))
    }

    /// 转换为 UI 显示的 ComicDownloadTask
    pub fn to_ui_task(&self) -> ComicDownloadTask {
        ComicDownloadTask {
            comic_id: self.meta.comic_id.clone(),
            comic_title: self.meta.comic_title.clone(),
            status: self.meta.state.to_ui_status(),
            current_episode: self.meta.state.current_episode(),
            total_episodes: self.meta.total_episodes,
            current_page: self.meta.state.current_page(),
            total_pages: self.current_episode_total_pages,
            save_path: self.meta.save_path.clone(),
            categories: self.meta.categories.clone(),
            tags: self.meta.tags.clone(),
        }
    }

    /// 开始下载
    pub fn start(&mut self) -> Result<(), String> {
        self.meta.update_state(DownloadState::Downloading {
            current_episode: 1,
            current_page: 0,
        })
    }

    /// 排队等待（设置为 Queued 状态）
    pub fn queue(&mut self) -> Result<(), String> {
        self.meta.update_state(DownloadState::Queued)
    }

    /// 更新进度
    pub fn update_progress(
        &mut self,
        episode: i32,
        page: i32,
        total_pages: i32,
    ) -> Result<(), String> {
        self.current_episode_total_pages = total_pages;
        self.meta.update_state(DownloadState::Downloading {
            current_episode: episode,
            current_page: page,
        })
    }

    /// 暂停
    pub fn pause(&mut self) -> Result<(), String> {
        let (episode, page) = match &self.meta.state {
            DownloadState::Downloading {
                current_episode,
                current_page,
            } => (*current_episode, *current_page),
            _ => (1, 0),
        };
        self.meta.update_state(DownloadState::Paused {
            current_episode: episode,
            current_page: page,
        })
    }

    /// 完成
    pub fn complete(&mut self) -> Result<(), String> {
        self.meta.update_state(DownloadState::Completed)
    }

    /// 失败
    pub fn fail(&mut self, error: String) -> Result<(), String> {
        self.meta.update_state(DownloadState::Failed(error))
    }

    /// 请求暂停（线程安全）
    pub fn request_pause(&self) {
        self.control.request_pause();
    }

    /// 检查是否应该暂停
    pub fn should_pause(&self) -> bool {
        self.control.is_pause_requested()
    }

    /// 获取控制器的克隆（用于传递给后台任务）
    pub fn get_control(&self) -> std::sync::Arc<SharedTaskControl> {
        self.control.clone()
    }
}

/// 单个漫画下载任务
#[derive(Debug, Clone)]
pub struct ComicDownloadTask {
    /// 漫画 ID
    pub comic_id: String,
    /// 漫画标题
    pub comic_title: String,
    /// 下载状态
    pub status: ComicDownloadStatus,
    /// 当前章节
    pub current_episode: i32,
    /// 总章节数
    pub total_episodes: i32,
    /// 当前章节已下载图片数
    pub current_page: i32,
    /// 当前章节总图片数
    pub total_pages: i32,
    /// 保存路径
    pub save_path: String,
    /// 分类列表
    pub categories: Vec<String>,
    /// 标签列表
    pub tags: Vec<String>,
}

/// 下载管理状态（FSM 架构）
#[derive(Resource, Default)]
pub struct DownloadManagerState {
    /// 下载任务 FSM 列表
    pub fsm_tasks: Vec<DownloadTaskFSM>,
    /// 正在下载的漫画 ID
    pub downloading_ids: std::collections::HashSet<String>,
}

impl DownloadManagerState {
    /// 根据 comic_id 查找任务
    pub fn find_task(&self, comic_id: &str) -> Option<&DownloadTaskFSM> {
        self.fsm_tasks.iter().find(|t| t.meta.comic_id == comic_id)
    }

    /// 根据 comic_id 查找任务（可变）
    pub fn find_task_mut(&mut self, comic_id: &str) -> Option<&mut DownloadTaskFSM> {
        self.fsm_tasks
            .iter_mut()
            .find(|t| t.meta.comic_id == comic_id)
    }

    /// 获取活跃任务列表（未完成的）
    pub fn active_tasks(&self) -> Vec<&DownloadTaskFSM> {
        self.fsm_tasks
            .iter()
            .filter(|t| !t.meta.state.is_completed())
            .collect()
    }

    /// 获取已完成任务列表
    pub fn completed_tasks(&self) -> Vec<&DownloadTaskFSM> {
        self.fsm_tasks
            .iter()
            .filter(|t| t.meta.state.is_completed())
            .collect()
    }

    /// 转换为 UI 显示的任务列表（兼容旧代码）
    pub fn tasks(&self) -> Vec<ComicDownloadTask> {
        self.fsm_tasks.iter().map(|t| t.to_ui_task()).collect()
    }

    /// 添加新任务
    pub fn add_task(&mut self, meta: DownloadTaskMeta) -> &mut DownloadTaskFSM {
        let fsm = DownloadTaskFSM::new(meta);
        self.fsm_tasks.push(fsm);
        self.fsm_tasks.last_mut().unwrap()
    }

    /// 移除任务
    pub fn remove_task(&mut self, comic_id: &str) {
        self.fsm_tasks.retain(|t| t.meta.comic_id != comic_id);
        self.downloading_ids.remove(comic_id);
    }

    /// 从数据库加载未完成的任务
    pub fn load_incomplete_tasks(&mut self) {
        use crate::db::database::{Database, run_db_operation};

        // 首先尝试从数据库加载
        let db_tasks: Vec<crate::db::models::DbDownloadTask> = run_db_operation(async {
            let db = Database::global().read();
            db.get_incomplete_download_tasks().await
        })
        .unwrap_or_default();

        for db_task in db_tasks {
            let meta = DownloadTaskMeta::from_db_task(&db_task);

            // 检查是否已经存在
            if self.find_task(&meta.comic_id).is_some() {
                tracing::debug!("任务已存在，跳过: {}", meta.comic_title);
                continue;
            }

            tracing::info!("从数据库加载下载任务: {}", meta.comic_title);
            let mut fsm = DownloadTaskFSM::new(meta);

            // 如果任务状态是 Downloading，自动转换为 Paused
            // （因为程序重启后后台下载任务已停止）
            if let DownloadState::Downloading {
                current_episode,
                current_page,
            } = fsm.meta.state.clone()
            {
                fsm.meta.state = DownloadState::Paused {
                    current_episode,
                    current_page,
                };
                // 保存状态变更
                let _ = fsm.meta.save();
                tracing::info!(
                    "将任务状态从 Downloading 转换为 Paused: {}",
                    fsm.meta.comic_title
                );
            }
            self.fsm_tasks.push(fsm);
        }

        tracing::info!("加载了 {} 个未完成的下载任务", self.active_tasks().len());
    }
}
