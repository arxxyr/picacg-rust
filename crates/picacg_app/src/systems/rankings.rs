//! 排行榜页面系统

use bevy::{prelude::*, window::PrimaryWindow};
use picacg_api::endpoints::RankTimeType;

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::scrollbar_config::SCROLLBAR_WIDTH,
        ui_common::{
            GridLayoutParams, TagColor, calculate_scroll_delta, format_number,
            measure_grid_content_height, spawn_comic_time_info, spawn_scrollbar,
            spawn_tag_badge_truncated, truncate_text,
        },
        waterfall::{RankingsCardCreationState, RankingsContext},
    },
    utils::content_filter::{
        FilterConfig, filter_comic_indices, load_filter_flags, load_filter_keywords,
    },
};

/// 滚动容器标记组件
#[derive(Component)]
pub struct ScrollContainer;

// ==================== 组件 ====================

/// 排行榜页面根标记
#[derive(Component)]
pub struct RankingsRoot;

/// 排行榜滚动容器标记
#[derive(Component)]
pub struct RankingsScrollContainer;

/// 排行榜内容容器（预留）
#[derive(Component)]
#[allow(dead_code)]
pub struct RankingsContentContainer;

/// Tab 按钮标记
#[derive(Component)]
pub struct RankingsTabButton {
    pub time_type: RankTimeType,
}

/// 排行榜卡片标记
#[derive(Component)]
pub struct RankingsComicCard {
    pub comic_id: String,
    /// 排名（用于显示）
    #[allow(dead_code)]
    pub rank: usize,
}

/// 排行榜封面图片标记
#[derive(Component)]
pub struct RankingsComicImage {
    pub comic_id: String,
}

/// 排名标签标记
#[derive(Component)]
pub struct RankBadge;

/// 加载中指示器标记
#[derive(Component)]
pub struct RankingsLoadingIndicator;

// ==================== 布局常量 ====================

mod layout {
    pub const CARD_WIDTH: f32 = 160.0;
    pub const CARD_HEIGHT: f32 = 300.0;
    pub const COVER_HEIGHT: f32 = 200.0;
    pub const COLUMN_GAP: f32 = 16.0;
    pub const ROW_GAP: f32 = 16.0;
    pub const PADDING_LEFT: f32 = 20.0;
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    pub const PADDING_TOP: f32 = 20.0;
    pub const PADDING_BOTTOM: f32 = 30.0;
}

// ==================== 设置/清理 ====================

/// 创建排行榜 UI
pub fn setup_rankings_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
    rankings_state: Res<RankingsState>,
) {
    let font: Handle<Font> = get_font();

    let content_area = match content_area_query.iter().next() {
        Some(entity) => entity,
        None => {
            tracing::warn!("排行榜页：找不到内容区域");
            return;
        }
    };

    commands.entity(content_area).with_children(|parent| {
        parent
            .spawn((
                RankingsRoot,
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
                // Tab 栏
                spawn_tab_bar(root, &font, &rankings_state);

                // 滚动区域包装器（与收藏/分类一致的结构）
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
                    // 滚动容器（直接使用 Wrap，不嵌套 ContentContainer）
                    let scroll_container_id = wrapper
                        .spawn((
                            RankingsScrollContainer,
                            ScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::FlexStart,
                                align_content: AlignContent::FlexStart,
                                padding: UiRect {
                                    left: Val::Px(layout::PADDING_LEFT),
                                    right: Val::Px(layout::PADDING_RIGHT),
                                    top: Val::Px(layout::PADDING_TOP),
                                    bottom: Val::Px(layout::PADDING_BOTTOM),
                                },
                                column_gap: Val::Px(layout::COLUMN_GAP),
                                row_gap: Val::Px(layout::ROW_GAP),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            ScrollPosition::default(),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|grid| {
                            if rankings_state.is_loading {
                                spawn_loading_indicator(grid, &font);
                            }
                        })
                        .id();

                    // 滚动条
                    spawn_scrollbar(wrapper, scroll_container_id);
                });
            });
    });

    tracing::info!("排行榜 UI 已创建");
}

