//! 分页组件定义
//!
//! 设计原则：
//! - `Pagination` 值组件是**单一事实源**：当前页/总页数只存在这里
//! - 翻页行为由控件内部的 `on(Pointer<Click>)` 观察者完成（见 scenes.rs），
//!   页面不再需要按钮交互系统，只消费 `Changed<Pagination>`
//! - 显示刷新由全局唯一的 `refresh_pagination_widgets` 系统完成（见
//!   systems.rs）

use std::marker::PhantomData;

use bevy::prelude::*;

/// 分页状态（挂在分页控件根实体上，单一事实源）
#[derive(Component, Default, Clone, PartialEq, Eq)]
pub struct Pagination {
    /// 当前页码（从 1 开始）
    pub current_page: u32,
    /// 总页数
    pub total_pages: u32,
}

impl Pagination {
    /// 是否可以向前翻页
    #[must_use]
    pub fn has_prev(&self) -> bool {
        self.current_page > 1
    }

    /// 是否可以向后翻页
    #[must_use]
    pub fn has_next(&self) -> bool {
        self.current_page < self.total_pages
    }
}

/// 为泛型标记组件手写 Default + Clone（derive 会错误地要求 `T:
/// Default/Clone`）。 Default + Clone 是 bsn! 模板系统（Template blanket
/// impl）的门槛。
macro_rules! impl_marker_default_clone {
    ($name:ident) => {
        impl<T: Send + Sync + 'static> Default for $name<T> {
            fn default() -> Self {
                Self {
                    _marker: PhantomData,
                }
            }
        }

        impl<T: Send + Sync + 'static> Clone for $name<T> {
            fn clone(&self) -> Self {
                Self::default()
            }
        }
    };
}

/// 分页控件根标记（泛型 T 区分页面，页面用它定位自己的 `Pagination`）
#[derive(Component)]
pub struct PaginationControl<T: Send + Sync + 'static> {
    _marker: PhantomData<fn() -> T>,
}

impl_marker_default_clone!(PaginationControl);

/// 上一页按钮标记（控件内部接线用，非泛型）
#[derive(Component, Default, Clone)]
pub struct PaginationPrev;

/// 下一页按钮标记（控件内部接线用，非泛型）
#[derive(Component, Default, Clone)]
pub struct PaginationNext;

/// 页码文本标记（控件内部接线用，非泛型）
#[derive(Component, Default, Clone)]
pub struct PaginationPageText;

/// 分页配置
pub struct PaginationConfig {
    /// 按钮宽度
    pub button_width: f32,
    /// 按钮高度
    pub button_height: f32,
    /// 容器高度
    pub container_height: f32,
    /// 元素间距
    pub gap: f32,
    /// 字体大小
    pub font_size: f32,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            button_width: 80.0,
            button_height: 32.0,
            container_height: 50.0,
            gap: 15.0,
            font_size: 14.0,
        }
    }
}
