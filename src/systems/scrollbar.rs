//! 自定义滚动条系统
//!
//! 实现类似 VSCode 风格的滚动条，支持：
//! - 滚动条轨道点击快速跳转
//! - 滑块拖拽滚动
//! - 自动计算滑块大小和位置

use bevy::{prelude::*, window::PrimaryWindow};

use crate::components::*;

/// 获取窗口的 scale_factor
fn get_scale_factor(window_query: &Query<&Window, With<PrimaryWindow>>) -> f32 {
    window_query
        .single()
        .ok()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0)
}

/// 滚动条配置常量
pub mod scrollbar_config {
    /// 滚动条宽度
    pub const SCROLLBAR_WIDTH: f32 = 12.0;
    /// 滑块最小高度
    pub const THUMB_MIN_HEIGHT: f32 = 30.0;
    /// 滚动条轨道颜色（透明）
    pub const TRACK_COLOR: bevy::color::Color = bevy::color::Color::srgba(0.2, 0.2, 0.25, 0.3);
    /// 滑块默认颜色
    pub const THUMB_COLOR: bevy::color::Color = bevy::color::Color::srgba(0.5, 0.5, 0.55, 0.6);
    /// 滑块悬停颜色
    pub const THUMB_HOVER_COLOR: bevy::color::Color =
        bevy::color::Color::srgba(0.6, 0.6, 0.65, 0.8);
    /// 滑块按下颜色
    pub const THUMB_PRESSED_COLOR: bevy::color::Color =
        bevy::color::Color::srgba(0.7, 0.7, 0.75, 0.9);
}

use scrollbar_config::*;

/// 更新所有滚动条的滑块位置
pub fn update_all_scrollbar_thumbs(
    window_query: Query<&Window, With<PrimaryWindow>>,
    scroll_query: Query<(
        Entity,
        &ScrollPosition,
        &ComputedNode,
        Option<&ContentSizeInfo>,
    )>,
    track_query: Query<(&ScrollbarTrack, &ComputedNode)>,
    mut thumb_query: Query<(&ScrollbarThumb, &mut Node, &mut BackgroundColor)>,
) {
    let scale_factor = get_scale_factor(&window_query);

    for (thumb_component, mut thumb_node, _bg) in &mut thumb_query {
        let scroll_container = thumb_component.scroll_container;

        // 获取滚动容器信息
        let Some((_, scroll_position, scroll_computed, content_size_info)) = scroll_query
            .iter()
            .find(|(entity, _, _, _)| *entity == scroll_container)
        else {
            continue;
        };

        // 获取轨道信息（用于计算轨道高度）
        let Some((_, track_computed)) = track_query
            .iter()
            .find(|(track, _)| track.scroll_container == scroll_container)
        else {
            continue;
        };

        // ComputedNode::size() 返回物理像素，转换为逻辑像素
        let track_height = track_computed.size().y / scale_factor;

        if track_height <= 0.0 {
            continue;
        }

        // 获取内容高度和视口高度（已经是逻辑像素）
        let (content_height, viewport_height) = match content_size_info {
            Some(info) => (info.content_height, info.viewport_height),
            None => {
                let outer_height = scroll_computed.size().y / scale_factor;
                (outer_height, outer_height)
            }
        };

        if viewport_height <= 0.0 {
            continue;
        }

        // 如果内容小于等于视口，隐藏滚动条滑块
        if content_height <= viewport_height {
            thumb_node.height = Val::Px(0.0);
            continue;
        }

        // 计算滑块高度比例（所有值都是逻辑像素）
        let thumb_ratio = (viewport_height / content_height).clamp(0.1, 1.0);
        let thumb_height = (track_height * thumb_ratio).max(THUMB_MIN_HEIGHT);

        // 计算滑块位置
        let max_scroll = content_height - viewport_height;
        let scroll_ratio = if max_scroll > 0.0 {
            (scroll_position.y / max_scroll).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let max_thumb_top = track_height - thumb_height;
        let thumb_top = scroll_ratio * max_thumb_top;

        // 更新滑块样式（使用逻辑像素，Bevy 会自动处理 DPI 缩放）
        thumb_node.height = Val::Px(thumb_height);
        thumb_node.top = Val::Px(thumb_top);
    }
}

/// 滑块交互系统（悬停、按下状态）
pub fn scrollbar_thumb_interaction(
    mut thumb_query: Query<
        (&Interaction, &mut BackgroundColor, &ScrollbarThumb),
        Changed<Interaction>,
    >,
    scroll_query: Query<(Entity, &ScrollPosition)>,
    mut drag_state: ResMut<ScrollbarDragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    for (interaction, mut bg_color, thumb) in &mut thumb_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(THUMB_PRESSED_COLOR);

                // 获取当前滚动位置
                let scroll_y = scroll_query
                    .iter()
                    .find(|(entity, _)| *entity == thumb.scroll_container)
                    .map(|(_, sp)| sp.y)
                    .unwrap_or(0.0);

                // 获取鼠标位置并开始拖拽
                if let Some(cursor_pos) = window.cursor_position() {
                    drag_state.start_drag(thumb.scroll_container, cursor_pos.y, scroll_y);
                }
            }
            Interaction::Hovered => {
                if !drag_state.is_dragging {
                    *bg_color = BackgroundColor(THUMB_HOVER_COLOR);
                }
            }
            Interaction::None => {
                if !drag_state.is_dragging {
                    *bg_color = BackgroundColor(THUMB_COLOR);
                }
            }
        }
    }
}

