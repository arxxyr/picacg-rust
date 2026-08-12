//! 图片缓存资源
//!
//! 管理远程图片的加载和缓存。
//! 每个 URL 的生命周期由单一状态机表达：
//! 入队（Pending）→ 加载中（Loading）→ 完成（Loaded）/ 失败（Failed）。
//!
//! ## 失败重试（有界 + 指数退避）
//!
//! 首次加载可能因网络抖动失败。失败后自动安排重试：第 1 次失败等 2s、
//! 第 2 次等 4s，累计 `MAX_ATTEMPTS` 次仍失败才进入**终局失败**。
//! `is_failed()` 只对终局失败返回真——消费系统（占位符替换等）因此在
//! 重试期间保持占位符存活，重试成功后图片正常落位；只有终局失败才摘除占位。
//! 重试重排队由 `requeue_ready_retries()` 驱动（api_plugin 的队列泵每帧调用，
//! 无待重试项时 O(1) 直返）。

use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use bevy::prelude::*;

/// 最大并发加载数量
const MAX_CONCURRENT_LOADS: usize = 15;

/// 单个 URL 的最大下载尝试次数（含首次）
const MAX_ATTEMPTS: u32 = 3;

/// 第 n 次失败后的重试等待（指数退避：2s、4s）
fn retry_backoff(attempts: u32) -> Duration {
    Duration::from_secs(2u64 << attempts.saturating_sub(1).min(4))
}

/// 图片加载状态
#[derive(Debug, Clone)]
pub enum ImageLoadState {
    /// 已入队等待加载
    Pending {
        /// 已尝试次数（重试重排队时保留）
        attempts: u32,
    },
    /// 正在加载
    Loading {
        /// 已尝试次数
        attempts: u32,
    },
    /// 加载完成
    Loaded(Handle<Image>),
    /// 加载失败
    Failed {
        /// 错误信息
        error: String,
        /// 已尝试次数
        attempts: u32,
        /// 下次重试时刻；`None` = 尝试耗尽，终局失败
        retry_at: Option<Instant>,
    },
}

/// 图片缓存资源
#[derive(Resource, Default)]
pub struct ImageCache {
    /// URL -> 加载状态（单一事实源）
    states: HashMap<String, ImageLoadState>,
    /// 待处理的图片 URL 队列（节流用）
    pending_queue: VecDeque<String>,
    /// 处于「等待重试」状态的条目数（快速路径：为 0 时重试扫描直接返回）
    retrying_count: usize,
}

impl ImageCache {
    /// 检查图片是否已加载
    pub fn is_loaded(&self, url: &str) -> bool {
        matches!(self.states.get(url), Some(ImageLoadState::Loaded(_)))
    }

    /// 检查图片是否正在加载
    pub fn is_loading(&self, url: &str) -> bool {
        matches!(self.states.get(url), Some(ImageLoadState::Loading { .. }))
    }

    /// 检查图片是否**终局失败**（重试次数耗尽）
    ///
    /// 等待重试期间返回 `false`——消费系统据此保持占位符存活。
    pub fn is_failed(&self, url: &str) -> bool {
        matches!(
            self.states.get(url),
            Some(ImageLoadState::Failed { retry_at: None, .. })
        )
    }

    /// URL 是否已有任何状态（入队/加载中/完成/失败）
    ///
    /// 消费系统用它避免对同一 URL 重复发起加载请求。
    pub fn is_known(&self, url: &str) -> bool {
        self.states.contains_key(url)
    }

    /// 获取图片 Handle
    pub fn get(&self, url: &str) -> Option<&Handle<Image>> {
        match self.states.get(url) {
            Some(ImageLoadState::Loaded(handle)) => Some(handle),
            _ => None,
        }
    }

    /// 添加到待处理队列
    ///
    /// 任何已有状态（含失败）的 URL 都不会重复入队；
    /// 失败重试由内部退避机制自动安排，外部强制重来用 `retry`。
    pub fn enqueue(&mut self, url: String) {
        if !self.states.contains_key(&url) {
            self.states
                .insert(url.clone(), ImageLoadState::Pending { attempts: 0 });
            self.pending_queue.push_back(url);
        }
    }

