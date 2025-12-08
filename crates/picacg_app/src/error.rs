//! 错误类型定义
//!
//! 从 picacg_core 重新导出错误类型

#![allow(dead_code)]

// 重新导出 picacg_core 的错误类型
pub use picacg_core::{PicacgError, Result};

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
