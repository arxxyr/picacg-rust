use crate::error::Result;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 代理设置
    pub proxy: ProxySettings,
    /// 下载并发数
    pub download_workers: usize,
    /// HTTP 并发数
    pub http_workers: usize,
    /// 缓存路径
    pub cache_path: PathBuf,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            proxy: ProxySettings::default(),
            download_workers: 5,
            http_workers: 5,
            cache_path: PathBuf::from("cache"),
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
}
