//! PicACG 数据库层
//!
//! 本地数据库和缓存管理

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod cache;
pub mod database;
pub mod models;

pub use cache::{CACHE_MANAGER, CacheManager, CacheStats};
pub use database::{
    Database,
    // 独立异步函数（避免跨 await 持有锁）
    add_completed_episode_async,
    clear_all_history_async,
    db_runtime,
    delete_download_task_async,
    delete_history_async,
    delete_like_record_async,
    get_all_download_tasks_async,
    get_all_histories_async,
    get_all_like_records_async,
    get_all_unique_tags_async,
    get_completed_download_tasks_async,
    get_download_task_async,
    get_history_count_async,
    get_incomplete_download_tasks_async,
    get_like_count_async,
    get_pool,
    insert_like_record_async,
    run_db_operation,
    upsert_download_task_async,
    upsert_history_async,
};
pub use models::{
    DbBook, DbCategoryCount, DbDownloadTask, DbFavorite, DbHistory, DbLikeRecord, DownloadStateData,
};
// 重新导出
pub use picacg_core::{PicacgError, Result};
pub use sqlx::sqlite::SqlitePool;
