use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use reqwest::Client;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::{Semaphore, mpsc},
    time::sleep,
};
use tracing::{debug, error, info, warn};

use crate::{
    download::task::{DownloadEvent, DownloadHandle, DownloadStatus, DownloadTask},
    error::{PicacgError, Result},
};

/// 全局下载管理器实例
pub static DOWNLOAD_MANAGER: Lazy<DownloadManager> = Lazy::new(DownloadManager::new);

/// 下载管理器配置
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// 最大并发下载数
    pub max_concurrent: usize,
    /// 下载超时时间（秒）
    pub timeout_secs: u64,
    /// 每个任务的最大重试次数
    pub max_retries: u32,
    /// 进度更新间隔（毫秒）
    pub progress_interval_ms: u64,
    /// 缓冲区大小（字节）
    pub buffer_size: usize,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            timeout_secs: 60,
            max_retries: 3,
            progress_interval_ms: 500,
            buffer_size: 8192,
        }
    }
}

/// 下载管理器
pub struct DownloadManager {
    /// HTTP 客户端
    client: Client,
    /// 任务映射表
    tasks: Arc<RwLock<HashMap<u64, DownloadTask>>>,
    /// 任务句柄映射表
    handles: Arc<RwLock<HashMap<u64, DownloadHandle>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<DownloadEvent>,
    /// 事件接收器（用于订阅）
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<DownloadEvent>>>>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 配置
    config: Arc<RwLock<DownloadConfig>>,
}

impl DownloadManager {
    /// 创建新的下载管理器
    pub fn new() -> Self {
        let config = DownloadConfig::default();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// 获取全局实例
    pub fn global() -> &'static DownloadManager {
        &DOWNLOAD_MANAGER
    }

    /// 更新配置
    pub fn set_config(&self, config: DownloadConfig) {
        *self.config.write() = config;
    }

    /// 获取配置
    pub fn get_config(&self) -> DownloadConfig {
        self.config.read().clone()
    }

    /// 订阅下载事件
    pub fn subscribe(&self) -> Option<mpsc::UnboundedReceiver<DownloadEvent>> {
        self.event_rx.write().take()
    }

    /// 添加下载任务
    pub fn add_task(&self, mut task: DownloadTask) -> Result<u64> {
        let task_id = task.id;

        // 检查保存路径的父目录是否存在
        if let Some(parent) = task.save_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 检查是否已存在同 ID 任务
        if self.tasks.read().contains_key(&task_id) {
            return Err(PicacgError::InvalidArgument(format!(
                "任务 {} 已存在",
                task_id
            )));
        }

        // 创建任务句柄
        let handle = DownloadHandle::new(task_id, self.event_tx.clone());

        // 保存任务和句柄
        task.status = DownloadStatus::Waiting;
        self.tasks.write().insert(task_id, task.clone());
        self.handles.write().insert(task_id, handle.clone());

        // 启动下载任务
        self.spawn_download_task(task, handle);

        Ok(task_id)
    }

    /// 取消任务
    pub fn cancel_task(&self, task_id: u64) -> Result<()> {
        if let Some(handle) = self.handles.read().get(&task_id) {
            handle.cancel();
            Ok(())
        } else {
            Err(PicacgError::NotFound(format!("任务 {} 不存在", task_id)))
        }
    }

