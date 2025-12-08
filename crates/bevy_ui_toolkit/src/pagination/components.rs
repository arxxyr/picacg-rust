//! 分页组件定义

use std::marker::PhantomData;

use bevy::prelude::*;

/// 分页容器标记（泛型 T 用于区分不同页面）
#[derive(Component)]
pub struct PaginationControl<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationControl<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 上一页按钮标记
#[derive(Component)]
pub struct PaginationPrevButton<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationPrevButton<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 下一页按钮标记
#[derive(Component)]
pub struct PaginationNextButton<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationNextButton<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 页码文本标记
#[derive(Component)]
pub struct PaginationPageText<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationPageText<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 分页配置
pub struct PaginationConfig {
    /// 按钮宽度
    pub button_width: f32,
    /// 按钮高度
    pub button_height: f32,
    /// 容器高度
    pub container_height: f32,
    /// 按钮间距
    pub gap: f32,
    /// 字体大小
    pub font_size: f32,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            button_width: 80.0,
            button_height: 36.0,
            container_height: 50.0,
            gap: 20.0,
            font_size: 14.0,
        }
    }
}

/// 分页状态 trait
///
/// 实现此 trait 可以让状态类型与分页系统配合使用
pub trait PaginationState {
    /// 获取当前页码
    fn current_page(&self) -> u32;

    /// 获取总页数
    fn total_pages(&self) -> u32;

    /// 设置当前页码
    fn set_page(&mut self, page: u32);

    /// 设置加载状态
    fn set_loading(&mut self, loading: bool);

    /// 清除数据（翻页时调用）
    fn clear_data(&mut self);
}
