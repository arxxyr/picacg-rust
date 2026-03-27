//! 点赞记录系统
//!
//! 实现点赞记录页面，展示用户点赞过的漫画

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

/// 点赞记录页面根节点
#[derive(Component)]
pub struct LikeRecordsRoot;

/// 点赞记录滚动容器
#[derive(Component)]
pub struct LikeRecordsScrollContainer;

/// 点赞记录卡片
#[derive(Component)]
pub struct LikeRecordCard {
    pub comic_id: String,
}

/// 点赞记录删除按钮（取消点赞）
#[derive(Component)]
pub struct LikeRecordDeleteButton {
    pub comic_id: String,
}

/// 点赞记录封面缩略图
#[derive(Component)]
pub struct LikeRecordThumbnail {
    #[allow(dead_code)]
    pub url: String,
}

/// 点赞记录空状态提示
#[derive(Component)]
pub struct LikeRecordsEmptyHint;

// ==================== 布局常量 ====================

mod like_records_layout {
    /// 卡片高度
    pub const CARD_HEIGHT: f32 = 90.0;
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
    pub const THUMB_WIDTH: f32 = 60.0;
    /// 封面高度
    pub const THUMB_HEIGHT: f32 = 75.0;
}

// ==================== 系统函数 ====================

/// 创建点赞记录界面（如果已存在则只显示）
pub fn setup_like_records_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    like_records_state: Res<LikeRecordsState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_messages: MessageWriter<LoadLikeRecordsRequest>,
    mut existing_query: Query<&mut Node, With<LikeRecordsRoot>>,
) {
    // 如果 LikeRecordsRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if like_records_state.records.is_empty() && !like_records_state.is_loading {
            load_messages.write(LoadLikeRecordsRequest);
        }
        return;
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let like_records_root = commands
        .spawn((
            LikeRecordsRoot,
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
                header
                    .spawn((Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },))
                    .with_children(|title_row| {
                        title_row.spawn((
                            Text::new(ICON_THUMB_UP),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(AppColors::PRIMARY),
                        ));
                        title_row.spawn((
                            Text::new("点赞记录"),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
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
                    // 列表（可滚动）
                    let scroll_container_id = wrapper
                        .spawn((
                            LikeRecordsScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect {
                                    left: Val::Px(like_records_layout::PADDING_LEFT),
                                    right: Val::Px(like_records_layout::PADDING_RIGHT),
                                    top: Val::Px(like_records_layout::PADDING_TOP),
                                    bottom: Val::Px(like_records_layout::PADDING_BOTTOM),
                                },
                                row_gap: Val::Px(like_records_layout::CARD_GAP),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            Scrollable,
                            ScrollPosition::default(),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|list| {
                            if like_records_state.is_loading {
                                list.spawn((
                                    LoadingIndicator,
                                    Text::new("加载中..."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT),
                                ));
                            } else if like_records_state.records.is_empty()
                                && like_records_state.error.is_none()
                            {
                                list.spawn((
                                    LikeRecordsEmptyHint,
                                    Text::new("暂无点赞记录"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT_SECONDARY),
                                ));
                            }
                        })
                        .id();

                    // 创建滚动条
                    spawn_scrollbar(wrapper, scroll_container_id);
                });
        })
        .id();

    // 如果有 ContentArea，将点赞记录作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(like_records_root);
    }

    // 发送加载请求
    if like_records_state.records.is_empty() && !like_records_state.is_loading {
        load_messages.write(LoadLikeRecordsRequest);
    }

    tracing::info!("点赞记录页面 UI 已创建");
}

