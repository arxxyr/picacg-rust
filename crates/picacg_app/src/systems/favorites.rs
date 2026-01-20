//! 收藏列表系统
//!
//! 实现我的收藏页面

use bevy::{input::mouse::MouseWheel, prelude::*, ui::FocusPolicy, window::PrimaryWindow};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::{AppColors, FONT_PATH},
        pagination::{
            PaginationNextButton, PaginationPageText, PaginationPrevButton,
            check_pagination_interaction, spawn_pagination_controls, update_pagination_display,
        },
        scrollbar::scrollbar_config::*,
    },
};

/// 收藏页面标记类型（用于分页组件的泛型参数）
pub struct FavoritesPage;

/// 收藏卡片布局常量
mod favorites_layout {
    /// 卡片宽度
    pub const CARD_WIDTH: f32 = 180.0;
    /// 卡片高度
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
    pub const PADDING_BOTTOM: f32 = 30.0;
}

/// 收藏页根标记
#[derive(Component)]
pub struct FavoritesRoot;

/// 收藏滚动容器标记
#[derive(Component)]
pub struct FavoritesScrollContainer;

/// 收藏卡片标记
#[derive(Component)]
pub struct FavoriteCard {
    pub comic_id: String,
}

/// 收藏卡片缩略图标记
#[derive(Component)]
pub struct FavoriteThumbnail {
    /// 图片 URL（用于图片加载）
    #[allow(dead_code)]
    pub url: String,
}

/// 收藏空状态提示标记
#[derive(Component)]
pub struct FavoritesEmptyHint;

/// 收藏卡片瀑布式创建状态
#[derive(Resource, Default)]
pub struct FavoritesCardCreationState {
    /// 是否正在创建
    pub is_creating: bool,
    /// 待创建的卡片总数
    pub total_cards: usize,
    /// 当前已显示的卡片数
    pub visible_count: usize,
    /// 每帧显示的卡片数
    pub cards_per_frame: usize,
    /// 字体句柄
    pub font: Option<Handle<Font>>,
}

impl FavoritesCardCreationState {
    /// 开始预创建模式
    pub fn start_precreate(&mut self, total: usize, font: Handle<Font>) {
        self.is_creating = true;
        self.total_cards = total;
        self.visible_count = 0;
        self.cards_per_frame = 3; // 每帧显示 3 个
        self.font = Some(font);
    }

    /// 清空状态
    pub fn clear(&mut self) {
        self.is_creating = false;
        self.total_cards = 0;
        self.visible_count = 0;
        self.font = None;
    }
}

