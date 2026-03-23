//! 漫画列表系统

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::scrollbar_config::SCROLLBAR_WIDTH,
        ui_common::{
            GridLayoutParams, calculate_scroll_delta, measure_grid_content_height,
            spawn_comic_time_info, spawn_scrollbar,
        },
        waterfall::ComicsCardCreationState,
    },
    utils::content_filter::{
        FilterConfig, filter_comic_indices, load_filter_flags, load_filter_keywords,
    },
};

/// 面包屑"分类"按钮，点击返回分类页
#[derive(Component)]
pub struct BreadcrumbBackToCategories;

/// 漫画卡片布局常量
mod comic_layout {
    /// 卡片宽度
    pub const CARD_WIDTH: f32 = 180.0;
    /// 列间距
    pub const COLUMN_GAP: f32 = 15.0;
    /// 行间距
    pub const ROW_GAP: f32 = 15.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 20.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
}

/// 创建漫画列表界面（在 ContentArea 内部）
pub fn setup_comics_list_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    comics_state: Res<ComicsListState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<ComicsCardCreationState>,
) {
    let font: Handle<Font> = get_font();

    // 清空之前的创建状态
    creation_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    let comics_root = commands
        .spawn((
            ComicsListRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // 标题栏（包含面包屑导航）
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|header| {
                // 面包屑: 分类 > 当前分类名（"分类"可点击返回）
                header
                    .spawn((
                        BreadcrumbBackToCategories,
                        Button,
                        Interaction::default(),
                        Node::default(),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("分类"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });

                header.spawn((
                    Text::new(">"),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                header.spawn((
                    Text::new(&comics_state.category),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // 滚动区域包装器（用于放置滚动条）
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
                    // 漫画网格（可滚动）
                    let scroll_container_id = wrapper
                        .spawn((
                            ComicsScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::FlexStart,
                                align_content: AlignContent::FlexStart,
                                padding: UiRect {
                                    left: Val::Px(comic_layout::PADDING_LEFT),
                                    right: Val::Px(comic_layout::PADDING_RIGHT),
                                    top: Val::Px(comic_layout::PADDING_TOP),
                                    bottom: Val::Px(comic_layout::PADDING_BOTTOM),
                                },
                                column_gap: Val::Px(comic_layout::COLUMN_GAP),
                                row_gap: Val::Px(comic_layout::ROW_GAP),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            ScrollPosition(Vec2::new(0.0, comics_state.scroll_y)),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|grid| {
                            if comics_state.is_loading {
                                grid.spawn((
                                    LoadingIndicator,
                                    Text::new("加载中..."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT),
                                ));
                            }
                            // 漫画卡片通过瀑布式创建系统添加
                        })
                        .id();

                    // 创建滚动条
                    spawn_scrollbar(wrapper, scroll_container_id);
                });

            // 无限滚动不再需要分页控件
        })
        .id();

    // 如果有 ContentArea，将漫画列表作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(comics_root);
    }

    // 启动预创建模式（在瀑布系统中一次性创建所有隐藏卡片，然后瀑布式显示）
    if !comics_state.comics.is_empty() && !comics_state.is_loading {
        creation_state.start_precreate(comics_state.comics.len(), font);
    }
}

/// 创建漫画卡片（返回 Entity，可选隐藏）
fn spawn_comic_card(
    parent: &mut ChildSpawnerCommands,
    comic: &picacg_api::models::Comic,
    font: &Handle<Font>,
    image_cache: &ImageCache,
    hidden: bool,
) -> Entity {
    parent
        .spawn((
            ComicCard {
                comic_id: comic.id.clone(),
            },
            Button,
            Node {
                width: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(AppColors::SURFACE),
            if hidden {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
        ))
        .with_children(|card| {
            // 封面图片
            let thumb_url = comic.thumb.url();
            if let Some(handle) = image_cache.get(&thumb_url) {
                card.spawn((
                    ComicThumbnail {
                        url: thumb_url.clone(),
                    },
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(164.0),
                        height: Val::Px(220.0),
                        ..default()
                    },
                ));
            } else {
                card.spawn((
                    PlaceholderImage,
                    Node {
                        width: Val::Px(164.0),
                        height: Val::Px(220.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                ));
            }

            // 标题
            card.spawn((
                Text::new(&comic.title),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ));

            // 作者
            card.spawn((
                Text::new(&comic.author),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // 分类标签容器
            if !comic.categories.is_empty() {
                card.spawn((Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    ..default()
                },))
                    .with_children(|tags_container| {
                        // 最多显示 3 个分类
                        for category in comic.categories.iter().take(3) {
                            tags_container
                                .spawn((
                                    Node {
                                        padding: UiRect::new(
                                            Val::Px(4.0),
                                            Val::Px(4.0),
                                            Val::Px(1.0),
                                            Val::Px(1.0),
                                        ),
                                        border_radius: BorderRadius::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.4, 0.8, 0.3)),
                                ))
                                .with_children(|badge| {
                                    badge.spawn((
                                        Text::new(category),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.6, 0.8, 1.0)),
                                    ));
                                });
                        }
                    });
            }

            // 标签容器
            if !comic.tags.is_empty() {
                card.spawn((Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    margin: UiRect::top(Val::Px(2.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },))
                    .with_children(|tags_container| {
                        // 最多显示 3 个标签
                        for tag in comic.tags.iter().take(3) {
                            tags_container
                                .spawn((
                                    Node {
                                        padding: UiRect::new(
                                            Val::Px(4.0),
                                            Val::Px(4.0),
                                            Val::Px(1.0),
                                            Val::Px(1.0),
                                        ),
                                        border_radius: BorderRadius::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.6, 0.3, 0.6, 0.3)),
                                ))
                                .with_children(|badge| {
                                    badge.spawn((
                                        Text::new(tag),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.7, 0.9)),
                                    ));
                                });
                        }
                    });
            }

            // 创建/更新时间
            spawn_comic_time_info(
                card,
                font,
                comic.created_at.as_deref(),
                comic.updated_at.as_deref(),
            );
        })
        .id()
}

/// 清理漫画列表界面（退出时保存滚动位置）
pub fn cleanup_comics_list_ui(
    mut commands: Commands,
    query: Query<Entity, With<ComicsListRoot>>,
    mut creation_state: ResMut<ComicsCardCreationState>,
    scroll_query: Query<&ScrollPosition, With<ComicsScrollContainer>>,
    mut comics_state: ResMut<ComicsListState>,
) {
    // 保存滚动位置，返回时恢复
    if let Ok(scroll_pos) = scroll_query.single() {
        comics_state.scroll_y = scroll_pos.y;
    }

    // 清空瀑布式创建状态（防止对已销毁的 Entity 操作）
    creation_state.clear();

    for entity in query.iter() {
        // Bevy 0.17: despawn() 自动递归删除子实体
        commands.entity(entity).despawn();
    }
}

/// 漫画卡片交互系统
pub fn comic_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ComicCard),
        Changed<Interaction>,
    >,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, mut bg_color, card) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));
                // 通过导航消息跳转到详情页（保留导航历史）
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

