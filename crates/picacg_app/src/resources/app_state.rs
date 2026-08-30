//! 应用状态资源
//!
//! 定义应用的全局状态

use bevy::prelude::*;
use picacg_api::{
    endpoints::{
        RankTimeType,
        fried::{AppInfo, FriedPost},
        rank::KnightUser,
    },
    models::{Category, Comic, Episode, Game, Picture},
};
use picacg_config::{AppSettings, ProxyType};

/// 应用路由状态
#[derive(Debug, Clone, PartialEq, Eq, Default, States, Hash)]
pub enum AppRoute {
    /// 登录页面
    #[default]
    Login,
    /// 注册页面
    Register,
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
    /// 阅读历史
    History,
    /// 评论页面
    Comments,
    /// 忘记密码
    ForgotPassword,
    /// 个人资料
    Profile,
    /// 本地阅读（已下载漫画离线浏览）
    LocalRead,
    /// 游戏列表
    Games,
    /// 游戏详情
    GameDetail,
    /// 点赞记录
    LikeRecords,
    /// 锅贴社区
    Fried,
    /// 图片格式转换工具
    ImageConvert,
    /// Waifu2x 超分辨率工具
    Waifu2x,
    /// 聊天大厅（房间列表）
    Chat,
    /// 聊天室
    ChatRoom,
    /// NAS 远程存储
    Nas,
}

/// 版本更新检查状态
#[derive(Resource, Default)]
pub struct UpdateCheckState {
    /// 是否正在检查
    pub is_checking: bool,
    /// 最新版本号
    pub latest_version: Option<String>,
    /// 是否有更新
    pub has_update: Option<bool>,
    /// 更新说明
    pub release_notes: Option<String>,
    /// 下载链接
    pub download_url: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 本地漫画条目（扫描到的已下载漫画）
#[derive(Debug, Clone)]
pub struct LocalComicEntry {
    /// 漫画文件夹名称
    pub name: String,
    /// 漫画文件夹完整路径
    pub path: String,
    /// 封面图片路径（第一个章节的第一张图片）
    pub cover_path: Option<String>,
    /// 章节数量（子文件夹数量）
    pub chapter_count: usize,
}

/// 本地阅读状态
#[derive(Resource, Default)]
pub struct LocalReadState {
    /// 本地漫画列表
    pub entries: Vec<LocalComicEntry>,
    /// 是否正在扫描
    pub is_scanning: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 用户个人资料状态
#[derive(Resource, Default)]
pub struct UserProfileState {
    /// 用户信息
    pub user: Option<picacg_api::models::User>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 点赞记录状态
#[derive(Resource, Default)]
pub struct LikeRecordsState {
    /// 点赞记录列表
    pub records: Vec<picacg_db::DbLikeRecord>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 总数
    pub total_count: i64,
}

/// 阅读历史状态
#[derive(Resource, Default)]
pub struct HistoryState {
    /// 历史记录列表
    pub records: Vec<picacg_db::DbHistory>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 历史总数
    pub total_count: i64,
}

/// 子评论状态
#[derive(Default, Clone)]
pub struct ChildCommentsState {
    /// 子评论列表
    pub comments: Vec<picacg_api::models::Comment>,
    /// 当前页码
    pub page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 是否加载中
    pub is_loading: bool,
}

/// 评论页面状态
#[derive(Resource, Default)]
pub struct CommentsState {
    /// 漫画 ID（评论目标）
    pub comic_id: String,
    /// 评论列表
    pub comments: Vec<picacg_api::models::Comment>,
    /// 当前页码
    pub page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 是否加载中
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 正在回复的评论 ID（None = 顶层评论）
    pub reply_to: Option<String>,
    /// 正在回复的用户名（用于显示提示）
    pub reply_to_name: Option<String>,
    /// 子评论展开状态（comment_id -> ChildCommentsState）
    pub children_map: std::collections::HashMap<String, ChildCommentsState>,
    /// 是否需要重建 UI
    pub needs_rebuild: bool,
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

/// 忘记密码页面状态
#[derive(Resource, Default)]
pub struct ForgotPasswordState {
    /// 邮箱/用户名
    pub email: String,
    /// 当前步骤: 0=输入邮箱获取安全问题, 1=回答安全问题重置密码
    pub step: u8,
    /// 安全问题列表（由 API 返回）
    pub question1: String,
    pub question2: String,
    pub question3: String,
    /// 选择的安全问题编号 (1, 2, 3)
    pub question_no: i32,
    /// 安全问题答案
    pub answer: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 成功信息
    pub success: Option<String>,
}

/// 性别选项
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gender {
    #[default]
    Male,
    Female,
    Bot,
}

impl Gender {
    /// 转换为 API 需要的字符串
    pub fn as_api_str(self) -> &'static str {
        match self {
            Gender::Male => "m",
            Gender::Female => "f",
            Gender::Bot => "bot",
        }
    }

