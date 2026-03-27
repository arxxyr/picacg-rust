//! 本地阅读系统
//!
//! 扫描下载目录，列出所有已下载漫画文件夹，支持离线浏览

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::scrollbar_config::SCROLLBAR_WIDTH,
        ui_common::{Scrollable, spawn_scrollbar},
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 本地阅读页面根节点
#[derive(Component)]
pub struct LocalReadRoot;

/// 本地阅读滚动容器
#[derive(Component)]
pub struct LocalReadScrollContainer;

/// 本地漫画卡片
#[derive(Component)]
pub struct LocalComicCard {
    /// 漫画文件夹路径
    #[allow(dead_code)]
    pub path: String,
}

/// 扫描按钮
#[derive(Component)]
pub struct LocalReadScanButton;

/// 本地漫画封面图片
#[derive(Component)]
pub struct LocalComicCoverImage {
    /// 封面图片路径
    pub path: String,
    /// 是否已加载
    pub loaded: bool,
}

/// 空状态提示
#[derive(Component)]
pub struct LocalReadEmptyHint;

/// 打开文件夹按钮
#[derive(Component)]
pub struct OpenLocalFolderButton {
    pub path: String,
}

// ==================== 布局常量 ====================

mod local_read_layout {
    /// 卡片高度
    pub const CARD_HEIGHT: f32 = 100.0;
    /// 卡片间距
    pub const CARD_GAP: f32 = 8.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 15.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
    /// 封面宽度
    pub const COVER_WIDTH: f32 = 70.0;
    /// 封面高度
    pub const COVER_HEIGHT: f32 = 85.0;
}

// ==================== 系统函数 ====================

