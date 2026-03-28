//! 聊天室系统
//!
//! WebSocket 实时聊天室，支持消息收发和自动滚动

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use picacg_api::endpoints::chat::{ChatMessage, ChatMessageType};

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

/// 聊天室根节点
#[derive(Component)]
pub struct ChatRoomRoot;

/// 聊天室消息滚动容器
#[derive(Component)]
pub struct ChatRoomScrollContainer;

/// 聊天室输入框容器
#[derive(Component)]
pub struct ChatRoomInputContainer;

/// 聊天室输入框文本
#[derive(Component)]
pub struct ChatRoomInputText;

/// 发送按钮
#[derive(Component)]
pub struct ChatRoomSendButton;

/// 返回按钮
#[derive(Component)]
pub struct ChatRoomBackButton;

/// 在线人数文本
#[derive(Component)]
pub struct OnlineCountText;

/// 连接状态文本
#[derive(Component)]
pub struct ConnectionStatusText;

/// 消息列表容器（放在滚动容器内部）
#[derive(Component)]
pub struct ChatMessageList;

// ==================== 布局常量 ====================

mod room_layout {
    /// 消息间距
    pub const MSG_GAP: f32 = 6.0;
    /// 消息区域内边距
    pub const PADDING: f32 = 12.0;
    /// 输入栏高度
    pub const INPUT_BAR_HEIGHT: f32 = 60.0;
}

// ==================== 系统函数 ====================

