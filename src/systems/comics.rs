//! 漫画列表系统

use bevy::{input::mouse::MouseWheel, prelude::*, ui::FocusPolicy, window::PrimaryWindow};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::{AppColors, FONT_PATH},
        scrollbar::scrollbar_config::*,
        waterfall::ComicsCardCreationState,
    },
};

/// 漫画卡片布局常量
mod comic_layout {
    /// 卡片宽度
    pub const CARD_WIDTH: f32 = 180.0;
    /// 卡片高度（封面 220px + 标题+作者约 50px + padding 16px）
    pub const CARD_HEIGHT: f32 = 300.0;
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
    pub const PADDING_BOTTOM: f32 = 20.0;
}

/// 创建漫画列表界面（在 ContentArea 内部）
pub fn setup_comics_list_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    comics_state: Res<ComicsListState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<ComicsCardCreationState>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

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
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BorderColor::all(AppColors::BORDER))
            .with_children(|header| {
                // 面包屑: 分类 > 当前分类名
                header.spawn((
                    Text::new("分类"),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

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
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                ..default()
            })
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
                        ScrollPosition::default(),
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
                spawn_scrollbar_inline(wrapper, scroll_container_id);
            });

            // 分页控件
            root.spawn((
                PaginationControl,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    border: UiRect::top(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
                BackgroundColor(AppColors::SURFACE),
                Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
            ))
            .with_children(|pagination| {
                // 上一页按钮
                pagination
                    .spawn((
                        PrevPageButton,
                        Button,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if comics_state.page > 1 {
                            AppColors::PRIMARY
                        } else {
                            AppColors::SECONDARY
                        }),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("上一页"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                // 页码显示
                pagination.spawn((
                    PageNumberText,
                    Text::new(format!(
                        "{} / {}",
                        comics_state.page, comics_state.total_pages
                    )),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 下一页按钮
                pagination
                    .spawn((
                        NextPageButton,
                        Button,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if comics_state.page < comics_state.total_pages {
                            AppColors::PRIMARY
                        } else {
                            AppColors::SECONDARY
                        }),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("下一页"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });
            });
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

/// 内联创建滚动条（用于 ChildSpawnerCommands）
///
/// 布局结构：
/// ScrollbarContainer (Absolute, right=0)
///   ├── ScrollbarTrack (Button, fills 100%, ZIndex=0)
///   └── ScrollbarThumb (Button, Absolute, ZIndex=1)
///
/// 滑块和轨道作为兄弟节点，避免父子节点交互事件冲突
fn spawn_scrollbar_inline(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
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
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|scrollbar| {
            // 滚动条轨道（与滑块同级，ZIndex 较低）
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
                // 添加 Transform 以获得 GlobalTransform（滚动条点击需要）
                Transform::default(),
            ));

            // 滚动条滑块（与轨道同级，ZIndex 较高以覆盖轨道）
            // 使用 FocusPolicy::Block 阻止事件穿透到轨道
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
                    ..default()
                },
                BackgroundColor(THUMB_COLOR),
                BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                ZIndex(1),
            ));
        });
}

/// 创建漫画卡片（返回 Entity，可选隐藏）
fn spawn_comic_card(
    parent: &mut ChildSpawnerCommands,
    comic: &crate::api::models::Comic,
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
                card.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    ..default()
                })
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
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.4, 0.8, 0.3)),
                                BorderRadius::all(Val::Px(2.0)),
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
                card.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    margin: UiRect::top(Val::Px(2.0)),
                    overflow: Overflow::clip(),
                    ..default()
                })
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
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.6, 0.3, 0.6, 0.3)),
                                BorderRadius::all(Val::Px(2.0)),
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
        })
        .id()
}