/// 创建收藏列表界面
pub fn setup_favorites_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    favorites_state: Res<FavoritesState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<FavoritesCardCreationState>,
    mut load_favorites_messages: MessageWriter<LoadFavoritesRequest>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    // 清空之前的创建状态
    creation_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    let favorites_root = commands
        .spawn((
            FavoritesRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|root| {
            // 标题栏
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
                Transform::default(),
            ))
            .with_children(|header| {
                header.spawn((
                    Text::new("我的收藏"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // 滚动区域包装器
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    ..default()
                },
                Transform::default(),
            ))
            .with_children(|wrapper| {
                // 收藏网格（可滚动）
                let scroll_container_id = wrapper
                    .spawn((
                        FavoritesScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: UiRect {
                                left: Val::Px(favorites_layout::PADDING_LEFT),
                                right: Val::Px(favorites_layout::PADDING_RIGHT),
                                top: Val::Px(favorites_layout::PADDING_TOP),
                                bottom: Val::Px(favorites_layout::PADDING_BOTTOM),
                            },
                            column_gap: Val::Px(favorites_layout::COLUMN_GAP),
                            row_gap: Val::Px(favorites_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|grid| {
                        if favorites_state.is_loading {
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
                        } else if favorites_state.comics.is_empty()
                            && favorites_state.error.is_none()
                        {
                            // 空状态提示（初始状态，数据加载后会被移除）
                            grid.spawn((
                                FavoritesEmptyHint,
                                Text::new("暂无收藏，去添加一些喜欢的漫画吧~"),
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
                spawn_scrollbar_inline(wrapper, scroll_container_id);
            });

            // 分页控件（使用通用分页组件）
            spawn_pagination_controls::<FavoritesPage>(
                root,
                &font,
                favorites_state.page.max(0) as u32,
                favorites_state.total_pages.max(0) as u32,
            );
        })
        .id();

    // 如果有 ContentArea，将收藏列表作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(favorites_root);
    }

    // 如果收藏列表为空且没有在加载，发送加载请求
    if favorites_state.comics.is_empty() && !favorites_state.is_loading {
        load_favorites_messages.write(LoadFavoritesRequest {
            page: favorites_state.page,
            sort: favorites_state.sort.clone(),
        });
    } else if !favorites_state.comics.is_empty() && !favorites_state.is_loading {
        // 启动预创建模式
        creation_state.start_precreate(favorites_state.comics.len(), font);
    }

    tracing::info!("收藏页面 UI 已创建");
}

/// 内联创建滚动条
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

/// 创建收藏卡片
fn spawn_favorite_card(
    parent: &mut ChildSpawnerCommands,
    comic: &picacg_api::models::Comic,
    font: &Handle<Font>,
    image_cache: &ImageCache,
    hidden: bool,
) -> Entity {
    parent
        .spawn((
            FavoriteCard {
                comic_id: comic.id.clone(),
            },
            Button,
            Interaction::default(),
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
                    FavoriteThumbnail {
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

            // 分类和标签容器
            if !comic.categories.is_empty() || !comic.tags.is_empty() {
                card.spawn((
                    Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(2.0),
                        max_width: Val::Px(164.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|tags_container| {
                    // 分类（蓝色）
                    for category in comic.categories.iter().take(2) {
                        spawn_tag_badge(tags_container, category, font, TagColor::Category);
                    }
                    // 标签（绿色）
                    for tag in comic.tags.iter().take(2) {
                        spawn_tag_badge(tags_container, tag, font, TagColor::Tag);
                    }
                });
            }
        })
        .id()
}

/// 标签颜色类型
enum TagColor {
    /// 分类（蓝色）
    Category,
    /// 标签（绿色）
    Tag,
}

/// 创建标签徽章
fn spawn_tag_badge(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font: &Handle<Font>,
    color_type: TagColor,
) {
    let (bg_color, text_color) = match color_type {
        TagColor::Category => (Color::srgba(0.2, 0.4, 0.8, 0.3), Color::srgb(0.6, 0.8, 1.0)),
        TagColor::Tag => (Color::srgba(0.2, 0.6, 0.4, 0.3), Color::srgb(0.5, 0.9, 0.7)),
    };

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

/// 清理收藏页面
pub fn cleanup_favorites_ui(
    mut commands: Commands,
    query: Query<Entity, With<FavoritesRoot>>,
    mut creation_state: ResMut<FavoritesCardCreationState>,
) {
    creation_state.clear();

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 收藏卡片交互系统
pub fn favorite_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &FavoriteCard),
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

/// 收藏分页按钮交互
pub fn favorites_pagination_interaction(
    mut commands: Commands,
    prev_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<PaginationPrevButton<FavoritesPage>>,
        ),
    >,
    next_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<PaginationNextButton<FavoritesPage>>,
        ),
    >,
    card_query: Query<Entity, With<FavoriteCard>>,
    mut favorites_state: ResMut<FavoritesState>,
    mut load_favorites_messages: MessageWriter<LoadFavoritesRequest>,
    mut creation_state: ResMut<FavoritesCardCreationState>,
    mut scroll_query: Query<&mut ScrollPosition, With<FavoritesScrollContainer>>,
) {
    // 使用通用分页交互检查函数
    let Some(is_next) = check_pagination_interaction::<FavoritesPage>(
        &prev_query,
        &next_query,
        favorites_state.page.max(0) as u32,
        favorites_state.total_pages.max(0) as u32,
    ) else {
        return;
    };

    // 更新页码
    if is_next {
        favorites_state.page += 1;
    } else {
        favorites_state.page -= 1;
    }

    // 删除所有旧卡片
    for entity in card_query.iter() {
        commands.entity(entity).despawn();
    }

    // 清除数据和状态
    favorites_state.comics.clear();
    favorites_state.is_loading = true;
    creation_state.clear();

    // 重置滚动位置
    for mut scroll_pos in scroll_query.iter_mut() {
        scroll_pos.y = 0.0;
    }

    // 发送加载请求
    load_favorites_messages.write(LoadFavoritesRequest {
        page: favorites_state.page,
        sort: favorites_state.sort.clone(),
    });

    tracing::debug!("切换到收藏第 {} 页", favorites_state.page);
}

/// 收藏页面滚动处理
pub fn handle_favorites_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut scroll_query: Query<
        (&mut ScrollPosition, &ComputedNode, Option<&ContentSizeInfo>),
        With<FavoritesScrollContainer>,
    >,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll_position, computed_node, content_size_info) in &mut scroll_query {
            let scroll_delta = match event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
                bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
            };

            let (content_height, viewport_height) = if let Some(info) = content_size_info {
                (info.content_height, info.viewport_height)
            } else {
                let size = computed_node.size();
                (size.y, size.y)
            };

            let max_scroll = (content_height - viewport_height).max(0.0);
            scroll_position.y = (scroll_position.y - scroll_delta).clamp(0.0, max_scroll);
        }
    }
}

/// 瀑布式创建收藏卡片
pub fn waterfall_create_favorite_cards(
    mut commands: Commands,
    mut creation_state: ResMut<FavoritesCardCreationState>,
    favorites_state: Res<FavoritesState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<FavoritesScrollContainer>>,
    card_query: Query<&FavoriteCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    empty_hint_query: Query<Entity, With<FavoritesEmptyHint>>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
) {
    // 如果没有滚动容器，退出
    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 自动检测：数据存在但没有卡片，启动预创建
    if !creation_state.is_creating
        && !favorites_state.comics.is_empty()
        && favorites_state.error.is_none()
    {
        let has_cards = children
            .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
            .unwrap_or(false);

        if !has_cards {
            // 删除加载指示器和空状态提示
            for entity in loading_query.iter() {
                commands.entity(entity).despawn();
            }
            for entity in empty_hint_query.iter() {
                commands.entity(entity).despawn();
            }

            let font: Handle<Font> = asset_server.load(FONT_PATH);
            creation_state.start_precreate(favorites_state.comics.len(), font);
        }
    }

    // 如果不在创建模式，退出
    if !creation_state.is_creating {
        return;
    }

    let Some(font) = creation_state.font.clone() else {
        return;
    };

    // 阶段1：预创建所有卡片（隐藏状态）
    let has_cards = children
        .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
        .unwrap_or(false);

    if !has_cards && creation_state.visible_count == 0 {
        // 一次性创建所有卡片（隐藏）
        commands.entity(scroll_entity).with_children(|parent| {
            for comic in favorites_state.comics.iter() {
                spawn_favorite_card(parent, comic, &font, &image_cache, true);
            }
        });
        return;
    }

    // 阶段2：逐帧显示卡片
    if creation_state.visible_count < creation_state.total_cards {
        let cards_to_show = creation_state.cards_per_frame;
        let start = creation_state.visible_count;
        let end = (start + cards_to_show).min(creation_state.total_cards);

        if let Some(children) = children {
            let card_entities: Vec<Entity> = children
                .iter()
                .filter(|e| card_query.get(*e).is_ok())
                .collect();

            for i in start..end {
                if let Some(entity) = card_entities.get(i) {
                    commands.entity(*entity).insert(Visibility::Inherited);
                }
            }
        }

        creation_state.visible_count = end;

        if creation_state.visible_count >= creation_state.total_cards {
            creation_state.is_creating = false;
            tracing::debug!("收藏卡片瀑布式创建完成: {} 个", creation_state.total_cards);
        }
    }
}

/// 更新收藏内容尺寸信息
pub fn update_favorites_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<FavoritesScrollContainer>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0);

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        // 计算内容高度（基于网格布局）
        let mut max_y: f32 = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(child) {
                let child_height = child_computed.size().y / scale_factor;
                max_y = max_y.max(child_height);
            }
        }

        // 估算行数和内容高度
        let card_count = children.len();
        let cards_per_row = ((scroll_computed.size().x / scale_factor
            - favorites_layout::PADDING_LEFT
            - favorites_layout::PADDING_RIGHT
            + favorites_layout::COLUMN_GAP)
            / (favorites_layout::CARD_WIDTH + favorites_layout::COLUMN_GAP))
            .floor()
            .max(1.0) as usize;

        let row_count = (card_count + cards_per_row - 1) / cards_per_row.max(1);
        let content_height = favorites_layout::PADDING_TOP
            + favorites_layout::PADDING_BOTTOM
            + (row_count as f32 * favorites_layout::CARD_HEIGHT)
            + ((row_count.saturating_sub(1)) as f32 * favorites_layout::ROW_GAP);

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 刷新收藏页面 UI（响应数据变化）
pub fn refresh_favorites_ui(
    favorites_state: Res<FavoritesState>,
    mut page_text_query: Query<&mut Text, With<PaginationPageText<FavoritesPage>>>,
    mut prev_btn_query: Query<
        &mut BackgroundColor,
        (
            With<PaginationPrevButton<FavoritesPage>>,
            Without<PaginationNextButton<FavoritesPage>>,
        ),
    >,
    mut next_btn_query: Query<
        &mut BackgroundColor,
        (
            With<PaginationNextButton<FavoritesPage>>,
            Without<PaginationPrevButton<FavoritesPage>>,
        ),
    >,
) {
    if !favorites_state.is_changed() {
        return;
    }

    // 使用通用分页显示更新函数
    update_pagination_display::<FavoritesPage>(
        &mut page_text_query,
        &mut prev_btn_query,
        &mut next_btn_query,
        favorites_state.page.max(0) as u32,
        favorites_state.total_pages.max(0) as u32,
    );
}

/// 处理收藏数据加载完成
pub fn handle_favorites_loaded(
    mut favorites_state: ResMut<FavoritesState>,
    mut messages: MessageReader<FavoritesLoadedEvent>,
) {
    for event in messages.read() {
        favorites_state.comics = event.comics.clone();
        favorites_state.total_pages = event.total_pages;
        favorites_state.is_loading = false;
        favorites_state.error = None;
        tracing::info!(
            "收藏列表加载完成: {} 个, 共 {} 页",
            favorites_state.comics.len(),
            favorites_state.total_pages
        );
    }
}

/// 处理收藏数据加载失败
pub fn handle_favorites_load_failed(
    mut favorites_state: ResMut<FavoritesState>,
    mut messages: MessageReader<FavoritesLoadFailedEvent>,
) {
    for event in messages.read() {
        favorites_state.is_loading = false;
        favorites_state.error = Some(event.error.clone());
        tracing::warn!("收藏列表加载失败: {}", event.error);
    }
}

/// 更新收藏封面图片（当图片加载完成时替换占位符）
pub fn update_favorites_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    favorites_state: Res<FavoritesState>,
    placeholder_query: Query<(Entity, &ChildOf), With<PlaceholderImage>>,
    card_query: Query<&FavoriteCard>,
) {
    // 每帧都检查占位符（不仅仅是 image_cache 变化时）
    let placeholder_count = placeholder_query.iter().count();
    if placeholder_count == 0 {
        return;
    }

    let mut replaced_count = 0;
    for (placeholder_entity, child_of) in placeholder_query.iter() {
        // 找到父卡片
        let parent_entity: Entity = child_of.parent();
        let Ok(card) = card_query.get(parent_entity) else {
            continue;
        };

        // 找到对应的漫画
        let Some(comic) = favorites_state
            .comics
            .iter()
            .find(|c| c.id == card.comic_id)
        else {
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
                    FavoriteThumbnail {
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
        tracing::trace!("[Favorites] 替换了 {} 个封面图片", replaced_count);
    }
}
