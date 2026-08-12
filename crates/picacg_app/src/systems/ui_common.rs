//! 通用 UI 构建函数
//!
//! 提取各页面共享的 UI 构建逻辑，避免代码重复。

use bevy::{
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
    window::PrimaryWindow,
};

use crate::{
    components::ContextMenuTarget,
    events::DownloadComicRequest,
    systems::{login::AppColors, scrollbar::scrollbar_config::*},
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

// ==================== 标签徽章（BSN 场景版） ====================

/// 标签徽章场景
pub fn tag_badge(text: &str, color_type: TagColor) -> impl Scene + use<> {
    let (bg_color, text_color) = color_type.colors();
    let text = text.to_string();

    // 单实体徽章：Text 节点自带 padding/圆角/底色
    bsn! {
        Text({text})
        TextFont { font_size: FontSize::Px(10.0) }
        TextColor(text_color)
        Node {
            padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(1.0), Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
        }
        BackgroundColor(bg_color)
    }
}

/// 带截断的标签徽章场景
pub fn tag_badge_truncated(
    text: &str,
    color_type: TagColor,
    max_chars: usize,
) -> impl Scene + use<> {
    let display_text = truncate_text(text, max_chars);
    let (bg_color, text_color) = color_type.colors();

    // 单实体徽章：Text 节点自带 padding/圆角/底色
    bsn! {
        Text({display_text})
        TextFont { font_size: FontSize::Px(9.0) }
        TextColor(text_color)
        Node {
            padding: UiRect::new(Val::Px(3.0), Val::Px(3.0), Val::Px(1.0), Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
        }
        BackgroundColor(bg_color)
    }
}

// ==================== 滚动条 ====================

// ==================== 滚动处理 ====================

/// 时间戳 → 本地时间字符串（history/like_records 共用；原两份字节级相同的拷贝）
/// 格式化时间戳为可读字符串
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::{Local, TimeZone};

    if timestamp == 0 {
        return "未知时间".to_string();
    }

    match Local.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => "未知时间".to_string(),
    }
}

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

/// 漫画卡片时间信息场景（两者皆 None 时返回空列表）
pub fn comic_time_info(created_at: Option<&str>, updated_at: Option<&str>) -> Box<dyn SceneList> {
    if created_at.is_none() && updated_at.is_none() {
        return Box::new(bsn_list![]);
    }

    let mut rows: Vec<Box<dyn Scene>> = Vec::new();
    if let Some(updated) = updated_at {
        let label = format!("更新 {}", format_api_date(updated));
        rows.push(Box::new(bsn! {
            Text({label})
            TextFont { font_size: FontSize::Px(9.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    }
    if let Some(created) = created_at {
        let label = format!("创建 {}", format_api_date(created));
        rows.push(Box::new(bsn! {
            Text({label})
            TextFont { font_size: FontSize::Px(9.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    }

    Box::new(bsn_list![(
        Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(Val::Px(2.0)),
            max_width: Val::Px(164.0),
            overflow: Overflow::clip(),
        }
        Children [ {rows} ]
    )])
}

// ==================== 全局右键菜单系统 ====================

/// 右键菜单根节点
#[derive(Component, Default, Clone)]
pub struct ComicContextMenu;

/// 右键菜单项类型
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextMenuAction {
    #[default]
    Download,
    Block,
}

/// 右键菜单项
#[derive(Component, Default, Clone)]
pub struct ComicContextMenuItem {
    pub action: ContextMenuAction,
    pub comic_id: String,
    pub comic_title: String,
}

/// 检测漫画卡片上的右键点击，弹出上下文菜单（全局，作用于所有带
/// ContextMenuTarget 的卡片）
pub fn comic_card_context_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&ContextMenuTarget, &Interaction)>,
    existing_menu: Query<Entity, With<ComicContextMenu>>,
) {
    // 右键刚按下
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    // 关闭已有菜单
    for entity in existing_menu.iter() {
        commands.entity(entity).despawn();
    }

    // 找到悬停中的卡片
    let hovered_card = card_query
        .iter()
        .find(|(_, interaction)| **interaction == Interaction::Hovered);

    let Some((target, _)) = hovered_card else {
        return;
    };

    // 获取光标位置
    let Some(window) = window_query.single().ok() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let comic_id = target.comic_id.clone();
    let comic_title = target.comic_title.clone();

    // 创建菜单
    commands.spawn_scene(context_menu(cursor, &comic_id, &comic_title));
}

/// 右键菜单场景
fn context_menu(cursor: Vec2, comic_id: &str, comic_title: &str) -> impl Scene + use<> {
    let download_label = format!("{} 下载", crate::utils::icons::ICON_DOWNLOAD);
    let block_label = format!("{} 屏蔽", crate::utils::icons::ICON_EYE_OFF);
    let download_item = ComicContextMenuItem {
        action: ContextMenuAction::Download,
        comic_id: comic_id.to_string(),
        comic_title: comic_title.to_string(),
    };
    let block_item = ComicContextMenuItem {
        action: ContextMenuAction::Block,
        comic_id: comic_id.to_string(),
        comic_title: comic_title.to_string(),
    };
    let (x, y) = (cursor.x, cursor.y);

    bsn! {
        ComicContextMenu
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(y),
            min_width: Val::Px(140.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        GlobalZIndex(100)
        BackgroundColor(Color::srgb(0.12, 0.12, 0.16))
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            // 下载按钮
            context_menu_item(download_label, download_item),
            (
                // 分割线
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(3.0)),
                }
                BackgroundColor(AppColors::BORDER)
            ),
            // 屏蔽按钮
            context_menu_item(block_label, block_item),
        ]
    }
}

/// 菜单项场景
fn context_menu_item(label: String, item: ComicContextMenuItem) -> impl Scene + use<> {
    bsn! {
        template_value(item)
        Button
        Interaction
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::NONE)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 创建菜单项/// 处理右键菜单项点击
pub fn comic_context_menu_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ComicContextMenuItem),
        Changed<Interaction>,
    >,
    menu_query: Query<Entity, With<ComicContextMenu>>,
    mut download_messages: MessageWriter<DownloadComicRequest>,
) {
    for (interaction, mut bg_color, item) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match item.action {
                    ContextMenuAction::Download => {
                        download_messages.write(DownloadComicRequest {
                            comic_id: item.comic_id.clone(),
                            comic_title: item.comic_title.clone(),
                            episodes: vec![], // 空 = 下载全部
                        });
                        tracing::info!("右键菜单：下载漫画 {}", item.comic_title);
                    }
                    ContextMenuAction::Block => {
                        // 将标题添加到屏蔽词
                        let title = item.comic_title.clone();
                        if !title.is_empty() {
                            let settings = picacg_config::AppSettings::global();
                            let mut s = settings.write();
                            if !s.filter.blocked_keywords.contains(&title) {
                                s.filter.blocked_keywords.push(title.clone());
                                if let Err(e) = s.save() {
                                    tracing::error!("保存屏蔽设置失败: {}", e);
                                } else {
                                    tracing::info!("右键菜单：已屏蔽「{}」", title);
                                }
                            }
                        }
                    }
                }
                // 关闭菜单
                for entity in menu_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 点击菜单外区域关闭菜单
pub fn dismiss_context_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    menu_query: Query<Entity, With<ComicContextMenu>>,
    menu_item_query: Query<&Interaction, With<ComicContextMenuItem>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    // 如果有菜单项被悬停/按下，说明点的是菜单内部，不关闭
    let hovering_menu = menu_item_query.iter().any(|i| *i != Interaction::None);
    if hovering_menu {
        return;
    }
    // 关闭所有菜单
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}