/// 创建本地阅读页面 UI（如果已存在则只显示）
pub fn setup_local_read_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    local_read_state: Res<LocalReadState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut scan_messages: MessageWriter<ScanLocalComicsRequest>,
    mut existing_query: Query<&mut Node, With<LocalReadRoot>>,
) {
    // 如果 LocalReadRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if local_read_state.entries.is_empty() && !local_read_state.is_scanning {
            scan_messages.write(ScanLocalComicsRequest);
        }
        return;
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let local_read_root = commands
        .spawn((
            LocalReadRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // 标题栏
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|header| {
                // 左侧标题
                header.spawn((
                    Text::new("本地阅读"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 右侧扫描按钮
                header
                    .spawn((
                        LocalReadScanButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(12.0),
                                Val::Px(6.0),
                                Val::Px(6.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                        BorderColor::all(AppColors::PRIMARY),
                        BackgroundColor(AppColors::PRIMARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_REFRESH),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        btn.spawn((
                            Text::new("扫描"),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });

            // 滚动区域包装器
            root.spawn((Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                ..default()
            },))
                .with_children(|wrapper| {
                    // 漫画列表（可滚动）
                    let scroll_container_id = wrapper
                        .spawn((
                            LocalReadScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect {
                                    left: Val::Px(local_read_layout::PADDING_LEFT),
                                    right: Val::Px(local_read_layout::PADDING_RIGHT),
                                    top: Val::Px(local_read_layout::PADDING_TOP),
                                    bottom: Val::Px(local_read_layout::PADDING_BOTTOM),
                                },
                                row_gap: Val::Px(local_read_layout::CARD_GAP),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            Scrollable,
                            ScrollPosition::default(),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|list| {
                            if local_read_state.is_scanning {
                                // 扫描中提示
                                list.spawn((
                                    LoadingIndicator,
                                    Text::new("正在扫描本地漫画..."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT),
                                ));
                            } else if local_read_state.entries.is_empty()
                                && local_read_state.error.is_none()
                            {
                                // 空状态提示
                                spawn_empty_hint(list, &font);
                            }
                        })
                        .id();

                    // 创建滚动条
                    spawn_scrollbar(wrapper, scroll_container_id);
                });
        })
        .id();

    // 如果有 ContentArea，将本地阅读页面作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(local_read_root);
    }

    // 首次进入时自动扫描
    if local_read_state.entries.is_empty() && !local_read_state.is_scanning {
        scan_messages.write(ScanLocalComicsRequest);
    }

    tracing::info!("本地阅读页面 UI 已创建");
}

/// 清理本地阅读页面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_local_read_ui(mut query: Query<&mut Node, With<LocalReadRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 创建空状态提示
fn spawn_empty_hint(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            LocalReadEmptyHint,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::vertical(Val::Px(60.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
        ))
        .with_children(|hint| {
            // 图标
            hint.spawn((
                Text::new(ICON_INBOX),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            // 主提示
            hint.spawn((
                Text::new("暂无本地漫画"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            // 副提示
            hint.spawn((
                Text::new("下载漫画后，在此处离线浏览"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.7)),
            ));
        });
}

/// 创建单个本地漫画卡片
fn spawn_local_comic_card(
    parent: &mut ChildSpawnerCommands,
    entry: &LocalComicEntry,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            LocalComicCard {
                path: entry.path.clone(),
            },
            Button,
            Interaction::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(local_read_layout::CARD_HEIGHT),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 1.0)),
        ))
        .with_children(|card| {
            // 封面占位/图片
            card.spawn((
                Node {
                    width: Val::Px(local_read_layout::COVER_WIDTH),
                    height: Val::Px(local_read_layout::COVER_HEIGHT),
                    min_width: Val::Px(local_read_layout::COVER_WIDTH),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 1.0)),
            ))
            .with_children(|cover_area| {
                if let Some(ref cover_path) = entry.cover_path {
                    // 带封面图片的占位（图片加载由 update 系统处理）
                    cover_area.spawn((
                        LocalComicCoverImage {
                            path: cover_path.clone(),
                            loaded: false,
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 1.0)),
                    ));
                } else {
                    // 无封面时显示图标
                    cover_area.spawn((
                        Text::new(ICON_BOOK),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                }
            });

            // 右侧信息区域
            card.spawn((Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(6.0),
                overflow: Overflow::clip(),
                ..default()
            },))
                .with_children(|info| {
                    // 漫画名称
                    info.spawn((
                        Text::new(&entry.name),
                        TextFont {
                            font: font.clone(),
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));

                    // 章节数
                    info.spawn((
                        Text::new(format!("{} 个章节", entry.chapter_count)),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));

                    // 路径（缩略显示）
                    let display_path = truncate_path(&entry.path, 50);
                    info.spawn((
                        Text::new(display_path),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.4, 0.4, 0.5, 0.8)),
                    ));
                });

            // 右侧操作区域
            card.spawn((Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|actions| {
                    // 打开文件夹按钮
                    actions
                        .spawn((
                            OpenLocalFolderButton {
                                path: entry.path.clone(),
                            },
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::all(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.0),
                                ..default()
                            },
                            BorderColor::all(AppColors::BORDER),
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(ICON_FOLDER_OPEN),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::PRIMARY),
                            ));
                            btn.spawn((
                                Text::new("打开"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        });
                });
        });
}

/// 刷新本地阅读列表 UI（响应数据变化）
pub fn refresh_local_read_ui(
    mut commands: Commands,
    local_read_state: Res<LocalReadState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<LocalReadScrollContainer>>,
    card_query: Query<&LocalComicCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    empty_hint_query: Query<Entity, With<LocalReadEmptyHint>>,
) {
    if !local_read_state.is_changed() {
        return;
    }

    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 如果有错误，显示错误信息
    if let Some(ref error) = local_read_state.error {
        // 删除加载指示器和空状态提示
        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in empty_hint_query.iter() {
            commands.entity(entity).despawn();
        }

        let font: Handle<Font> = get_font();
        commands.entity(scroll_entity).with_children(|parent| {
            parent.spawn((
                ErrorMessage,
                Text::new(format!("扫描失败: {}", error)),
                TextFont {
                    font,
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.3, 0.3)),
            ));
        });
        return;
    }

    // 如果正在扫描，不做操作（setup 时已显示加载指示器）
    if local_read_state.is_scanning {
        return;
    }

    // 检查是否已有卡片
    let has_cards = children
        .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
        .unwrap_or(false);

    // 数据为空，显示空状态
    if local_read_state.entries.is_empty() {
        if !has_cards && empty_hint_query.is_empty() && loading_query.is_empty() {
            // 先删除加载指示器
            for entity in loading_query.iter() {
                commands.entity(entity).despawn();
            }
            let font: Handle<Font> = get_font();
            commands.entity(scroll_entity).with_children(|parent| {
                spawn_empty_hint(parent, &font);
            });
        }
        return;
    }

    // 有数据但已有卡片，不重复创建
    if has_cards {
        return;
    }

    // 删除加载指示器和空状态提示
    for entity in loading_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in empty_hint_query.iter() {
        commands.entity(entity).despawn();
    }

    // 创建所有漫画卡片
    let font: Handle<Font> = get_font();
    commands.entity(scroll_entity).with_children(|parent| {
        for entry in local_read_state.entries.iter() {
            spawn_local_comic_card(parent, entry, &font);
        }
    });
}

/// 扫描按钮交互
pub fn local_read_scan_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<LocalReadScanButton>),
    >,
    local_read_state: Res<LocalReadState>,
    mut scan_messages: MessageWriter<ScanLocalComicsRequest>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 如果未在扫描中，发送扫描请求
                if !local_read_state.is_scanning {
                    scan_messages.write(ScanLocalComicsRequest);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
        }
    }
}

