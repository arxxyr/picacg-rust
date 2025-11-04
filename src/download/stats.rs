use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::RwLock;

/// 下载速度样本
#[derive(Debug, Clone, Copy)]
struct SpeedSample {
    /// 采样时间
    timestamp: Instant,
    /// 已下载字节数
    bytes: u64,
}

/// 下载统计信息
#[derive(Debug, Clone)]
pub struct DownloadStats {
    /// 任务 ID
    pub task_id: u64,
    /// 开始时间
    pub start_time: Instant,
    /// 总字节数
    pub total_bytes: u64,
    /// 已下载字节数
    pub downloaded_bytes: u64,
    /// 当前速度（字节/秒）
    pub current_speed: f64,
    /// 平均速度（字节/秒）
    pub average_speed: f64,
    /// 预计剩余时间（秒）
    pub eta_seconds: u64,
    /// 下载进度（0-100）
    pub progress_percentage: f64,
}

impl DownloadStats {
    /// 创建新的统计信息
    pub fn new(task_id: u64, total_bytes: u64) -> Self {
        Self {
            task_id,
            start_time: Instant::now(),
            total_bytes,
            downloaded_bytes: 0,
            current_speed: 0.0,
            average_speed: 0.0,
            eta_seconds: 0,
            progress_percentage: 0.0,
        }
    }

    /// 更新已下载字节数
    pub fn update_downloaded(&mut self, downloaded_bytes: u64) {
        self.downloaded_bytes = downloaded_bytes;
        self.progress_percentage = if self.total_bytes > 0 {
            (downloaded_bytes as f64 / self.total_bytes as f64) * 100.0
        } else {
            0.0
        };
    }

    /// 更新速度
    pub fn update_speed(&mut self, current_speed: f64) {
        self.current_speed = current_speed;

        // 计算平均速度
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.average_speed = self.downloaded_bytes as f64 / elapsed;
        }

        // 计算 ETA
        if current_speed > 0.0 && self.total_bytes > self.downloaded_bytes {
            self.eta_seconds =
                ((self.total_bytes - self.downloaded_bytes) as f64 / current_speed) as u64;
        } else {
            self.eta_seconds = 0;
        }
    }

    /// 格式化速度（人类可读）
    pub fn format_speed(&self) -> String {
        format_bytes_per_second(self.current_speed)
    }

    /// 格式化 ETA（人类可读）
    pub fn format_eta(&self) -> String {
        if self.eta_seconds == 0 {
            "未知".to_string()
        } else {
            format_duration(Duration::from_secs(self.eta_seconds))
        }
    }

    /// 格式化已下载大小
    pub fn format_downloaded(&self) -> String {
        format_bytes(self.downloaded_bytes)
    }

    /// 格式化总大小
    pub fn format_total(&self) -> String {
        format_bytes(self.total_bytes)
    }
}

/// 下载统计追踪器
pub struct StatsTracker {
    /// 速度样本队列（最多保留最近 10 个样本）
    samples: Arc<RwLock<VecDeque<SpeedSample>>>,
    /// 最大样本数
    max_samples: usize,
}

impl StatsTracker {
    /// 创建新的统计追踪器
    pub fn new() -> Self {
        Self {
            samples: Arc::new(RwLock::new(VecDeque::new())),
            max_samples: 10,
        }
    }

    /// 添加样本
    pub fn add_sample(&self, bytes: u64) {
        let mut samples = self.samples.write();
        samples.push_back(SpeedSample {
            timestamp: Instant::now(),
            bytes,
        });

        // 保持最多 max_samples 个样本
        while samples.len() > self.max_samples {
            samples.pop_front();
        }
    }

    /// 计算当前速度（基于最近两个样本）
    pub fn calculate_current_speed(&self) -> f64 {
        let samples = self.samples.read();
        if samples.len() < 2 {
            return 0.0;
        }

        let last = samples.back().unwrap();
        let prev = samples.get(samples.len() - 2).unwrap();

        let elapsed = last.timestamp.duration_since(prev.timestamp).as_secs_f64();
        if elapsed > 0.0 {
            (last.bytes - prev.bytes) as f64 / elapsed
        } else {
            0.0
        }
    }

    /// 计算平均速度
    pub fn calculate_average_speed(&self) -> f64 {
        let samples = self.samples.read();
        if samples.len() < 2 {
            return 0.0;
        }

        let first = samples.front().unwrap();
        let last = samples.back().unwrap();

        let elapsed = last.timestamp.duration_since(first.timestamp).as_secs_f64();
        if elapsed > 0.0 {
            (last.bytes - first.bytes) as f64 / elapsed
        } else {
            0.0
        }
    }

    /// 重置统计
    pub fn reset(&self) {
        self.samples.write().clear();
    }
}

impl Default for StatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局下载统计
#[derive(Debug, Clone, Default)]
pub struct GlobalDownloadStats {
    /// 总下载任务数
    pub total_tasks: u64,
    /// 成功任务数
    pub successful_tasks: u64,
    /// 失败任务数
    pub failed_tasks: u64,
    /// 总下载字节数
    pub total_downloaded_bytes: u64,
    /// 总上传字节数（如果有）
    pub total_uploaded_bytes: u64,
}

impl GlobalDownloadStats {
    /// 记录任务完成
    pub fn record_success(&mut self, downloaded_bytes: u64) {
        self.successful_tasks += 1;
        self.total_downloaded_bytes += downloaded_bytes;
    }

    /// 记录任务失败
    pub fn record_failure(&mut self) {
        self.failed_tasks += 1;
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.successful_tasks as f64 / self.total_tasks as f64) * 100.0
        }
    }

    /// 格式化总下载量
    pub fn format_total_downloaded(&self) -> String {
        format_bytes(self.total_downloaded_bytes)
    }
}

/// 格式化字节数（人类可读）
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// 格式化速度（字节/秒）
pub fn format_bytes_per_second(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec as u64))
}

/// 格式化时间间隔
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}:{:02}", minutes, seconds)
    } else {
        format!("{}秒", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1536 * 1024 * 1024), "1.50 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30秒");
        assert_eq!(format_duration(Duration::from_secs(90)), "1:30");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1:01:05");
    }

    #[test]
    fn test_stats_tracker() {
        let tracker = StatsTracker::new();

        tracker.add_sample(0);
        std::thread::sleep(Duration::from_millis(100));
        tracker.add_sample(1024);

        let speed = tracker.calculate_current_speed();
        assert!(speed > 0.0);
    }

    #[test]
    fn test_global_stats() {
        let mut stats = GlobalDownloadStats::default();

        stats.total_tasks = 10;
        stats.record_success(1024);
        stats.record_success(2048);
        stats.record_failure();

        assert_eq!(stats.successful_tasks, 2);
        assert_eq!(stats.failed_tasks, 1);
        assert_eq!(stats.total_downloaded_bytes, 3072);
        assert_eq!(stats.success_rate(), 20.0);
    }
}
