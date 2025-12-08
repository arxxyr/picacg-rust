//! 错误类型定义

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PicacgError {
    // 网络错误
    #[error("网络连接失败: {0}")]
    NetworkError(String),

    #[error("HTTP 请求失败: {status}, {message}")]
    HttpError { status: u16, message: String },

    #[error("连接超时")]
    Timeout,

    #[error("代理错误: {0}")]
    ProxyError(String),

    // API 错误
    #[error("API 错误: code={code}, message={message}")]
    ApiError {
        code: i32,
        message: String,
        error: String,
    },

    #[error("认证失败: {0}")]
    AuthError(String),

    #[error("未登录")]
    NotLoggedIn,

    // 数据错误
    #[error("序列化错误: {0}")]
    SerializationError(String),

    #[error("数据不存在: {0}")]
    NotFound(String),

    // 文件错误
    #[error("文件 I/O 错误: {0}")]
    IoError(String),

    // 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    // 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    // 业务错误
    #[error("任务已取消")]
    Cancelled,

    #[error("无效的参数: {0}")]
    InvalidArgument(String),

    #[error("内部错误: {0}")]
    InternalError(String),

    // 其他
    #[error("未知错误: {0}")]
    Unknown(String),
}

// 从标准库错误转换
impl From<std::io::Error> for PicacgError {
    fn from(err: std::io::Error) -> Self {
        PicacgError::IoError(err.to_string())
    }
}

impl From<reqwest::Error> for PicacgError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            PicacgError::Timeout
        } else if err.is_connect() {
            PicacgError::NetworkError(err.to_string())
        } else {
            PicacgError::HttpError {
                status: err.status().map(|s| s.as_u16()).unwrap_or(0),
                message: err.to_string(),
            }
        }
    }
}

impl From<serde_json::Error> for PicacgError {
    fn from(err: serde_json::Error) -> Self {
        PicacgError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for PicacgError {
    fn from(err: toml::de::Error) -> Self {
        PicacgError::SerializationError(err.to_string())
    }
}

impl From<toml::ser::Error> for PicacgError {
    fn from(err: toml::ser::Error) -> Self {
        PicacgError::SerializationError(err.to_string())
    }
}

// sqlx 错误转换
impl From<sqlx::Error> for PicacgError {
    fn from(err: sqlx::Error) -> Self {
        PicacgError::DatabaseError(err.to_string())
    }
}

// sqlx::migrate::MigrateError 转换
impl From<sqlx::migrate::MigrateError> for PicacgError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        PicacgError::DatabaseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PicacgError>;

// 状态码定义(对应 Python 的 Status 类)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok = 0,
    Error = 1,
    NetError = 2,
    TimeOut = 3,
    ConnectErr = 4,
    Cancel = 8,
}

impl From<&PicacgError> for Status {
    fn from(err: &PicacgError) -> Self {
        match err {
            PicacgError::Timeout => Status::TimeOut,
            PicacgError::NetworkError(_) => Status::NetError,
            PicacgError::HttpError { .. } => Status::NetError,
            PicacgError::Cancelled => Status::Cancel,
            _ => Status::Error,
        }
    }
}
