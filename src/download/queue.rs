use crate::download::task::{DownloadStatus, DownloadTask};
use crate::error::Result;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// 下载优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DownloadPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
}

/// 队列中的任务项
#[derive(Debug, Clone)]
pub struct QueueItem {
    /// 任务
    pub task: DownloadTask,
    /// 优先级
    pub priority: DownloadPriority,
    /// 添加时间（Unix 时间戳）
    pub added_at: i64,
}

impl QueueItem {
    pub fn new(task: DownloadTask, priority: DownloadPriority) -> Self {
        Self {
            task,
            priority,
            added_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// 下载队列
pub struct DownloadQueue {
    /// 等待队列（按优先级和添加时间排序）
    waiting: Arc<RwLock<VecDeque<QueueItem>>>,
    /// 下载中的任务（task_id -> QueueItem）
    downloading: Arc<RwLock<HashMap<u64, QueueItem>>>,
    /// 暂停的任务
    paused: Arc<RwLock<HashMap<u64, QueueItem>>>,
    /// 已完成的任务
    completed: Arc<RwLock<Vec<QueueItem>>>,
    /// 失败的任务
    failed: Arc<RwLock<Vec<QueueItem>>>,
}

impl DownloadQueue {
    /// 创建新的下载队列
    pub fn new() -> Self {
        Self {
            waiting: Arc::new(RwLock::new(VecDeque::new())),
            downloading: Arc::new(RwLock::new(HashMap::new())),
            paused: Arc::new(RwLock::new(HashMap::new())),
            completed: Arc::new(RwLock::new(Vec::new())),
            failed: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加任务到队列
    pub fn enqueue(&self, task: DownloadTask, priority: DownloadPriority) -> Result<()> {
        let item = QueueItem::new(task, priority);
        let mut waiting = self.waiting.write();

        // 按优先级插入（高优先级在前）
        let insert_pos = waiting
            .iter()
            .position(|existing| existing.priority < priority)
            .unwrap_or(waiting.len());

        waiting.insert(insert_pos, item);
        Ok(())
    }

    /// 从等待队列中取出下一个任务
    pub fn dequeue(&self) -> Option<QueueItem> {
        self.waiting.write().pop_front()
    }

    /// 将任务标记为下载中
    pub fn mark_downloading(&self, item: QueueItem) {
        self.downloading.write().insert(item.task.id, item);
    }

    /// 将任务标记为已完成
    pub fn mark_completed(&self, task_id: u64) -> Option<QueueItem> {
        if let Some(item) = self.downloading.write().remove(&task_id) {
            self.completed.write().push(item.clone());
            Some(item)
        } else {
            None
        }
    }

    /// 将任务标记为失败
    pub fn mark_failed(&self, task_id: u64) -> Option<QueueItem> {
        if let Some(item) = self.downloading.write().remove(&task_id) {
            self.failed.write().push(item.clone());
            Some(item)
        } else {
            None
        }
    }

    /// 暂停任务
    pub fn pause_task(&self, task_id: u64) -> Result<()> {
        // 从下载中移除
        if let Some(item) = self.downloading.write().remove(&task_id) {
            self.paused.write().insert(task_id, item);
            return Ok(());
        }

        // 从等待队列中移除
        let mut waiting = self.waiting.write();
        if let Some(pos) = waiting.iter().position(|item| item.task.id == task_id) {
            let item = waiting.remove(pos).unwrap();
            self.paused.write().insert(task_id, item);
            return Ok(());
        }

        Err(crate::error::PicacgError::NotFound(format!(
            "任务 {} 不存在",
            task_id
        )))
    }

    /// 恢复任务
    pub fn resume_task(&self, task_id: u64) -> Result<QueueItem> {
        if let Some(item) = self.paused.write().remove(&task_id) {
            // 重新加入等待队列
            let mut waiting = self.waiting.write();
            let insert_pos = waiting
                .iter()
                .position(|existing| existing.priority < item.priority)
                .unwrap_or(waiting.len());
            waiting.insert(insert_pos, item.clone());
            Ok(item)
        } else {
            Err(crate::error::PicacgError::NotFound(format!(
                "暂停任务 {} 不存在",
                task_id
            )))
        }
    }

    /// 移除任务（从任何队列中）
    pub fn remove_task(&self, task_id: u64) -> Option<QueueItem> {
        // 尝试从各个队列中移除
        if let Some(item) = self.downloading.write().remove(&task_id) {
            return Some(item);
        }

        if let Some(item) = self.paused.write().remove(&task_id) {
            return Some(item);
        }

        let mut waiting = self.waiting.write();
        if let Some(pos) = waiting.iter().position(|item| item.task.id == task_id) {
            return waiting.remove(pos);
        }

        None
    }

    /// 获取等待队列大小
    pub fn waiting_count(&self) -> usize {
        self.waiting.read().len()
    }

    /// 获取下载中任务数量
    pub fn downloading_count(&self) -> usize {
        self.downloading.read().len()
    }

    /// 获取暂停任务数量
    pub fn paused_count(&self) -> usize {
        self.paused.read().len()
    }

    /// 获取已完成任务数量
    pub fn completed_count(&self) -> usize {
        self.completed.read().len()
    }

    /// 获取失败任务数量
    pub fn failed_count(&self) -> usize {
        self.failed.read().len()
    }

    /// 获取所有等待中的任务
    pub fn get_waiting_tasks(&self) -> Vec<QueueItem> {
        self.waiting.read().iter().cloned().collect()
    }

    /// 获取所有下载中的任务
    pub fn get_downloading_tasks(&self) -> Vec<QueueItem> {
        self.downloading.read().values().cloned().collect()
    }

    /// 获取所有暂停的任务
    pub fn get_paused_tasks(&self) -> Vec<QueueItem> {
        self.paused.read().values().cloned().collect()
    }

    /// 获取所有已完成的任务
    pub fn get_completed_tasks(&self) -> Vec<QueueItem> {
        self.completed.read().clone()
    }

    /// 获取所有失败的任务
    pub fn get_failed_tasks(&self) -> Vec<QueueItem> {
        self.failed.read().clone()
    }

    /// 清空已完成的任务
    pub fn clear_completed(&self) {
        self.completed.write().clear();
    }

    /// 清空失败的任务
    pub fn clear_failed(&self) {
        self.failed.write().clear();
    }

    /// 重试所有失败的任务
    pub fn retry_failed(&self) -> Vec<QueueItem> {
        let mut failed = self.failed.write();
        let tasks = failed.drain(..).collect::<Vec<_>>();

        // 重新加入等待队列
        let mut waiting = self.waiting.write();
        for item in &tasks {
            let insert_pos = waiting
                .iter()
                .position(|existing| existing.priority < item.priority)
                .unwrap_or(waiting.len());
            waiting.insert(insert_pos, item.clone());
        }

        tasks
    }

    /// 获取队列统计信息
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            waiting: self.waiting_count(),
            downloading: self.downloading_count(),
            paused: self.paused_count(),
            completed: self.completed_count(),
            failed: self.failed_count(),
        }
    }

    /// 检查队列是否为空（不包括已完成和失败）
    pub fn is_empty(&self) -> bool {
        self.waiting_count() == 0 && self.downloading_count() == 0 && self.paused_count() == 0
    }

    /// 获取总任务数（不包括已完成和失败）
    pub fn total_active_count(&self) -> usize {
        self.waiting_count() + self.downloading_count() + self.paused_count()
    }
}

impl Default for DownloadQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 队列统计信息
#[derive(Debug, Clone, Copy)]
pub struct QueueStats {
    pub waiting: usize,
    pub downloading: usize,
    pub paused: usize,
    pub completed: usize,
    pub failed: usize,
}

impl QueueStats {
    /// 获取活跃任务总数（等待+下载中+暂停）
    pub fn active_total(&self) -> usize {
        self.waiting + self.downloading + self.paused
    }

    /// 获取历史任务总数（已完成+失败）
    pub fn history_total(&self) -> usize {
        self.completed + self.failed
    }

    /// 获取所有任务总数
    pub fn total(&self) -> usize {
        self.active_total() + self.history_total()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_queue_operations() {
        let queue = DownloadQueue::new();

        // 添加任务
        let task1 = DownloadTask::new("url1".to_string(), PathBuf::from("/tmp/file1"));
        let task2 = DownloadTask::new("url2".to_string(), PathBuf::from("/tmp/file2"));
        let task3 = DownloadTask::new("url3".to_string(), PathBuf::from("/tmp/file3"));

        queue
            .enqueue(task1.clone(), DownloadPriority::Normal)
            .unwrap();
        queue
            .enqueue(task2.clone(), DownloadPriority::High)
            .unwrap();
        queue
            .enqueue(task3.clone(), DownloadPriority::Low)
            .unwrap();

        // 检查队列大小
        assert_eq!(queue.waiting_count(), 3);

        // 出队（应该按优先级：High -> Normal -> Low）
        let item = queue.dequeue().unwrap();
        assert_eq!(item.task.id, task2.id); // High priority

        let item = queue.dequeue().unwrap();
        assert_eq!(item.task.id, task1.id); // Normal priority

        let item = queue.dequeue().unwrap();
        assert_eq!(item.task.id, task3.id); // Low priority

        assert_eq!(queue.waiting_count(), 0);
    }

    #[test]
    fn test_pause_resume() {
        let queue = DownloadQueue::new();

        let task = DownloadTask::new("url".to_string(), PathBuf::from("/tmp/file"));
        let task_id = task.id;

        queue.enqueue(task, DownloadPriority::Normal).unwrap();
        assert_eq!(queue.waiting_count(), 1);

        // 暂停
        queue.pause_task(task_id).unwrap();
        assert_eq!(queue.waiting_count(), 0);
        assert_eq!(queue.paused_count(), 1);

        // 恢复
        queue.resume_task(task_id).unwrap();
        assert_eq!(queue.waiting_count(), 1);
        assert_eq!(queue.paused_count(), 0);
    }

    #[test]
    fn test_stats() {
        let queue = DownloadQueue::new();

        let task1 = DownloadTask::new("url1".to_string(), PathBuf::from("/tmp/file1"));
        let task2 = DownloadTask::new("url2".to_string(), PathBuf::from("/tmp/file2"));

        queue
            .enqueue(task1.clone(), DownloadPriority::Normal)
            .unwrap();
        queue
            .enqueue(task2.clone(), DownloadPriority::Normal)
            .unwrap();

        let stats = queue.stats();
        assert_eq!(stats.waiting, 2);
        assert_eq!(stats.active_total(), 2);
    }
}
