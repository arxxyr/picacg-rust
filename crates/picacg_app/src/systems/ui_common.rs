//! 通用 UI 构建函数
//!
//! 提取各页面共享的 UI 构建逻辑，避免代码重复。

use bevy::{
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
    window::PrimaryWindow,
};

use crate::{
    components::{ContentSizeInfo, ContextMenuTarget},
    events::DownloadComicRequest,
    systems::{
        ScrollbarContainer, ScrollbarThumb, ScrollbarTrack, login::AppColors,
        scrollbar::scrollbar_config::*,
    },
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

/// 可滚动容器标记（所有需要鼠标滚轮滚动的容器都应添加此组件）
#[derive(Component)]
pub struct Scrollable;

/// 计算滚动增量（统一处理 Line 和 Pixel 单位）
#[must_use]
pub fn calculate_scroll_delta(event: &bevy::input::mouse::MouseWheel) -> f32 {
    match event.unit {
        bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
        bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
    }
}

/// 从子节点计算滚动容器的实际内容高度
///
/// 统一处理 margin / row_gap / padding，避免每个模块各自补偿。
fn compute_content_height(
    container_node: &Node,
    children: &Children,
    children_query: &Query<(&ComputedNode, &Node)>,
    scale: f32,
) -> f32 {
    let mut h = 0.0_f32;
    for child in children.iter() {
        if let Ok((cn, child_node)) = children_query.get(child) {
            h += cn.size().y / scale;
            // 加上子节点的上下 margin（ComputedNode::size 不含 margin）
            if let Val::Px(mt) = child_node.margin.top {
                h += mt;
            }
            if let Val::Px(mb) = child_node.margin.bottom {
                h += mb;
            }
        }
    }
    // row_gap（子节点间距）
    if children.len() > 1 {
        let gap = match container_node.row_gap {
            Val::Px(px) => px,
            _ => 0.0,
        };
        h += gap * (children.len() - 1) as f32;
    }
    // 容器的 padding
    if let Val::Px(pt) = container_node.padding.top {
        h += pt;
    }
    if let Val::Px(pb) = container_node.padding.bottom {
        h += pb;
    }
    h
}

/// 全局滚轮分发系统
///
/// 根据光标 X 坐标判断在侧边栏还是内容区域，分发滚轮事件到对应容器。
/// 内容高度统一从子节点计算（含 margin/gap/padding），不依赖各模块的
/// ContentSizeInfo。 仅 flex-wrap 网格布局的页面使用
/// ContentSizeInfo（网格高度无法从子节点直接累加）。
pub fn global_scroll_dispatch(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut sidebar_scroll: Query<
        (&Node, &mut ScrollPosition, &ComputedNode, &Children),
        With<super::main_layout::SidebarMenuArea>,
    >,
    mut content_scroll: Query<
        (
            &Node,
            &mut ScrollPosition,
            &ComputedNode,
            Option<&ContentSizeInfo>,
            Option<&Children>,
        ),
        (
            With<Scrollable>,
            Without<super::main_layout::SidebarMenuArea>,
        ),
    >,
    children_query: Query<(&ComputedNode, &Node)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let Some(window) = window_query.single().ok() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let scale = window.scale_factor();

    // 收集本帧所有滚轮事件的总增量
    let mut total_delta = 0.0_f32;
    for event in mouse_wheel_events.read() {
        total_delta += calculate_scroll_delta(event);
    }
    if total_delta == 0.0 {
        return;
    }

    let sidebar_width = super::main_layout::SIDEBAR_WIDTH;

    if cursor_pos.x < sidebar_width {
        // 侧边栏区域
        for (node, mut scroll_pos, computed, children) in sidebar_scroll.iter_mut() {
            let viewport_h = computed.size().y / scale;
            let content_h = compute_content_height(node, children, &children_query, scale);
            let max_scroll = (content_h - viewport_h).max(0.0);
            scroll_pos.y = (scroll_pos.y - total_delta).clamp(0.0, max_scroll);
        }
    } else {
        // 内容区域
        for (node, mut scroll_pos, computed, content_info, children) in content_scroll.iter_mut() {
            if node.display == Display::None {
                continue;
            }
            let size = computed.size();
            if size.y < 1.0 {
                continue;
            }
            let viewport_h = size.y / scale;

            // flex-wrap 网格布局使用 ContentSizeInfo（子节点不是简单纵向排列）
            // 普通 column 布局从子节点精确计算
            let content_h = match (content_info, children) {
                (Some(info), _)
                    if info.content_height > 0.0 && node.flex_wrap != FlexWrap::NoWrap =>
                {
                    // 网格布局：信任各模块计算的 ContentSizeInfo
                    info.content_height
                }
                (_, Some(ch)) => {
                    // 纵向布局：从子节点精确计算
                    compute_content_height(node, ch, &children_query, scale)
                }
                (Some(info), None) if info.content_height > 0.0 => info.content_height,
                _ => 0.0,
            };

            let max_scroll = (content_h - viewport_h).max(0.0);
            scroll_pos.y = (scroll_pos.y - total_delta).clamp(0.0, max_scroll);
        }
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

// ==================== 全局右键菜单系统 ====================

/// 右键菜单根节点
#[derive(Component)]
pub struct ComicContextMenu;

/// 右键菜单项类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuAction {
    Download,
    Block,
}

/// 右键菜单项
#[derive(Component)]
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

    let font = super::font_loader::get_font();
    let comic_id = target.comic_id.clone();
    let comic_title = target.comic_title.clone();

    // 创建菜单
    commands
        .spawn((
            ComicContextMenu,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cursor.x),
                top: Val::Px(cursor.y),
                min_width: Val::Px(140.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            GlobalZIndex(100),
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|menu| {
            // 下载按钮
            spawn_context_menu_item(
                menu,
                &font,
                &format!("{} 下载", crate::utils::icons::ICON_DOWNLOAD),
                ContextMenuAction::Download,
                &comic_id,
                &comic_title,
            );

            // 分割线
            menu.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(AppColors::BORDER),
            ));

            // 屏蔽按钮
            spawn_context_menu_item(
                menu,
                &font,
                &format!("{} 屏蔽", crate::utils::icons::ICON_EYE_OFF),
                ContextMenuAction::Block,
                &comic_id,
                &comic_title,
            );
        });
}

/// 创建菜单项
fn spawn_context_menu_item(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    action: ContextMenuAction,
    comic_id: &str,
    comic_title: &str,
) {
    parent
        .spawn((
            ComicContextMenuItem {
                action,
                comic_id: comic_id.to_string(),
                comic_title: comic_title.to_string(),
            },
            Button,
            Interaction::default(),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 处理右键菜单项点击
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