/// 清理点赞记录页面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_like_records_ui(mut query: Query<&mut Node, With<LikeRecordsRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新点赞记录列表 UI（响应数据变化）
pub fn refresh_like_records_ui(
    mut commands: Commands,
    like_records_state: Res<LikeRecordsState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<LikeRecordsScrollContainer>>,
    card_query: Query<&LikeRecordCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    empty_hint_query: Query<Entity, With<LikeRecordsEmptyHint>>,
    image_cache: Res<ImageCache>,
) {
    if !like_records_state.is_changed() {
        return;
    }

    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 如果有错误，显示错误信息
    if let Some(ref error) = like_records_state.error {
        // 删除加载指示器
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
                Text::new(format!("加载失败: {}", error)),
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

    // 检查是否已有卡片
    let has_cards = children
        .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
        .unwrap_or(false);

    // 如果数据存在或已有卡片，不重建
    if has_cards || like_records_state.records.is_empty() {
        // 如果记录为空且没有空状态提示，添加空状态提示
        if like_records_state.records.is_empty()
            && !like_records_state.is_loading
            && !has_cards
            && empty_hint_query.is_empty()
            && loading_query.is_empty()
        {
            let font: Handle<Font> = get_font();
            commands.entity(scroll_entity).with_children(|parent| {
                parent.spawn((
                    LikeRecordsEmptyHint,
                    Text::new("暂无点赞记录"),
                    TextFont {
                        font,
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });
        }
        return;
    }

    // 删除加载指示器和空状态提示
    for entity in loading_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in empty_hint_query.iter() {
        commands.entity(entity).despawn();
    }

    // 创建所有点赞记录卡片
    let font: Handle<Font> = get_font();
    commands.entity(scroll_entity).with_children(|parent| {
        for record in like_records_state.records.iter() {
            spawn_like_record_card(parent, record, &font, &image_cache);
        }
    });
}

/// 创建单个点赞记录卡片
fn spawn_like_record_card(
    parent: &mut ChildSpawnerCommands,
    record: &picacg_db::DbLikeRecord,
    font: &Handle<Font>,
    image_cache: &ImageCache,
) {
    let time_str = format_timestamp(record.liked_at);

    parent
        .spawn((
            LikeRecordCard {
                comic_id: record.comic_id.clone(),
            },
            ContextMenuTarget {
                comic_id: record.comic_id.clone(),
                comic_title: record.comic_title.clone(),
            },
            Button,
            Interaction::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(like_records_layout::CARD_HEIGHT),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                column_gap: Val::Px(12.0),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(AppColors::SURFACE),
        ))
        .with_children(|card| {
            // 封面缩略图
            let thumb_url = record.thumb_url.as_deref().unwrap_or("");
            if !thumb_url.is_empty() {
                if let Some(handle) = image_cache.get(thumb_url) {
                    card.spawn((
                        LikeRecordThumbnail {
                            url: thumb_url.to_string(),
                        },
                        ImageNode::new(handle.clone()),
                        Node {
                            width: Val::Px(like_records_layout::THUMB_WIDTH),
                            height: Val::Px(like_records_layout::THUMB_HEIGHT),
                            flex_shrink: 0.0,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                } else {
                    // 占位符
                    card.spawn((
                        PlaceholderImage,
                        Node {
                            width: Val::Px(like_records_layout::THUMB_WIDTH),
                            height: Val::Px(like_records_layout::THUMB_HEIGHT),
                            flex_shrink: 0.0,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    ));
                }
            } else {
                // 无封面占位符
                card.spawn((
                    Node {
                        width: Val::Px(like_records_layout::THUMB_WIDTH),
                        height: Val::Px(like_records_layout::THUMB_HEIGHT),
                        flex_shrink: 0.0,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                ))
                .with_children(|ph| {
                    ph.spawn((
                        Text::new(ICON_BOOK),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });
            }

            // 中间信息区域
            card.spawn((Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(4.0),
                overflow: Overflow::clip(),
                ..default()
            },))
                .with_children(|info| {
                    // 漫画标题
                    info.spawn((
                        Text::new(record.comic_title.as_str()),
                        TextFont {
                            font: font.clone(),
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                        Node {
                            max_width: Val::Percent(100.0),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                    ));

                    // 点赞时间
                    info.spawn((
                        Text::new(format!("点赞于 {}", time_str)),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

            // 右侧取消点赞按钮
            card.spawn((
                LikeRecordDeleteButton {
                    comic_id: record.comic_id.clone(),
                },
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(ICON_CLOSE),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.55)),
                ));
            });
        });
}

/// 格式化时间戳为可读字符串
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{Local, TimeZone};

    if timestamp == 0 {
        return "未知时间".to_string();
    }

    match Local.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => "未知时间".to_string(),
    }
}

/// 点赞记录卡片点击交互（跳转到漫画详情）
pub fn like_record_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &LikeRecordCard),
        (Changed<Interaction>, Without<LikeRecordDeleteButton>),
    >,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, mut bg_color, card) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));
                detail_messages.write(NavigateToComicDetailEvent {
                    comic_id: card.comic_id.clone(),
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::SURFACE);
            }
        }
    }
}

/// 点赞记录删除按钮交互（取消点赞）
pub fn like_record_delete_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &LikeRecordDeleteButton),
        Changed<Interaction>,
    >,
    mut delete_messages: MessageWriter<DeleteLikeRecordRequest>,
) {
    for (interaction, mut bg_color, btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.9, 0.3, 0.3, 0.3));
                delete_messages.write(DeleteLikeRecordRequest {
                    comic_id: btn.comic_id.clone(),
                });
                tracing::info!("取消点赞: {}", btn.comic_id);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.2));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 点赞记录页面滚动处理