/// 创建 Tab 栏
fn spawn_tab_bar(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, state: &RankingsState) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                padding: UiRect::horizontal(Val::Px(layout::PADDING_LEFT)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(AppColors::CARD_BG),
            Transform::default(),
        ))
        .with_children(|bar| {
            // 标题
            bar.spawn((
                Text::new("🏆 排行榜"),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    margin: UiRect::right(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Tab 按钮
            for time_type in [RankTimeType::H24, RankTimeType::D7, RankTimeType::D30] {
                let is_active = state.current_type == time_type;
                spawn_tab_button(bar, font, time_type, is_active);
            }
        });
}

/// 创建 Tab 按钮
fn spawn_tab_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    time_type: RankTimeType,
    is_active: bool,
) {
    let bg_color = if is_active {
        AppColors::PRIMARY
    } else {
        Color::srgba(0.2, 0.2, 0.25, 0.8)
    };

    parent
        .spawn((
            RankingsTabButton { time_type },
            Button,
            Interaction::default(),
            Node {
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            Transform::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(time_type.display_name()),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 清理排行榜 UI
pub fn cleanup_rankings_ui(
    mut commands: Commands,
    query: Query<Entity, With<RankingsRoot>>,
    mut creation_state: ResMut<RankingsCardCreationState>,
) {
    // 清空瀑布式创建状态（防止对已销毁的 Entity 操作）
    creation_state.clear();

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ==================== 交互系统 ====================

/// Tab 按钮交互
pub fn rankings_tab_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &RankingsTabButton),
        Changed<Interaction>,
    >,
    mut rankings_state: ResMut<RankingsState>,
    mut load_messages: MessageWriter<LoadRankingsRequest>,
    mut creation_state: ResMut<RankingsCardCreationState>,
    card_query: Query<Entity, With<RankingsComicCard>>,
    mut scroll_query: Query<&mut ScrollPosition, With<RankingsScrollContainer>>,
) {
    for (interaction, mut bg_color, tab) in interaction_query.iter_mut() {
        let is_active = rankings_state.current_type == tab.time_type;

        match *interaction {
            Interaction::Pressed => {
                if !is_active {
                    let start = std::time::Instant::now();

                    // 立即清除旧卡片
                    for entity in card_query.iter() {
                        commands.entity(entity).despawn();
                    }

                    // 清除瀑布流状态
                    creation_state.clear();

                    // 重置滚动位置
                    for mut scroll_pos in scroll_query.iter_mut() {
                        scroll_pos.y = 0.0;
                    }

                    // 切换当前类型
                    rankings_state.current_type = tab.time_type;
                    *bg_color = BackgroundColor(AppColors::PRIMARY);

                    // 如果该类型还没有加载数据，发送加载请求
                    if !rankings_state.is_loaded(tab.time_type) {
                        rankings_state.is_loading = true;
                        load_messages.write(LoadRankingsRequest {
                            time_type: tab.time_type,
                        });
                        tracing::info!(
                            "切换到 {} 榜（需要加载）: {:?}",
                            tab.time_type.display_name(),
                            start.elapsed()
                        );
                    } else {
                        tracing::info!(
                            "切换到 {} 榜（使用缓存）: {:?}",
                            tab.time_type.display_name(),
                            start.elapsed()
                        );
                    }
                }
            }
            Interaction::Hovered => {
                if !is_active {
                    *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.9));
                }
            }
            Interaction::None => {
                if !is_active {
                    *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.8));
                }
            }
        }
    }
}

/// 漫画卡片点击交互
pub fn rankings_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &RankingsComicCard),
        Changed<Interaction>,
    >,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, mut bg_color, card) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.9));
                // 通过导航消息跳转到详情页（保留导航历史）
                detail_messages.write(NavigateToComicDetailEvent {
                    comic_id: card.comic_id.clone(),
                });
                tracing::info!("点击排行榜漫画: {}", card.comic_id);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 1.0));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::CARD_BG);
            }
        }
    }
}

// ==================== 刷新系统 ====================