/// 创建聊天室界面
pub fn setup_chat_room_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    chat_room_state: Res<ChatRoomState>,
    chat_state: Res<ChatState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut connect_messages: MessageWriter<ConnectChatRoomRequest>,
    existing_query: Query<Entity, With<ChatRoomRoot>>,
) {
    // 每次进入聊天室都销毁旧的重建（不同聊天室数据不同，不适合缓存）
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let chat_room_root = commands
        .spawn((
            ChatRoomRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // 顶部工具栏
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                    flex_shrink: 0.0,
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|header| {
                // 返回按钮
                header
                    .spawn((
                        ChatRoomBackButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_CHEVRON_LEFT),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                // 房间标题
                header.spawn((
                    Text::new(&chat_room_state.room_title),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 弹性空间
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // 连接状态
                header.spawn((
                    ConnectionStatusText,
                    Text::new("连接中..."),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                // 在线人数
                header.spawn((
                    OnlineCountText,
                    Text::new(" "),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });

            // 消息区域（中间可滚动）
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
                            ChatRoomScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                overflow: Overflow::scroll_y(),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(room_layout::PADDING)),
                                row_gap: Val::Px(room_layout::MSG_GAP),
                                ..default()
                            },
                            Scrollable,
                            ScrollPosition::default(),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|scroll| {
                            // 消息列表（初始为空）
                            scroll.spawn((
                                ChatMessageList,
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(room_layout::MSG_GAP),
                                    ..default()
                                },
                            ));
                        })
                        .id();

                    // 滚动条
                    spawn_scrollbar(wrapper, scroll_container);
                });

            // 底部输入栏
            root.spawn((
                ChatRoomInputContainer,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(room_layout::INPUT_BAR_HEIGHT),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    border: UiRect::top(Val::Px(1.0)),
                    flex_shrink: 0.0,
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
                BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
            ))
            .with_children(|input_bar| {
                // 输入框
                input_bar
                    .spawn((
                        Node {
                            flex_grow: 1.0,
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            min_height: Val::Px(36.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(AppColors::BORDER),
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                    ))
                    .with_children(|input_area| {
                        input_area.spawn((
                            ChatRoomInputText,
                            Text::new("输入消息..."),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });

                // 发送按钮
                input_bar
                    .spawn((
                        ChatRoomSendButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            min_height: Val::Px(36.0),
                            ..default()
                        },
                        BackgroundColor(AppColors::PRIMARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("发送"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        })
        .id();

    // 挂载到内容区域
    if let Some(content) = content_area {
        commands.entity(content).add_children(&[chat_room_root]);
    }

    // 发送 WebSocket 连接请求
    if let Some(ref token) = chat_state.chat_token {
        connect_messages.write(ConnectChatRoomRequest {
            room_id: chat_room_state.room_id.clone(),
            token: token.clone(),
        });
    }
}

/// 清理聊天室界面（销毁 UI + 关闭 WebSocket 连接）
pub fn cleanup_chat_room_ui(
    mut commands: Commands,
    query: Query<Entity, With<ChatRoomRoot>>,
    mut chat_room_state: ResMut<ChatRoomState>,
) {
    // 销毁聊天室 UI（参数化页面，不同聊天室数据不同，不适合缓存）
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // 关闭 WebSocket 连接（离开页面时必须断开）
    if let Some(close_sender) = chat_room_state.ws_close_sender.take() {
        let _ = close_sender.send(());
    }
    chat_room_state.is_connected = false;
    chat_room_state.is_connecting = false;
    chat_room_state.ws_receiver = None;
    chat_room_state.ws_sender = None;
}

// ==================== 消息处理系统 ====================

/// 轮询 WebSocket 接收通道，处理新消息
pub fn poll_chat_messages(mut chat_room_state: ResMut<ChatRoomState>) {
    // 从 ws_receiver 中取出所有可用消息
    let mut new_messages = Vec::new();
    let mut online_count_changed = false;

    if let Some(ref receiver_mutex) = chat_room_state.ws_receiver
        && let Ok(mut receiver) = receiver_mutex.try_lock()
    {
        // 非阻塞地取消息
        loop {
            match receiver.try_recv() {
                Ok(raw_msg) => new_messages.push(raw_msg),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    tracing::warn!("WebSocket 接收通道已断开");
                    break;
                }
            }
        }
    }

    // 解析并处理消息
    for raw_msg in new_messages {
        match serde_json::from_str::<ChatMessage>(&raw_msg) {
            Ok(msg) => {
                let parsed = msg.parse();
                match &parsed.msg_type {
                    ChatMessageType::Text | ChatMessageType::Image => {
                        chat_room_state.messages.push(parsed);
                        chat_room_state.needs_rebuild = true;
                    }
                    ChatMessageType::Connected => {
                        chat_room_state.is_connected = true;
                        chat_room_state.is_connecting = false;
                    }
                    ChatMessageType::InitialMessages => {
                        // 解析初始消息列表
                        if let Some(messages) = msg.data.get("messages")
                            && let Some(msg_array) = messages.as_array()
                        {
                            for msg_value in msg_array {
                                if let Ok(sub_msg) =
                                    serde_json::from_value::<ChatMessage>(msg_value.clone())
                                {
                                    let sub_parsed = sub_msg.parse();
                                    match &sub_parsed.msg_type {
                                        ChatMessageType::Text | ChatMessageType::Image => {
                                            chat_room_state.messages.push(sub_parsed);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            chat_room_state.needs_rebuild = true;
                        }
                    }
                    ChatMessageType::UpdateOnlineCount => {
                        if let Some(count) = msg.data.get("onlineCount")
                            && let Some(n) = count.as_u64()
                        {
                            chat_room_state.online_count = n as u32;
                            online_count_changed = true;
                        }
                    }
                    ChatMessageType::DeleteMessage => {
                        // 暂不处理消息删除
                    }
                    ChatMessageType::Unknown(_) => {}
                }
            }
            Err(e) => {
                tracing::debug!(
                    "解析 WebSocket 消息失败: {}, 原始: {}",
                    e,
                    &raw_msg[..raw_msg.len().min(200)]
                );
            }
        }
    }

    // 限制消息缓存数量
    let max = chat_room_state.max_messages;
    if chat_room_state.messages.len() > max {
        let excess = chat_room_state.messages.len() - max;
        chat_room_state.messages.drain(..excess);
    }

    let _ = online_count_changed;
}

/// 重建消息列表 UI
pub fn rebuild_chat_messages_ui(
    mut commands: Commands,
    mut chat_room_state: ResMut<ChatRoomState>,
    message_list_query: Query<(Entity, Option<&Children>), With<ChatMessageList>>,
) {
    if !chat_room_state.needs_rebuild {
        return;
    }
    chat_room_state.needs_rebuild = false;

    let font: Handle<Font> = get_font();

    for (list_entity, children) in message_list_query.iter() {
        // 清空旧消息：逐个 despawn 子实体
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 渲染所有消息
        commands.entity(list_entity).with_children(|list| {
            for msg in &chat_room_state.messages {
                spawn_chat_message_bubble(list, &font, msg);
            }
        });
    }
}

/// 创建消息气泡
fn spawn_chat_message_bubble(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    msg: &picacg_api::endpoints::chat::ParsedChatMessage,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            row_gap: Val::Px(2.0),
            ..default()
        },))
        .with_children(|bubble| {
            // 发送者信息行
            bubble
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },))
                .with_children(|row| {
                    // 用户名
                    row.spawn((
                        Text::new(&msg.sender_name),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(AppColors::PRIMARY),
                    ));

                    // 等级
                    if msg.sender_level > 0 {
                        row.spawn((
                            Text::new(format!("LV.{}", msg.sender_level)),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    }

                    // 称号
                    if !msg.sender_title.is_empty() {
                        row.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BorderColor::all(AppColors::TEXT_SECONDARY),
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                Text::new(&msg.sender_title),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        });
                    }

                    // 角色标签（VIP 等）
                    for character in &msg.sender_characters {
                        let (label, color) = match character.as_str() {
                            "vip" => ("VIP", Color::srgb(1.0, 0.84, 0.0)),
                            "girl" => ("♀", Color::srgb(1.0, 0.5, 0.7)),
                            "manager" => ("管理", Color::srgb(0.3, 0.8, 0.3)),
                            "official" => ("官方", Color::srgb(0.3, 0.6, 1.0)),
                            _ => continue,
                        };
                        row.spawn((
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }

                    // 时间戳
                    if !msg.created_at.is_empty() {
                        // 简单提取时间部分（HH:MM:SS）
                        let time_str = extract_time(&msg.created_at);
                        row.spawn((
                            Text::new(time_str),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.7)),
                        ));
                    }
                });

            // @提及
            for mention in &msg.mentions {
                bubble.spawn((
                    Text::new(format!("@{}", mention)),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.2, 0.4, 0.8)),
                ));
            }

            // 回复信息
            if let Some(ref reply) = msg.reply {
                bubble
                    .spawn((
                        Node {
                            padding: UiRect::all(Val::Px(6.0)),
                            border: UiRect::left(Val::Px(3.0)),
                            margin: UiRect::vertical(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.3, 0.3, 0.5, 0.5)),
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.5)),
                    ))
                    .with_children(|reply_area| {
                        reply_area.spawn((
                            Text::new(format!("@{}: {}", reply.name, reply.message)),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });
            }

            // 消息正文
            if !msg.message.is_empty() {
                bubble.spawn((
                    Text::new(&msg.message),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            }

            // 图片消息的图片 URL（简单显示）
            for url in &msg.media_urls {
                bubble.spawn((
                    Text::new(format!("[图片] {}", url)),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.6, 0.8)),
                ));
            }
        });
}

/// 从 ISO 时间字符串提取 HH:MM:SS
fn extract_time(datetime: &str) -> String {
    // 尝试解析 ISO 格式：2024-01-01T12:34:56.789Z
    if let Some(t_pos) = datetime.find('T') {
        let time_part = &datetime[t_pos + 1..];
        // 取前 8 个字符（HH:MM:SS）
        let end = time_part
            .find('.')
            .or_else(|| time_part.find('Z'))
            .unwrap_or(time_part.len())
            .min(8);
        return time_part[..end].to_string();
    }
    datetime.to_string()
}

/// 更新连接状态文本
pub fn update_connection_status(
    chat_room_state: Res<ChatRoomState>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<ConnectionStatusText>>,
    mut online_query: Query<&mut Text, (With<OnlineCountText>, Without<ConnectionStatusText>)>,
) {
    if !chat_room_state.is_changed() {
        return;
    }

    for (mut text, mut color) in status_query.iter_mut() {
        if chat_room_state.is_connected {
            **text = "已连接".to_string();
            *color = TextColor(Color::srgb(0.3, 0.8, 0.3));
        } else if chat_room_state.is_connecting {
            **text = "连接中...".to_string();
            *color = TextColor(AppColors::TEXT_SECONDARY);
        } else if chat_room_state.error.is_some() {
            **text = "连接失败".to_string();
            *color = TextColor(Color::srgb(0.8, 0.3, 0.3));
        } else {
            **text = "未连接".to_string();
            *color = TextColor(AppColors::TEXT_SECONDARY);
        }
    }

    for mut text in online_query.iter_mut() {
        if chat_room_state.online_count > 0 {
            **text = format!("在线: {}", chat_room_state.online_count);
        }
    }
}

/// 自动滚动到底部
pub fn auto_scroll_chat(
    chat_room_state: Res<ChatRoomState>,
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<ChatRoomScrollContainer>,
    >,
) {
    if !chat_room_state.auto_scroll || !chat_room_state.is_changed() {
        return;
    }

    for (mut scroll_pos, content_info) in scroll_query.iter_mut() {
        if let Some(info) = content_info {
            let max_scroll = (info.content_height - info.viewport_height).max(0.0);
            scroll_pos.y = max_scroll;
        }
    }
}

// ==================== 交互系统 ====================

/// 返回按钮交互
pub fn chat_room_back_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ChatRoomBackButton>),
    >,
    mut back_events: MessageWriter<NavigateBackEvent>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                back_events.write(NavigateBackEvent);
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

/// 发送按钮交互
pub fn chat_room_send_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ChatRoomSendButton>),
    >,
    mut chat_room_state: ResMut<ChatRoomState>,
    mut send_messages: MessageWriter<SendChatMessageRequest>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let text = chat_room_state.input_text.trim().to_string();
                if !text.is_empty() && chat_room_state.is_connected {
                    let room_id = chat_room_state.room_id.clone();
                    send_messages.write(SendChatMessageRequest {
                        room_id,
                        message: text,
                    });
                    chat_room_state.input_text.clear();
                }
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
        }
    }
}

