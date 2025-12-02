//! 数据库模块
//!
//! 本地数据库和缓存管理，部分功能预留

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod cache;
pub mod database;
pub mod models;

pub use cache::{CacheManager, CacheStats};
pub use database::Database;
pub use models::{DbBook, DbCategoryCount, DbFavorite, DbHistory};
