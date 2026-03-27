//! 游戏列表系统
//!
//! 实现游戏区页面，展示可用的游戏列表

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

/// 游戏列表根节点
#[derive(Component)]
pub struct GamesRoot;

/// 游戏列表滚动容器
#[derive(Component)]
pub struct GamesScrollContainer;

/// 游戏卡片
#[derive(Component)]
pub struct GameCard {
    pub game_id: String,
}

/// 游戏图标缩略图
#[derive(Component)]
pub struct GameIconThumbnail {
    #[allow(dead_code)]
    pub url: String,
}

/// 游戏列表分页：上一页按钮
#[derive(Component)]
pub struct GamesPrevPageButton;

/// 游戏列表分页：下一页按钮
#[derive(Component)]
pub struct GamesNextPageButton;

/// 游戏列表分页：页码文本
#[derive(Component)]
pub struct GamesPageText;

// ==================== 布局常量 ====================

mod games_layout {
    /// 卡片高度
    pub const CARD_HEIGHT: f32 = 100.0;
    /// 卡片间距
    pub const CARD_GAP: f32 = 10.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 15.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
    /// 图标宽度
    pub const ICON_SIZE: f32 = 72.0;
}

// ==================== 系统函数 ====================

/// 创建游戏列表界面（如果已存在则只显示）
pub fn setup_games_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    games_state: Res<GamesState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_games_messages: MessageWriter<LoadGamesRequest>,
    mut existing_query: Query<&mut Node, With<GamesRoot>>,
) {
    // 如果 GamesRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if games_state.games.is_empty() && !games_state.is_loading && games_state.error.is_none() {
            load_games_messages.write(LoadGamesRequest {
                page: games_state.page.max(1),
            });
        }
        return;
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let games_root = commands
        .spawn((
            GamesRoot,
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
                    column_gap: Val::Px(10.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|header| {
                // 图标
                header.spawn((
                    Text::new(ICON_GAMEPAD),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(AppColors::PRIMARY),
                ));

                header.spawn((
                    Text::new("游戏区"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
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
                // 可滚动内容区域
                let scroll_container_id = wrapper
                    .spawn((
                        GamesScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect {
                                left: Val::Px(games_layout::PADDING_LEFT),
                                right: Val::Px(games_layout::PADDING_RIGHT),
                                top: Val::Px(games_layout::PADDING_TOP),
                                bottom: Val::Px(games_layout::PADDING_BOTTOM),
                            },
                            row_gap: Val::Px(games_layout::CARD_GAP),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        Scrollable,
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|content| {
                        if games_state.is_loading {
                            content.spawn((
                                LoadingIndicator,
                                Text::new("加载中..."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        } else if let Some(ref error) = games_state.error {
                            content.spawn((
                                ErrorMessage,
                                Text::new(format!("加载失败: {}", error)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.4, 0.4)),
                            ));
                        } else if games_state.games.is_empty() {
                            content.spawn((
                                Text::new("暂无游戏"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        } else {
                            // 显示游戏列表
                            for game in &games_state.games {
                                spawn_game_card(content, &font, game);
                            }

                            // 分页控件
                            if games_state.total_pages > 1 {
                                spawn_games_pagination(
                                    content,
                                    &font,
                                    games_state.page,
                                    games_state.total_pages,
                                );
                            }
                        }

                        // 底部间距
                        content.spawn(Node {
                            height: Val::Px(30.0),
                            min_height: Val::Px(30.0),
                            ..default()
                        });
                    })
                    .id();

                // 滚动条
                spawn_scrollbar(wrapper, scroll_container_id);
            });
        })
        .id();

    // 挂载到内容区域
    if let Some(content_area) = content_area {
        commands.entity(content_area).add_child(games_root);
    }

    // 如果数据未加载，触发加载
    if games_state.games.is_empty() && !games_state.is_loading && games_state.error.is_none() {
        load_games_messages.write(LoadGamesRequest {
            page: games_state.page.max(1),
        });
    }
}

/// 创建单个游戏卡片
fn spawn_game_card(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    game: &picacg_api::models::Game,
) {
    parent
        .spawn((
            GameCard {
                game_id: game.id.clone(),
            },
            Button,
            Interaction::default(),
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(games_layout::CARD_HEIGHT),
                padding: UiRect::all(Val::Px(12.0)),
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|card| {
            // 游戏图标占位
            let icon_url = game.icon.url();
            card.spawn((
                GameIconThumbnail { url: icon_url },
                Node {
                    width: Val::Px(games_layout::ICON_SIZE),
                    height: Val::Px(games_layout::ICON_SIZE),
                    min_width: Val::Px(games_layout::ICON_SIZE),
                    min_height: Val::Px(games_layout::ICON_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|icon_area| {
                icon_area.spawn((
                    Text::new(ICON_GAMEPAD),
                    TextFont {
                        font: font.clone(),
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });

            // 文字信息区域
            card.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(4.0),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|info| {
                // 标题
                info.spawn((
                    Text::new(&game.title),
                    TextFont {
                        font: font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 描述（截取前 60 个字符）
                let desc = if game.description.len() > 60 {
                    format!(
                        "{}...",
                        &game.description.chars().take(60).collect::<String>()
                    )
                } else {
                    game.description.clone()
                };
                info.spawn((
                    Text::new(desc),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                // 发布者信息
                if let Some(ref publisher) = game.publisher {
                    info.spawn(Node {
                        column_gap: Val::Px(6.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("开发者:"),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                        row.spawn((
                            Text::new(publisher.as_str()),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::PRIMARY),
                        ));
                    });
                }
            });

            // 右侧箭头
            card.spawn((
                Text::new(ICON_CHEVRON_RIGHT),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 创建分页控件
fn spawn_games_pagination(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_page: i32,
    total_pages: i32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(15.0),
            padding: UiRect::vertical(Val::Px(15.0)),
            ..default()
        })
        .with_children(|row| {
            // 上一页
            let prev_enabled = current_page > 1;
            let prev_color = if prev_enabled {
                AppColors::PRIMARY
            } else {
                Color::srgb(0.3, 0.3, 0.3)
            };
            row.spawn((
                GamesPrevPageButton,
                Button,
                Interaction::default(),
                Node {
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(prev_color),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("上一页"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 页码
            row.spawn((
                GamesPageText,
                Text::new(format!("{} / {}", current_page, total_pages)),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));

            // 下一页
            let next_enabled = current_page < total_pages;
            let next_color = if next_enabled {
                AppColors::PRIMARY
            } else {
                Color::srgb(0.3, 0.3, 0.3)
            };
            row.spawn((
                GamesNextPageButton,
                Button,
                Interaction::default(),
                Node {
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(next_color),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("下一页"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 清理游戏列表界面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_games_ui(mut query: Query<&mut Node, With<GamesRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新游戏列表 UI（数据变化时重建滚动容器内容）
pub fn refresh_games_ui(
    mut commands: Commands,
    games_state: Res<GamesState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<GamesScrollContainer>>,
) {
    if !games_state.is_changed() {
        return;
    }

    // 只在有数据/错误变化时才重建（跳过仅 is_loading 变化的场景）
    let has_data = !games_state.games.is_empty();
    let has_error = games_state.error.is_some();
    let is_loading = games_state.is_loading;

    if is_loading && !has_data && !has_error {
        return;
    }

    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 清除滚动容器内的所有子元素
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // 重建滚动容器内容
    let font: Handle<Font> = get_font();
    commands.entity(scroll_entity).with_children(|content| {
        if is_loading {
            content.spawn((
                LoadingIndicator,
                Text::new("加载中..."),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        } else if let Some(ref error) = games_state.error {
            content.spawn((
                ErrorMessage,
                Text::new(format!("加载失败: {}", error)),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.4)),
            ));
        } else if games_state.games.is_empty() {
            content.spawn((
                Text::new("暂无游戏"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        } else {
            for game in &games_state.games {
                spawn_game_card(content, &font, game);
            }
            if games_state.total_pages > 1 {
                spawn_games_pagination(content, &font, games_state.page, games_state.total_pages);
            }
        }

        // 底部间距
        content.spawn(Node {
            height: Val::Px(30.0),
            min_height: Val::Px(30.0),
            ..default()
        });
    });
}

/// 游戏卡片点击交互：导航到游戏详情
pub fn game_card_interaction(
    interaction_query: Query<(&Interaction, &GameCard), Changed<Interaction>>,
    mut navigate_messages: MessageWriter<NavigateToGameDetailEvent>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction == Interaction::Pressed {
            tracing::info!("点击游戏卡片: game_id={}", card.game_id);
            navigate_messages.write(NavigateToGameDetailEvent {
                game_id: card.game_id.clone(),
            });
        }
    }
}

/// 分页按钮交互
pub fn games_pagination_interaction(
    prev_query: Query<&Interaction, (Changed<Interaction>, With<GamesPrevPageButton>)>,
    next_query: Query<&Interaction, (Changed<Interaction>, With<GamesNextPageButton>)>,
    mut games_state: ResMut<GamesState>,
    mut load_messages: MessageWriter<LoadGamesRequest>,
) {
    // 上一页
    for interaction in &prev_query {
        if *interaction == Interaction::Pressed && games_state.page > 1 {
            games_state.page -= 1;
            games_state.games.clear();
            games_state.is_loading = true;
            games_state.error = None;
            load_messages.write(LoadGamesRequest {
                page: games_state.page,
            });
        }
    }

    // 下一页
    for interaction in &next_query {
        if *interaction == Interaction::Pressed && games_state.page < games_state.total_pages {
            games_state.page += 1;
            games_state.games.clear();
            games_state.is_loading = true;
            games_state.error = None;
            load_messages.write(LoadGamesRequest {
                page: games_state.page,
            });
        }
    }
}

/// 处理游戏列表滚动
pub fn handle_games_scroll(
    _scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<GamesScrollContainer>,
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新游戏列表内容尺寸
pub fn update_games_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<GamesScrollContainer>,
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
        let child_count = children.len();
        if child_count > 1 {
            content_height += (child_count - 1) as f32 * games_layout::CARD_GAP;
        }

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 更新游戏图标图片
pub fn update_games_images(
    mut icon_query: Query<(
        Entity,
        &GameIconThumbnail,
        &mut BackgroundColor,
        Option<&Children>,
    )>,
    image_cache: Res<ImageCache>,
    mut commands: Commands,
) {
    for (entity, thumb, mut bg_color, children) in icon_query.iter_mut() {
        if let Some(handle) = image_cache.get(&thumb.url) {
            // 通过背景色判断是否还是占位状态
            let is_placeholder = bg_color.0 != Color::NONE;

            if is_placeholder {
                // 清除占位子节点
                if let Some(children) = children {
                    for child in children.iter() {
                        commands.entity(child).despawn();
                    }
                }
                // 添加图片节点
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        ImageNode {
                            image: handle.clone(),
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            ..default()
                        },
                    ));
                });
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 处理游戏列表加载完成事件
pub fn handle_games_loaded(
    mut loaded_messages: MessageReader<GamesLoadedEvent>,
    mut games_state: ResMut<GamesState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        games_state.games = event.games.clone();
        games_state.total_pages = event.total_pages;
        games_state.is_loading = false;
        games_state.error = None;
        tracing::info!(
            "游戏列表加载完成: {} 个, 共 {} 页",
            games_state.games.len(),
            games_state.total_pages
        );

        // 触发加载游戏图标
        for game in &games_state.games {
            image_messages.write(LoadImageRequest {
                url: game.icon.url(),
            });
        }
    }
}

/// 处理游戏列表加载失败事件
pub fn handle_games_load_failed(
    mut failed_messages: MessageReader<GamesLoadFailedEvent>,
    mut games_state: ResMut<GamesState>,
) {
    for event in failed_messages.read() {
        games_state.is_loading = false;
        games_state.error = Some(event.error.clone());
        tracing::warn!("游戏列表加载失败: {}", event.error);
    }
}