/// 无限滚动自动加载更多漫画
pub fn auto_load_more_comics(
    scroll_query: Query<(&ScrollPosition, Option<&ContentSizeInfo>), With<ComicsScrollContainer>>,
    mut comics_state: ResMut<ComicsListState>,
    mut load_messages: MessageWriter<LoadComicsRequest>,
) {
    let Ok((scroll_pos, content_info)) = scroll_query.single() else {
        return;
    };

    let Some(info) = content_info else {
        return;
    };

    // 视口或内容高度为 0 时不触发
    if info.viewport_height <= 0.0 || info.content_height <= 0.0 {
        return;
    }

    let remaining = info.content_height - info.viewport_height - scroll_pos.y;

    // 距底部 200px 时触发加载下一页
    if remaining < 200.0
        && !comics_state.is_loading
        && !comics_state.is_loading_more
        && comics_state.page < comics_state.total_pages
    {
        comics_state.page += 1;
        comics_state.is_loading_more = true;
        load_messages.write(LoadComicsRequest {
            category: comics_state.category.clone(),
            page: comics_state.page,
            sort: comics_state.sort.clone(),
        });
        tracing::debug!(
            "无限滚动：加载第 {}/{} 页",
            comics_state.page,
            comics_state.total_pages
        );
    }
}

