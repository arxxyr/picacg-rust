//! 聊天大厅系统
//!
//! 展示聊天房间列表，点击进入聊天室

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

/// 聊天大厅根节点
#[derive(Component)]
pub struct ChatRoot;

/// 聊天大厅滚动容器
#[derive(Component)]
pub struct ChatScrollContainer;

/// 聊天房间卡片
#[derive(Component)]
pub struct ChatRoomCard {
    pub room_id: String,
    pub room_title: String,
}

/// 聊天房间图标
#[derive(Component)]
pub struct ChatRoomIcon {
    pub url: String,
}

/// 刷新按钮
#[derive(Component)]
pub struct ChatRefreshButton;

// ==================== 布局常量 ====================

mod chat_layout {
    /// 卡片间距
    pub const CARD_GAP: f32 = 12.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 30.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 30.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 15.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
}

// ==================== 系统函数 ====================

/// 创建聊天大厅界面（如果已存在则只显示）
pub fn setup_chat_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    chat_state: Res<ChatState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_rooms_messages: MessageWriter<LoadChatRoomsRequest>,
    mut existing_query: Query<&mut Node, With<ChatRoot>>,
) {
    // 如果 ChatRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if chat_state.rooms.is_empty() && !chat_state.is_loading {
            load_rooms_messages.write(LoadChatRoomsRequest);
        }
        return;
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let chat_root = commands
        .spawn((
            ChatRoot,
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
                    Text::new(ICON_CHAT),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(AppColors::PRIMARY),
                ));
                // 标题
                header.spawn((
                    Text::new("聊天室"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 弹性空间
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // 刷新按钮
                header
                    .spawn((
                        ChatRefreshButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.0),
                            ..default()
                        },
                        BorderColor::all(AppColors::BORDER),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_REFRESH),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                        btn.spawn((
                            Text::new("刷新"),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });
            });

            // 内容区域（可滚动）
            root.spawn((Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },))
                .with_children(|wrapper| {
                    let scroll_container = wrapper
                        .spawn((
                            ChatScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                overflow: Overflow::scroll_y(),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect {
                                    left: Val::Px(chat_layout::PADDING_LEFT),
                                    right: Val::Px(chat_layout::PADDING_RIGHT),
                                    top: Val::Px(chat_layout::PADDING_TOP),
                                    bottom: Val::Px(chat_layout::PADDING_BOTTOM),
                                },
                                row_gap: Val::Px(chat_layout::CARD_GAP),
                                ..default()
                            },
                            Scrollable,
                            ScrollPosition::default(),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|scroll| {
                            // 加载中提示
                            if chat_state.is_loading {
                                spawn_loading_indicator(scroll, &font);
                            } else if let Some(ref error) = chat_state.error {
                                spawn_error_message(scroll, &font, error);
                            } else if chat_state.rooms.is_empty() {
                                spawn_empty_hint(scroll, &font, "点击刷新加载聊天房间列表");
                            } else {
                                // 渲染房间列表
                                for room in &chat_state.rooms {
                                    spawn_room_card(scroll, &font, room);
                                }
                            }
                        })
                        .id();

                    // 滚动条
                    spawn_scrollbar(wrapper, scroll_container);
                });
        })
        .id();

    // 挂载到内容区域
    if let Some(content) = content_area {
        commands.entity(content).add_children(&[chat_root]);
    }

    // 如果没有房间数据，自动加载
    if chat_state.rooms.is_empty() && !chat_state.is_loading {
        load_rooms_messages.write(LoadChatRoomsRequest);
    }
}

/// 清理聊天大厅界面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_chat_ui(mut query: Query<&mut Node, With<ChatRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 创建房间卡片
fn spawn_room_card(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    room: &picacg_api::endpoints::chat::ChatRoom,
) {
    let is_available = room.is_available;
    let card_bg = if is_available {
        Color::srgb(0.12, 0.12, 0.18)
    } else {
        Color::srgb(0.08, 0.08, 0.12)
    };
    let text_color = if is_available {
        AppColors::TEXT
    } else {
        AppColors::TEXT_SECONDARY
    };

    parent
        .spawn((
            ChatRoomCard {
                room_id: room.id.clone(),
                room_title: room.title.clone(),
            },
            Button,
            Interaction::default(),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(card_bg),
        ))
        .with_children(|card| {
            // 房间图标占位
            card.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.22)),
            ))
            .with_children(|icon_area| {
                // 如果有图标 URL，标记为待加载
                if !room.icon.is_empty() {
                    icon_area.spawn((
                        ChatRoomIcon {
                            url: room.icon.clone(),
                        },
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(60.0),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                } else {
                    icon_area.spawn((
                        Text::new(ICON_CHAT),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(AppColors::PRIMARY),
                    ));
                }
            });

            // 房间信息
            card.spawn((Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(6.0),
                ..default()
            },))
                .with_children(|info| {
                    // 房间标题
                    info.spawn((
                        Text::new(&room.title),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(text_color),
                    ));

                    // 房间描述
                    if !room.description.is_empty() {
                        info.spawn((
                            Text::new(&room.description),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    }

                    // 准入条件
                    info.spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        ..default()
                    },))
                        .with_children(|row| {
                            // 等级要求
                            row.spawn((
                                Text::new(format!("LV.{} 以上", room.min_level)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));

                            // 注册天数要求
                            if room.min_register_days > 0 {
                                row.spawn((
                                    Text::new(format!("注册 {} 天以上", room.min_register_days)),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT_SECONDARY),
                                ));
                            }

                            // 可用状态
                            if !is_available {
                                row.spawn((
                                    Text::new("(不可用)"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.8, 0.3, 0.3)),
                                ));
                            }
                        });
                });

            // 右箭头
            if is_available {
                card.spawn((
                    Text::new(ICON_CHEVRON_RIGHT),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            }
        });
}