/// 清理漫画列表界面
pub fn cleanup_comics_list_ui(
    mut commands: Commands,
    query: Query<Entity, With<ComicsListRoot>>,
    mut creation_state: ResMut<ComicsCardCreationState>,
) {
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
    mut detail_state: ResMut<ComicDetailState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut load_detail_messages: MessageWriter<LoadComicDetailRequest>,
) {
    for (interaction, mut bg_color, card) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));

                // 设置当前漫画 ID
                detail_state.comic_id = card.comic_id.clone();
                detail_state.comic = None;
                detail_state.episodes.clear();

                next_route.set(AppRoute::ComicDetail);

                // 触发加载漫画详情
                load_detail_messages.write(LoadComicDetailRequest {
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

/// 分页按钮交互系统
pub fn pagination_interaction(
    prev_query: Query<&Interaction, (Changed<Interaction>, With<PrevPageButton>)>,
    next_query: Query<&Interaction, (Changed<Interaction>, With<NextPageButton>)>,
    mut comics_state: ResMut<ComicsListState>,
    mut load_comics_messages: MessageWriter<LoadComicsRequest>,
) {
    // 上一页
    for interaction in prev_query.iter() {
        if *interaction == Interaction::Pressed && comics_state.page > 1 {
            comics_state.page -= 1;
            load_comics_messages.write(LoadComicsRequest {
                category: comics_state.category.clone(),
                page: comics_state.page,
                sort: comics_state.sort.clone(),
            });
        }
    }

    // 下一页
    for interaction in next_query.iter() {
        if *interaction == Interaction::Pressed && comics_state.page < comics_state.total_pages {
            comics_state.page += 1;
            load_comics_messages.write(LoadComicsRequest {
                category: comics_state.category.clone(),
                page: comics_state.page,
                sort: comics_state.sort.clone(),
            });
        }
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
        for (mut scroll_position, computed_node, content_size_info) in &mut scroll_query {
            let scroll_delta = match event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
                bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
            };

            // 获取内容和视口高度
            let (content_height, viewport_height) = if let Some(info) = content_size_info {
                (info.content_height, info.viewport_height)
            } else {
                let size = computed_node.size();
                (size.y, size.y)
            };

            let max_scroll = (content_height - viewport_height).max(0.0);

            // 更新滚动位置
            let old_scroll = scroll_position.y;
            scroll_position.y = (scroll_position.y - scroll_delta).clamp(0.0, max_scroll);

            // 详细日志（trace 级别）
            tracing::trace!(
                "[Comics] 滚动: delta={:.1}, old={:.1}, new={:.1}, max={:.1}, content={:.1}, viewport={:.1}",
                scroll_delta,
                old_scroll,
                scroll_position.y,
                max_scroll,
                content_height,
                viewport_height
            );
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
            let max_scroll = (content_info.content_height - content_info.viewport_height).max(0.0);
            if scroll_position.y > max_scroll {
                scroll_position.y = max_scroll;
            }
        }
    }
}

/// 更新漫画列表内容尺寸信息
///
/// 使用手动网格计算（基于卡片数量和布局常量）。
pub fn update_comics_content_size(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scroll_query: Query<(&ComputedNode, &mut ContentSizeInfo), With<ComicsScrollContainer>>,
    card_query: Query<Entity, With<ComicCard>>,
) {
    use comic_layout::*;

    // 获取 scale_factor
    let scale_factor = windows
        .single()
        .ok()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0);

    for (scroll_computed, mut content_size_info) in &mut scroll_query {
        let viewport_size = scroll_computed.size();
        // ComputedNode::size() 返回物理像素，转换为逻辑像素
        let viewport_width = viewport_size.x / scale_factor;
        let viewport_height = viewport_size.y / scale_factor;

        // 如果视口尺寸为0，说明布局还没完成
        if viewport_height <= 0.0 || viewport_width <= 0.0 {
            continue;
        }

        // 计算卡片数量
        let card_count = card_query.iter().count();
        if card_count == 0 {
            content_size_info.content_height = 0.0;
            content_size_info.viewport_height = viewport_height;
            continue;
        }

        // 计算列数（所有值都是逻辑像素）
        let available_width = viewport_width - PADDING_LEFT - PADDING_RIGHT;
        let card_with_gap = CARD_WIDTH + COLUMN_GAP;
        let columns = ((available_width + COLUMN_GAP) / card_with_gap)
            .floor()
            .max(1.0) as usize;
        let rows = (card_count + columns - 1) / columns;

        // 计算内容高度（逻辑像素）
        let content_height = PADDING_TOP
            + (rows as f32) * CARD_HEIGHT
            + ((rows.saturating_sub(1)) as f32) * ROW_GAP
            + PADDING_BOTTOM;

        // 调试日志（值变化时输出）
        static LAST_DEBUG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let current_hash =
            ((content_height as u32) as u64) << 32 | ((viewport_height as u32) as u64);
        let last = LAST_DEBUG.load(std::sync::atomic::Ordering::Relaxed);
        if current_hash != last {
            LAST_DEBUG.store(current_hash, std::sync::atomic::Ordering::Relaxed);
            tracing::trace!(
                "[Comics] scale={:.2}, cards={}, cols={}, rows={}, viewport={:.0}, content={:.0}, max_scroll={:.0}",
                scale_factor,
                card_count,
                columns,
                rows,
                viewport_height,
                content_height,
                (content_height - viewport_height).max(0.0)
            );
        }

        content_size_info.content_height = content_height;
        content_size_info.viewport_height = viewport_height;
    }
}

