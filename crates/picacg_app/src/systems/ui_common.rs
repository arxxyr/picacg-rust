//! 通用 UI 构建函数
//!
//! 提取各页面共享的 UI 构建逻辑，避免代码重复。

use bevy::{
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
};

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
}

impl TagColor {
    /// 获取背景色和文字颜色
    #[must_use]
    pub fn colors(self) -> (Color, Color) {
        match self {
            Self::Category => (Color::srgba(0.2, 0.4, 0.8, 0.3), Color::srgb(0.6, 0.8, 1.0)),
            Self::Tag => (Color::srgba(0.2, 0.6, 0.4, 0.3), Color::srgb(0.5, 0.9, 0.7)),
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
                RelativeCursorPosition::default(),
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
    pub column_gap: f32,
    pub row_gap: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
}

/// 通过测量子节点的实际渲染高度计算 flex-wrap 网格的内容高度
///
/// 读取每个子节点的 `ComputedNode::size()` 获取实际渲染高度，
/// 按行分组取最大值，避免因卡片高度不一致导致滚动条位置偏移。
#[must_use]
pub fn measure_grid_content_height(
    children: Option<&Children>,
    child_computed_query: &Query<&ComputedNode>,
    scale_factor: f32,
    viewport_width: f32,
    params: &GridLayoutParams,
) -> f32 {
    let Some(children) = children else {
        return 0.0;
    };

    // 收集所有子节点的实际渲染高度
    let mut child_heights: Vec<f32> = Vec::new();
    for child in children.iter() {
        if let Ok(child_computed) = child_computed_query.get(child) {
            let h = child_computed.size().y / scale_factor;
            if h > 0.0 {
                child_heights.push(h);
            }
        }
    }

    if child_heights.is_empty() {
        return 0.0;
    }

    // 计算列数（与 Bevy flex-wrap 布局一致）
    let available_width = viewport_width - params.padding_left - params.padding_right;
    let card_with_gap = params.card_width + params.column_gap;
    let columns = ((available_width + params.column_gap) / card_with_gap)
        .floor()
        .max(1.0) as usize;

    // 按行分组，每行取最大高度
    let mut content_height = params.padding_top;
    let mut row_count = 0usize;
    for row in child_heights.chunks(columns) {
        content_height += row.iter().copied().fold(0.0_f32, f32::max);
        row_count += 1;
    }
    content_height += (row_count.saturating_sub(1) as f32) * params.row_gap;
    content_height += params.padding_bottom;

    content_height
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

/// 格式化 API 返回的 ISO 8601 时间字符串为日期
///
/// `"2023-01-01T12:00:00.000Z"` → `"2023-01-01"`
#[must_use]
pub fn format_api_date(iso_str: &str) -> &str {
    iso_str.split('T').next().unwrap_or(iso_str)
}

/// 在漫画卡片中显示创建/更新时间
pub fn spawn_comic_time_info(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    created_at: Option<&str>,
    updated_at: Option<&str>,
) {
    if created_at.is_none() && updated_at.is_none() {
        return;
    }

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(Val::Px(2.0)),
            max_width: Val::Px(164.0),
            overflow: Overflow::clip(),
            ..default()
        })
        .with_children(|container| {
            if let Some(updated) = updated_at {
                let date = format_api_date(updated);
                container.spawn((
                    Text::new(format!("更新 {date}")),
                    TextFont {
                        font: font.clone(),
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            }
            if let Some(created) = created_at {
                let date = format_api_date(created);
                container.spawn((
                    Text::new(format!("创建 {date}")),
                    TextFont {
                        font: font.clone(),
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            }
        });
}