pub fn handle_like_records_scroll(
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
    _scroll_query: Query<
        (&mut ScrollPosition, &ComputedNode, Option<&ContentSizeInfo>),
        With<LikeRecordsScrollContainer>,
    >,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新点赞记录内容尺寸信息
pub fn update_like_records_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<LikeRecordsScrollContainer>,
    >,
    _children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        // 计算内容高度（列表布局：每个卡片高度 + 间距）
        let card_count = children.len();
        let content_height = like_records_layout::PADDING_TOP
            + like_records_layout::PADDING_BOTTOM
            + (card_count as f32 * like_records_layout::CARD_HEIGHT)
            + (card_count.saturating_sub(1) as f32 * like_records_layout::CARD_GAP);

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 处理点赞记录数据加载完成
pub fn handle_like_records_loaded(
    mut like_records_state: ResMut<LikeRecordsState>,
    mut messages: MessageReader<LikeRecordsLoadedEvent>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    image_cache: Res<ImageCache>,
) {
    for event in messages.read() {
        like_records_state.records = event.records.clone();
        like_records_state.total_count = event.total_count;
        like_records_state.is_loading = false;
        like_records_state.error = None;

        // 预加载封面图片
        for record in &like_records_state.records {
            if let Some(ref url) = record.thumb_url
                && !url.is_empty()
                && image_cache.get(url).is_none()
            {
                load_image_messages.write(LoadImageRequest { url: url.clone() });
            }
        }

        tracing::info!(
            "点赞记录加载完成: {} 条记录",
            like_records_state.records.len()
        );
    }
}

/// 处理点赞记录数据加载失败
pub fn handle_like_records_load_failed(
    mut like_records_state: ResMut<LikeRecordsState>,
    mut messages: MessageReader<LikeRecordsLoadFailedEvent>,
) {
    for event in messages.read() {
        like_records_state.is_loading = false;
        like_records_state.error = Some(event.error.clone());
        tracing::warn!("点赞记录加载失败: {}", event.error);
    }
}

/// 更新点赞记录封面图片（当图片加载完成时替换占位符）
pub fn update_like_records_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    like_records_state: Res<LikeRecordsState>,
    placeholder_query: Query<(Entity, &ChildOf), With<PlaceholderImage>>,
    card_query: Query<&LikeRecordCard>,
) {
    let placeholder_count = placeholder_query.iter().count();
    if placeholder_count == 0 {
        return;
    }

    let mut replaced_count = 0;
    for (placeholder_entity, child_of) in placeholder_query.iter() {
        let parent_entity: Entity = child_of.parent();
        let Ok(card) = card_query.get(parent_entity) else {
            continue;
        };

        // 找到对应的点赞记录
        let Some(record) = like_records_state
            .records
            .iter()
            .find(|r| r.comic_id == card.comic_id)
        else {
            continue;
        };

        let Some(ref thumb_url) = record.thumb_url else {
            continue;
        };

        if thumb_url.is_empty() {
            continue;
        }

        // 检查图片是否已加载
        if let Some(handle) = image_cache.get(thumb_url) {
            commands.entity(placeholder_entity).despawn();
            let image_entity = commands
                .spawn((
                    LikeRecordThumbnail {
                        url: thumb_url.clone(),
                    },
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(like_records_layout::THUMB_WIDTH),
                        height: Val::Px(like_records_layout::THUMB_HEIGHT),
                        flex_shrink: 0.0,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                ))
                .id();

            // 插入到第一个位置（在信息区域之前）
            commands
                .entity(parent_entity)
                .insert_children(0, &[image_entity]);
            replaced_count += 1;
        }
    }

    if replaced_count > 0 {
        tracing::trace!("[LikeRecords] 替换了 {} 个封面图片", replaced_count);
    }
}
