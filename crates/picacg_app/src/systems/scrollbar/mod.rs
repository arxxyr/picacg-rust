//! 滚动条（上游 `bevy_ui_widgets` 包装）
//!
//! 自研滚动条已退役：轨道点击/滑块拖拽/DPI 换算/内容尺寸全部由
//! Bevy 0.19 上游件承担（`UiWidgetsPlugins` 随 DefaultPlugins 加载）：
//! - `bevy::ui_widgets::Scrollbar`：滑块尺寸=可视/内容比例，布局后自动定位
//! - `bevy::ui_widgets::ScrollArea`：滚轮/触控板滚动（`Pointer<Scroll>`
//!   冒泡派发）
//! - `ComputedNode::content_size()`：内容尺寸由引擎布局原生输出
//!
//! 本模块只保留：VSCode 风格外观的包装场景函数 + 滑块悬停/拖拽配色系统。
//!
//! ## 使用方法
//!
//! ```ignore
//! bsn! {
//!     Node { .. }
//!     Children [
//!         ( #ContentScroll ScrollArea Node { overflow: Overflow::scroll_y(), .. } ),
//!         scrollbar(#ContentScroll),
//!     ]
//! }
//! ```

mod scenes;
mod systems;

pub use bevy::ui_widgets::{ControlOrientation, ScrollArea, Scrollbar, ScrollbarThumb};
pub use scenes::*;
pub use systems::*;

/// 滚动条外观常量
pub mod scrollbar_config {
    use bevy::color::Color;

    /// 滚动条宽度
    pub const SCROLLBAR_WIDTH: f32 = 12.0;
    /// 滑块最小高度
    pub const THUMB_MIN_HEIGHT: f32 = 30.0;
    /// 滚动条轨道颜色（透明）
    pub const TRACK_COLOR: Color = Color::srgba(0.2, 0.2, 0.25, 0.3);
    /// 滑块默认颜色
    pub const THUMB_COLOR: Color = Color::srgba(0.5, 0.5, 0.55, 0.6);
    /// 滑块悬停颜色
    pub const THUMB_HOVER_COLOR: Color = Color::srgba(0.6, 0.6, 0.65, 0.8);
    /// 滑块按下颜色
    pub const THUMB_PRESSED_COLOR: Color = Color::srgba(0.7, 0.7, 0.75, 0.9);
}