/// 聊天室键盘输入
pub fn chat_room_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut chat_room_state: ResMut<ChatRoomState>,
    mut send_messages: MessageWriter<SendChatMessageRequest>,
    mut input_text_query: Query<(&mut Text, &mut TextColor), With<ChatRoomInputText>>,
) {
    for event in keyboard_events.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            bevy::input::keyboard::Key::Backspace => {
                chat_room_state.input_text.pop();
            }
            bevy::input::keyboard::Key::Enter => {
                let text = chat_room_state.input_text.trim().to_string();
                if !text.is_empty() && chat_room_state.is_connected {
                    let room_id = chat_room_state.room_id.clone();
                    send_messages.write(SendChatMessageRequest {
                        room_id,
                        message: text,
                    });
                    chat_room_state.input_text.clear();
                }
            }
            bevy::input::keyboard::Key::Character(input) => {
                for c in input.chars() {
                    if !c.is_control() {
                        chat_room_state.input_text.push(c);
                    }
                }
            }
            _ => {}
        }
    }

    // 更新输入框文本显示
    for (mut text, mut color) in input_text_query.iter_mut() {
        if chat_room_state.input_text.is_empty() {
            **text = "输入消息...".to_string();
            *color = TextColor(AppColors::TEXT_SECONDARY);
        } else {
            **text = chat_room_state.input_text.clone();
            *color = TextColor(AppColors::TEXT);
        }
    }
}

