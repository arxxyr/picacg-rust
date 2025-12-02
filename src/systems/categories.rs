//! 分类浏览系统

use bevy::{input::mouse::MouseWheel, prelude::*, ui::FocusPolicy};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::{AppColors, FONT_PATH},
        scrollbar::scrollbar_config::*,
    },
};

/// 分类卡片布局常量
mod category_layout {
    /// 卡片宽度
    pub const CARD_WIDTH: f32 = 150.0;
    /// 卡片高度
    pub const CARD_HEIGHT: f32 = 180.0;
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

/// 创建分类界面（在 ContentArea 内部）
pub fn setup_categories_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    categories_state: Res<CategoriesState>,
    image_cache: Res<ImageCache>,
    content_area_query: Query<Entity, With<ContentArea>>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    // 创建分类内容
    let categories_root = commands
        .spawn((
            CategoriesRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // 页面标题栏
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BorderColor::all(AppColors::BORDER))
            .with_children(|header| {
                header.spawn((
                    Text::new("分类浏览"),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
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
                // 分类网格容器（可滚动）
                let scroll_container_id = wrapper
                    .spawn((
                        CategoriesScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: UiRect {
                                left: Val::Px(category_layout::PADDING_LEFT),
                                right: Val::Px(category_layout::PADDING_RIGHT),
                                top: Val::Px(category_layout::PADDING_TOP),
                                bottom: Val::Px(category_layout::PADDING_BOTTOM),
                            },
                            column_gap: Val::Px(category_layout::COLUMN_GAP),
                            row_gap: Val::Px(category_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|grid| {
                        if let Some(ref error) = categories_state.error {
                            // 错误信息
                            grid.spawn((
                                ErrorMessage,
                                Text::new(error.clone()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::ERROR),
                            ));
                        } else if categories_state.categories.is_empty() {
                            // 加载中（categories 为空时显示）
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
                        } else {
                            // 分类卡片
                            for category in &categories_state.categories {
                                spawn_category_card(grid, category, &font, &image_cache);
                            }
                        }
                    })
                    .id();

                // 创建滚动条
                spawn_scrollbar_inline(wrapper, scroll_container_id);
            });
        })
        .id();

    // 如果有 ContentArea，将分类内容作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(categories_root);
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

/// 创建分类卡片
fn spawn_category_card(
    parent: &mut ChildSpawnerCommands,
    category: &crate::api::models::Category,
    font: &Handle<Font>,
    image_cache: &ImageCache,
) {
    parent
        .spawn((
            CategoryCard {
                title: category.title.clone(),
            },
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(AppColors::SURFACE),
        ))
        .with_children(|card| {
            // 图片区域
            let thumb_url = category.thumb.url();
            if let Some(handle) = image_cache.get(&thumb_url) {
                card.spawn((
                    CategoryImage { url: thumb_url },
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(134.0),
                        height: Val::Px(134.0),
                        ..default()
                    },
                ));
            } else {
                // 占位符
                card.spawn((
                    PlaceholderImage,
                    Node {
                        width: Val::Px(134.0),
                        height: Val::Px(134.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                ));
            }

            // 标题
            card.spawn((
                Text::new(&category.title),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        });
}

/// 清理分类界面
pub fn cleanup_categories_ui(mut commands: Commands, query: Query<Entity, With<CategoriesRoot>>) {
    for entity in query.iter() {
        // Bevy 0.17: despawn() 自动递归删除子实体
        commands.entity(entity).despawn();
    }
}

/// 刷新分类界面（监听 CategoriesState 变化）
pub fn refresh_categories_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    categories_state: Res<CategoriesState>,
    image_cache: Res<ImageCache>,
    root_query: Query<Entity, With<CategoriesRoot>>,
    content_area_query: Query<Entity, With<ContentArea>>,
) {
    // 只在状态变化时刷新
    if !categories_state.is_changed() {
        return;
    }

    // 如果还在加载中，不刷新（等待加载完成）
    if categories_state.is_loading && categories_state.categories.is_empty() {
        return;
    }

    // 删除旧的 UI
    for entity in root_query.iter() {
        commands.entity(entity).despawn();
    }

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    // 重新创建 UI
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    let categories_root = commands
        .spawn((
            CategoriesRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // 页面标题栏
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BorderColor::all(AppColors::BORDER))
            .with_children(|header| {
                header.spawn((
                    Text::new("分类浏览"),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
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
                // 分类网格容器（可滚动）
                let scroll_container_id = wrapper
                    .spawn((
                        CategoriesScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: UiRect {
                                left: Val::Px(category_layout::PADDING_LEFT),
                                right: Val::Px(category_layout::PADDING_RIGHT),
                                top: Val::Px(category_layout::PADDING_TOP),
                                bottom: Val::Px(category_layout::PADDING_BOTTOM),
                            },
                            column_gap: Val::Px(category_layout::COLUMN_GAP),
                            row_gap: Val::Px(category_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|grid| {
                        if let Some(ref error) = categories_state.error {
                            // 错误信息
                            grid.spawn((
                                ErrorMessage,
                                Text::new(error.clone()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::ERROR),
                            ));
                        } else if categories_state.categories.is_empty() {
                            // 仍在加载
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
                        } else {
                            // 分类卡片
                            for category in &categories_state.categories {
                                spawn_category_card(grid, category, &font, &image_cache);
                            }
                        }
                    })
                    .id();

                // 创建滚动条
                spawn_scrollbar_inline(wrapper, scroll_container_id);
            });
        })
        .id();

    // 如果有 ContentArea，将分类内容作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(categories_root);
    }
}

/// 分类卡片交互系统
pub fn category_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &CategoryCard),
        Changed<Interaction>,
    >,
    mut comics_list_state: ResMut<ComicsListState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut load_comics_messages: MessageWriter<LoadComicsRequest>,
) {
    for (interaction, mut bg_color, card) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));

                // 设置当前分类并导航
                comics_list_state.category = card.title.clone();
                comics_list_state.page = 1;
                comics_list_state.comics.clear();

                next_route.set(AppRoute::ComicsList);

                // 触发加载漫画列表
                load_comics_messages.write(LoadComicsRequest {
                    category: card.title.clone(),
                    page: 1,
                    sort: comics_list_state.sort.clone(),
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

/// 分类页面滚动处理系统
pub fn handle_categories_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut scroll_query: Query<
        (&mut ScrollPosition, &ComputedNode, Option<&ContentSizeInfo>),
        With<CategoriesScrollContainer>,
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

            // 详细日志：每次滚动时输出
            tracing::info!(
                "[Categories] 滚动: delta={:.1}, old={:.1}, new={:.1}, max={:.1}, content={:.1}, viewport={:.1}",
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

/// 限制分类页面滚动范围（防止越界）
pub fn clamp_categories_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<CategoriesScrollContainer>,
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

/// 更新分类页面内容尺寸信息
///
/// 使用手动网格计算（基于卡片数量和布局常量）。
pub fn update_categories_content_size(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut scroll_query: Query<(&ComputedNode, &mut ContentSizeInfo), With<CategoriesScrollContainer>>,
    card_query: Query<Entity, With<CategoryCard>>,
) {
    use category_layout::*;

    // 获取 scale_factor 用于日志
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
            tracing::info!(
                "[Categories] scale={:.2}, cards={}, cols={}, rows={}, viewport={:.0}, content={:.0}, max_scroll={:.0}",
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

/// 更新分类页面图片（当图片加载完成时）
pub fn update_categories_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    categories_state: Res<CategoriesState>,
    placeholder_query: Query<(Entity, &ChildOf), With<PlaceholderImage>>,
    card_query: Query<&CategoryCard>,
) {
    // 如果图片缓存没有变化，跳过
    if !image_cache.is_changed() {
        return;
    }

    for (placeholder_entity, child_of) in placeholder_query.iter() {
        // 找到父卡片
        let parent_entity: Entity = child_of.parent();
        let Ok(card) = card_query.get(parent_entity) else {
            continue;
        };

        // 找到对应的分类
        let Some(category) = categories_state
            .categories
            .iter()
            .find(|c| c.title == card.title)
        else {
            continue;
        };

        let thumb_url = category.thumb.url();

        // 检查图片是否已加载
        if let Some(handle) = image_cache.get(&thumb_url) {
            // 删除占位符，添加实际图片
            commands.entity(placeholder_entity).despawn();
            // 创建新的图片实体并插入到父卡片的第一个位置
            let image_entity = commands
                .spawn((
                    CategoryImage {
                        url: thumb_url.clone(),
                    },
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(134.0),
                        height: Val::Px(134.0),
                        ..default()
                    },
                ))
                .id();

            // 插入到第一个位置（在标题之前）
            commands
                .entity(parent_entity)
                .insert_children(0, &[image_entity]);
        }
    }
}