/// 刷新漫画列表界面（只处理错误状态，卡片由瀑布式系统创建）
///
/// 注意：这个函数**不应该**在数据加载完成后重建整个
/// UI，否则会覆盖瀑布式系统创建的卡片。 它只在出现错误时处理错误显示。
pub fn refresh_comics_list_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    comics_state: Res<ComicsListState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ComicsScrollContainer>>,
    card_query: Query<&ComicCard>,
    error_query: Query<Entity, With<ErrorMessage>>,
) {
    // 只在状态变化时检查
    if !comics_state.is_changed() {
        return;
    }

    // 如果有错误，显示错误信息
    if let Some(ref error) = comics_state.error {
        // 如果还没有错误信息 UI，添加它
        if error_query.is_empty() {
            if let Ok((container_entity, _)) = scroll_container_query.single() {
                let font: Handle<Font> = asset_server.load(FONT_PATH);
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
        return;
    }

    // 如果数据存在或已有卡片，让瀑布式系统处理，不干涉
    if !comics_state.comics.is_empty() {
        return;
    }

    // 检查是否已有卡片
    if let Ok((_, children)) = scroll_container_query.single() {
        let has_cards = children
            .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
            .unwrap_or(false);
        if has_cards {
            return;
        }
    }

    // 数据为空且没有卡片，不做任何操作（保持加载中状态）
}

/// 瀑布式显示漫画卡片（预创建所有隐藏卡片，然后分批显示）
pub fn waterfall_create_comic_cards(
    mut commands: Commands,
    mut creation_state: ResMut<ComicsCardCreationState>,
    comics_state: Res<ComicsListState>,
    image_cache: Res<ImageCache>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ComicsScrollContainer>>,
    card_query: Query<&ComicCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    // 如果数据已加载但 creation_state 未启动，主动启动预创建
    // （解决系统执行顺序导致 is_changed() 检测失败的问题）
    if !creation_state.is_creating
        && !comics_state.comics.is_empty()
        && comics_state.error.is_none()
    {
        // 检查当前容器中是否有卡片
        if let Ok((container_entity, children)) = scroll_container_query.single() {
            // 检查容器的子元素中是否有 ComicCard
            let has_cards = children
                .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
                .unwrap_or(false);

            if !has_cards {
                // 删除"加载中..."指示器（安全删除，实体可能已被其他系统删除）
                for entity in loading_query.iter() {
                    if let Ok(mut entity_commands) = commands.get_entity(entity) {
                        entity_commands.despawn();
                    }
                }
                let font: Handle<Font> = asset_server.load(FONT_PATH);
                creation_state.start_precreate(comics_state.comics.len(), font);
                tracing::debug!("自动启动漫画卡片预创建: {} 个", comics_state.comics.len());
            }
            let _ = container_entity; // suppress warning
        }
    }

    // 检查是否需要预创建
    if creation_state.needs_precreate() {
        let Ok((container_entity, _)) = scroll_container_query.single() else {
            return;
        };

        let Some(font) = creation_state.font_handle.clone() else {
            return;
        };

        let comics = &comics_state.comics;
        let count = creation_state.get_precreate_count();

        if comics.is_empty() || count == 0 {
            creation_state.clear();
            return;
        }

        // 一次性创建所有隐藏卡片
        let mut entities = Vec::with_capacity(count);
        commands.entity(container_entity).with_children(|parent| {
            for i in 0..count {
                if let Some(comic) = comics.get(i) {
                    let entity = spawn_comic_card(parent, comic, &font, &image_cache, true);
                    entities.push(entity);
                }
            }
        });

        // 设置预创建完成后的实体列表
        let entity_count = entities.len();
        creation_state.set_precreated_entities(entities);
        tracing::debug!("漫画卡片预创建完成: {} 个", entity_count);
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
        tracing::debug!("漫画卡片瀑布式显示完成: {} 个", comics_state.comics.len());
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