/// 轨道点击跳转系统
/// 使用 GlobalTransform 和 ComputedNode 精确计算鼠标相对于轨道的位置
pub fn scrollbar_track_click(
    window_query: Query<&Window, With<PrimaryWindow>>,
    track_query: Query<
        (
            &ScrollbarTrack,
            &Interaction,
            &GlobalTransform,
            &ComputedNode,
        ),
        (Changed<Interaction>, With<ScrollbarTrack>),
    >,
    thumb_query: Query<(&ScrollbarThumb, &Node)>,
    mut scroll_query: Query<(
        Entity,
        &mut ScrollPosition,
        &ComputedNode,
        Option<&ContentSizeInfo>,
    )>,
    mut drag_state: ResMut<ScrollbarDragState>,
) {
    // 如果正在拖拽滑块，不处理轨道点击
    if drag_state.is_dragging {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let scale_factor = window.scale_factor() as f32;
    let window_height = window.height();

    for (track, interaction, track_transform, track_computed) in &track_query {
        // 只处理刚变为 Pressed 状态的轨道
        if *interaction != Interaction::Pressed {
            continue;
        }

        // ComputedNode::size() 返回物理像素，转换为逻辑像素
        let track_height = track_computed.size().y / scale_factor;
        if track_height <= 0.0 {
            continue;
        }

        // GlobalTransform 在 Bevy UI 中可能返回物理像素坐标，需要转换为逻辑像素
        // 注意：这是一个假设，需要通过日志验证
        let track_center_y = track_transform.translation().y / scale_factor;

        // 将鼠标从屏幕坐标转换为 Bevy UI 坐标
        // 屏幕坐标: (0, 0) 在左上角，Y向下
        // Bevy UI 坐标: (0, 0) 在窗口中心，Y向上
        let cursor_y_bevy = window_height / 2.0 - cursor_pos.y;

        // 轨道顶部 Y 坐标（Bevy 坐标系中，Y 增大 = 向上，所以顶部 = 中心 + 高度/2）
        let track_top_y = track_center_y + track_height / 2.0;

        // 点击位置相对于轨道顶部的偏移（向下为正）
        let click_offset_from_top = track_top_y - cursor_y_bevy;

        // 计算点击比例（0.0 = 顶部，1.0 = 底部）
        let click_ratio = (click_offset_from_top / track_height).clamp(0.0, 1.0);

        let scroll_container = track.scroll_container;

        // 检查滑块的当前位置和高度，判断点击是否在滑块范围内
        let mut thumb_top_ratio = 0.0;
        let mut thumb_bottom_ratio = 0.0;
        let mut has_thumb = false;

        for (thumb, thumb_node) in &thumb_query {
            if thumb.scroll_container != scroll_container {
                continue;
            }

            // 获取滑块的 top 和 height（都是 Val::Px 逻辑像素值）
            let thumb_top = match thumb_node.top {
                Val::Px(v) => v,
                _ => 0.0,
            };
            let thumb_height = match thumb_node.height {
                Val::Px(v) => v,
                _ => 0.0,
            };

            if thumb_height > 0.0 && track_height > 0.0 {
                thumb_top_ratio = thumb_top / track_height;
                thumb_bottom_ratio = (thumb_top + thumb_height) / track_height;
                has_thumb = true;
            }
            break;
        }

        // 如果点击在滑块范围内，开始拖拽（因为滑块接收不到事件，由轨道代为处理）
        if has_thumb && click_ratio >= thumb_top_ratio && click_ratio <= thumb_bottom_ratio {
            // 获取当前滚动位置
            let scroll_y = scroll_query
                .iter()
                .find(|(entity, _, _, _)| *entity == scroll_container)
                .map(|(_, sp, _, _)| sp.y)
                .unwrap_or(0.0);

            // 获取鼠标位置并开始拖拽
            if let Some(cursor_pos) = window.cursor_position() {
                drag_state.start_drag(scroll_container, cursor_pos.y, scroll_y);
            }
            continue;
        }

        // 查找对应的滚动容器并更新滚动位置
        for (entity, mut scroll_position, scroll_computed, content_size_info) in &mut scroll_query {
            if entity != scroll_container {
                continue;
            }

            let (content_height, viewport_height) = match content_size_info {
                Some(info) => (info.content_height, info.viewport_height),
                None => {
                    let outer_height = scroll_computed.size().y / scale_factor;
                    (outer_height, outer_height)
                }
            };

            let max_scroll = (content_height - viewport_height).max(0.0);
            let target_scroll = click_ratio * max_scroll;
            scroll_position.y = target_scroll.clamp(0.0, max_scroll);

            break;
        }
    }
}

/// 滑块拖拽系统
pub fn scrollbar_thumb_drag(
    mut drag_state: ResMut<ScrollbarDragState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    track_query: Query<(&ScrollbarTrack, &ComputedNode)>,
    thumb_query: Query<(&ScrollbarThumb, &ComputedNode)>,
    mut scroll_query: Query<(
        Entity,
        &mut ScrollPosition,
        &ComputedNode,
        Option<&ContentSizeInfo>,
    )>,
) {
    // 如果鼠标没有按下，不处理
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    // 如果没有在拖拽，返回
    if !drag_state.is_dragging {
        return;
    }

    let Some(scroll_container) = drag_state.dragging_thumb else {
        return;
    };

    let Ok(window) = window_query.single() else {
        return;
    };

    // 获取当前鼠标位置
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // 获取轨道信息
    let Some((_, track_computed)) = track_query
        .iter()
        .find(|(track, _)| track.scroll_container == scroll_container)
    else {
        return;
    };

    // 获取滑块信息
    let Some((_, thumb_computed)) = thumb_query
        .iter()
        .find(|(thumb, _)| thumb.scroll_container == scroll_container)
    else {
        return;
    };

    let scale_factor = get_scale_factor(&window_query);

    // 获取滚动容器
    for (entity, mut scroll_position, scroll_computed, content_size_info) in &mut scroll_query {
        if entity != scroll_container {
            continue;
        }

        // ComputedNode::size() 返回物理像素，转换为逻辑像素
        let track_height = track_computed.size().y / scale_factor;
        let thumb_height = thumb_computed.size().y / scale_factor;

        if track_height <= 0.0 || thumb_height <= 0.0 {
            continue;
        }

        // 计算滑块可移动范围
        let max_thumb_travel = track_height - thumb_height;
        if max_thumb_travel <= 0.0 {
            continue;
        }

        // 计算鼠标移动距离（cursor_pos 已经是逻辑像素）
        let mouse_delta = cursor_pos.y - drag_state.drag_start_y;

        // 获取内容高度和视口高度（已经是逻辑像素）
        let (content_height, viewport_height) = match content_size_info {
            Some(info) => (info.content_height, info.viewport_height),
            None => {
                let outer_height = scroll_computed.size().y / scale_factor;
                (outer_height, outer_height)
            }
        };

        let max_scroll = (content_height - viewport_height).max(0.0);

        // 将鼠标移动转换为滚动距离
        let scroll_per_pixel = max_scroll / max_thumb_travel;
        let scroll_delta = mouse_delta * scroll_per_pixel;

        // 更新滚动位置
        let new_scroll = drag_state.drag_start_scroll + scroll_delta;
        scroll_position.y = new_scroll.clamp(0.0, max_scroll);

        break;
    }
}

/// 重置拖拽状态（当鼠标释放时）
pub fn reset_drag_state_on_release(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<ScrollbarDragState>,
    mut thumb_query: Query<&mut BackgroundColor, With<ScrollbarThumb>>,
) {
    // 如果鼠标释放，结束拖拽
    if mouse_button.just_released(MouseButton::Left) && drag_state.is_dragging {
        drag_state.end_drag();

        // 重置滑块颜色
        for mut bg_color in &mut thumb_query {
            *bg_color = BackgroundColor(THUMB_COLOR);
        }
    }
}
