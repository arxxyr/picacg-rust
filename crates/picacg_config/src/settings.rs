//! 应用配置

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use parking_lot::RwLock;
use picacg_core::Result;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{EnvFilter, Registry, reload::Handle};

/// 日志等级 reload handle 类型
pub type LogLevelHandle = Handle<EnvFilter, Registry>;

/// 全局日志等级 reload handle
static LOG_LEVEL_HANDLE: OnceLock<Arc<LogLevelHandle>> = OnceLock::new();

/// 设置日志等级 reload handle（程序启动时调用一次）
pub fn set_log_level_handle(handle: LogLevelHandle) {
    let _ = LOG_LEVEL_HANDLE.set(Arc::new(handle));
}

/// 获取日志等级 reload handle
pub fn get_log_level_handle() -> Option<Arc<LogLevelHandle>> {
    LOG_LEVEL_HANDLE.get().cloned()
}

/// 动态更新日志等级
pub fn update_log_level(level: LogLevel) {
    if let Some(handle) = get_log_level_handle() {
        let filter = EnvFilter::new(level.as_str());
        if let Err(e) = handle.reload(filter) {
            tracing::error!("更新日志等级失败: {}", e);
        } else {
            tracing::info!("日志等级已更新为: {}", level.as_str());
        }
    }
}

/// 代理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProxyType {
    /// HTTP 代理
    #[default]
    Http,
    /// HTTPS 代理
    Https,
    /// SOCKS5 代理
    Socks5,
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    /// 是否启用代理
    pub enabled: bool,
    /// 代理类型
    pub proxy_type: ProxyType,
    /// 代理地址（例如：127.0.0.1）
    pub host: String,
    /// 代理端口（例如：7890）
    pub port: u16,
    /// 是否需要认证
    pub use_auth: bool,
    /// 用户名（可选）
    pub username: String,
    /// 密码（可选）
    pub password: String,
}

impl Default for ProxySettings {
    fn default() -> Self {
        ProxySettings {
            enabled: false,
            proxy_type: ProxyType::Http,
            host: String::from("127.0.0.1"),
            port: 7890,
            use_auth: false,
            username: String::new(),
            password: String::new(),
        }
    }
}

impl ProxySettings {
    /// 构建代理 URL
    pub fn to_proxy_url(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let scheme = match self.proxy_type {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks5 => "socks5",
        };

        let auth = if self.use_auth && !self.username.is_empty() {
            format!("{}:{}@", self.username, self.password)
        } else {
            String::new()
        };

        Some(format!("{}://{}{}:{}", scheme, auth, self.host, self.port))
    }
}

/// 登录设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoginSettings {
    /// 是否保存密码
    pub save_password: bool,
    /// 是否自动登录
    pub auto_login: bool,
    /// 是否自动打卡
    pub auto_punch_in: bool,
    /// 保存的用户名/邮箱
    pub saved_email: String,
    /// 保存的密码（注意：明文存储，生产环境应加密）
    pub saved_password: String,
}

/// 日志等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// 转换为 tracing 的 Level
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }

    /// 转换为 EnvFilter 字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "warn,picacg=trace",
            LogLevel::Debug => "warn,picacg=debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            LogLevel::Trace => "Trace (最详细)",
            LogLevel::Debug => "Debug (调试)",
            LogLevel::Info => "Info (默认)",
            LogLevel::Warn => "Warn (警告)",
            LogLevel::Error => "Error (错误)",
        }
    }
}

/// 分流通道类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChannelType {
    /// 直连（默认）
    #[default]
    Direct,
    /// CDN IP 1 (104.21.91.145)
    CdnIp1,
    /// CDN IP 2 (188.114.98.153)
    CdnIp2,
    /// 自定义 CDN IP
    CustomCdnIp,
    /// 日本反代
    JpProxy,
    /// 美国反代
    UsProxy,
}

