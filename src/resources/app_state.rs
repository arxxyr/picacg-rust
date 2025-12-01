//! 应用状态资源
//!
//! 定义应用的全局状态

use bevy::prelude::*;

use crate::{
    api::models::{Category, Comic, Episode, Picture},
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
