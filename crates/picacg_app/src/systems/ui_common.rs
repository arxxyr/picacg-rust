//! 通用 UI 构建函数
//!
//! 提取各页面共享的 UI 构建逻辑，避免代码重复。

use bevy::{prelude::*, ui::FocusPolicy};

use crate::systems::{
    ScrollbarContainer, ScrollbarThumb, ScrollbarTrack, login::AppColors,
    scrollbar::scrollbar_config::*,
};

// ==================== 标签徽章 ====================

/// 标签颜色类型
#[derive(Clone, Copy)]
pub enum TagColor {
    /// 分类（蓝色）
    Category,
    /// 标签（绿色）- 用于收藏和排行榜
    Tag,
    /// 标签（紫色）- 用于漫画列表和搜索
    TagPurple,
}

impl TagColor {
    /// 获取背景色和文字颜色
    #[must_use]
    pub fn colors(self) -> (Color, Color) {
        match self {
            Self::Category => (Color::srgba(0.2, 0.4, 0.8, 0.3), Color::srgb(0.6, 0.8, 1.0)),
            Self::Tag => (Color::srgba(0.2, 0.6, 0.4, 0.3), Color::srgb(0.5, 0.9, 0.7)),
            Self::TagPurple => (Color::srgba(0.6, 0.3, 0.6, 0.3), Color::srgb(0.9, 0.7, 0.9)),
        }
    }
}

/// 创建标签徽章
pub fn spawn_tag_badge(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font: &Handle<Font>,
    color_type: TagColor,
) {
    let (bg_color, text_color) = color_type.colors();

    parent
        .spawn((
            Node {
                padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(1.0), Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
        ))
        .with_children(|badge| {
            badge.spawn((
                Text::new(text),
                TextFont {
                    font: font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

/// 创建带截断的标签徽章
pub fn spawn_tag_badge_truncated(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font: &Handle<Font>,
    color_type: TagColor,
    max_chars: usize,
) {
    let display_text = truncate_text(text, max_chars);
    let (bg_color, text_color) = color_type.colors();

    parent
        .spawn((
            Node {
                padding: UiRect::new(Val::Px(3.0), Val::Px(3.0), Val::Px(1.0), Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
        ))
        .with_children(|badge| {
            badge.spawn((
                Text::new(display_text),
                TextFont {
                    font: font.clone(),
                    font_size: 9.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

// ==================== 滚动条 ====================

/// 创建滚动条（通用实现）
///
/// 布局结构：
/// ScrollbarContainer (Absolute, right=0)
///   ├── ScrollbarTrack (Button, fills 100%, ZIndex=0)
///   └── ScrollbarThumb (Button, Absolute, ZIndex=1)
pub fn spawn_scrollbar(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
    parent
        .spawn((
            ScrollbarContainer { scroll_container },
            Node {
                width: Val::Px(SCROLLBAR_WIDTH),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(10),
            Transform::default(),
        ))
        .with_children(|scrollbar| {
            // 滚动条轨道
            scrollbar.spawn((
                ScrollbarTrack { scroll_container },
                Button,
                Interaction::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(TRACK_COLOR),
                ZIndex(0),
                Transform::default(),
            ));

            // 滚动条滑块
            scrollbar.spawn((
                ScrollbarThumb { scroll_container },
                Button,
                Interaction::default(),
                FocusPolicy::Block,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(THUMB_MIN_HEIGHT),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    border_radius: BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                    ..default()
                },
                BackgroundColor(THUMB_COLOR),
                ZIndex(1),
            ));
        });
}

// ==================== 滚动处理 ====================

/// 计算滚动增量（统一处理 Line 和 Pixel 单位）
#[must_use]
pub fn calculate_scroll_delta(event: &bevy::input::mouse::MouseWheel) -> f32 {
    match event.unit {
        bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
        bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
    }
}

// ==================== 内容尺寸计算 ====================

/// 网格布局参数
pub struct GridLayoutParams {
    pub card_width: f32,
    pub card_height: f32,
    pub column_gap: f32,
    pub row_gap: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
}

/// 计算网格布局的内容高度
#[must_use]
pub fn calculate_grid_content_height(
    viewport_width: f32,
    card_count: usize,
    params: &GridLayoutParams,
) -> f32 {
    if card_count == 0 {
        return 0.0;
    }

    let available_width = viewport_width - params.padding_left - params.padding_right;
    let card_with_gap = params.card_width + params.column_gap;
    let columns = ((available_width + params.column_gap) / card_with_gap)
        .floor()
        .max(1.0) as usize;
    let rows = card_count.div_ceil(columns);

    params.padding_top
        + (rows as f32) * params.card_height
        + ((rows.saturating_sub(1)) as f32) * params.row_gap
        + params.padding_bottom
}

// ==================== 文本工具 ====================

/// 截断文本
#[must_use]
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        format!("{}...", text.chars().take(max_chars).collect::<String>())
    } else {
        text.to_string()
    }
}

/// 格式化数字（支持万和k）
#[must_use]
pub fn format_number(n: i64) -> String {
    if n >= 10000 {
        format!("{:.1}万", n as f64 / 10000.0)
    } else if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}