    /// 显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Gender::Male => "男",
            Gender::Female => "女",
            Gender::Bot => "扶她",
        }
    }
}

/// 注册表单状态
#[derive(Resource, Default)]
pub struct RegisterFormState {
    /// 邮箱/用户名
    pub email: String,
    /// 密码
    pub password: String,
    /// 确认密码
    pub confirm_password: String,
    /// 昵称
    pub name: String,
    /// 生日 (格式: yyyy-MM-dd)
    pub birthday: String,
    /// 性别
    pub gender: Gender,
    /// 安全问题1
    pub question1: String,
    /// 安全问题2
    pub question2: String,
    /// 安全问题3
    pub question3: String,
    /// 安全问题答案1
    pub answer1: String,
    /// 安全问题答案2
    pub answer2: String,
    /// 安全问题答案3
    pub answer3: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 成功信息
    pub success: Option<String>,
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

/// 缓存的标签列表（从数据库 book 表聚合，用于屏蔽词建议）
#[derive(Resource, Default)]
pub struct CachedTagsState {
    /// 去重排序后的所有标签
    pub tags: Vec<String>,
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
    /// 是否正在加载（首次加载）
    pub is_loading: bool,
    /// 是否正在加载更多（无限滚动追加）
    pub is_loading_more: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 保存的滚动位置（用于返回时恢复）
    pub scroll_y: f32,
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
            is_loading_more: false,
            error: None,
            scroll_y: 0.0,
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
    /// 选中的分类过滤列表
    pub selected_categories: Vec<String>,
    /// 是否展开分类过滤面板
    pub show_category_filter: bool,
    /// 是否需要重建 UI（仅在搜索结果/排序/分类/翻页/错误变化时设置，
    /// 输入文字不触发）
    pub needs_rebuild: bool,
    /// 热门搜索关键词
    pub hot_keywords: Vec<String>,
    /// 热词是否已加载
    pub hot_keywords_loaded: bool,
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
            selected_categories: Vec::new(),
            show_category_filter: false,
            needs_rebuild: false,
            hot_keywords: Vec::new(),
            hot_keywords_loaded: false,
        }
    }
}

/// 排行榜标签类型（包含漫画排行和骑士榜）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RankingsTabType {
    /// 漫画排行（日/周/月）
    Comics(RankTimeType),
    /// 骑士榜
    Knight,
}

impl RankingsTabType {
    /// 显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            RankingsTabType::Comics(time_type) => time_type.display_name(),
            RankingsTabType::Knight => "骑士榜",
        }
    }

    /// 是否为漫画排行标签
    pub fn is_comics(&self) -> bool {
        matches!(self, RankingsTabType::Comics(_))
    }

    /// 是否为骑士榜标签
    pub fn is_knight(&self) -> bool {
        matches!(self, RankingsTabType::Knight)
    }
}

/// 排行榜状态
#[derive(Resource)]
pub struct RankingsState {
    /// 当前选中的标签类型
    pub current_tab: RankingsTabType,
    /// 当前选中的时间类型（仅漫画排行时使用）
    pub current_type: RankTimeType,
    /// 日榜数据
    pub h24_comics: Vec<Comic>,
    /// 周榜数据
    pub d7_comics: Vec<Comic>,
    /// 月榜数据
    pub d30_comics: Vec<Comic>,
    /// 骑士榜数据
    pub knight_users: Vec<KnightUser>,
    /// 骑士榜是否加载中
    pub knight_loading: bool,
    /// 骑士榜错误
    pub knight_error: Option<String>,
    /// 是否正在加载（漫画排行）
    pub is_loading: bool,
    /// 错误信息（漫画排行）
    pub error: Option<String>,
}