/// 漫画列表页面滚动处理系统
pub fn handle_comics_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut scroll_query: Query<
        (&mut ScrollPosition, &ComputedNode, Option<&ContentSizeInfo>),
        With<ComicsScrollContainer>,
    >,
) {
    for event in mouse_wheel_events.read() {
        let scroll_delta = calculate_scroll_delta(event);

        for (mut scroll_position, computed_node, content_size_info) in &mut scroll_query {
            let (content_height, viewport_height) = content_size_info
                .map(|info| (info.content_height, info.viewport_height))
                .unwrap_or_else(|| {
                    let size = computed_node.size();
                    (size.y, size.y)
                });

            let max_scroll = (content_height - viewport_height).max(0.0);
            scroll_position.y = (scroll_position.y - scroll_delta).clamp(0.0, max_scroll);
        }
    }
}

/// 限制漫画列表页面滚动范围（防止越界）
pub fn clamp_comics_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<ComicsScrollContainer>,
    >,
) {
    for (mut scroll_position, content_size_info) in &mut scroll_query {
        if scroll_position.y < 0.0 {
            scroll_position.y = 0.0;
        }

        if let Some(content_info) = content_size_info {
            // 内容尺寸尚未计算时（卡片还没创建），不做上限 clamp，
            // 避免返回页面时滚动位置在卡片创建前被错误重置为 0
            if content_info.content_height > 0.0 {
                let max_scroll =
                    (content_info.content_height - content_info.viewport_height).max(0.0);
                if scroll_position.y > max_scroll {
                    scroll_position.y = max_scroll;
                }
            }
        }
    }
}

/// 更新漫画列表内容尺寸信息
///
/// 通过测量子节点的实际渲染高度来计算内容高度，
/// 避免因卡片高度不一致导致滚动条位置偏移。
pub fn update_comics_content_size(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, Option<&Children>),
        With<ComicsScrollContainer>,
    >,
    child_computed_query: Query<&ComputedNode>,
) {
    use comic_layout::*;

    let scale_factor = windows
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    let layout_params = GridLayoutParams {
        card_width: CARD_WIDTH,
        column_gap: COLUMN_GAP,
        row_gap: ROW_GAP,
        padding_left: PADDING_LEFT,
        padding_right: PADDING_RIGHT,
        padding_top: PADDING_TOP,
        padding_bottom: PADDING_BOTTOM,
    };

    for (scroll_computed, mut content_size_info, children) in &mut scroll_query {
        let viewport_size = scroll_computed.size();
        let viewport_width = viewport_size.x / scale_factor;
        let viewport_height = viewport_size.y / scale_factor;

        if viewport_height <= 0.0 || viewport_width <= 0.0 {
            continue;
        }

        content_size_info.viewport_height = viewport_height;
        content_size_info.content_height = measure_grid_content_height(
            children,
            &child_computed_query,
            scale_factor,
            viewport_width,
            &layout_params,
        );
    }
}

