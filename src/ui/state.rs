use crate::{
    api::models::{Category, Comic},
    config::settings::{AppSettings, ProxyType},
    download::DownloadTask,
    ui::image_loader::ImageCache,
};

/// 路由枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// 登录页面
    Login,
    /// 代理设置（登录前）
    ProxySettings,
    /// 主页
    Home,
    /// 分类浏览
    Categories,
    /// 漫画列表（按分类）
    ComicsList(String), // category
    /// 搜索
    Search,
    /// 收藏
    Favorites,
    /// 下载管理
    Downloads,
    /// 设置
    Settings,
    /// 漫画详情
    ComicDetail(String), // comic_id
    /// 阅读界面
    ReadView {
        comic_id: String,
        episode_order: i32,
    },
}

/// 登录界面焦点位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginFocus {
    /// 用户名输入框
    Username,
    /// 密码输入框
    Password,
    /// 登录按钮
    LoginButton,
    /// 代理设置按钮
    ProxyButton,
}

/// 登录状态
#[derive(Debug, Clone)]
pub struct LoginState {
    /// 用户名
    pub email: String,
    /// 密码
    pub password: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 当前焦点位置
    pub focus: LoginFocus,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            is_loading: false,
            error: None,
            focus: LoginFocus::Username,
        }
    }
}

/// 分类列表状态
#[derive(Debug, Clone, Default)]
pub struct CategoriesState {
    /// 分类列表
    pub categories: Vec<Category>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 分类缩略图 (URL -> Handle)
    pub thumbnails: std::collections::HashMap<String, iced::widget::image::Handle>,
}

/// 漫画列表状态
#[derive(Debug, Clone)]
pub struct ComicsListState {
    /// 当前分类
    pub category: String,
    /// 漫画列表
    pub comics: Vec<Comic>,
    /// 当前页码
    pub page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 排序方式 (dd, da, ld, vd)
    pub sort: String,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 漫画缩略图 (URL -> Handle)
    pub thumbnails: std::collections::HashMap<String, iced::widget::image::Handle>,
}

impl Default for ComicsListState {
    fn default() -> Self {
        Self {
            category: String::new(),
            comics: Vec::new(),
            page: 1,
            total_pages: 1,
            sort: "dd".to_string(), // 默认按更新时间降序
            is_loading: false,
            error: None,
            thumbnails: std::collections::HashMap::new(),
        }
    }
}

/// 漫画详情状态
#[derive(Debug, Clone)]
pub struct ComicDetailState {
    /// 当前漫画 ID
    pub comic_id: String,
    /// 漫画信息
    pub comic: Option<Comic>,
    /// 封面图片
    pub cover_image: Option<iced::widget::image::Handle>,
    /// 章节列表
    pub episodes: Vec<crate::api::models::Episode>,
    /// 章节当前页码
    pub episodes_page: i32,
    /// 章节总页数
    pub episodes_total_pages: i32,
    /// 是否正在加载章节
    pub is_loading_episodes: bool,
    /// 是否正在加载详情
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 是否收藏
    pub is_favorite: bool,
    /// 是否点赞
    pub is_liked: bool,
}

impl ComicDetailState {
    pub fn new(comic_id: String) -> Self {
        Self {
            comic_id,
            comic: None,
            cover_image: None,
            episodes: Vec::new(),
            episodes_page: 1,
            episodes_total_pages: 1,
            is_loading_episodes: false,
            is_loading: false,
            error: None,
            is_favorite: false,
            is_liked: false,
        }
    }
}

/// 阅读界面状态
#[derive(Debug, Clone)]
pub struct ReadViewState {
    /// 当前漫画 ID
    pub comic_id: String,
    /// 当前章节顺序
    pub episode_order: i32,
    /// 当前页码
    pub current_page: i32,
    /// 总页数
    pub total_pages: i32,
    /// 图片列表
    pub pictures: Vec<crate::api::models::Picture>,
    /// 当前显示的图片 Handle
    pub current_image: Option<iced::widget::image::Handle>,
    /// 图片缓存 (page -> Handle)
    pub image_cache: std::collections::HashMap<i32, iced::widget::image::Handle>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 缩放比例
    pub scale: f32,
    /// 阅读模式（单页/双页等）
    pub read_mode: ReadMode,
}

/// 阅读模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadMode {
    /// 单页模式
    SinglePage,
    /// 双页模式
    DoublePage,
    /// 连续滚动
    Scroll,
}

impl ReadViewState {
    pub fn new(comic_id: String, episode_order: i32) -> Self {
        Self {
            comic_id,
            episode_order,
            current_page: 1,
            total_pages: 0,
            pictures: Vec::new(),
            current_image: None,
            image_cache: std::collections::HashMap::new(),
            is_loading: false,
            error: None,
            scale: 1.0,
            read_mode: ReadMode::SinglePage,
        }
    }
}

/// 下载管理状态
#[derive(Debug, Clone, Default)]
pub struct DownloadsState {
    /// 下载任务列表
    pub tasks: Vec<DownloadTask>,
    /// 是否显示详情
    pub show_details: bool,
}

/// 代理设置状态
#[derive(Debug, Clone)]
pub struct ProxySettingsState {
    /// 是否启用代理
    pub enabled: bool,
    /// 代理类型
    pub proxy_type: ProxyType,
    /// 代理主机
    pub host: String,
    /// 代理端口
    pub port: String,
    /// 是否使用认证
    pub use_auth: bool,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否正在测试
    pub is_testing: bool,
    /// 测试结果消息
    pub test_message: Option<String>,
}

impl Default for ProxySettingsState {
    fn default() -> Self {
        // 从全局配置加载
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

/// 应用主状态
#[derive(Debug, Clone)]
pub struct AppState {
    /// 当前路由
    pub route: Route,
    /// 认证 token
    pub token: Option<String>,
    /// 登录状态
    pub login_state: LoginState,
    /// 分类列表状态
    pub categories_state: CategoriesState,
    /// 漫画列表状态
    pub comics_list_state: ComicsListState,
    /// 漫画详情状态
    pub comic_detail_state: Option<ComicDetailState>,
    /// 阅读界面状态
    pub read_view_state: Option<ReadViewState>,
    /// 下载管理状态
    pub downloads_state: DownloadsState,
    /// 代理设置状态
    pub proxy_settings_state: ProxySettingsState,
    /// 图片缓存
    pub image_cache: ImageCache,
    /// 全局错误信息
    pub global_error: Option<String>,
    /// 全局成功信息
    pub global_success: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            route: Route::Login,
            token: None,
            login_state: LoginState::default(),
            categories_state: CategoriesState::default(),
            comics_list_state: ComicsListState::default(),
            comic_detail_state: None,
            read_view_state: None,
            downloads_state: DownloadsState::default(),
            proxy_settings_state: ProxySettingsState::default(),
            image_cache: ImageCache::new(),
            global_error: None,
            global_success: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 切换路由
    pub fn navigate_to(&mut self, route: Route) {
        self.route = route;
    }

    /// 判断是否已登录
    pub fn is_logged_in(&self) -> bool {
        self.token.is_some()
    }

    /// 设置 token
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// 清除 token
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// 设置错误消息
    pub fn set_error(&mut self, error: String) {
        self.global_error = Some(error);
        self.global_success = None;
    }

    /// 设置成功消息
    pub fn set_success(&mut self, success: String) {
        self.global_success = Some(success);
        self.global_error = None;
    }

    /// 清除所有消息
    pub fn clear_messages(&mut self) {
        self.global_error = None;
        self.global_success = None;
    }
}
