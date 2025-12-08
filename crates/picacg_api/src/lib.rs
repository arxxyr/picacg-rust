//! PicACG API 客户端库
//!
//! 与 PicACG 后端 API 通信的客户端实现

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod client;
pub mod endpoints;
pub mod models;
pub mod signer;

pub use client::{API_BASE_URL, ApiClient, ApiRequest, ApiResponse};
pub use models::*;
// 重新导出错误类型
pub use picacg_core::{PicacgError, Result};
pub use signer::Signer;
