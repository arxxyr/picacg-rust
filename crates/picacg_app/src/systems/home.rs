//! 首页系统
//!
//! 实现首页推荐漫画展示

use bevy::{input::mouse::MouseWheel, prelude::*, ui::FocusPolicy, window::PrimaryWindow};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::{AppColors, FONT_PATH},
        scrollbar::scrollbar_config::*,
    },
};

/// 首页卡片布局常量
mod home_layout {
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

/// 首页根标记
#[derive(Component)]
pub struct HomeRoot;

/// 首页滚动容器标记
#[derive(Component)]
pub struct HomeScrollContainer;

/// 首页漫画卡片标记
#[derive(Component)]
pub struct HomeComicCard {
    pub comic_id: String,
}

/// 首页卡片缩略图标记
#[derive(Component)]
pub struct HomeThumbnail {
    pub url: String,
}

/// 刷新按钮标记
#[derive(Component)]
pub struct HomeRefreshButton;

/// 首页加载指示器
#[derive(Component)]
pub struct HomeLoadingIndicator;

/// 首页卡片瀑布式创建状态
#[derive(Resource, Default)]
pub struct HomeCardCreationState {
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

impl HomeCardCreationState {
    /// 开始预创建模式
    pub fn start_precreate(&mut self, total: usize, font: Handle<Font>) {
        self.is_creating = true;
        self.total_cards = total;
        self.visible_count = 0;
        self.cards_per_frame = 3;
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

/// 创建首页界面
pub fn setup_home_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    home_state: Res<HomeState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<HomeCardCreationState>,
    mut load_recommendations: MessageWriter<LoadRecommendationsRequest>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    // 清空之前的创建状态
    creation_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    let home_root = commands
        .spawn((
            HomeRoot,
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
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BorderColor::all(AppColors::BORDER))
            .with_children(|header| {
                // 标题
                header.spawn((
                    Text::new("推荐漫画"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 刷新按钮
                header
                    .spawn((
                        HomeRefreshButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(12.0),
                                Val::Px(6.0),
                                Val::Px(6.0),
                            ),
                            ..default()
                        },
                        BackgroundColor(AppColors::PRIMARY),
                        BorderRadius::all(Val::Px(4.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("换一批"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });
            });

            // 滚动区域包装器
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
                // 内容网格（可滚动）
                let scroll_container_id = wrapper
                    .spawn((
                        HomeScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: UiRect {
                                left: Val::Px(home_layout::PADDING_LEFT),
                                right: Val::Px(home_layout::PADDING_RIGHT),
                                top: Val::Px(home_layout::PADDING_TOP),
                                bottom: Val::Px(home_layout::PADDING_BOTTOM),
                            },
                            column_gap: Val::Px(home_layout::COLUMN_GAP),
                            row_gap: Val::Px(home_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|grid| {
                        if home_state.is_loading {
                            grid.spawn((
                                HomeLoadingIndicator,
                                Text::new("加载中..."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        }
                    })
                    .id();

                // 创建滚动条
                spawn_scrollbar_inline(wrapper, scroll_container_id);
            });
        })
        .id();

    // 如果有 ContentArea，将首页作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(home_root);
    }

    // 如果推荐列表为空且没有在加载，发送加载请求
    if home_state.recommendations.is_empty() && !home_state.is_loading {
        load_recommendations.write(LoadRecommendationsRequest);
    } else if !home_state.recommendations.is_empty() && !home_state.is_loading {
        // 启动预创建模式
        creation_state.start_precreate(home_state.recommendations.len(), font);
    }

    tracing::info!("首页 UI 已创建");
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
                    ..default()
                },
                BackgroundColor(THUMB_COLOR),
                BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                ZIndex(1),
            ));
        });
}

/// 创建漫画卡片
fn spawn_home_card(
    parent: &mut ChildSpawnerCommands,
    comic: &picacg_api::models::Comic,
    font: &Handle<Font>,
    image_cache: &ImageCache,
    hidden: bool,
) -> Entity {
    parent
        .spawn((
            HomeComicCard {
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
                    HomeThumbnail {
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
                card.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    ..default()
                })
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
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(2.0)),
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

/// 清理首页
pub fn cleanup_home_ui(
    mut commands: Commands,
    query: Query<Entity, With<HomeRoot>>,
    mut creation_state: ResMut<HomeCardCreationState>,
) {
    creation_state.clear();

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 首页卡片交互系统
pub fn home_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &HomeComicCard),
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

/// 刷新按钮交互
pub fn home_refresh_button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HomeRefreshButton>),
    >,
    mut home_state: ResMut<HomeState>,
    mut load_recommendations: MessageWriter<LoadRecommendationsRequest>,
    mut creation_state: ResMut<HomeCardCreationState>,
    card_query: Query<Entity, With<HomeComicCard>>,
    mut scroll_query: Query<&mut ScrollPosition, With<HomeScrollContainer>>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.5, 0.9));

