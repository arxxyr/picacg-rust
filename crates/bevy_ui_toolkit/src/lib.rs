//! Bevy UI 工具库
//!
//! 提供通用的 UI 组件和系统，包括：
//! - 主题/颜色系统 (`theme`)
//! - 滚动条系统 (`scrollbar`)
//! - 分页组件 (`pagination`)
//! - 瀑布流系统 (`waterfall`)

pub mod pagination;
pub mod scrollbar;
pub mod theme;
pub mod waterfall;

// 重新导出常用类型
use bevy::prelude::*;
pub use pagination::{
    NextBtnFilter, PaginationConfig, PaginationControl, PaginationNextButton, PaginationPageText,
    PaginationPrevButton, PaginationState, PrevBtnFilter, check_pagination_interaction,
    spawn_pagination_controls, spawn_pagination_controls_with_config,
    spawn_pagination_controls_with_theme, update_pagination_display,
    update_pagination_display_with_theme,
};
pub use scrollbar::{
    ContentSizeInfo, GridLayoutParams, ScrollbarConfig, ScrollbarContainer, ScrollbarDragState,
    ScrollbarThumb, ScrollbarTrack, TrackClickFilter, reset_drag_state_on_release,
    scrollbar_thumb_drag, scrollbar_thumb_interaction, scrollbar_track_click,
    update_all_scrollbar_thumbs,
};
pub use theme::{CurrentTheme, Theme};
pub use waterfall::WaterfallState;

/// Bevy UI 工具库插件
///
/// 初始化主题资源和滚动条拖拽状态
#[derive(Default)]
pub struct BevyUiToolkitPlugin {
    /// 自定义主题（可选，默认使用深色主题）
    pub theme: Option<Theme>,
}

impl Plugin for BevyUiToolkitPlugin {
    fn build(&self, app: &mut App) {
        // 初始化主题资源
        let theme = self.theme.clone().unwrap_or_default();
        app.insert_resource(CurrentTheme(theme));

        // 初始化滚动条拖拽状态
        app.init_resource::<ScrollbarDragState>();
    }
}
