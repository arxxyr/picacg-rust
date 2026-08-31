//! PicACG API 客户端库
//!
//! 与 PicACG 后端 API 通信的客户端实现

#![allow(unused_imports)]

pub mod channel;
pub mod client;
pub mod endpoints;
pub mod models;
pub mod signer;

pub use channel::{
    ChannelRoute, apply_api_dns_override, apply_image_dns_override, resolve_api_route,
    transform_image_url,
};
pub use client::{API_BASE_URL, ApiClient, ApiRequest, ApiResponse};
pub use models::*;
// 重新导出错误类型
pub use picacg_core::{PicacgError, Result};
pub use signer::Signer;