/// 漫画卡片悬停交互（视觉反馈）
pub fn local_comic_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<LocalComicCard>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.18, 0.18, 0.24, 1.0));
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 1.0));
                *border_color = BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 1.0));
                *border_color = BorderColor::all(AppColors::BORDER);
            }
        }
    }
}

/// 打开文件夹按钮交互
pub fn open_local_folder_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &OpenLocalFolderButton,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, mut bg_color, mut border_color, btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.28, 1.0));
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 用系统文件管理器打开目录
                open_directory(&btn.path);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 1.0));
                *border_color = BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
                *border_color = BorderColor::all(AppColors::BORDER);
            }
        }
    }
}

/// 处理扫描请求（异步扫描下载目录）
pub fn handle_scan_local_comics(
    runtime: ResMut<crate::utils::TokioTasksRuntime>,
    mut messages: MessageReader<ScanLocalComicsRequest>,
    mut local_read_state: ResMut<LocalReadState>,
) {
    for _ in messages.read() {
        if local_read_state.is_scanning {
            continue;
        }

        tracing::info!("开始扫描本地已下载漫画...");
        local_read_state.is_scanning = true;
        local_read_state.error = None;
        local_read_state.entries.clear();

        // 获取下载根目录
        let download_base = crate::systems::downloads::get_download_base_path();
        let images_dir = download_base.join("Images");

        runtime.spawn_background_task(|mut ctx| async move {
            let result = scan_local_comics(images_dir).await;

            ctx.run_on_main_thread(move |ctx| match result {
                Ok(entries) => {
                    ctx.world
                        .write_message(ScanLocalComicsCompletedEvent { entries });
                }
                Err(error) => {
                    ctx.world
                        .write_message(ScanLocalComicsFailedEvent { error });
                }
            })
            .await;
        });
    }
}

/// 处理扫描完成事件
pub fn handle_scan_completed(
    mut messages: MessageReader<ScanLocalComicsCompletedEvent>,
    mut local_read_state: ResMut<LocalReadState>,
    mut commands: Commands,
    scroll_container_query: Query<(Entity, Option<&Children>), With<LocalReadScrollContainer>>,
) {
    for event in messages.read() {
        tracing::info!("本地漫画扫描完成，找到 {} 部漫画", event.entries.len());

        // 扫描完成时清除旧的 UI 元素（卡片、加载指示器、空状态、错误提示）
        if let Ok((_scroll_entity, Some(children))) = scroll_container_query.single() {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 更新状态（触发 refresh_local_read_ui 重建卡片）
        local_read_state.is_scanning = false;
        local_read_state.entries = event.entries.clone();
        local_read_state.error = None;
    }
}

/// 处理扫描失败事件
pub fn handle_scan_failed(
    mut messages: MessageReader<ScanLocalComicsFailedEvent>,
    mut local_read_state: ResMut<LocalReadState>,
) {
    for event in messages.read() {
        tracing::error!("本地漫画扫描失败: {}", event.error);
        local_read_state.is_scanning = false;
        local_read_state.error = Some(event.error.clone());
    }
}

/// 处理滚动事件
pub fn handle_local_read_scroll(
    _scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<LocalReadScrollContainer>,
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新内容尺寸信息
pub fn update_local_read_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<LocalReadScrollContainer>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        let mut content_height = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(child) {
                content_height += child_computed.size().y / scale_factor;
            }
        }

        // 加上间距
        let gap_count = children.len().saturating_sub(1);
        content_height += gap_count as f32 * local_read_layout::CARD_GAP;

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 加载本地封面图片
pub fn update_local_cover_images(
    mut commands: Commands,
    mut cover_query: Query<(Entity, &mut LocalComicCoverImage)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, mut cover) in cover_query.iter_mut() {
        if cover.loaded {
            continue;
        }

        // 检查封面文件是否存在
        let cover_path = std::path::Path::new(&cover.path);
        if cover_path.exists() {
            // 使用 Bevy 的 AssetServer 加载本地图片
            let image_handle: Handle<Image> = asset_server.load(cover.path.clone());
            commands.entity(entity).insert(ImageNode {
                image: image_handle,
                ..default()
            });
            cover.loaded = true;
        } else {
            cover.loaded = true; // 标记为已处理，避免重复尝试
        }
    }
}

