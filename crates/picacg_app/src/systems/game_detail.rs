//! 游戏详情系统
//!
//! 实现游戏详情页面的 UI 和交互

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        navigation::NavigationHistory,
        scrollbar::scrollbar_config::SCROLLBAR_WIDTH,
        ui_common::{Scrollable, spawn_scrollbar},
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 游戏详情根节点
#[derive(Component)]
pub struct GameDetailRoot;

/// 游戏详情滚动容器
#[derive(Component)]
pub struct GameDetailScrollContainer;

/// 游戏详情返回按钮
#[derive(Component)]
pub struct GameDetailBackButton;

/// 游戏详情图标
#[derive(Component)]
pub struct GameDetailIcon {
    #[allow(dead_code)]
    pub url: String,
}

// ==================== 系统函数 ====================

/// 创建游戏详情界面
pub fn setup_game_detail_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    game_detail_state: Res<GameDetailState>,
    image_cache: Res<ImageCache>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_detail_messages: MessageWriter<LoadGameDetailRequest>,
    existing_query: Query<Entity, With<GameDetailRoot>>,
) {
    // 每次进入游戏详情页面都销毁旧的重建（不同游戏的数据不同，不适合缓存）
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let detail_root =
        create_game_detail_ui_internal(&mut commands, &font, &game_detail_state, &image_cache);

    // 挂载到内容区域
    if let Some(content_area) = content_area {
        commands.entity(content_area).add_child(detail_root);
    }

    // 如果数据未加载，触发加载
    if !game_detail_state.game_id.is_empty()
        && game_detail_state.game.is_none()
        && !game_detail_state.is_loading
    {
        load_detail_messages.write(LoadGameDetailRequest {
            game_id: game_detail_state.game_id.clone(),
        });
    }
}

