//! 滚动条系统：滑块悬停/拖拽配色
//!
//! 交互本体（拖拽/点击/滚轮）全部在上游，这里只负责 VSCode 风格的
//! 状态配色。`Changed` 过滤保证静止零开销。

use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{ScrollbarDragState, ScrollbarThumb},
};

use super::scrollbar_config::{THUMB_COLOR, THUMB_HOVER_COLOR, THUMB_PRESSED_COLOR};

/// 按悬停/拖拽状态更新滑块颜色
pub fn update_scrollbar_thumb_colors(
    mut thumbs: Query<
        (&Hovered, &ScrollbarDragState, &mut BackgroundColor),
        (
            With<ScrollbarThumb>,
            Or<(Changed<Hovered>, Changed<ScrollbarDragState>)>,
        ),
    >,
) {
    for (hovered, drag_state, mut color) in &mut thumbs {
        let target = if drag_state.dragging {
            THUMB_PRESSED_COLOR
        } else if hovered.0 {
            THUMB_HOVER_COLOR
        } else {
            THUMB_COLOR
        };
        if color.0 != target {
            color.0 = target;
        }
    }
}