// ==================== 辅助函数 ====================

/// 异步扫描本地已下载漫画目录
async fn scan_local_comics(images_dir: std::path::PathBuf) -> Result<Vec<LocalComicEntry>, String> {
    use tokio::fs;

    // 检查 Images 目录是否存在
    if !images_dir.exists() {
        tracing::info!("下载目录不存在: {}，尝试扫描父目录", images_dir.display());
        // 如果 Images 目录不存在，返回空列表
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    let mut dir = fs::read_dir(&images_dir)
        .await
        .map_err(|e| format!("读取目录失败: {}", e))?;

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| format!("遍历目录失败: {}", e))?
    {
        let path = entry.path();

        // 跳过非目录
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知")
            .to_string();

        // 统计子文件夹数量（章节数）
        let mut chapter_count = 0;
        let mut cover_path: Option<String> = None;

        if let Ok(mut sub_dir) = fs::read_dir(&path).await {
            while let Ok(Some(sub_entry)) = sub_dir.next_entry().await {
                let sub_path = sub_entry.path();
                if sub_path.is_dir() {
                    chapter_count += 1;

                    // 尝试找第一个章节的第一张图片作为封面
                    if cover_path.is_none() {
                        cover_path = find_first_image(&sub_path).await;
                    }
                }
            }
        }

        // 如果没有子文件夹（可能漫画图片直接放在根目录）
        // 也检查根目录是否有图片
        if chapter_count == 0
            && let Some(img) = find_first_image(&path).await
        {
            cover_path = Some(img);
            // 如果根目录有图片但没有子文件夹，当作 1 个章节
            chapter_count = 1;
        }

        // 只添加有内容的文件夹
        if chapter_count > 0 || has_any_content(&path).await {
            entries.push(LocalComicEntry {
                name,
                path: path.to_string_lossy().to_string(),
                cover_path,
                chapter_count,
            });
        }
    }

    // 按名称排序
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    tracing::info!(
        "扫描完成: 在 {} 中找到 {} 部本地漫画",
        images_dir.display(),
        entries.len()
    );

    Ok(entries)
}

/// 在目录中查找第一张图片文件
async fn find_first_image(dir: &std::path::Path) -> Option<String> {
    use tokio::fs;

    let mut entries = fs::read_dir(dir).await.ok()?;
    let mut image_paths = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if is_image_file(&path) {
            image_paths.push(path);
        }
    }

    // 排序确保获取第一张
    image_paths.sort();
    image_paths.first().map(|p| p.to_string_lossy().to_string())
}

/// 检查文件是否为图片
fn is_image_file(path: &std::path::Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
    )
}

/// 检查目录是否有任何内容（文件或子目录）
async fn has_any_content(dir: &std::path::Path) -> bool {
    use tokio::fs;

    let Ok(mut entries) = fs::read_dir(dir).await else {
        return false;
    };

    entries.next_entry().await.ok().flatten().is_some()
}

/// 截断路径显示（保留末尾部分）
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}

/// 用系统文件管理器打开目录
fn open_directory(path: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
}
