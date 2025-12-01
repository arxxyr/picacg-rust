//! 图片缓存资源
//!
//! 管理远程图片的加载和缓存

use std::collections::HashMap;

use bevy::prelude::*;

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

    /// 获取图片 Handle
    pub fn get(&self, url: &str) -> Option<&Handle<Image>> {
        self.handles.get(url)
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
}