                // 清除旧卡片
                for entity in card_query.iter() {
                    commands.entity(entity).despawn();
                }

                // 清除状态
                home_state.recommendations.clear();
                home_state.is_loading = true;
                creation_state.clear();

                // 重置滚动位置
                for mut scroll_pos in scroll_query.iter_mut() {
                    scroll_pos.y = 0.0;
                }

                // 发送加载请求
                load_recommendations.write(LoadRecommendationsRequest);

                tracing::info!("刷新推荐漫画");
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.6, 1.0));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
        }
    }
}

/// 首页滚动处理
pub fn handle_home_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut scroll_query: Query<
        (&mut ScrollPosition, &ComputedNode, Option<&ContentSizeInfo>),
        With<HomeScrollContainer>,
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

/// 瀑布式创建首页卡片
pub fn waterfall_create_home_cards(
    mut commands: Commands,
    mut creation_state: ResMut<HomeCardCreationState>,
    home_state: Res<HomeState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<HomeScrollContainer>>,
    card_query: Query<&HomeComicCard>,
    loading_query: Query<Entity, With<HomeLoadingIndicator>>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
) {
    // 如果没有滚动容器，退出
    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 自动检测：数据存在但没有卡片，启动预创建
    if !creation_state.is_creating
        && !home_state.recommendations.is_empty()
        && home_state.error.is_none()
    {
        let has_cards = children
            .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
            .unwrap_or(false);

        if !has_cards {
            // 删除加载指示器
            for entity in loading_query.iter() {
                commands.entity(entity).despawn();
            }

            let font: Handle<Font> = asset_server.load(FONT_PATH);
            creation_state.start_precreate(home_state.recommendations.len(), font);
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
            for comic in home_state.recommendations.iter() {
                spawn_home_card(parent, comic, &font, &image_cache, true);
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
            tracing::debug!("首页卡片瀑布式创建完成: {} 个", creation_state.total_cards);
        }
    }
}

/// 更新首页内容尺寸信息
pub fn update_home_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<HomeScrollContainer>,
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
            - home_layout::PADDING_LEFT
            - home_layout::PADDING_RIGHT
            + home_layout::COLUMN_GAP)
            / (home_layout::CARD_WIDTH + home_layout::COLUMN_GAP))
            .floor()
            .max(1.0) as usize;

        let row_count = (card_count + cards_per_row - 1) / cards_per_row.max(1);
        let content_height = home_layout::PADDING_TOP
            + home_layout::PADDING_BOTTOM
            + (row_count as f32 * home_layout::CARD_HEIGHT)
            + ((row_count.saturating_sub(1)) as f32 * home_layout::ROW_GAP);

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 更新首页封面图片
pub fn update_home_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    home_state: Res<HomeState>,
    placeholder_query: Query<(Entity, &ChildOf), With<PlaceholderImage>>,
    card_query: Query<&HomeComicCard>,
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

        let Some(comic) = home_state
            .recommendations
            .iter()
            .find(|c| c.id == card.comic_id)
        else {
            continue;
        };

        let thumb_url = comic.thumb.url();

        if let Some(handle) = image_cache.get(&thumb_url) {
            commands.entity(placeholder_entity).despawn();
            let image_entity = commands
                .spawn((
                    HomeThumbnail {
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

            commands
                .entity(parent_entity)
                .insert_children(0, &[image_entity]);
            replaced_count += 1;
        }
    }

    if replaced_count > 0 {
        tracing::trace!("[Home] 替换了 {} 个封面图片", replaced_count);
    }
}

/// 处理推荐数据加载完成
pub fn handle_recommendations_loaded(
    mut home_state: ResMut<HomeState>,
    mut messages: MessageReader<RecommendationsLoadedEvent>,
) {
    for event in messages.read() {
        home_state.recommendations = event.comics.clone();
        home_state.is_loading = false;
        home_state.error = None;
        tracing::info!("推荐漫画加载完成: {} 个", home_state.recommendations.len());
    }
}

/// 处理推荐数据加载失败
pub fn handle_recommendations_load_failed(
    mut home_state: ResMut<HomeState>,
    mut messages: MessageReader<RecommendationsLoadFailedEvent>,
) {
    for event in messages.read() {
        home_state.is_loading = false;
        home_state.error = Some(event.error.clone());
        tracing::warn!("推荐漫画加载失败: {}", event.error);
    }
}