/// 刷新排行榜 UI（只处理 Tab 按钮和加载状态）
pub fn refresh_rankings_ui(
    mut commands: Commands,
    rankings_state: Res<RankingsState>,
    _asset_server: Res<AssetServer>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<RankingsScrollContainer>>,
    tab_query: Query<(Entity, &RankingsTabButton)>,
    card_query: Query<&RankingsComicCard>,
) {
    if !rankings_state.is_changed() {
        return;
    }

    let start = std::time::Instant::now();
    tracing::debug!("refresh_rankings_ui 开始");

    let font: Handle<Font> = get_font();

    // 更新 Tab 按钮状态
    for (entity, tab) in tab_query.iter() {
        let is_active = rankings_state.current_type == tab.time_type;
        let bg_color = if is_active {
            AppColors::PRIMARY
        } else {
            Color::srgba(0.2, 0.2, 0.25, 0.8)
        };
        commands.entity(entity).insert(BackgroundColor(bg_color));
    }

    // 如果已有卡片或数据已加载，让 waterfall_create_cards 处理
    // 这里只处理加载中和空状态的显示
    let comics = rankings_state.current_comics();
    if !comics.is_empty() || !card_query.is_empty() {
        tracing::debug!(
            "refresh_rankings_ui 完成（数据由 waterfall 处理）: {:?}",
            start.elapsed()
        );
        return;
    }

    // 更新内容区域（只在加载中或空状态时）
    let Ok((container_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 清除现有内容
    if let Some(children) = children {
        for child in children.iter() {
            if let Ok(mut entity_commands) = commands.get_entity(child) {
                entity_commands.despawn();
            }
        }
    }

    if rankings_state.is_loading {
        // 显示加载中
        commands.entity(container_entity).with_children(|parent| {
            spawn_loading_indicator(parent, &font);
        });
    } else {
        // 显示空状态
        commands.entity(container_entity).with_children(|parent| {
            spawn_empty_state(parent, &font, "点击上方标签加载排行榜");
        });
    }

    tracing::debug!("refresh_rankings_ui 完成: {:?}", start.elapsed());
}

/// 瀑布式显示卡片（预创建所有隐藏卡片，然后分批显示）
pub fn waterfall_create_cards(
    mut commands: Commands,
    mut creation_state: ResMut<RankingsCardCreationState>,
    rankings_state: Res<RankingsState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<RankingsScrollContainer>>,
    card_query: Query<&RankingsComicCard>,
    loading_query: Query<Entity, With<RankingsLoadingIndicator>>,
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

    // 如果数据已加载但 creation_state 未启动，主动启动预创建
    // （解决系统执行顺序导致 is_changed() 检测失败的问题）
    if !creation_state.is_creating && !rankings_state.is_loading {
        let comics = rankings_state.current_comics();
        let filtered_indices = filter_comic_indices(comics, &filter_config);
        if !filtered_indices.is_empty() {
            // 检查当前容器中是否有卡片
            if let Ok((container_entity, children)) = scroll_container_query.single() {
                // 检查容器的子元素中是否有 RankingsComicCard
                let has_cards = children
                    .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
                    .unwrap_or(false);

                // 检查类型是否匹配（处理标签切换的情况）
                let type_matches = creation_state
                    .context
                    .current_type
                    .map(|t| t == rankings_state.current_type)
                    .unwrap_or(false);

                // 如果有卡片但类型不匹配，需要清除旧卡片
                if has_cards && !type_matches {
                    tracing::debug!(
                        "排行榜类型切换，清除旧卡片: {:?} -> {:?}",
                        creation_state.context.current_type,
                        rankings_state.current_type
                    );
                    // 清除所有子元素（包括卡片和加载指示器）
                    if let Some(children) = children {
                        for child in children.iter() {
                            if let Ok(mut entity_commands) = commands.get_entity(child) {
                                entity_commands.despawn();
                            }
                        }
                    }
                    creation_state.clear();
                    // 下一帧会检测到没有卡片，启动预创建
                    return;
                }

                if !has_cards {
                    // 删除"加载中..."指示器（安全删除，实体可能已被其他系统删除）
                    for entity in loading_query.iter() {
                        if let Ok(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.despawn();
                        }
                    }
                    let font: Handle<Font> = get_font();
                    let context = RankingsContext {
                        current_type: Some(rankings_state.current_type),
                    };
                    creation_state.start_precreate_with_context(
                        filtered_indices.len(),
                        font,
                        context,
                    );
                    tracing::debug!(
                        "自动启动排行榜卡片预创建: {} 个（过滤后，{:?}）",
                        filtered_indices.len(),
                        rankings_state.current_type
                    );
                }
                let _ = container_entity; // suppress warning
            }
        }
    }

    // 确保类型匹配（防止切换 Tab 后创建错误的卡片）
    if let Some(current_type) = creation_state.context.current_type {
        if current_type != rankings_state.current_type {
            // 类型不匹配，清空状态（卡片已在上面清除）
            creation_state.clear();
            return;
        }
    } else if creation_state.is_creating {
        // 没有类型但正在创建，说明状态异常，清空
        creation_state.clear();
        return;
    }

    // 检查是否需要预创建
    if creation_state.needs_precreate() {
        let Ok((container_entity, _)) = scroll_container_query.single() else {
            return;
        };

        let Some(font) = creation_state.font_handle.clone() else {
            return;
        };

        let comics = rankings_state.current_comics();
        let filtered_indices = filter_comic_indices(comics, &filter_config);
        let count = creation_state.get_precreate_count();

        if filtered_indices.is_empty() || count == 0 {
            creation_state.clear();
            return;
        }

        // 一次性创建所有隐藏卡片（使用过滤后的索引，保留原始排名号）
        let mut entities = Vec::with_capacity(count);
        commands.entity(container_entity).with_children(|parent| {
            for i in 0..count {
                if let Some(&original_index) = filtered_indices.get(i)
                    && let Some(comic) = comics.get(original_index)
                {
                    // 排名号使用原始索引 +1，保留真实排名
                    let entity = spawn_comic_card(parent, &font, comic, original_index + 1, true);
                    entities.push(entity);
                }
            }
        });

        // 设置预创建完成后的实体列表
        creation_state.set_precreated_entities(entities);
        tracing::debug!("排行榜卡片预创建完成: {} 个（过滤后）", count);
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

    // 日志（仅在显示完成时输出）
    if !creation_state.has_pending() {
        creation_state.finish();
        tracing::info!(
            "排行榜卡片瀑布式显示完成: {} 个",
            rankings_state.current_comics().len()
        );
    }
}

/// 创建加载指示器
fn spawn_loading_indicator(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            RankingsLoadingIndicator,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(200.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|loading| {
            loading.spawn((
                Text::new("⏳"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            loading.spawn((
                Text::new("加载中..."),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 创建空状态
fn spawn_empty_state(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, message: &str) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(200.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            Transform::default(),
        ))
        .with_children(|empty| {
            empty.spawn((
                Text::new("📋"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            empty.spawn((
                Text::new(message),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 创建漫画卡片（返回 Entity，可选隐藏）
fn spawn_comic_card(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    comic: &picacg_api::models::Comic,
    rank: usize,
    hidden: bool,
) -> Entity {
    parent
        .spawn((
            RankingsComicCard {
                comic_id: comic.id.clone(),
                rank,
            },
            Button,
            Node {
                width: Val::Px(layout::CARD_WIDTH),
                height: Val::Px(layout::CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(AppColors::CARD_BG),
            BorderColor::all(AppColors::BORDER),
            if hidden {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
        ))
        .with_children(|card| {
            // 封面区域（带排名标签）
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(layout::COVER_HEIGHT),
                    position_type: PositionType::Relative,
                    ..default()
                },
                Transform::default(),
            ))
            .with_children(|cover_area| {
                // 封面图片占位
                cover_area
                    .spawn((
                        RankingsComicImage {
                            comic_id: comic.id.clone(),
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 1.0)),
                    ))
                    .with_children(|img_area| {
                        // 加载中文字
                        img_area.spawn((
                            Text::new("📖"),
                            TextFont {
                                font: font.clone(),
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });

                // 排名标签
                let (badge_color, badge_text_color) = match rank {
                    1 => (Color::srgb(1.0, 0.84, 0.0), Color::BLACK), // 金色
                    2 => (Color::srgb(0.75, 0.75, 0.75), Color::BLACK), // 银色
                    3 => (Color::srgb(0.8, 0.5, 0.2), Color::WHITE),  // 铜色
                    _ => (Color::srgba(0.0, 0.0, 0.0, 0.7), Color::WHITE),
                };

                cover_area
                    .spawn((
                        RankBadge,
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(8.0),
                            left: Val::Px(8.0),
                            padding: UiRect::new(
                                Val::Px(8.0),
                                Val::Px(8.0),
                                Val::Px(4.0),
                                Val::Px(4.0),
                            ),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(badge_color),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(format!("#{}", rank)),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(badge_text_color),
                        ));
                    });
            });

            // 信息区域
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                Transform::default(),
            ))
            .with_children(|info| {
                // 标题
                info.spawn((
                    Text::new(truncate_text(&comic.title, 12)),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 作者
                info.spawn((
                    Text::new(truncate_text(&comic.author, 10)),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                // 点赞数
                info.spawn((
                    Text::new(format!("❤️ {}", format_number(comic.likes_count))),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                // 分类和标签容器
                if !comic.categories.is_empty() || !comic.tags.is_empty() {
                    info.spawn((
                        Node {
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(3.0),
                            row_gap: Val::Px(2.0),
                            max_width: Val::Px(layout::CARD_WIDTH - 16.0),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        Transform::default(),
                    ))
                    .with_children(|tags_container| {
                        // 分类（蓝色）
                        for category in comic.categories.iter().take(2) {
                            spawn_tag_badge_truncated(
                                tags_container,
                                category,
                                font,
                                TagColor::Category,
                                6,
                            );
                        }
                        // 标签（绿色）
                        for tag in comic.tags.iter().take(2) {
                            spawn_tag_badge_truncated(tags_container, tag, font, TagColor::Tag, 6);
                        }
                    });
                }

                // 创建/更新时间
                spawn_comic_time_info(
                    info,
                    font,
                    comic.created_at.as_deref(),
                    comic.updated_at.as_deref(),
                );
            });
        })
        .id()
}

// ==================== 图片加载 ====================

/// 更新排行榜图片
pub fn update_rankings_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    image_query: Query<(Entity, &RankingsComicImage)>,
    rankings_state: Res<RankingsState>,
    children_query: Query<&Children>,
    image_node_query: Query<&ImageNode>,
) {
    // 注意：不使用 is_changed() 检查，因为系统执行顺序可能导致检测失败
    // 已设置图片的实体会通过 has_image 检查跳过，性能影响不大

    let comics = rankings_state.current_comics();

    for (entity, img_marker) in image_query.iter() {
        // 找到对应的漫画
        let Some(comic) = comics.iter().find(|c| c.id == img_marker.comic_id) else {
            continue;
        };

        let url = comic.thumb.url();

        // 检查缓存中是否有图片
        if let Some(handle) = image_cache.get(&url) {
            // 检查是否已经设置了 ImageNode（避免重复添加）
            if let Ok(children) = children_query.get(entity) {
                let has_image = children
                    .iter()
                    .any(|child| image_node_query.get(child).is_ok());
                if has_image {
                    continue;
                }
            }

            // 清除占位内容（文字等）
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }

            // 添加图片
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::top(Val::Px(8.0)),
                        ..default()
                    },
                ));
            });
        }
        // 图片加载请求已在 handle_rankings_response 中发送，无需重复请求
    }
}

// ==================== 滚动处理 ====================

/// 处理排行榜滚动
pub fn handle_rankings_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<RankingsScrollContainer>,
    >,
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        let scroll_delta = calculate_scroll_delta(event);

        for (mut scroll_pos, content_info) in scroll_query.iter_mut() {
            let max_scroll = content_info
                .map(|info| (info.content_height - info.viewport_height).max(0.0))
                .unwrap_or(0.0);
            scroll_pos.y = (scroll_pos.y - scroll_delta).clamp(0.0, max_scroll);
        }
    }
}

/// 更新排行榜内容尺寸
pub fn update_rankings_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, Option<&Children>),
        With<RankingsScrollContainer>,
    >,
    child_computed_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    use layout::*;

    let scale_factor = window_query
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

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_size = scroll_computed.size();
        let viewport_width = viewport_size.x / scale_factor;
        let viewport_height = viewport_size.y / scale_factor;

        if viewport_height <= 0.0 || viewport_width <= 0.0 {
            continue;
        }

        content_info.viewport_height = viewport_height;
        content_info.content_height = measure_grid_content_height(
            children,
            &child_computed_query,
            scale_factor,
            viewport_width,
            &layout_params,
        );
    }
}

// ==================== 进入页面时触发加载 ====================

/// 进入排行榜页面时触发加载
pub fn trigger_load_rankings(
    rankings_state: Res<RankingsState>,
    mut load_messages: MessageWriter<LoadRankingsRequest>,
) {
    let current_type = rankings_state.current_type;

    // 如果数据还没有加载，发送加载请求
    // 预创建由 waterfall_create_cards 的自动检测来处理
    if !rankings_state.is_loaded(current_type) && !rankings_state.is_loading {
        load_messages.write(LoadRankingsRequest {
            time_type: current_type,
        });
        tracing::info!("自动加载 {} 榜", current_type.display_name());
    }
}
