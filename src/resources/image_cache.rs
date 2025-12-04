//! 图片缓存资源
//!
//! 管理远程图片的加载和缓存

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

/// 最大并发加载数量
const MAX_CONCURRENT_LOADS: usize = 15;

/// 图片加载状态
#[derive(Debug, Clone)]
pub enum ImageLoadState {
    /// 正在加载
    Loading,
    /// 加载完成
    Loaded(Handle<Image>),
    /// 加载失败
    Failed(String),
}

/// 图片缓存资源
#[derive(Resource, Default)]
pub struct ImageCache {
    /// URL -> Handle 映射
    pub handles: HashMap<String, Handle<Image>>,
    /// 正在加载的 URL
    pub loading: HashMap<String, bool>,
    /// 加载失败的 URL
    pub failed: HashMap<String, String>,
    /// 待处理的图片 URL 队列（节流用）
    pub pending_queue: VecDeque<String>,
}

impl ImageCache {
    /// 检查图片是否已加载
    pub fn is_loaded(&self, url: &str) -> bool {
        self.handles.contains_key(url)
    }

    /// 检查图片是否正在加载
    pub fn is_loading(&self, url: &str) -> bool {
        self.loading.contains_key(url)
    }

    /// 检查图片是否在队列中
    pub fn is_pending(&self, url: &str) -> bool {
        self.pending_queue.iter().any(|u| u == url)
    }

    /// 获取图片 Handle
    pub fn get(&self, url: &str) -> Option<&Handle<Image>> {
        self.handles.get(url)
    }

    /// 添加到待处理队列（如果未加载、未在队列中、未正在加载）
    pub fn enqueue(&mut self, url: String) {
        if !self.is_loaded(&url) && !self.is_loading(&url) && !self.is_pending(&url) {
            self.pending_queue.push_back(url);
        }
    }

    /// 获取可以开始加载的 URL 列表（受并发限制）
    pub fn take_pending_batch(&mut self) -> Vec<String> {
        let current_loading = self.loading.len();
        let available_slots = MAX_CONCURRENT_LOADS.saturating_sub(current_loading);
        let batch_size = available_slots.min(self.pending_queue.len());

        (0..batch_size)
            .filter_map(|_| self.pending_queue.pop_front())
            .collect()
    }

    /// 标记为正在加载
    pub fn mark_loading(&mut self, url: String) {
        self.loading.insert(url, true);
    }

    /// 设置加载完成
    pub fn set_loaded(&mut self, url: String, handle: Handle<Image>) {
        self.loading.remove(&url);
        self.handles.insert(url, handle);
    }

    /// 设置加载失败
    pub fn set_failed(&mut self, url: String, error: String) {
        self.loading.remove(&url);
        self.failed.insert(url, error);
    }

    /// 获取已加载图片数量
    pub fn loaded_count(&self) -> usize {
        self.handles.len()
    }

    /// 获取正在加载的图片数量
    pub fn loading_count(&self) -> usize {
        self.loading.len()
    }

    /// 获取待处理队列长度
    pub fn pending_count(&self) -> usize {
        self.pending_queue.len()
    }
}