impl ChannelType {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ChannelType::Direct => "直连",
            ChannelType::CdnIp1 => "CDN 1",
            ChannelType::CdnIp2 => "CDN 2",
            ChannelType::CustomCdnIp => "自定义IP",
            ChannelType::JpProxy => "日本反代",
            ChannelType::UsProxy => "美国反代",
        }
    }

    /// 所有通道类型（用于 UI 渲染按钮）
    pub fn all() -> &'static [ChannelType] {
        &[
            ChannelType::Direct,
            ChannelType::CdnIp1,
            ChannelType::CdnIp2,
            ChannelType::CustomCdnIp,
            ChannelType::JpProxy,
            ChannelType::UsProxy,
        ]
    }
}

/// 分流通道设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelSettings {
    /// API 分流通道
    #[serde(default)]
    pub api_channel: ChannelType,
    /// 图片分流通道
    #[serde(default)]
    pub image_channel: ChannelType,
    /// 自定义 API CDN IP
    #[serde(default)]
    pub custom_cdn_api_ip: String,
    /// 自定义图片 CDN IP
    #[serde(default)]
    pub custom_cdn_img_ip: String,
}

/// 内容过滤设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterSettings {
    /// 屏蔽词列表
    #[serde(default)]
    pub blocked_keywords: Vec<String>,
    /// 是否按分类屏蔽
    #[serde(default)]
    pub filter_by_category: bool,
    /// 是否按标签屏蔽
    #[serde(default)]
    pub filter_by_tag: bool,
    /// 是否按标题屏蔽
    #[serde(default)]
    pub filter_by_title: bool,
}

/// 关闭行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CloseBehavior {
    /// 直接关闭（默认）
    #[default]
    Close,
    /// 最小化到任务栏
    Minimize,
    /// 每次询问（暂不弹窗，行为同关闭）
    Ask,
}

impl CloseBehavior {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            CloseBehavior::Close => "直接退出",
            CloseBehavior::Minimize => "最小化",
            CloseBehavior::Ask => "每次询问",
        }
    }

    /// 所有关闭行为（用于 UI 渲染按钮）
    pub fn all() -> &'static [CloseBehavior] {
        &[
            CloseBehavior::Close,
            CloseBehavior::Minimize,
            CloseBehavior::Ask,
        ]
    }
}

/// 界面语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    /// 简体中文（默认）
    #[default]
    ZhCN,
    /// 繁體中文
    ZhTW,
    /// English
    En,
}

impl Language {
    /// 获取显示名称（始终用对应语言的原生名称）
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::ZhCN => "简体中文",
            Language::ZhTW => "繁體中文",
            Language::En => "English",
        }
    }

    /// 所有语言选项（用于 UI 渲染按钮）
    pub fn all() -> &'static [Language] {
        &[Language::ZhCN, Language::ZhTW, Language::En]
    }
}

/// 主题模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    /// 深色主题（默认）
    #[default]
    Dark,
    /// 浅色主题
    Light,
    /// 跟随系统
    Auto,
}

impl ThemeMode {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "深色",
            ThemeMode::Light => "浅色",
            ThemeMode::Auto => "跟随系统",
        }
    }

    /// 所有主题模式（用于 UI 渲染按钮）
    pub fn all() -> &'static [ThemeMode] {
        &[ThemeMode::Dark, ThemeMode::Light, ThemeMode::Auto]
    }
}

/// Waifu2x 超分辨率设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waifu2xSettings {
    /// waifu2x-ncnn-vulkan 可执行文件路径
    #[serde(default)]
    pub executable_path: String,
    /// 缩放倍数 (1, 2, 4, 8, 16, 32)
    #[serde(default = "default_waifu2x_scale")]
    pub scale: i32,
    /// 降噪等级 (-1, 0, 1, 2, 3)
    #[serde(default)]
    pub noise_level: i32,
    /// GPU ID (-1 = CPU)
    #[serde(default)]
    pub gpu_id: i32,
    /// 输出格式 (png, jpg, webp)
    #[serde(default = "default_waifu2x_output_format")]
    pub output_format: String,
}

fn default_waifu2x_scale() -> i32 {
    2
}

fn default_waifu2x_output_format() -> String {
    "png".to_string()
}

impl Default for Waifu2xSettings {
    fn default() -> Self {
        Self {
            executable_path: String::new(),
            scale: 2,
            noise_level: 0,
            gpu_id: 0,
            output_format: "png".to_string(),
        }
    }
}

