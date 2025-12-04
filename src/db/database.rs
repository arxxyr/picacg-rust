use std::{path::PathBuf, str::FromStr};

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use sqlx::{
    Row,
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
};
use tracing::{debug, info};

use crate::{
    db::models::{
        DbBook, DbCategoryCount, DbDownloadTask, DbFavorite, DbHistory, DownloadStateData,
    },
    error::Result,
};

// 数据库单例
static DATABASE: OnceCell<RwLock<Database>> = OnceCell::new();

// 数据库专用的 tokio 运行时（用于在非异步上下文中执行数据库操作）
static DB_RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
    once_cell::sync::Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("无法创建数据库运行时")
    });

/// 获取数据库专用运行时
pub fn db_runtime() -> &'static tokio::runtime::Runtime {
    &DB_RUNTIME
}

/// 在适当的运行时上下文中执行异步操作
///
/// 如果当前已经在 tokio 运行时中（例如 bevy-tokio-tasks 的工作线程），
/// 则使用 `spawn_blocking` 来避免嵌套 `block_on` 的问题。
/// 否则使用专用的数据库运行时。
pub fn run_db_operation<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    // 检查是否在 tokio 运行时中
    if let Ok(_handle) = tokio::runtime::Handle::try_current() {
        // 已经在 tokio 运行时中，使用 spawn_blocking + block_on 来执行
        // 这样可以避免 "Cannot start a runtime from within a runtime" 错误
        std::thread::scope(|s| {
            s.spawn(|| {
                // 在新线程中使用 db_runtime
                db_runtime().block_on(future)
            })
            .join()
            .expect("数据库操作线程 panic")
        })
    } else {
        // 不在 tokio 运行时中，直接使用 db_runtime
        db_runtime().block_on(future)
    }
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 初始化数据库（单例模式）
    pub async fn init(db_path: PathBuf) -> Result<()> {
        let database = Self::new(db_path).await?;
        DATABASE
            .set(RwLock::new(database))
            .map_err(|_| crate::error::PicacgError::DatabaseError("数据库已初始化".to_string()))?;
        Ok(())
    }

    /// 获取全局数据库实例
    pub fn global() -> &'static RwLock<Database> {
        DATABASE
            .get()
            .expect("数据库未初始化，请先调用 Database::init()")
    }

    /// 创建新的数据库连接
    async fn new(db_path: PathBuf) -> Result<Self> {
        // 确保数据库目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("正在连接数据库: {:?}", db_path);

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // 运行迁移
        info!("正在运行数据库迁移...");
        sqlx::query(include_str!(
            "../../migrations/20250104000001_initial_schema.sql"
        ))
        .execute(&pool)
        .await?;

        // 运行下载任务表迁移
        sqlx::query(include_str!(
            "../../migrations/20250104000002_download_tasks.sql"
        ))
        .execute(&pool)
        .await?;

        // 运行下载任务字段迁移（添加 categories, tags, completed_episodes 列）
        // 使用单独的语句，忽略 "duplicate column" 错误（列已存在的情况）
        for sql in [
            "ALTER TABLE download_task ADD COLUMN categories TEXT",
            "ALTER TABLE download_task ADD COLUMN tags TEXT",
            "ALTER TABLE download_task ADD COLUMN completed_episodes TEXT",
        ] {
            match sqlx::query(sql).execute(&pool).await {
                Ok(_) => debug!("迁移成功: {}", sql),
                Err(e) => {
                    let err_str = e.to_string();
                    // SQLite 错误：duplicate column name
                    if err_str.contains("duplicate column") {
                        debug!("列已存在，跳过: {}", sql);
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }

        info!("数据库初始化完成");

        Ok(Self { pool })
    }

    // ==================== 漫画相关操作 ====================

    /// 插入或更新漫画
    pub async fn upsert_book(&self, book: &DbBook) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO book (
                id, title, title2, author, chinese_team, description,
                eps_count, pages, finished, categories, tags, likes_count,
                created_at, updated_at, path, file_server, original_name,
                creator, total_likes, total_views
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                title2 = excluded.title2,
                author = excluded.author,
                chinese_team = excluded.chinese_team,
                description = excluded.description,
                eps_count = excluded.eps_count,
                pages = excluded.pages,
                finished = excluded.finished,
                categories = excluded.categories,
                tags = excluded.tags,
                likes_count = excluded.likes_count,
                updated_at = excluded.updated_at,
                path = excluded.path,
                file_server = excluded.file_server,
                original_name = excluded.original_name,
                creator = excluded.creator,
                total_likes = excluded.total_likes,
                total_views = excluded.total_views
            "#,
        )
        .bind(&book.id)
        .bind(&book.title)
        .bind(&book.title2)
        .bind(&book.author)
        .bind(&book.chinese_team)
        .bind(&book.description)
        .bind(book.eps_count)
        .bind(book.pages)
        .bind(book.finished)
        .bind(&book.categories)
        .bind(&book.tags)
        .bind(book.likes_count)
        .bind(book.created_at)
        .bind(book.updated_at)
        .bind(&book.path)
        .bind(&book.file_server)
        .bind(&book.original_name)
        .bind(&book.creator)
        .bind(book.total_likes)
        .bind(book.total_views)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据 ID 查询漫画
    pub async fn get_book_by_id(&self, id: &str) -> Result<Option<DbBook>> {
        let book = sqlx::query_as::<_, DbBook>("SELECT * FROM book WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(book)
    }

    /// 搜索漫画（按标题）
    pub async fn search_books(&self, keyword: &str, limit: i64) -> Result<Vec<DbBook>> {
        let pattern = format!("%{}%", keyword);
        let books = sqlx::query_as::<_, DbBook>(
            "SELECT * FROM book WHERE title LIKE ? OR title2 LIKE ? ORDER BY updated_at DESC LIMIT ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(books)
    }

    /// 批量插入漫画
    pub async fn batch_upsert_books(&self, books: &[DbBook]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for book in books {
            sqlx::query(
                r#"
                INSERT INTO book (
                    id, title, title2, author, chinese_team, description,
                    eps_count, pages, finished, categories, tags, likes_count,
                    created_at, updated_at, path, file_server, original_name,
                    creator, total_likes, total_views
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    updated_at = excluded.updated_at
                "#,
            )
            .bind(&book.id)
            .bind(&book.title)
            .bind(&book.title2)
            .bind(&book.author)
            .bind(&book.chinese_team)
            .bind(&book.description)
            .bind(book.eps_count)
            .bind(book.pages)
            .bind(book.finished)
            .bind(&book.categories)
            .bind(&book.tags)
            .bind(book.likes_count)
            .bind(book.created_at)
            .bind(book.updated_at)
            .bind(&book.path)
            .bind(&book.file_server)
            .bind(&book.original_name)
            .bind(&book.creator)
            .bind(book.total_likes)
            .bind(book.total_views)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // ==================== 收藏相关操作 ====================

    /// 添加收藏
    pub async fn add_favorite(&self, book_id: &str) -> Result<()> {
        let favorite = DbFavorite::new(book_id.to_string());
        sqlx::query(
            "INSERT INTO favorite (book_id, added_at) VALUES (?, ?) ON CONFLICT(book_id) DO NOTHING",
        )
        .bind(&favorite.book_id)
        .bind(favorite.added_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 移除收藏
    pub async fn remove_favorite(&self, book_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM favorite WHERE book_id = ?")
            .bind(book_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 检查是否已收藏
    pub async fn is_favorited(&self, book_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM favorite WHERE book_id = ?")
            .bind(book_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count > 0)
    }

    /// 获取所有收藏
    pub async fn get_favorites(&self) -> Result<Vec<DbFavorite>> {
        let favorites =
            sqlx::query_as::<_, DbFavorite>("SELECT * FROM favorite ORDER BY added_at DESC")
                .fetch_all(&self.pool)
                .await?;

        Ok(favorites)
    }

    // ==================== 历史相关操作 ====================

    /// 更新浏览历史
    pub async fn update_history(&self, history: &DbHistory) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO history (book_id, last_read, last_eps, last_page)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(book_id) DO UPDATE SET
                last_read = excluded.last_read,
                last_eps = excluded.last_eps,
                last_page = excluded.last_page
            "#,
        )
        .bind(&history.book_id)
        .bind(history.last_read)
        .bind(history.last_eps)
        .bind(history.last_page)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取浏览历史
    pub async fn get_history(&self, book_id: &str) -> Result<Option<DbHistory>> {
        let history = sqlx::query_as::<_, DbHistory>("SELECT * FROM history WHERE book_id = ?")
            .bind(book_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(history)
    }

    /// 获取所有浏览历史
    pub async fn get_all_histories(&self) -> Result<Vec<DbHistory>> {
        let histories =
            sqlx::query_as::<_, DbHistory>("SELECT * FROM history ORDER BY last_read DESC")
                .fetch_all(&self.pool)
                .await?;

        Ok(histories)
    }

    // ==================== 分类计数相关操作 ====================

    /// 更新分类计数
    pub async fn update_category_count(&self, category: &str, count: i64) -> Result<()> {
        sqlx::query(
            "INSERT INTO category_count (category, count) VALUES (?, ?) ON CONFLICT(category) DO UPDATE SET count = excluded.count",
        )
        .bind(category)
        .bind(count)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取分类计数
    pub async fn get_category_count(&self, category: &str) -> Result<Option<i64>> {
        let count: Option<i64> =
            sqlx::query_scalar("SELECT count FROM category_count WHERE category = ?")
                .bind(category)
                .fetch_optional(&self.pool)
                .await?;

        Ok(count)
    }

    /// 获取所有分类计数
    pub async fn get_all_category_counts(&self) -> Result<Vec<DbCategoryCount>> {
        let counts =
            sqlx::query_as::<_, DbCategoryCount>("SELECT * FROM category_count ORDER BY category")
                .fetch_all(&self.pool)
                .await?;

        Ok(counts)
    }

    // ==================== 统计相关操作 ====================

    /// 获取漫画总数
    pub async fn get_book_count(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM book")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    /// 获取收藏总数
    pub async fn get_favorite_count(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM favorite")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    // ==================== 下载任务相关操作 ====================

    /// 插入或更新下载任务
    pub async fn upsert_download_task(&self, task: &DbDownloadTask) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO download_task (
                comic_id, comic_title, total_episodes, episode_orders,
                save_path, state, state_data, created_at, updated_at,
                categories, tags, completed_episodes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(comic_id) DO UPDATE SET
                comic_title = excluded.comic_title,
                total_episodes = excluded.total_episodes,
                episode_orders = excluded.episode_orders,
                save_path = excluded.save_path,
                state = excluded.state,
                state_data = excluded.state_data,
                updated_at = excluded.updated_at,
                categories = excluded.categories,
                tags = excluded.tags,
                completed_episodes = excluded.completed_episodes
            "#,
        )
        .bind(&task.comic_id)
        .bind(&task.comic_title)
        .bind(task.total_episodes)
        .bind(&task.episode_orders)
        .bind(&task.save_path)
        .bind(&task.state)
        .bind(&task.state_data)
        .bind(task.created_at)
        .bind(task.updated_at)
        .bind(&task.categories)
        .bind(&task.tags)
        .bind(&task.completed_episodes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新下载任务状态
    pub async fn update_download_task_state(
        &self,
        comic_id: &str,
        state: &str,
        state_data: Option<&DownloadStateData>,
    ) -> Result<()> {
        let state_data_json = state_data.and_then(|d| serde_json::to_string(d).ok());
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "UPDATE download_task SET state = ?, state_data = ?, updated_at = ? WHERE comic_id = ?",
        )
        .bind(state)
        .bind(&state_data_json)
        .bind(now)
        .bind(comic_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 添加已完成的章节
    pub async fn add_completed_episode(&self, comic_id: &str, episode: i32) -> Result<()> {
        // 先获取当前的已完成章节列表
        let task = self.get_download_task(comic_id).await?;
        let mut completed = task
            .as_ref()
            .map(|t| t.get_completed_episodes())
            .unwrap_or_default();

        // 添加新章节（如果不存在）
        if !completed.contains(&episode) {
            completed.push(episode);
            completed.sort();
        }

        let completed_json = serde_json::to_string(&completed).ok();
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "UPDATE download_task SET completed_episodes = ?, updated_at = ? WHERE comic_id = ?",
        )
        .bind(&completed_json)
        .bind(now)
        .bind(comic_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 检查章节是否已完成
    pub async fn is_episode_completed(&self, comic_id: &str, episode: i32) -> Result<bool> {
        let task = self.get_download_task(comic_id).await?;
        Ok(task
            .map(|t| t.get_completed_episodes().contains(&episode))
            .unwrap_or(false))
    }

    /// 获取下载任务
    pub async fn get_download_task(&self, comic_id: &str) -> Result<Option<DbDownloadTask>> {
        let task =
            sqlx::query_as::<_, DbDownloadTask>("SELECT * FROM download_task WHERE comic_id = ?")
                .bind(comic_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(task)
    }

    /// 获取所有未完成的下载任务
    pub async fn get_incomplete_download_tasks(&self) -> Result<Vec<DbDownloadTask>> {
        let tasks = sqlx::query_as::<_, DbDownloadTask>(
            "SELECT * FROM download_task WHERE state != 'Completed' ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    /// 获取所有已完成的下载任务
    pub async fn get_completed_download_tasks(&self) -> Result<Vec<DbDownloadTask>> {
        let tasks = sqlx::query_as::<_, DbDownloadTask>(
            "SELECT * FROM download_task WHERE state = 'Completed' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    /// 获取所有下载任务
    pub async fn get_all_download_tasks(&self) -> Result<Vec<DbDownloadTask>> {
        let tasks = sqlx::query_as::<_, DbDownloadTask>(
            "SELECT * FROM download_task ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    /// 删除下载任务
    pub async fn delete_download_task(&self, comic_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM download_task WHERE comic_id = ?")
            .bind(comic_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 检查下载任务是否存在
    pub async fn download_task_exists(&self, comic_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM download_task WHERE comic_id = ?")
                .bind(comic_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }
}
