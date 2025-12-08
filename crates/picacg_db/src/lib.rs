//! PicACG 数据库层
//!
//! 本地数据库和缓存管理

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod cache;
pub mod database;
pub mod models;

pub use cache::{CACHE_MANAGER, CacheManager, CacheStats};
pub use database::{Database, db_runtime, run_db_operation};
pub use models::{
    DbBook, DbCategoryCount, DbDownloadTask, DbFavorite, DbHistory, DownloadStateData,
};
// 重新导出
pub use picacg_core::{PicacgError, Result};
