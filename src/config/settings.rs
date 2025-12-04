use std::{fs, path::PathBuf, sync::Arc};

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{EnvFilter, Registry, reload::Handle};

use crate::error::Result;

/// 日志等级 reload handle 类型
pub type LogLevelHandle = Handle<EnvFilter, Registry>;

/// 全局日志等级 reload handle
static LOG_LEVEL_HANDLE: OnceCell<Arc<LogLevelHandle>> = OnceCell::new();

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    /// HTTP 代理
    Http,
    /// HTTPS 代理
    Https,
    /// SOCKS5 代理
    Socks5,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Http
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
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
    /// 对于 trace/debug 级别，只对 picacg crate 启用，第三方库保持 warn 级别
    pub fn as_str(&self) -> &'static str {
        match self {
            // trace/debug 只对我们的代码生效，减少第三方库噪音
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
    /// 最大同时下载漫画数量（默认 3）
    #[serde(default = "default_max_concurrent_downloads")]
    pub max_concurrent_downloads: usize,
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
            download_path: String::new(), // 空字符串表示使用默认路径
            database_path: String::new(), // 空字符串表示使用默认路径
            auto_resume_downloads: false, // 默认不自动恢复下载
            max_concurrent_downloads: 3,  // 默认最多同时下载 3 本漫画
        }
    }
}

impl AppSettings {
    /// 获取全局单例
    pub fn global() -> &'static RwLock<AppSettings> {
        static INSTANCE: OnceCell<RwLock<AppSettings>> = OnceCell::new();
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
}