impl Default for RankingsState {
    fn default() -> Self {
        Self {
            current_tab: RankingsTabType::Comics(RankTimeType::H24),
            current_type: RankTimeType::H24,
            h24_comics: Vec::new(),
            d7_comics: Vec::new(),
            d30_comics: Vec::new(),
            knight_users: Vec::new(),
            knight_loading: false,
            knight_error: None,
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

    /// 检查骑士榜是否已加载
    pub fn is_knight_loaded(&self) -> bool {
        !self.knight_users.is_empty()
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

/// 签到状态
#[derive(Resource, Default)]
pub struct PunchInState {
    /// 是否已签到
    pub is_punched: bool,
    /// 签到结果消息
    pub message: Option<String>,
    /// 是否签到成功
    pub is_success: bool,
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
    /// 单页模式（翻页）
    SinglePage,
    /// Webtoon 模式（条漫，垂直无限滚动）
    #[default]
    Webtoon,
}

/// 阅读器状态
#[derive(Resource)]
pub struct ReaderState {
    /// 漫画 ID
    pub comic_id: String,
    /// 漫画标题（本地文件查找用）
    pub comic_title: String,
    /// 所有章节列表（从 ComicDetailState 复制）
    pub episodes: Vec<Episode>,
    /// 当前章节在 episodes 中的索引
    pub current_episode_idx: usize,
    /// 当前章节 order
    pub episode_order: i32,
    /// 当前页码（0-indexed，条漫模式下为全局页码）
    pub current_page: usize,
    /// 总图片数
    pub total_pages: usize,
    /// 图片列表（条漫模式：所有章节扁平化；单页模式：当前章节）
    pub pictures: Vec<Picture>,
    /// 条漫模式：每张图片的章节归属元数据（与 pictures 平行）
    pub page_metas: Vec<crate::events::WebtoonPageMeta>,
    /// 缩放比例 (1.0 = 100%)
    pub scale: f32,
    /// 阅读模式
    pub read_mode: ReadMode,
    /// 条漫滚动锚点：(锚定页, 页内偏移逻辑像素)
    ///
    /// **滚动位置的唯一真相**：每帧由锚点算出 `ScrollPosition` 写下去，
    /// 而不是反过来读 `ScrollPosition` 再去纠正。图片真实高度陆续就位时，
    /// 锚定页上方的高度变化会被这次换算自然吸收，视觉位置纹丝不动——
    /// 且**与用户是否正在滚动无关**。
    ///
    /// 旧实现相反：`ScrollPosition` 是真相、锚点当补丁，且补偿只在
    /// 「非用户滚动帧」执行——用户一路拖到底时每帧都是用户滚动帧，
    /// 补偿全被跳过，于是新图加载就错位。
    pub webtoon_anchor: (usize, f32),
    /// 条漫每页高度（逻辑像素），未测量的页用占位高度
    ///
    /// 与 `pictures` 平行。测量值只增不改（由布局实测覆盖占位值），
    /// 不做「按已测均值重估未测页」——那会让锚点上方的估算高度变动，
    /// 反而制造跳动。
    pub webtoon_page_heights: Vec<f32>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 是否正在加载下一章（单页模式跨章节切换时）
    pub is_loading_next_chapter: bool,
    /// 下一章的图片列表（单页模式预加载）
    pub next_chapter_pictures: Vec<Picture>,
    /// 条漫模式是否正在加载全章节图片列表
    pub is_loading_all_chapters: bool,
}

impl Default for ReaderState {
    fn default() -> Self {
        Self {
            webtoon_anchor: (0, 0.0),
            webtoon_page_heights: Vec::new(),
            comic_id: String::new(),
            comic_title: String::new(),
            episodes: Vec::new(),
            current_episode_idx: 0,
            episode_order: 1,
            current_page: 0,
            total_pages: 0,
            pictures: Vec::new(),
            page_metas: Vec::new(),
            scale: 1.0,
            read_mode: ReadMode::default(),
            is_loading: false,
            error: None,
            is_loading_next_chapter: false,
            next_chapter_pictures: Vec::new(),
            is_loading_all_chapters: false,
        }
    }
}

impl ReaderState {
    /// 获取下一章的 episode（如果存在）
    pub fn next_episode(&self) -> Option<&Episode> {
        if self.current_episode_idx + 1 < self.episodes.len() {
            Some(&self.episodes[self.current_episode_idx + 1])
        } else {
            None
        }
    }

    /// 获取上一章的 episode（如果存在）
    pub fn prev_episode(&self) -> Option<&Episode> {
        if self.current_episode_idx > 0 {
            Some(&self.episodes[self.current_episode_idx - 1])
        } else {
            None
        }
    }

    /// 是否为当前章节的最后一页
    pub fn is_last_page(&self) -> bool {
        self.total_pages == 0 || self.current_page >= self.total_pages.saturating_sub(1)
    }

    /// 是否为当前章节的第一页
    pub fn is_first_page(&self) -> bool {
        self.current_page == 0
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum DownloadState {
    /// 排队中（等待下载）
    #[default]
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
    /// 独立下载路径（None 时使用全局设置）
    #[serde(default)]
    pub custom_download_path: Option<String>,
    /// 独立 CBZ 打包开关（None 时使用全局设置）
    #[serde(default)]
    pub custom_auto_pack_cbz: Option<bool>,
    /// 下载/更新当时服务端 `epsCount` 的快照（None = 老记录，未知）
    ///
    /// 更新检测的基准，详见
    /// `picacg_db::models::DbDownloadTask::remote_eps_count`
    #[serde(default)]
    pub remote_eps_count: Option<i32>,
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
            custom_download_path: None,
            custom_auto_pack_cbz: None,
            remote_eps_count: None,
        }
    }

    /// 获取实际保存路径（始终返回 save_path，即 base/image/漫画名）
    pub fn effective_download_path(&self) -> &str {
        &self.save_path
    }

    /// 获取有效的 CBZ 打包开关（优先使用独立设置，回退到全局）
    pub fn effective_auto_pack_cbz(&self) -> bool {
        self.custom_auto_pack_cbz
            .unwrap_or_else(|| picacg_config::AppSettings::global().read().auto_pack_cbz)
    }

    /// 转换为数据库实体
    pub fn to_db_task(&self) -> picacg_db::models::DbDownloadTask {
        use picacg_db::models::{DbDownloadTask, DownloadStateData};

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
        db_task.custom_download_path = self.custom_download_path.clone();
        db_task.set_custom_auto_pack_cbz(self.custom_auto_pack_cbz);
        db_task.remote_eps_count = self.remote_eps_count.map(i64::from);

        db_task
    }

    /// 从数据库实体创建
    pub fn from_db_task(db_task: &picacg_db::models::DbDownloadTask) -> Self {
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
            custom_download_path: db_task.custom_download_path.clone(),
            custom_auto_pack_cbz: db_task.get_custom_auto_pack_cbz(),
            remote_eps_count: db_task.remote_eps_count.map(|v| v as i32),
        }
    }

    /// 保存元数据到数据库
    pub fn save(&self) -> Result<(), String> {
        use picacg_db::{get_pool, run_db_operation, upsert_download_task_async};

        let db_task = self.to_db_task();
        let pool = get_pool();

        // 使用 run_db_operation 自动处理运行时上下文
        run_db_operation(async move {
            upsert_download_task_async(&pool, &db_task)
                .await
                .map_err(|e| format!("保存到数据库失败: {}", e))
        })
    }

    /// 从数据库加载元数据
    pub fn load(save_path: &str) -> Result<Self, String> {
        use picacg_db::{get_all_download_tasks_async, get_pool, run_db_operation};

        let save_path_owned = save_path.to_string();
        let pool = get_pool();

        // 从数据库加载（通过 save_path 查找）
        run_db_operation(async move {
            let tasks = get_all_download_tasks_async(&pool)
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
        use picacg_db::{get_download_task_async, get_pool, run_db_operation};

        let comic_id_owned = comic_id.to_string();
        let pool = get_pool();

        run_db_operation(async move {
            let task = get_download_task_async(&pool, &comic_id_owned)
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
        use picacg_db::{delete_download_task_async, get_pool, run_db_operation};

        let comic_id = self.comic_id.clone();
        let pool = get_pool();

        // 从数据库删除
        run_db_operation(async move {
            delete_download_task_async(&pool, &comic_id)
                .await
                .map_err(|e| format!("删除下载任务失败: {}", e))
        })
    }
}

/// 自动修复被污染的 save_path
///
/// 之前错误的迁移可能把 save_path 设成了基础目录（如 `/repo/comic`）
/// 而不是完整路径（如 `/repo/comic/image/漫画名`）。
/// 修复方式：用 comic_title 重建正确的 save_path。
pub fn repair_save_path(meta: &mut DownloadTaskMeta) {
    let expected_suffix = format!(
        "image/{}",
        crate::utils::sanitize_filename(&meta.comic_title)
    );
    // 已经是正确格式就跳过
    if meta.save_path.ends_with(&expected_suffix) {
        return;
    }

    // 尝试从 save_path 推导基础目录，然后重建
    let base = if let Some(base) = meta.custom_download_path.as_deref() {
        base.to_string()
    } else {
        // save_path 本身可能就是基础目录，或者取其父级
        let p = std::path::Path::new(&meta.save_path);
        // 如果以 /image/xxx 结尾说明没被污染（但 file_name 不对）
        if p.parent()
            .and_then(|pp| pp.file_name())
            .map(|n| n == "image")
            == Some(true)
        {
            p.parent()
                .unwrap()
                .parent()
                .unwrap_or(p)
                .to_string_lossy()
                .to_string()
        } else {
            meta.save_path.clone()
        }
    };

    let new_path = std::path::Path::new(&base)
        .join("image")
        .join(crate::utils::sanitize_filename(&meta.comic_title))
        .to_string_lossy()
        .to_string();

    if new_path != meta.save_path {
        tracing::info!(
            "修复路径: {} -> {} (漫画: {})",
            meta.save_path,
            new_path,
            meta.comic_title
        );
        meta.save_path = new_path;
        // 保存修复后的路径到数据库
        let _ = meta.save();
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
            custom_download_path: self.meta.custom_download_path.clone(),
            custom_auto_pack_cbz: self.meta.custom_auto_pack_cbz,
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
    /// 独立下载路径
    pub custom_download_path: Option<String>,
    /// 独立 CBZ 打包开关
    pub custom_auto_pack_cbz: Option<bool>,
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
        // 移除同 comic_id 的旧任务，避免重复
        let comic_id = meta.comic_id.clone();
        self.fsm_tasks.retain(|t| t.meta.comic_id != comic_id);
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
        use picacg_db::{get_incomplete_download_tasks_async, get_pool, run_db_operation};

        let pool = get_pool();

        // 首先尝试从数据库加载
        let db_tasks: Vec<picacg_db::models::DbDownloadTask> =
            run_db_operation(async move { get_incomplete_download_tasks_async(&pool).await })
                .unwrap_or_default();

        for db_task in db_tasks {
            let mut meta = DownloadTaskMeta::from_db_task(&db_task);

            // 自动修复被污染的 save_path（之前错误迁移可能把路径设成了基础目录）
            repair_save_path(&mut meta);

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

// ==================== 游戏系统 ====================

/// 游戏列表状态
#[derive(Resource, Default)]
pub struct GamesState {
    /// 游戏列表
    pub games: Vec<Game>,
    /// 当前页码
    pub page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 游戏详情状态
#[derive(Resource, Default)]
pub struct GameDetailState {
    /// 游戏 ID
    pub game_id: String,
    /// 游戏详情
    pub game: Option<Game>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 网络诊断状态
#[derive(Resource, Default)]
pub struct NetworkDiagState {
    /// 是否正在测速
    pub is_testing_speed: bool,
    /// 是否正在 Ping
    pub is_testing_ping: bool,
    /// 下载速度（KB/s）
    pub download_speed: Option<f64>,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
}

// ==================== 锅贴社区 ====================

/// 锅贴社区状态
#[derive(Resource, Default)]
pub struct FriedState {
    /// 小程序列表（从 /pica-apps 获取）
    pub apps: Vec<AppInfo>,
    /// 锅贴帖子列表
    pub posts: Vec<FriedPost>,
    /// 锅贴 token（通过 PicACG token 换取）
    pub fried_token: Option<String>,
    /// 当前页码（从 0 开始的偏移量）
    pub page: i32,
    /// 总帖子数
    pub total: i32,
    /// 每页条数
    pub limit: i32,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

// ==================== 图片格式转换 ====================

/// 目标图片格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetImageFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
    Bmp,
}

impl TargetImageFormat {
    /// 显示名称
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WebP",
            Self::Bmp => "BMP",
        }
    }

    /// 文件扩展名
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Bmp => "bmp",
        }
    }

    /// 所有可选格式
    pub const ALL: [TargetImageFormat; 4] = [Self::Png, Self::Jpeg, Self::Webp, Self::Bmp];
}

/// 图片格式转换状态
#[derive(Resource, Default)]
pub struct ImageConvertState {
    /// 源目录路径
    pub source_dir: String,
    /// 目标格式
    pub target_format: TargetImageFormat,
    /// 是否正在转换
    pub is_converting: bool,
    /// 已完成数量
    pub progress: u32,
    /// 总文件数量
    pub total: u32,
    /// 错误信息
    pub error: Option<String>,
    /// 成功信息
    pub success: Option<String>,
}

/// 目录选择器异步结果（图片转换专用）
#[derive(Resource, Default)]
pub struct ImageConvertPickerResult {
    /// 异步接收通道
    pub receiver: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Option<String>>>>,
}

/// 图片转换进度异步结果
#[derive(Resource, Default)]
pub struct ImageConvertProgressResult {
    /// 异步接收通道（进度: (已完成, 总数, 错误信息)）
    pub receiver: Option<std::sync::Mutex<std::sync::mpsc::Receiver<ImageConvertProgressMsg>>>,
}

/// 图片转换进度消息
#[derive(Debug, Clone)]
pub enum ImageConvertProgressMsg {
    /// 进度更新
    Progress { done: u32, total: u32 },
    /// 转换完成
    Completed { done: u32, total: u32 },
    /// 转换出错
    Error(String),
}

// ==================== Waifu2x 超分辨率 ====================

/// Waifu2x 超分辨率状态
#[derive(Resource)]
pub struct Waifu2xState {
    /// waifu2x-ncnn-vulkan 可执行文件路径
    pub executable_path: String,
    /// 缩放倍数
    pub scale: i32,
    /// 降噪等级
    pub noise_level: i32,
    /// GPU ID
    pub gpu_id: i32,
    /// 输出格式
    pub output_format: String,
    /// 输入目录
    pub input_dir: String,
    /// 输出目录
    pub output_dir: String,
    /// 是否正在处理
    pub is_processing: bool,
    /// 已完成数量
    pub progress: u32,
    /// 总文件数量
    pub total: u32,
    /// 当前正在处理的文件名
    pub current_file: String,
    /// 错误信息
    pub error: Option<String>,
    /// 成功信息
    pub success: Option<String>,
}

impl Default for Waifu2xState {
    fn default() -> Self {
        // 从配置加载已保存的设置
        let settings = picacg_config::AppSettings::global().read();
        Self {
            executable_path: settings.waifu2x.executable_path.clone(),
            scale: settings.waifu2x.scale,
            noise_level: settings.waifu2x.noise_level,
            gpu_id: settings.waifu2x.gpu_id,
            output_format: settings.waifu2x.output_format.clone(),
            input_dir: String::new(),
            output_dir: String::new(),
            is_processing: false,
            progress: 0,
            total: 0,
            current_file: String::new(),
            error: None,
            success: None,
        }
    }
}

/// Waifu2x 目录选择器接收通道类型
type Waifu2xPickerReceiver =
    std::sync::Mutex<std::sync::mpsc::Receiver<(Option<String>, Waifu2xPickerType)>>;

/// Waifu2x 目录选择器异步结果
#[derive(Resource, Default)]
pub struct Waifu2xPickerResult {
    /// 异步接收通道（(选择的路径, 选择类型)）
    pub receiver: Option<Waifu2xPickerReceiver>,
}

/// Waifu2x 目录选择器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waifu2xPickerType {
    /// 选择可执行文件路径
    Executable,
    /// 选择输入目录
    InputDir,
    /// 选择输出目录
    OutputDir,
}

/// Waifu2x 处理进度异步结果
#[derive(Resource, Default)]
pub struct Waifu2xProgressResult {
    /// 异步接收通道
    pub receiver: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Waifu2xProgressMsg>>>,
}

/// Waifu2x 处理进度消息
#[derive(Debug, Clone)]
pub enum Waifu2xProgressMsg {
    /// 进度更新
    Progress {
        done: u32,
        total: u32,
        current_file: String,
    },
    /// 处理完成
    Completed { done: u32, total: u32 },
    /// 处理出错
    Error(String),
}

// ==================== 聊天室 ====================

/// 聊天大厅状态（房间列表）
#[derive(Resource, Default)]
pub struct ChatState {
    /// 房间列表
    pub rooms: Vec<picacg_api::endpoints::chat::ChatRoom>,
    /// 聊天服务 token
    pub chat_token: Option<String>,
    /// 聊天用户资料
    pub profile: Option<picacg_api::endpoints::chat::ChatProfile>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 聊天室状态
#[derive(Resource)]
pub struct ChatRoomState {
    /// 当前房间 ID
    pub room_id: String,
    /// 当前房间标题
    pub room_title: String,
    /// 聊天消息列表（解析后的）
    pub messages: Vec<picacg_api::endpoints::chat::ParsedChatMessage>,
    /// 在线人数
    pub online_count: u32,
    /// 是否已连接 WebSocket
    pub is_connected: bool,
    /// 是否正在连接
    pub is_connecting: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 最大消息缓存数量
    pub max_messages: usize,
    /// WebSocket 消息接收通道
    pub ws_receiver: Option<std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<String>>>,
    /// WebSocket 消息发送通道（发送给 WebSocket 写入端）
    pub ws_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    /// WebSocket 关闭信号发送端
    pub ws_close_sender: Option<tokio::sync::oneshot::Sender<()>>,
    /// 是否需要自动滚动到底部
    pub auto_scroll: bool,
    /// 需要重建 UI
    pub needs_rebuild: bool,
}

impl Default for ChatRoomState {
    fn default() -> Self {
        Self {
            room_id: String::new(),
            room_title: String::new(),
            messages: Vec::new(),
            online_count: 0,
            is_connected: false,
            is_connecting: false,
            error: None,
            max_messages: 500,
            ws_receiver: None,
            ws_sender: None,
            ws_close_sender: None,
            auto_scroll: true,
            needs_rebuild: false,
        }
    }
}

// ==================== NAS 远程存储状态 ====================

/// NAS 上传任务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NasUploadStatus {
    /// 等待中
    Waiting,
    /// 上传中
    Uploading,
    /// 已完成
    Completed,
    /// 失败
    Failed(String),
}

/// NAS 上传任务条目
#[derive(Debug, Clone)]
pub struct NasUploadTask {
    /// 漫画标题
    pub comic_title: String,
    /// 本地文件路径
    pub local_path: String,
    /// 远程目标路径
    pub remote_path: String,
    /// 上传状态
    pub status: NasUploadStatus,
    /// 已上传文件数
    pub uploaded_files: u32,
    /// 总文件数
    pub total_files: u32,
}

/// WebDAV 远程文件条目
#[derive(Debug, Clone)]
pub struct NasRemoteEntry {
    /// 文件/目录名称
    pub name: String,
    /// 完整远程路径
    pub path: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 文件大小（字节）
    pub size: u64,
}

/// NAS 页面状态
#[derive(Resource, Default)]
pub struct NasState {
    /// WebDAV 连接状态
    pub is_connected: bool,
    /// 是否正在测试连接
    pub is_testing: bool,
    /// 连接测试结果消息
    pub test_message: Option<String>,
    /// 测试是否成功
    pub test_success: bool,
    /// 是否正在上传
    pub is_uploading: bool,
    /// 上传任务列表
    pub upload_tasks: Vec<NasUploadTask>,
    /// 远程文件列表（浏览用）
    pub remote_entries: Vec<NasRemoteEntry>,
    /// 当前浏览的远程路径
    pub browse_path: String,
    /// 是否正在浏览
    pub is_browsing: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 成功消息
    pub success: Option<String>,
    /// 需要重建 UI
    pub needs_rebuild: bool,
}

// ==================== 漫画列表批量选择 ====================

/// 漫画列表的批量选择状态
///
/// 只在「漫画列表」页使用。开启选择模式后卡片点击变成勾选/取消，
/// 而不是跳转详情页；选完一次性发下载请求。
#[derive(Resource, Default)]
pub struct ComicsSelectionState {
    /// 是否处于选择模式
    pub active: bool,
    /// 已选中的漫画 ID
    pub selected: std::collections::HashSet<String>,
}

impl ComicsSelectionState {
    /// 切换一本漫画的选中态，返回切换后是否选中
    pub fn toggle(&mut self, comic_id: &str) -> bool {
        if self.selected.remove(comic_id) {
            false
        } else {
            self.selected.insert(comic_id.to_string());
            true
        }
    }

    /// 退出选择模式并清空选中项
    pub fn exit(&mut self) {
        self.active = false;
        self.selected.clear();
    }
}

// ==================== 已下载漫画索引 ====================

/// 封面下载角标状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadBadgeState {
    /// 已下载，且服务端章节数与下载当时相同
    Downloaded,
    /// 已下载，但服务端 `epsCount` 比下载当时变大（有新章节）
    UpdateAvailable,
}

/// 已下载漫画索引（漫画卡片封面角标的数据源）
///
/// 卡片建卡是高频路径，不能每张卡都查一次数据库——这里把「已完成下载」的
/// 漫画 ID 与 `epsCount` 快照缓存成内存表，建卡与刷新都只做一次哈希查找。
/// 数据来源是下载任务表，故只在启动、下载完成、删除记录三处刷新。
#[derive(Resource, Default)]
pub struct DownloadedComicsIndex {
    /// comic_id → 下载当时的 `epsCount` 快照（None = 老记录，基准未知）
    snapshots: std::collections::HashMap<String, Option<i32>>,
}

impl DownloadedComicsIndex {
    /// 判断某漫画的角标状态
    ///
    /// `remote_episodes` 是**当前**列表接口给的 `Comic::eps_count`，与下载
    /// 当时的快照比：变大 = 有新章节。
    ///
    /// ⚠️ **不能拿 `eps_count` 直接跟本地章节数比**——实测该字段与
    /// `/comics/{id}/eps` 的真实条数长期对不上，且两个方向都会偏
    /// （48↔49、46↔48、12↔15、55↔53）。它是个漂移的冗余计数，直接比会让
    /// 早已下完的漫画常年亮「有更新」。同一字段自比，系统偏差才相消。
    ///
    /// 快照缺失（老记录）或当前值为 0（接口没给）→ 只报「已下载」，不猜更新。
    #[must_use]
    pub fn badge_state(&self, comic_id: &str, remote_episodes: i32) -> Option<DownloadBadgeState> {
        let snapshot = *self.snapshots.get(comic_id)?;
        match snapshot {
            Some(base) if remote_episodes > 0 && remote_episodes > base => {
                Some(DownloadBadgeState::UpdateAvailable)
            }
            _ => Some(DownloadBadgeState::Downloaded),
        }
    }

    /// 是否已下载
    #[must_use]
    pub fn contains(&self, comic_id: &str) -> bool {
        self.snapshots.contains_key(comic_id)
    }

    /// 记录/更新一本漫画的 `epsCount` 快照
    pub fn insert(&mut self, comic_id: impl Into<String>, remote_eps_count: Option<i32>) {
        self.snapshots.insert(comic_id.into(), remote_eps_count);
    }

    /// 移除一本漫画（删除下载记录时调用）
    pub fn remove(&mut self, comic_id: &str) {
        self.snapshots.remove(comic_id);
    }

    /// 从数据库全量重建索引
    pub fn reload(&mut self) {
        use picacg_db::{get_completed_download_tasks_async, get_pool, run_db_operation};

        let pool = get_pool();
        let tasks =
            run_db_operation(async move { get_completed_download_tasks_async(&pool).await })
                .unwrap_or_default();

        self.snapshots = tasks
            .iter()
            .map(|task| {
                (
                    task.comic_id.clone(),
                    task.remote_eps_count.map(|v| v as i32),
                )
            })
            .collect();

        let with_base = self.snapshots.values().filter(|v| v.is_some()).count();
        tracing::info!(
            "已下载漫画索引重建完成: {} 本（{} 本有更新基准）",
            self.snapshots.len(),
            with_base
        );
    }
}
