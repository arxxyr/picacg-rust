pub mod manager;
pub mod queue;
pub mod stats;
pub mod task;

pub use manager::{DownloadConfig, DownloadManager};
pub use queue::{DownloadPriority, DownloadQueue, QueueItem, QueueStats};
pub use stats::{
    DownloadStats, GlobalDownloadStats, StatsTracker, format_bytes, format_bytes_per_second,
    format_duration,
};
pub use task::{DownloadEvent, DownloadHandle, DownloadProgress, DownloadStatus, DownloadTask};