/// 内部创建游戏详情 UI（供 setup 和 refresh 共用）
fn create_game_detail_ui_internal(
    commands: &mut Commands,
    font: &Handle<Font>,
    game_detail_state: &GameDetailState,
    image_cache: &ImageCache,
) -> Entity {
    commands
        .spawn((
            GameDetailRoot,
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
                // 返回按钮
                header
                    .spawn((
                        GameDetailBackButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_CHEVRON_LEFT),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                header.spawn((
                    Text::new("游戏详情"),
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
                        GameDetailScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect {
                                left: Val::Px(20.0),
                                right: Val::Px(20.0 + SCROLLBAR_WIDTH),
                                top: Val::Px(20.0),
                                bottom: Val::Px(20.0),
                            },
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        Scrollable,
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|content| {
                        if game_detail_state.is_loading {
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
                        } else if let Some(ref error) = game_detail_state.error {
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
                        } else if let Some(ref game) = game_detail_state.game {
                            spawn_game_detail_content(content, font, game, image_cache);
                        } else {
                            content.spawn((
                                Text::new("暂无数据"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
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
        .id()
}

/// 创建游戏详情内容
fn spawn_game_detail_content(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    game: &picacg_api::models::Game,
    image_cache: &ImageCache,
) {
    // 基本信息区域（图标 + 基本信息）
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        })
        .with_children(|header| {
            // 游戏图标
            let icon_url = game.icon.url();
            let mut icon_entity = header.spawn((
                GameDetailIcon {
                    url: icon_url.clone(),
                },
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(120.0),
                    min_width: Val::Px(120.0),
                    min_height: Val::Px(120.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                BorderColor::all(AppColors::BORDER),
            ));

            // 尝试从缓存加载图标
            if let Some(handle) = image_cache.get(&icon_url) {
                icon_entity.with_children(|icon_area| {
                    icon_area.spawn((
                        ImageNode {
                            image: handle.clone(),
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(20.0)),
                            ..default()
                        },
                    ));
                });
            } else {
                icon_entity.with_children(|icon_area| {
                    icon_area.spawn((
                        Text::new(ICON_GAMEPAD),
                        TextFont {
                            font: font.clone(),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });
            }

            // 基本信息
            header
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|info| {
                    // 标题
                    info.spawn((
                        Text::new(&game.title),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));

                    // 发布者
                    if let Some(ref publisher) = game.publisher {
                        spawn_info_row(info, font, "开发者", publisher);
                    }

                    // 版本
                    if let Some(ref version) = game.version {
                        spawn_info_row(info, font, "版本", version);
                    }

                    // 互动数据
                    info.spawn(Node {
                        column_gap: Val::Px(15.0),
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    })
                    .with_children(|stats| {
                        if let Some(likes) = game.likes_count {
                            spawn_stat_badge(stats, font, ICON_HEART, &format!("{}", likes));
                        }
                        if let Some(comments) = game.comments_count {
                            spawn_stat_badge(stats, font, "💬", &format!("{}", comments));
                        }
                    });
                });
        });

    // 分隔线
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(AppColors::BORDER),
    ));

    // 描述区域
    spawn_section(parent, font, "简介", &game.description);

    // 更新内容
    if let Some(ref update_content) = game.update_content
        && !update_content.is_empty()
    {
        spawn_section(parent, font, "更新内容", update_content);
    }

    // 下载链接区域
    let has_android = game.android_link.is_some()
        || game
            .android_links
            .as_ref()
            .is_some_and(|links| !links.is_empty());
    let has_ios = game.ios_link.is_some()
        || game
            .ios_links
            .as_ref()
            .is_some_and(|links| !links.is_empty());

    if has_android || has_ios {
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(1.0),
                margin: UiRect::vertical(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(AppColors::BORDER),
        ));

        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(8.0),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            })
            .with_children(|links_section| {
                links_section.spawn((
                    Text::new("下载链接"),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                // Android 链接
                if let Some(ref link) = game.android_link {
                    spawn_link_row(links_section, font, "Android", link);
                }
                if let Some(ref links) = game.android_links {
                    for (i, link) in links.iter().enumerate() {
                        let label = if links.len() > 1 {
                            format!("Android {}", i + 1)
                        } else {
                            "Android".to_string()
                        };
                        spawn_link_row(links_section, font, &label, link);
                    }
                }

                // iOS 链接
                if let Some(ref link) = game.ios_link {
                    spawn_link_row(links_section, font, "iOS", link);
                }
                if let Some(ref links) = game.ios_links {
                    for (i, link) in links.iter().enumerate() {
                        let label = if links.len() > 1 {
                            format!("iOS {}", i + 1)
                        } else {
                            "iOS".to_string()
                        };
                        spawn_link_row(links_section, font, &label, link);
                    }
                }
            });
    }

    // 截图区域
    if let Some(ref screenshots) = game.screenshots
        && !screenshots.is_empty()
    {
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(1.0),
                margin: UiRect::vertical(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(AppColors::BORDER),
        ));

        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|screenshots_section| {
                screenshots_section.spawn((
                    Text::new(format!("截图 ({})", screenshots.len())),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                // 截图列表（横向滚动）
                screenshots_section
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(10.0),
                        overflow: Overflow::scroll_x(),
                        ..default()
                    })
                    .with_children(|row| {
                        for screenshot in screenshots {
                            let url = screenshot.url();
                            let mut img_container = row.spawn((
                                GameDetailIcon { url: url.clone() },
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(150.0),
                                    min_width: Val::Px(200.0),
                                    min_height: Val::Px(150.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                                BorderColor::all(AppColors::BORDER),
                            ));

                            if let Some(handle) = image_cache.get(&url) {
                                img_container.with_children(|parent| {
                                    parent.spawn((
                                        ImageNode {
                                            image: handle.clone(),
                                            ..default()
                                        },
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            border_radius: BorderRadius::all(Val::Px(8.0)),
                                            ..default()
                                        },
                                    ));
                                });
                            } else {
                                img_container.with_children(|parent| {
                                    parent.spawn((
                                        Text::new(ICON_TIMER_SAND),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(AppColors::TEXT_SECONDARY),
                                    ));
                                });
                            }
                        }
                    });
            });
    }
}

/// 创建信息行（标签 + 值）
fn spawn_info_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    value: &str,
) {
    parent
        .spawn(Node {
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{}:", label)),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            row.spawn((
                Text::new(value),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 创建统计徽章
fn spawn_stat_badge(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon: &str,
    value: &str,
) {
    parent
        .spawn(Node {
            column_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|badge| {
            badge.spawn((
                Text::new(icon),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::PRIMARY),
            ));
            badge.spawn((
                Text::new(value),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 创建内容段落（标题 + 正文）
fn spawn_section(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, title: &str, body: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            row_gap: Val::Px(8.0),
            margin: UiRect::bottom(Val::Px(15.0)),
            ..default()
        })
        .with_children(|section| {
            section.spawn((
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
            section.spawn((
                Text::new(body),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 创建链接行
fn spawn_link_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    platform: &str,
    url: &str,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(8.0)),
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgb(0.12, 0.12, 0.16)))
        .insert(BorderColor::all(AppColors::BORDER))
        .with_children(|row| {
            // 平台标签
            row.spawn((
                Node {
                    padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(2.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(AppColors::PRIMARY),
            ))
            .with_children(|badge| {
                badge.spawn((
                    Text::new(platform),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // URL（截断显示）
            let display_url = if url.len() > 50 {
                format!("{}...", &url[..50])
            } else {
                url.to_string()
            };
            row.spawn((
                Text::new(display_url),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    flex_shrink: 1.0,
                    ..default()
                },
            ));
        });
}

/// 清理游戏详情界面（销毁 UI，参数化页面不适合缓存）
pub fn cleanup_game_detail_ui(mut commands: Commands, query: Query<Entity, With<GameDetailRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 刷新游戏详情 UI（despawn 旧根节点并重建）
pub fn refresh_game_detail_ui(
    mut commands: Commands,
    game_detail_state: Res<GameDetailState>,
    detail_root_query: Query<Entity, With<GameDetailRoot>>,
    content_area_query: Query<Entity, With<ContentArea>>,
    _asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    mut load_detail_messages: MessageWriter<LoadGameDetailRequest>,
) {
    if !game_detail_state.is_changed() {
        return;
    }

    // 如果正在加载且没数据也没错误，暂不重建
    if game_detail_state.is_loading
        && game_detail_state.game.is_none()
        && game_detail_state.error.is_none()
    {
        return;
    }

    // 删除旧 UI
    for entity in detail_root_query.iter() {
        commands.entity(entity).despawn();
    }

    // 重建（直接内联创建逻辑，不调用 setup 避免 deferred despawn 导致
    // existing_query 误判）
    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();
    let detail_root =
        create_game_detail_ui_internal(&mut commands, &font, &game_detail_state, &image_cache);

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(detail_root);
    }

    // 如果数据未加载，触发加载
    if !game_detail_state.game_id.is_empty()
        && game_detail_state.game.is_none()
        && !game_detail_state.is_loading
    {
        load_detail_messages.write(LoadGameDetailRequest {
            game_id: game_detail_state.game_id.clone(),
        });
    }
}

/// 返回按钮交互
pub fn game_detail_back_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<GameDetailBackButton>)>,
    mut back_messages: MessageWriter<NavigateBackEvent>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            back_messages.write(NavigateBackEvent);
        }
    }
}

/// 处理游戏详情滚动
pub fn handle_game_detail_scroll(
    _scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<GameDetailScrollContainer>,
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新游戏详情内容尺寸
pub fn update_game_detail_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<GameDetailScrollContainer>,
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

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 更新游戏详情图标和截图图片
pub fn update_game_detail_images(
    mut icon_query: Query<(
        Entity,
        &GameDetailIcon,
        &mut BackgroundColor,
        Option<&Children>,
    )>,
    image_cache: Res<ImageCache>,
    mut commands: Commands,
) {
    for (entity, icon, mut bg_color, children) in icon_query.iter_mut() {
        if let Some(handle) = image_cache.get(&icon.url) {
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
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                });
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 处理游戏详情加载完成事件
pub fn handle_game_detail_loaded(
    mut loaded_messages: MessageReader<GameDetailLoadedEvent>,
    mut game_detail_state: ResMut<GameDetailState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        tracing::info!("游戏详情加载完成: {}", event.game.title);

        // 触发加载图标
        image_messages.write(LoadImageRequest {
            url: event.game.icon.url(),
        });

        // 触发加载截图
        if let Some(ref screenshots) = event.game.screenshots {
            for screenshot in screenshots {
                image_messages.write(LoadImageRequest {
                    url: screenshot.url(),
                });
            }
        }

        game_detail_state.game = Some(event.game.clone());
        game_detail_state.is_loading = false;
        game_detail_state.error = None;
    }
}

/// 处理游戏详情加载失败事件
pub fn handle_game_detail_load_failed(
    mut failed_messages: MessageReader<GameDetailLoadFailedEvent>,
    mut game_detail_state: ResMut<GameDetailState>,
) {
    for event in failed_messages.read() {
        game_detail_state.is_loading = false;
        game_detail_state.error = Some(event.error.clone());
        tracing::warn!("游戏详情加载失败: {}", event.error);
    }
}