/// 刷新漫画列表界面（只处理错误状态，卡片由瀑布式系统创建）
///
/// 注意：这个函数**不应该**在数据加载完成后重建整个
/// UI，否则会覆盖瀑布式系统创建的卡片。 它只在出现错误时处理错误显示。
pub fn refresh_comics_list_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    comics_state: Res<ComicsListState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ComicsScrollContainer>>,
    error_query: Query<Entity, With<ErrorMessage>>,
) {
    // 只在状态变化时检查
    if !comics_state.is_changed() {
        return;
    }

    // 如果有错误，显示错误信息
    if let Some(ref error) = comics_state.error {
        // 如果还没有错误信息 UI，添加它
        if error_query.is_empty()
            && let Ok((container_entity, _)) = scroll_container_query.single()
        {
            let font: Handle<Font> = get_font();
            let error_entity = commands
                .spawn((
                    ErrorMessage,
                    Text::new(format!("加载失败: {}", error)),
                    TextFont {
                        font,
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                ))
                .id();
            commands.entity(container_entity).add_child(error_entity);
        }
    }

    // 如果数据存在或已有卡片，让瀑布式系统处理，不干涉
    // 数据为空且没有卡片则保持加载中状态
}

/// 瀑布式显示漫画卡片（支持无限滚动增量创建）
#[allow(clippy::too_many_arguments)]
pub fn waterfall_create_comic_cards(
    mut commands: Commands,
    mut creation_state: ResMut<ComicsCardCreationState>,
    comics_state: Res<ComicsListState>,
    image_cache: Res<ImageCache>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ComicsScrollContainer>>,
    card_query: Query<&ComicCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    time: Res<Time>,
    _asset_server: Res<AssetServer>,
) {
    // 构建屏蔽过滤配置
    let blocked_keywords = load_filter_keywords();
    let (filter_by_category, filter_by_tag, filter_by_title) = load_filter_flags();
    let filter_config = FilterConfig {
        blocked_keywords: &blocked_keywords,
        filter_by_category,
        filter_by_tag,
        filter_by_title,
    };

    // 计算过滤后的索引列表
    let filtered_indices = filter_comic_indices(&comics_state.comics, &filter_config);

    // 如果数据已加载但 creation_state 未启动，主动启动预创建
    if !creation_state.is_creating
        && !comics_state.comics.is_empty()
        && comics_state.error.is_none()
        && let Ok((_, children)) = scroll_container_query.single()
    {
        // 统计容器中已有的卡片数量
        let existing_card_count = children
            .map(|c| {
                c.iter()
                    .filter(|child| card_query.get(*child).is_ok())
                    .count()
            })
            .unwrap_or(0);

        let total_filtered = filtered_indices.len();

        if existing_card_count == 0 && total_filtered > 0 {
            // 首次加载：没有卡片，创建全部（过滤后）
            for entity in loading_query.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                }
            }
            let font: Handle<Font> = get_font();
            creation_state.start_precreate(total_filtered, font);
            tracing::debug!("自动启动漫画卡片预创建: {} 个（过滤后）", total_filtered);
        } else if existing_card_count < total_filtered && existing_card_count > 0 {
            // 无限滚动追加：有新数据追加，增量创建新卡片
            let new_count = total_filtered - existing_card_count;
            let font: Handle<Font> = get_font();
            creation_state.start_precreate(new_count, font);
            tracing::debug!(
                "无限滚动追加卡片: 已有 {}，新增 {}",
                existing_card_count,
                new_count
            );
        }
    }

    // 检查是否需要预创建
    if creation_state.needs_precreate() {
        let Ok((container_entity, children)) = scroll_container_query.single() else {
            return;
        };

        let Some(font) = creation_state.font_handle.clone() else {
            return;
        };

        let comics = &comics_state.comics;
        let count = creation_state.get_precreate_count();

        if filtered_indices.is_empty() || count == 0 {
            creation_state.clear();
            return;
        }

        // 计算已有卡片数，从该偏移量开始创建新卡片
        let existing_card_count = children
            .map(|c| {
                c.iter()
                    .filter(|child| card_query.get(*child).is_ok())
                    .count()
            })
            .unwrap_or(0);

        let start_index = existing_card_count;
        let end_index = (start_index + count).min(filtered_indices.len());

        // 一次性创建所有隐藏卡片（使用过滤后的索引）
        let mut entities = Vec::with_capacity(end_index - start_index);
        commands.entity(container_entity).with_children(|parent| {
            for &original_index in &filtered_indices[start_index..end_index] {
                if let Some(comic) = comics.get(original_index) {
                    let entity = spawn_comic_card(parent, comic, &font, &image_cache, true);
                    entities.push(entity);
                }
            }
        });

        // 设置预创建完成后的实体列表
        let entity_count = entities.len();
        creation_state.set_precreated_entities(entities);
        tracing::debug!(
            "漫画卡片预创建完成: {} 个（从索引 {} 开始）",
            entity_count,
            start_index
        );
        return;
    }

    // 检查是否应该显示下一批
    if !creation_state.should_show_batch(time.delta()) {
        return;
    }

    // 获取这一批要显示的实体
    let batch = creation_state.take_batch();
    if batch.is_empty() {
        return;
    }

    // 显示这一批卡片（设置 Visibility::Inherited）
    for entity in batch {
        // 安全检查：实体可能在清理时已被销毁
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(Visibility::Inherited);
        }
    }

    // 标记显示完成
    if !creation_state.has_pending() {
        creation_state.finish();
        tracing::debug!(
            "漫画卡片瀑布式显示完成: 共 {} 个",
            comics_state.comics.len()
        );
    }
}