impl Waifu2xSettings {
    /// 所有支持的缩放倍数
    pub const SCALES: &'static [i32] = &[1, 2, 4, 8, 16, 32];

    /// 所有支持的降噪等级
    pub const NOISE_LEVELS: &'static [i32] = &[-1, 0, 1, 2, 3];

    /// 所有支持的 GPU ID 选项
    pub const GPU_IDS: &'static [i32] = &[-1, 0, 1];

    /// 所有支持的输出格式
    pub const OUTPUT_FORMATS: &'static [&'static str] = &["png", "jpg", "webp"];

    /// 获取降噪等级显示名称
    pub fn noise_level_display(level: i32) -> &'static str {
        match level {
            -1 => "-1 (无降噪)",
            0 => "0 (轻微)",
            1 => "1 (适中)",
            2 => "2 (强)",
            3 => "3 (最强)",
            _ => "未知",
        }
    }

    /// 获取 GPU ID 显示名称
    pub fn gpu_id_display(id: i32) -> &'static str {
        match id {
            -1 => "-1 (CPU)",
            0 => "GPU 0",
            1 => "GPU 1",
            _ => "未知",
        }
    }
}

/// NAS 远程存储设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NasSettings {
    /// WebDAV 服务器 URL（如 http://192.168.1.100:5005/webdav）
    #[serde(default)]
    pub server_url: String,
    /// 用户名
    #[serde(default)]
    pub username: String,
    /// 密码（注意：明文存储，生产环境应加密）
    #[serde(default)]
    pub password: String,
    /// 远程根目录（如 /picacg/）
    #[serde(default)]
    pub remote_path: String,
    /// 是否启用 NAS 远程存储
    #[serde(default)]
    pub enabled: bool,
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 代理设置
    pub proxy: ProxySettings,
    /// 登录设置
    #[serde(default)]
    pub login: LoginSettings,
    /// 日志等级
    #[serde(default)]
    pub log_level: LogLevel,
    /// 下载并发数
    pub download_workers: usize,
    /// HTTP 并发数
    pub http_workers: usize,
    /// 缓存路径
    pub cache_path: PathBuf,
    /// 下载保存路径（空字符串表示使用默认路径：程序目录/Downloads）
    #[serde(default)]
    pub download_path: String,
    /// 数据库路径（空字符串表示使用默认路径）
    #[serde(default)]
    pub database_path: String,
    /// 启动后自动开始未完成的下载
    #[serde(default)]
    pub auto_resume_downloads: bool,
    /// 下载队列全部完成后自动退出程序（挂机下载用）
    #[serde(default)]
    pub exit_after_downloads: bool,
    /// 启动后自动检查新版本（只查不装，发现新版本时在设置页给出下载入口）
    #[serde(default)]
    pub auto_check_update: bool,
    /// 启用系统耗时追踪（设置页开关，**重启后生效**）
    ///
    /// 为什么要重启：bevy 在系统初始化时**一次性**建好 `info_span!("system")`，
    /// 那一刻若没有订阅者感兴趣，span 就是禁用态，之后再打开开关也不会重建。
    #[serde(default)]
    pub enable_profiling: bool,
    /// 最大同时下载漫画数量（默认 3）
    #[serde(default = "default_max_concurrent_downloads")]
    pub max_concurrent_downloads: usize,
    /// 下载完成后自动打包为 CBZ 格式
    #[serde(default)]
    pub auto_pack_cbz: bool,
    /// 打包 CBZ 后删除原图文件夹
    #[serde(default)]
    pub delete_images_after_cbz: bool,
    /// 内容过滤设置
    #[serde(default)]
    pub filter: FilterSettings,
    /// 分流通道设置
    #[serde(default)]
    pub channel: ChannelSettings,
    /// 主题模式
    #[serde(default)]
    pub theme: ThemeMode,
    /// 界面语言
    #[serde(default)]
    pub language: Language,
    /// 界面缩放比例（0.0 = 自动，1.0-2.0 = 手动缩放）
    #[serde(default)]
    pub ui_scale: f32,
    /// 自定义字体路径（空字符串表示使用内置字体）
    #[serde(default)]
    pub custom_font_path: String,
    /// 是否启用 SNI 伪装（绕过 SNI 封锁）
    #[serde(default)]
    pub use_sni_pretend: bool,
    /// 是否优先使用 IPv6
    #[serde(default)]
    pub prefer_ipv6: bool,
    /// 关闭行为（点击关闭按钮时的行为）
    #[serde(default)]
    pub close_behavior: CloseBehavior,
    /// Waifu2x 超分辨率设置
    #[serde(default)]
    pub waifu2x: Waifu2xSettings,
    /// NAS 远程存储设置
    #[serde(default)]
    pub nas: NasSettings,
    /// 窗口 X 坐标（物理像素，None 表示未保存）
    #[serde(default)]
    pub window_x: Option<f32>,
    /// 窗口 Y 坐标（物理像素，None 表示未保存）
    #[serde(default)]
    pub window_y: Option<f32>,
    /// 窗口宽度（逻辑像素，None 表示使用默认值）
    #[serde(default)]
    pub window_width: Option<f32>,
    /// 窗口高度（逻辑像素，None 表示使用默认值）
    #[serde(default)]
    pub window_height: Option<f32>,
}