/// 聊天室 IME 输入
pub fn chat_room_ime_input(
    mut ime_events: MessageReader<bevy::window::Ime>,
    mut chat_room_state: ResMut<ChatRoomState>,
) {
    for event in ime_events.read() {
        if let bevy::window::Ime::Commit { value, .. } = event {
            chat_room_state.input_text.push_str(value);
        }
    }
}

/// 聊天室滚动处理
pub fn handle_chat_room_scroll(
    scroll_query: Query<
        (&ScrollPosition, Option<&ContentSizeInfo>),
        (Changed<ScrollPosition>, With<ChatRoomScrollContainer>),
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
    mut chat_room_state: ResMut<ChatRoomState>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
    // 这里只监听 ScrollPosition 变化来更新自动滚动状态
    for (scroll_pos, content_info) in scroll_query.iter() {
        let max_scroll = content_info
            .map(|info| (info.content_height - info.viewport_height).max(0.0))
            .unwrap_or(0.0);

        // 如果用户滚动到底部附近，恢复自动滚动；否则禁止自动滚动
        if max_scroll > 0.0 {
            chat_room_state.auto_scroll = (max_scroll - scroll_pos.y) < 50.0;
        }
    }
}

/// 更新聊天室内容尺寸
pub fn update_chat_room_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<ChatRoomScrollContainer>,
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

        content_height += room_layout::PADDING * 2.0;

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 处理发送消息响应
pub fn handle_send_chat_message_response(mut messages: MessageReader<SendChatMessageResponse>) {
    for event in messages.read() {
        if !event.success
            && let Some(ref error) = event.error
        {
            tracing::error!("发送聊天消息失败: {}", error);
        }
    }
}