    /// 清除既有状态并重新入队（手动强制重试入口，尝试计数归零）
    #[allow(dead_code)]
    pub fn retry(&mut self, url: String) {
        if let Some(ImageLoadState::Failed { retry_at, .. }) = self.states.get(&url) {
            if retry_at.is_some() {
                self.retrying_count = self.retrying_count.saturating_sub(1);
            }
            self.states
                .insert(url.clone(), ImageLoadState::Pending { attempts: 0 });
            self.pending_queue.push_back(url);
        }
    }

    /// 把退避期已过的失败条目重新排队（队列泵每帧调用；无待重试项 O(1) 直返）
    pub fn requeue_ready_retries(&mut self) {
        if self.retrying_count == 0 {
            return;
        }
        let now = Instant::now();
        let ready: Vec<String> = self
            .states
            .iter()
            .filter_map(|(url, state)| match state {
                ImageLoadState::Failed {
                    retry_at: Some(at), ..
                } if *at <= now => Some(url.clone()),
                _ => None,
            })
            .collect();
        for url in ready {
            let attempts = match self.states.get(&url) {
                Some(ImageLoadState::Failed { attempts, .. }) => *attempts,
                _ => 0,
            };
            tracing::debug!("图片重试入队: url={} 第 {} 次尝试", url, attempts + 1);
            self.states
                .insert(url.clone(), ImageLoadState::Pending { attempts });
            self.pending_queue.push_back(url);
            self.retrying_count = self.retrying_count.saturating_sub(1);
        }
    }

    /// 获取可以开始加载的 URL 列表（受并发限制）
    pub fn take_pending_batch(&mut self) -> Vec<String> {
        let current_loading = self.loading_count();
        let available_slots = MAX_CONCURRENT_LOADS.saturating_sub(current_loading);
        let batch_size = available_slots.min(self.pending_queue.len());

        (0..batch_size)
            .filter_map(|_| self.pending_queue.pop_front())
            .collect()
    }

    /// 标记为正在加载（保留已尝试次数）
    pub fn mark_loading(&mut self, url: String) {
        let attempts = match self.states.get(&url) {
            Some(ImageLoadState::Pending { attempts }) => *attempts,
            Some(ImageLoadState::Loading { attempts }) => *attempts,
            _ => 0,
        };
        self.states
            .insert(url, ImageLoadState::Loading { attempts });
    }

    /// 设置加载完成
    pub fn set_loaded(&mut self, url: String, handle: Handle<Image>) {
        self.states.insert(url, ImageLoadState::Loaded(handle));
    }

    /// 设置加载失败：未耗尽尝试次数则安排退避重试，否则进入终局失败
    pub fn set_failed(&mut self, url: String, error: String) {
        let attempts = match self.states.get(&url) {
            Some(ImageLoadState::Loading { attempts }) => attempts + 1,
            Some(ImageLoadState::Failed { attempts, .. }) => attempts + 1,
            _ => 1,
        };
        let retry_at = if attempts < MAX_ATTEMPTS {
            let backoff = retry_backoff(attempts);
            tracing::debug!(
                "图片加载失败（第 {}/{} 次尝试），{}s 后重试: url={} error={}",
                attempts,
                MAX_ATTEMPTS,
                backoff.as_secs(),
                url,
                error
            );
            self.retrying_count += 1;
            Some(Instant::now() + backoff)
        } else {
            tracing::warn!(
                "图片加载失败（{} 次尝试耗尽，终局失败）: url={} error={}",
                attempts,
                url,
                error
            );
            None
        };
        self.states.insert(
            url,
            ImageLoadState::Failed {
                error,
                attempts,
                retry_at,
            },
        );
    }

    /// 获取已加载图片数量
    pub fn loaded_count(&self) -> usize {
        self.states
            .values()
            .filter(|s| matches!(s, ImageLoadState::Loaded(_)))
            .count()
    }

    /// 获取正在加载的图片数量
    pub fn loading_count(&self) -> usize {
        self.states
            .values()
            .filter(|s| matches!(s, ImageLoadState::Loading { .. }))
            .count()
    }

    /// 获取待处理队列长度
    pub fn pending_count(&self) -> usize {
        self.pending_queue.len()
    }
}
