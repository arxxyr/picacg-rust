//! API 端点定义
//!
//! 各种 API 请求/响应结构，部分功能预留

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod auth;
pub mod category;
pub mod comic;
pub mod comment;
pub mod game;
pub mod rank;

pub use auth::*;
pub use category::*;
pub use comic::*;
pub use comment::*;
pub use game::*;
pub use rank::*;