fn default_max_concurrent_downloads() -> usize {
    3
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            proxy: ProxySettings::default(),
            login: LoginSettings::default(),
            log_level: LogLevel::default(),
            download_workers: 5,
            http_workers: 5,
            cache_path: PathBuf::from("cache"),
            download_path: String::new(),
            database_path: String::new(),
            auto_resume_downloads: false,
            exit_after_downloads: false,
            auto_check_update: false,
            enable_profiling: false,
            max_concurrent_downloads: 3,
            auto_pack_cbz: false,
            delete_images_after_cbz: false,
            filter: FilterSettings::default(),
            channel: ChannelSettings::default(),
            theme: ThemeMode::default(),
            language: Language::default(),
            ui_scale: 0.0,
            custom_font_path: String::new(),
            use_sni_pretend: false,
            prefer_ipv6: false,
            close_behavior: CloseBehavior::default(),
            waifu2x: Waifu2xSettings::default(),
            nas: NasSettings::default(),
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
        }
    }
}

impl AppSettings {
    /// 获取全局单例
    pub fn global() -> &'static RwLock<AppSettings> {
        static INSTANCE: OnceLock<RwLock<AppSettings>> = OnceLock::new();
        INSTANCE.get_or_init(|| RwLock::new(Self::load().unwrap_or_default()))
    }

    /// 从文件加载配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        tracing::debug!("配置文件路径: {:?}", config_path);
        if !config_path.exists() {
            tracing::info!("配置文件不存在，使用默认配置");
            return Ok(Self::default());
        }

        tracing::debug!("正在加载配置文件...");
        let content = fs::read_to_string(&config_path)?;
        let settings: AppSettings = toml::from_str(&content)?;
        tracing::info!("配置加载成功");
        Ok(settings)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// 获取配置文件路径
    fn config_path() -> PathBuf {
        let mut path = directories::ProjectDirs::from("com", "picacg", "picacg")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        path.push("config.toml");
        path
    }

    /// 获取数据库文件路径
    pub fn get_database_path(&self) -> PathBuf {
        if !self.database_path.is_empty() {
            return PathBuf::from(&self.database_path);
        }

        // 默认路径：配置目录/picacg.db
        directories::ProjectDirs::from("com", "picacg", "picacg")
            .map(|dirs| dirs.data_dir().join("picacg.db"))
            .unwrap_or_else(|| PathBuf::from("picacg.db"))
    }

    /// 日志目录（与数据库同级的 `logs/`）
    ///
    /// 性能榜单等诊断产物落在这里——打包成 .app 之后没有终端，
    /// 报告必须有个能用 Finder 打开、能直接发出来的落点。
    #[must_use]
    pub fn log_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "picacg", "picacg")
            .map(|dirs| dirs.data_dir().join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs"))
    }

    /// 系统耗时榜的落盘路径
    #[must_use]
    pub fn profiling_log_path() -> PathBuf {
        Self::log_dir().join("profiling.log")
    }
}
