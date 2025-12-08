//! API 端点定义

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