    /// 获取任务信息
    pub fn get_task(&self, task_id: u64) -> Option<DownloadTask> {
        self.tasks.read().get(&task_id).cloned()
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<DownloadTask> {
        self.tasks.read().values().cloned().collect()
    }

    /// 清理已完成/失败的任务
    pub fn cleanup_finished_tasks(&self) {
        let mut tasks = self.tasks.write();
        let mut handles = self.handles.write();

        tasks.retain(|id, task| {
            let should_keep = matches!(
                task.status,
                DownloadStatus::Waiting | DownloadStatus::Downloading | DownloadStatus::Paused
            );

            if !should_keep {
                handles.remove(id);
            }

            should_keep
        });
    }

    /// 生成下载任务
    fn spawn_download_task(&self, task: DownloadTask, handle: DownloadHandle) {
        let client = self.client.clone();
        let tasks = self.tasks.clone();
        let handles = self.handles.clone();
        let semaphore = self.semaphore.clone();
        let config = self.config.read().clone();

        tokio::spawn(async move {
            // 获取信号量许可（控制并发数）
            let _permit = semaphore.acquire().await.unwrap();

            // 发送开始事件
            handle.send_event(DownloadEvent::Started(task.id));

            // 更新状态为下载中
            if let Some(t) = tasks.write().get_mut(&task.id) {
                t.status = DownloadStatus::Downloading;
            }

            // 执行下载
            let result =
                Self::download_file(client, task.clone(), handle.clone(), tasks.clone(), config)
                    .await;

            // 处理下载结果
            match result {
                Ok(_) => {
                    // 更新状态为完成
                    if let Some(t) = tasks.write().get_mut(&task.id) {
                        t.status = DownloadStatus::Completed;
                    }
                    handle.send_event(DownloadEvent::Completed(task.id));
                    info!("下载任务 {} 完成", task.id);
                }
                Err(e) => {
                    if handle.is_cancelled() {
                        // 任务被取消
                        if let Some(t) = tasks.write().get_mut(&task.id) {
                            t.status = DownloadStatus::Cancelled;
                        }
                        handle.send_event(DownloadEvent::Cancelled(task.id));
                        info!("下载任务 {} 已取消", task.id);
                    } else {
                        // 下载失败
                        let error_msg = e.to_string();
                        if let Some(t) = tasks.write().get_mut(&task.id) {
                            t.status = DownloadStatus::Failed;
                            t.error = Some(error_msg.clone());
                        }
                        handle.send_event(DownloadEvent::Failed {
                            task_id: task.id,
                            error: error_msg.clone(),
                        });
                        error!("下载任务 {} 失败: {}", task.id, error_msg);
                    }
                }
            }

            // 清理句柄
            handles.write().remove(&task.id);
        });
    }

    /// 下载文件
    async fn download_file(
        client: Client,
        task: DownloadTask,
        handle: DownloadHandle,
        tasks: Arc<RwLock<HashMap<u64, DownloadTask>>>,
        config: DownloadConfig,
    ) -> Result<()> {
        let mut retries = 0;

        loop {
            // 检查是否取消
            if handle.is_cancelled() {
                return Err(PicacgError::Cancelled);
            }

            // 尝试下载
            match Self::download_with_resume(&client, &task, &handle, &tasks, &config).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    retries += 1;
                    if retries >= config.max_retries {
                        return Err(e);
                    }

                    warn!(
                        "下载任务 {} 失败，重试 {}/{}",
                        task.id, retries, config.max_retries
                    );

                    // 等待后重试
                    sleep(Duration::from_secs(2u64.pow(retries))).await;
                }
            }
        }
    }

    /// 带断点续传的下载
    async fn download_with_resume(
        client: &Client,
        task: &DownloadTask,
        handle: &DownloadHandle,
        tasks: &Arc<RwLock<HashMap<u64, DownloadTask>>>,
        config: &DownloadConfig,
    ) -> Result<()> {
        // 检查文件是否已存在
        let existing_size = if task.save_path.exists() {
            tokio::fs::metadata(&task.save_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        // 构建请求
        let mut request = client.get(&task.url);
        if existing_size > 0 {
            request = request.header("Range", format!("bytes={}-", existing_size));
        }

        // 发送请求
        let response = request.send().await?;

        // 检查状态码
        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            return Err(PicacgError::HttpError {
                status: status.as_u16(),
                message: format!("HTTP 错误: {}", status),
            });
        }

        // 获取总大小
        let content_length = response.content_length().unwrap_or(0);
        let total_bytes = if status.as_u16() == 206 {
            existing_size + content_length
        } else {
            content_length
        };

        // 更新任务信息
        {
            if let Some(t) = tasks.write().get_mut(&task.id) {
                t.total_bytes = total_bytes;
                t.downloaded_bytes = existing_size;
                t.supports_resume = status.as_u16() == 206;
            }
        }

        // 打开文件
        let mut file = if existing_size > 0 {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&task.save_path)
                .await?;
            file.seek(std::io::SeekFrom::End(0)).await?;
            file
        } else {
            File::create(&task.save_path).await?
        };

        // 下载数据
        let mut stream = response.bytes_stream();
        let mut downloaded = existing_size;
        let mut last_progress_time = Instant::now();
        let mut last_downloaded = downloaded;

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            // 检查是否取消
            if handle.is_cancelled() {
                return Err(PicacgError::Cancelled);
            }

            let chunk = chunk?;
            file.write_all(&chunk).await?;

            downloaded += chunk.len() as u64;

            // 更新进度
            let now = Instant::now();
            if now.duration_since(last_progress_time).as_millis()
                >= config.progress_interval_ms as u128
            {
                let elapsed = now.duration_since(last_progress_time).as_secs_f64();
                let speed = if elapsed > 0.0 {
                    (downloaded - last_downloaded) as f64 / elapsed
                } else {
                    0.0
                };

                // 更新任务进度
                if let Some(t) = tasks.write().get_mut(&task.id) {
                    t.downloaded_bytes = downloaded;
                }

                // 发送进度事件
                let progress = {
                    let tasks_read = tasks.read();
                    if let Some(t) = tasks_read.get(&task.id) {
                        t.progress(speed)
                    } else {
                        return Err(PicacgError::NotFound("任务不存在".to_string()));
                    }
                };

                handle.send_event(DownloadEvent::Progress(progress));

                last_progress_time = now;
                last_downloaded = downloaded;
            }
        }

        file.flush().await?;

        Ok(())
    }
}
