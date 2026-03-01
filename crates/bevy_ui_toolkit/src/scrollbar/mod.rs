//! 自定义滚动条系统
//!
//! 实现类似 VSCode 风格的滚动条，支持：
//! - 滚动条轨道点击快速跳转
//! - 滑块拖拽滚动
//! - 自动计算滑块大小和位置
//! - DPI 缩放适配
//!
//! ## 使用方法
//!
//! 1. 在 Plugin 中注册系统：
//! ```ignore
//! app.add_plugins(BevyUiToolkitPlugin::default())
//!    .add_systems(Update, (
//!        update_all_scrollbar_thumbs,
//!        scrollbar_thumb_interaction,
//!        scrollbar_track_click,
//!        scrollbar_thumb_drag,
//!        reset_drag_state_on_release,
//!    ));
//! ```
//!
//! 2. 创建滚动容器时保存 Entity ID：
//! ```ignore
//! let scroll_container = parent.spawn((
//!     ScrollContainer,
//!     Node {
//!         overflow: Overflow::scroll_y(),
//!         ..default()
//!     },
//!     ScrollPosition::default(),
//!     ContentSizeInfo::default(),
//! )).id();
//! ```
//!
//! 3. 创建滚动条组件：
//! ```ignore
//! spawn_scrollbar(parent, scroll_container);
//! ```

mod components;
mod systems;

pub use components::*;
pub use systems::*;
