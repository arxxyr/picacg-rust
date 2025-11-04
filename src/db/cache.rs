use std::{sync::Arc, time::Duration};

use moka::future::Cache;
use once_cell::sync::Lazy;

use crate::{
    api::models::{Comic, User},
    db::models::DbBook,
};

// 全局缓存实例
pub static CACHE_MANAGER: Lazy<CacheManager> = Lazy::new(CacheManager::new);

/// 缓存管理器
pub struct CacheManager {
    /// 漫画信息缓存（API 返回的 Comic）
    comic_cache: Cache<String, Arc<Comic>>,

    /// 用户信息缓存
    user_cache: Cache<String, Arc<User>>,

    /// 漫画数据库实体缓存
    db_book_cache: Cache<String, Arc<DbBook>>,

    /// 图片 URL 缓存（图片 ID -> 完整 URL）
    image_url_cache: Cache<String, String>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            // 漫画缓存: 最多 1000 个，30 分钟过期
            comic_cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(30 * 60))
                .build(),

            // 用户缓存: 最多 100 个，10 分钟过期
            user_cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(10 * 60))
                .build(),

            // 数据库实体缓存: 最多 5000 个，1 小时过期
            db_book_cache: Cache::builder()
                .max_capacity(5000)
                .time_to_live(Duration::from_secs(60 * 60))
                .build(),

            // 图片 URL 缓存: 最多 10000 个，2 小时过期
            image_url_cache: Cache::builder()
                .max_capacity(10000)
                .time_to_live(Duration::from_secs(2 * 60 * 60))
                .build(),
        }
    }

    /// 获取全局缓存管理器实例
    pub fn global() -> &'static CacheManager {
        &CACHE_MANAGER
    }

    // ==================== 漫画缓存 ====================

    /// 缓存漫画信息
    pub async fn set_comic(&self, id: String, comic: Comic) {
        self.comic_cache.insert(id, Arc::new(comic)).await;
    }

    /// 获取缓存的漫画信息
    pub async fn get_comic(&self, id: &str) -> Option<Arc<Comic>> {
        self.comic_cache.get(id).await
    }

    /// 移除漫画缓存
    pub async fn remove_comic(&self, id: &str) {
        self.comic_cache.invalidate(id).await;
    }

    // ==================== 用户缓存 ====================

    /// 缓存用户信息
    pub async fn set_user(&self, id: String, user: User) {
        self.user_cache.insert(id, Arc::new(user)).await;
    }

    /// 获取缓存的用户信息
    pub async fn get_user(&self, id: &str) -> Option<Arc<User>> {
        self.user_cache.get(id).await
    }

    /// 移除用户缓存
    pub async fn remove_user(&self, id: &str) {
        self.user_cache.invalidate(id).await;
    }

    // ==================== 数据库实体缓存 ====================

    /// 缓存数据库漫画实体
    pub async fn set_db_book(&self, id: String, book: DbBook) {
        self.db_book_cache.insert(id, Arc::new(book)).await;
    }

    /// 获取缓存的数据库漫画实体
    pub async fn get_db_book(&self, id: &str) -> Option<Arc<DbBook>> {
        self.db_book_cache.get(id).await
    }

    /// 移除数据库实体缓存
    pub async fn remove_db_book(&self, id: &str) {
        self.db_book_cache.invalidate(id).await;
    }

    // ==================== 图片 URL 缓存 ====================

    /// 缓存图片 URL
    pub async fn set_image_url(&self, image_id: String, url: String) {
        self.image_url_cache.insert(image_id, url).await;
    }

    /// 获取缓存的图片 URL
    pub async fn get_image_url(&self, image_id: &str) -> Option<String> {
        self.image_url_cache.get(image_id).await
    }

    /// 移除图片 URL 缓存
    pub async fn remove_image_url(&self, image_id: &str) {
        self.image_url_cache.invalidate(image_id).await;
    }

    // ==================== 批量操作 ====================

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        self.comic_cache.invalidate_all();
        self.user_cache.invalidate_all();
        self.db_book_cache.invalidate_all();
        self.image_url_cache.invalidate_all();
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            comic_count: self.comic_cache.entry_count(),
            user_count: self.user_cache.entry_count(),
            db_book_count: self.db_book_cache.entry_count(),
            image_url_count: self.image_url_cache.entry_count(),
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub comic_count: u64,
    pub user_count: u64,
    pub db_book_count: u64,
    pub image_url_count: u64,
}

impl CacheStats {
    pub fn total_count(&self) -> u64 {
        self.comic_count + self.user_count + self.db_book_count + self.image_url_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_manager() {
        let cache = CacheManager::new();

        // 测试图片 URL 缓存
        cache
            .set_image_url(
                "img1".to_string(),
                "http://example.com/img1.jpg".to_string(),
            )
            .await;

        let url = cache.get_image_url("img1").await;
        assert_eq!(url, Some("http://example.com/img1.jpg".to_string()));

        // 测试移除
        cache.remove_image_url("img1").await;
        let url = cache.get_image_url("img1").await;
        assert_eq!(url, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = CacheManager::new();

        cache
            .set_image_url("img1".to_string(), "url1".to_string())
            .await;
        cache
            .set_image_url("img2".to_string(), "url2".to_string())
            .await;

        let stats = cache.stats().await;
        assert_eq!(stats.image_url_count, 2);
    }
}