/// 更新漫画封面图片（当图片加载完成时替换占位符）
pub fn update_comics_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    comics_state: Res<ComicsListState>,
    placeholder_query: Query<(Entity, &ChildOf), With<PlaceholderImage>>,
    card_query: Query<&ComicCard>,
) {
    // 每帧都检查占位符（不仅仅是 image_cache 变化时）
    // 因为占位符可能在 image_cache 变化后的帧才创建
    let placeholder_count = placeholder_query.iter().count();
    if placeholder_count == 0 {
        return;
    }

    // 调试日志（trace 级别）
    static LAST_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let last = LAST_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    if last != placeholder_count {
        LAST_COUNT.store(placeholder_count, std::sync::atomic::Ordering::Relaxed);
        tracing::trace!(
            "[Comics] 占位符数量: {}, 漫画数量: {}, 缓存图片数: {}",
            placeholder_count,
            comics_state.comics.len(),
            image_cache.loaded_count()
        );
    }

    let mut replaced_count = 0;
    for (placeholder_entity, child_of) in placeholder_query.iter() {
        // 找到父卡片
        let parent_entity: Entity = child_of.parent();
        let Ok(card) = card_query.get(parent_entity) else {
            tracing::warn!("[Comics] 找不到占位符的父卡片");
            continue;
        };

        // 找到对应的漫画
        let Some(comic) = comics_state.comics.iter().find(|c| c.id == card.comic_id) else {
            tracing::warn!("[Comics] 找不到漫画: {}", card.comic_id);
            continue;
        };

        let thumb_url = comic.thumb.url();

        // 检查图片是否已加载
        if let Some(handle) = image_cache.get(&thumb_url) {
            // 删除占位符，添加实际图片
            commands.entity(placeholder_entity).despawn();
            // 创建新的图片实体并插入到父卡片的第一个位置
            let image_entity = commands
                .spawn((
                    ComicThumbnail {
                        url: thumb_url.clone(),
                    },
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(164.0),
                        height: Val::Px(220.0),
                        ..default()
                    },
                ))
                .id();

            // 插入到第一个位置（在标题之前）
            commands
                .entity(parent_entity)
                .insert_children(0, &[image_entity]);
            replaced_count += 1;
        }
    }

    if replaced_count > 0 {
        tracing::trace!("[Comics] 替换了 {} 个封面图片", replaced_count);
    }
}

/// 面包屑"分类"按钮交互：点击返回分类列表页
pub fn breadcrumb_back_to_categories(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<BreadcrumbBackToCategories>),
    >,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_route.set(AppRoute::Categories);
        }
    }
}
