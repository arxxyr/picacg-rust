//! 滚动条 BSN 场景函数（上游件 + VSCode 风格外观）

use bevy::{
    ecs::template::EntityTemplate,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{ControlOrientation, Scrollbar, ScrollbarThumb},
};

use super::scrollbar_config::{SCROLLBAR_WIDTH, THUMB_COLOR, THUMB_MIN_HEIGHT, TRACK_COLOR};

/// 垂直滚动条场景
///
/// `scroll_container` 接受：
/// - `bsn!` 内的 `#Name` 命名引用（同一宏调用内的滚动容器）
/// - 具体 `Entity`（容器已存在的动态场景）
///
/// 滑块尺寸/位置、轨道点击、拖拽全部由上游系统处理。
pub fn scrollbar(scroll_container: impl Into<EntityTemplate>) -> impl Scene {
    let target: EntityTemplate = scroll_container.into();
    bsn! {
        Scrollbar {
            target: {target},
            orientation: ControlOrientation::Vertical,
            min_thumb_length: THUMB_MIN_HEIGHT,
        }
        Node {
            width: Val::Px(SCROLLBAR_WIDTH),
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
        }
        BackgroundColor(TRACK_COLOR)
        ZIndex(10)
        Children [
            (
                // 滑块：上游组件（无 Node，布局后阶段自动定位）
                ScrollbarThumb {
                    border_radius: BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                }
                Hovered
                BackgroundColor(THUMB_COLOR)
            ),
        ]
    }
}