/// 加载中提示
fn spawn_loading_indicator(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|center| {
            center.spawn((
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

/// 错误提示
fn spawn_error_message(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, error: &str) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },))
        .with_children(|center| {
            center.spawn((
                Text::new("加载失败"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.3, 0.3)),
            ));
            center.spawn((
                Text::new(error),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 空状态提示
fn spawn_empty_hint(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, hint: &str) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|center| {
            center.spawn((
                Text::new(hint),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

// ==================== 交互系统 ====================

/// 房间卡片交互：点击进入聊天室
pub fn chat_room_card_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ChatRoomCard,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    chat_state: Res<ChatState>,
    mut chat_room_state: ResMut<ChatRoomState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
    current_route: Res<State<AppRoute>>,
) {
    for (interaction, card, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // 检查是否有 token
                if let Some(ref token) = chat_state.chat_token {
                    // 设置聊天室状态
                    chat_room_state.room_id = card.room_id.clone();
                    chat_room_state.room_title = card.room_title.clone();
                    chat_room_state.messages.clear();
                    chat_room_state.online_count = 0;
                    chat_room_state.input_text.clear();
                    chat_room_state.is_connected = false;
                    chat_room_state.is_connecting = false;
                    chat_room_state.error = None;
                    chat_room_state.auto_scroll = true;
                    chat_room_state.needs_rebuild = false;

                    // 关闭旧的 WebSocket 连接
                    if let Some(close_sender) = chat_room_state.ws_close_sender.take() {
                        let _ = close_sender.send(());
                    }
                    chat_room_state.ws_receiver = None;
                    chat_room_state.ws_sender = None;

                    // 导航到聊天室
                    history.push(current_route.get().clone());
                    next_route.set(AppRoute::ChatRoom);

                    let _ = token;
                } else {
                    tracing::warn!("聊天服务未登录，无法进入聊天室");
                }
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.22));
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
                *border_color = BorderColor::all(AppColors::BORDER);
            }
        }
    }
}

/// 刷新按钮交互
pub fn chat_refresh_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ChatRefreshButton>),
    >,
    mut load_rooms_messages: MessageWriter<LoadChatRoomsRequest>,
    chat_state: Res<ChatState>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if !chat_state.is_loading {
                    load_rooms_messages.write(LoadChatRoomsRequest);
                }
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 刷新聊天房间列表 UI（数据变化时重建滚动容器内容）
pub fn refresh_chat_ui(
    mut commands: Commands,
    chat_state: Res<ChatState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ChatScrollContainer>>,
) {
    if !chat_state.is_changed() {
        return;
    }

    // 跳过仅 is_loading 变化的场景
    let has_data = !chat_state.rooms.is_empty();
    let has_error = chat_state.error.is_some();
    let is_loading = chat_state.is_loading;

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
    commands.entity(scroll_entity).with_children(|scroll| {
        if is_loading {
            spawn_loading_indicator(scroll, &font);
        } else if let Some(ref error) = chat_state.error {
            spawn_error_message(scroll, &font, error);
        } else if chat_state.rooms.is_empty() {
            spawn_empty_hint(scroll, &font, "点击刷新加载聊天房间列表");
        } else {
            for room in &chat_state.rooms {
                spawn_room_card(scroll, &font, room);
            }
        }
    });
}

/// 处理聊天大厅滚动
pub fn handle_chat_scroll(
    _scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<ChatScrollContainer>,
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新聊天大厅内容尺寸
pub fn update_chat_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<ChatScrollContainer>,
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

        // 加上 padding 和 gap
        content_height += chat_layout::PADDING_TOP + chat_layout::PADDING_BOTTOM;
        if children.len() > 1 {
            content_height += chat_layout::CARD_GAP * (children.len() as f32 - 1.0);
        }

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 处理房间列表加载完成
pub fn handle_chat_rooms_loaded(
    mut loaded_messages: MessageReader<ChatRoomsLoadedEvent>,
    mut failed_messages: MessageReader<ChatRoomsLoadFailedEvent>,
    mut chat_state: ResMut<ChatState>,
) {
    for event in loaded_messages.read() {
        chat_state.is_loading = false;
        chat_state.rooms = event.rooms.clone();
        chat_state.chat_token = Some(event.token.clone());
        chat_state.profile = event.profile.clone();
        chat_state.error = None;
        tracing::info!("聊天房间列表加载完成，共 {} 个房间", chat_state.rooms.len());
    }

    for event in failed_messages.read() {
        chat_state.is_loading = false;
        chat_state.error = Some(event.error.clone());
        tracing::error!("聊天房间列表加载失败: {}", event.error);
    }
}

/// 更新房间图标图片
pub fn update_chat_room_icons(
    mut commands: Commands,
    icon_query: Query<(Entity, &ChatRoomIcon), Without<ImageNode>>,
    image_cache: Res<ImageCache>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for (entity, icon) in icon_query.iter() {
        if let Some(handle) = image_cache.get(&icon.url) {
            commands.entity(entity).insert(ImageNode {
                image: handle.clone(),
                ..default()
            });
        } else {
            image_messages.write(LoadImageRequest {
                url: icon.url.clone(),
            });
        }
    }
}
