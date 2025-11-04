use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// 下载任务 ID 生成器
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// 下载任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// 等待中
    Waiting,
    /// 下载中
    Downloading,
    /// 暂停
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Waiting => write!(f, "等待中"),
            Self::Downloading => write!(f, "下载中"),
            Self::Paused => write!(f, "暂停"),
            Self::Completed => write!(f, "已完成"),
            Self::Failed => write!(f, "失败"),
            Self::Cancelled => write!(f, "已取消"),
        }
    }
}

/// 下载进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// 任务 ID
    pub task_id: u64,
    /// 已下载字节数
    pub downloaded_bytes: u64,
    /// 总字节数（0 表示未知）
    pub total_bytes: u64,
    /// 下载速度（字节/秒）
    pub speed: f64,
    /// 预计剩余时间（秒，0 表示未知）
    pub eta: u64,
}

impl DownloadProgress {
    /// 计算下载进度百分比
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// 是否完成
    pub fn is_complete(&self) -> bool {
        self.total_bytes > 0 && self.downloaded_bytes >= self.total_bytes
    }
}

/// 下载任务
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// 任务 ID
    pub id: u64,
    /// 下载 URL
    pub url: String,
    /// 保存路径
    pub save_path: PathBuf,
    /// 任务状态
    pub status: DownloadStatus,
    /// 已下载字节数
    pub downloaded_bytes: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 是否支持断点续传
    pub supports_resume: bool,
    /// 自定义数据（用于业务关联）
    pub user_data: Option<serde_json::Value>,
}

impl DownloadTask {
    /// 创建新的下载任务
    pub fn new(url: String, save_path: PathBuf) -> Self {
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id,
            url,
            save_path,
            status: DownloadStatus::Waiting,
            downloaded_bytes: 0,
            total_bytes: 0,
            error: None,
            supports_resume: false,
            user_data: None,
        }
    }

    /// 设置自定义数据
    pub fn with_user_data(mut self, user_data: serde_json::Value) -> Self {
        self.user_data = Some(user_data);
        self
    }

    /// 创建进度信息
    pub fn progress(&self, speed: f64) -> DownloadProgress {
        let eta = if speed > 0.0 && self.total_bytes > self.downloaded_bytes {
            ((self.total_bytes - self.downloaded_bytes) as f64 / speed) as u64
        } else {
            0
        };

        DownloadProgress {
            task_id: self.id,
            downloaded_bytes: self.downloaded_bytes,
            total_bytes: self.total_bytes,
            speed,
            eta,
        }
    }
}

/// 下载事件
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    /// 任务开始
    Started(u64),
    /// 进度更新
    Progress(DownloadProgress),
    /// 任务完成
    Completed(u64),
    /// 任务失败
    Failed { task_id: u64, error: String },
    /// 任务取消
    Cancelled(u64),
}

/// 下载任务句柄（用于控制任务）
#[derive(Clone)]
pub struct DownloadHandle {
    /// 任务 ID
    pub task_id: u64,
    /// 取消令牌
    cancel_token: CancellationToken,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<DownloadEvent>,
}

impl DownloadHandle {
    /// 创建新的任务句柄
    pub fn new(task_id: u64, event_tx: mpsc::UnboundedSender<DownloadEvent>) -> Self {
        Self {
            task_id,
            cancel_token: CancellationToken::new(),
            event_tx,
        }
    }

    /// 取消任务
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// 检查是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// 获取取消令牌
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// 发送事件
    pub fn send_event(&self, event: DownloadEvent) {
        let _ = self.event_tx.send(event);
    }
}
