//! 聊天室系统
//!
//! WebSocket 实时聊天室，支持消息收发和自动滚动

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::InputFocus,
    prelude::*,
    ui::RelativeCursorPosition,
};
use picacg_api::endpoints::chat::{ChatMessage, ChatMessageType, ParsedChatMessage};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar},
        widgets::ButtonStyle,
    },
    utils::{
        icons::*,
        text_input::{TextInput, TextInputDisplay},
    },
};

// ==================== 组件定义 ====================

/// 聊天室根节点
#[derive(Component, Default, Clone)]
pub struct ChatRoomRoot;

/// 聊天室消息滚动容器
#[derive(Component, Default, Clone)]
pub struct ChatRoomScrollContainer;

/// 聊天室输入框容器
#[derive(Component, Default, Clone)]
pub struct ChatRoomInputContainer;

/// 聊天室输入框（配合通用 `TextInput` 使用）
#[derive(Component, Default, Clone)]
pub struct ChatRoomInputField;

/// 发送按钮
#[derive(Component, Default, Clone)]
pub struct ChatRoomSendButton;

/// 返回按钮
#[derive(Component, Default, Clone)]
pub struct ChatRoomBackButton;

/// 在线人数文本
#[derive(Component, Default, Clone)]
pub struct OnlineCountText;

/// 连接状态文本
#[derive(Component, Default, Clone)]
pub struct ConnectionStatusText;

/// 消息列表容器（放在滚动容器内部）
#[derive(Component, Default, Clone)]
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

    let content_area = content_area_query.single().ok();

    let chat_room_root = commands.spawn_scene(chat_room_page(&chat_room_state)).id();

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

/// 聊天室页面场景
fn chat_room_page(chat_room_state: &ChatRoomState) -> impl Scene + use<> {
    let room_title = chat_room_state.room_title.clone();

    bsn! {
        ChatRoomRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 顶部工具栏
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                    flex_shrink: 0.0,
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // 返回按钮
                        ChatRoomBackButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_CHEVRON_LEFT)
                                TextFont { font_size: FontSize::Px(18.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 房间标题
                        Text({room_title})
                        TextFont { font_size: FontSize::Px(16.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 弹性空间
                        Node { flex_grow: 1.0 }
                    ),
                    (
                        // 连接状态
                        ConnectionStatusText
                        Text("连接中...")
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        // 在线人数
                        OnlineCountText
                        Text(" ")
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            (
                // 消息区域（中间可滚动）
                Node {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                }
                Children [
                    (
                        #ChatRoomScroll
                        ChatRoomScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            overflow: Overflow::scroll_y(),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(room_layout::PADDING)),
                            row_gap: Val::Px(room_layout::MSG_GAP),
                        }
                        ScrollArea
                        Children [
                            (
                                // 消息列表（初始为空）
                                ChatMessageList
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(room_layout::MSG_GAP),
                                }
                            )
                        ]
                    ),
                    // 滚动条
                    scrollbar(#ChatRoomScroll),
                ]
            ),
            (
                // 底部输入栏
                ChatRoomInputContainer
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(room_layout::INPUT_BAR_HEIGHT),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    border: UiRect::top(Val::Px(1.0)),
                    flex_shrink: 0.0,
                }
                template_value(BorderColor::all(AppColors::BORDER))
                BackgroundColor(AppColors::HEADER_BG)
                Children [
                    (
                        // 输入框（通用 TextInput 组件：点击聚焦、光标、IME 全由通用系统接管）
                        ChatRoomInputField
                        template_value(TextInput::new("输入消息..."))
                        Button
                        Node {
                            flex_grow: 1.0,
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            min_height: Val::Px(36.0),
                            align_items: AlignItems::Center,
                        }
                        template_value(BorderColor::all(AppColors::BORDER))
                        BackgroundColor(AppColors::BACKGROUND)
                        RelativeCursorPosition
                        Children [
                            (
                                TextInputDisplay
                                Text("输入消息...")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    (
                        // 发送按钮
                        ChatRoomSendButton
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            min_height: Val::Px(36.0),
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        Children [
                            (
                                Text("发送")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(Color::WHITE)
                            )
                        ]
                    ),
                ]
            ),
        ]
    }
}

/// 清理聊天室界面（销毁 UI + 关闭 WebSocket 连接）
pub fn cleanup_chat_room_ui(
    mut commands: Commands,
    query: Query<Entity, With<ChatRoomRoot>>,
    mut chat_room_state: ResMut<ChatRoomState>,
    mut input_focus: ResMut<InputFocus>,
    input_query: Query<Entity, With<ChatRoomInputField>>,
) {
    // 交还输入焦点（IME 随焦点由 text_input_focus_visuals 关闭），
    // 否则焦点留在被销毁的输入框上，输入法状态会泄漏到下一个页面
    if let Some(focused) = input_focus.get()
        && input_query.contains(focused)
    {
        input_focus.clear();
    }

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

    for (list_entity, children) in message_list_query.iter() {
        // 清空旧消息：逐个 despawn 子实体
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 渲染所有消息
        for msg in &chat_room_state.messages {
            commands
                .spawn_scene(chat_message_bubble(msg))
                .insert(ChildOf(list_entity));
        }
    }
}

/// 消息气泡场景
fn chat_message_bubble(msg: &ParsedChatMessage) -> impl Scene + use<> {
    let sender_name = msg.sender_name.clone();

    // 等级
    let level: Box<dyn SceneList> = if msg.sender_level > 0 {
        let level_label = format!("LV.{}", msg.sender_level);
        Box::new(bsn_list![(
            Text({level_label})
            TextFont { font_size: FontSize::Px(10.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 称号
    let title: Box<dyn SceneList> = if msg.sender_title.is_empty() {
        Box::new(bsn_list![])
    } else {
        let sender_title = msg.sender_title.clone();
        Box::new(bsn_list![(
            Node {
                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
            }
            template_value(BorderColor::all(AppColors::TEXT_SECONDARY))
            Children [
                (
                    Text({sender_title})
                    TextFont { font_size: FontSize::Px(10.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                )
            ]
        )])
    };

    // 角色标签（VIP 等）
    let mut characters: Vec<Box<dyn Scene>> = Vec::new();
    for character in &msg.sender_characters {
        let (label, color) = match character.as_str() {
            "vip" => ("VIP", Color::srgb(1.0, 0.84, 0.0)),
            "girl" => ("♀", Color::srgb(1.0, 0.5, 0.7)),
            "manager" => ("管理", Color::srgb(0.3, 0.8, 0.3)),
            "official" => ("官方", Color::srgb(0.3, 0.6, 1.0)),
            _ => continue,
        };
        let label = label.to_string();
        characters.push(Box::new(bsn! {
            Text({label})
            TextFont { font_size: FontSize::Px(10.0) }
            TextColor(color)
        }));
    }

    // 时间戳
    let timestamp: Box<dyn SceneList> = if msg.created_at.is_empty() {
        Box::new(bsn_list![])
    } else {
        // 简单提取时间部分（HH:MM:SS）
        let time_str = extract_time(&msg.created_at);
        Box::new(bsn_list![(
            Text({time_str})
            TextFont { font_size: FontSize::Px(10.0) }
            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.7))
        )])
    };

    // @提及
    let mut mentions: Vec<Box<dyn Scene>> = Vec::new();
    for mention in &msg.mentions {
        let mention_label = format!("@{}", mention);
        mentions.push(Box::new(bsn! {
            Text({mention_label})
            TextFont { font_size: FontSize::Px(12.0) }
            TextColor(Color::srgb(0.2, 0.4, 0.8))
        }));
    }

    // 回复信息
    let reply: Box<dyn SceneList> = if let Some(ref reply_info) = msg.reply {
        let reply_label = format!("@{}: {}", reply_info.name, reply_info.message);
        Box::new(bsn_list![(
            Node {
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::left(Val::Px(3.0)),
                margin: UiRect::vertical(Val::Px(2.0)),
            }
            template_value(BorderColor::all(Color::srgba(0.3, 0.3, 0.5, 0.5)))
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.5))
            Children [
                (
                    Text({reply_label})
                    TextFont { font_size: FontSize::Px(11.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                )
            ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 消息正文
    let body: Box<dyn SceneList> = if msg.message.is_empty() {
        Box::new(bsn_list![])
    } else {
        let message = msg.message.clone();
        Box::new(bsn_list![(
            Text({message})
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(AppColors::TEXT)
        )])
    };

    // 图片消息的图片 URL（简单显示）
    let mut media: Vec<Box<dyn Scene>> = Vec::new();
    for url in &msg.media_urls {
        let media_label = format!("[图片] {}", url);
        media.push(Box::new(bsn! {
            Text({media_label})
            TextFont { font_size: FontSize::Px(12.0) }
            TextColor(Color::srgb(0.4, 0.6, 0.8))
        }));
    }

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            row_gap: Val::Px(2.0),
        }
        Children [
            (
                // 发送者信息行
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                }
                Children [
                    (
                        // 用户名
                        Text({sender_name})
                        TextFont { font_size: FontSize::Px(13.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                    {level},
                    {title},
                    {characters},
                    {timestamp},
                ]
            ),
            {mentions},
            {reply},
            {body},
            {media},
        ]
    }
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
            *color = TextColor(AppColors::ERROR);
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
    mut scroll_query: Query<(&ComputedNode, &mut ScrollPosition), With<ChatRoomScrollContainer>>,
) {
    if !chat_room_state.auto_scroll || !chat_room_state.is_changed() {
        return;
    }

    // 内容/视口尺寸由引擎布局输出（物理像素 → 逻辑像素）
    for (computed, mut scroll_pos) in scroll_query.iter_mut() {
        let content_h = computed.content_size().y * computed.inverse_scale_factor;
        let viewport_h = computed.size().y * computed.inverse_scale_factor;
        scroll_pos.y = (content_h - viewport_h).max(0.0);
    }
}

// ==================== 交互系统 ====================

/// 返回按钮交互
pub fn chat_room_back_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ChatRoomBackButton>)>,
    mut back_events: MessageWriter<NavigateBackEvent>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            back_events.write(NavigateBackEvent);
        }
    }
}

/// 发送按钮交互
pub fn chat_room_send_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ChatRoomSendButton>)>,
    mut input_query: Query<&mut TextInput, With<ChatRoomInputField>>,
    chat_room_state: Res<ChatRoomState>,
    mut send_messages: MessageWriter<SendChatMessageRequest>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(mut input) = input_query.single_mut() else {
            continue;
        };
        send_chat_message(&chat_room_state, &mut input, &mut send_messages);
    }
}

/// 输入框动作键（Enter 发送 / Escape 失焦）
///
/// 字符编辑、光标、剪贴板、IME 全归通用 TextInput 系统，这里只认动作键，
/// 且只在焦点确实落在聊天输入框上时才响应。
pub fn chat_room_input_action_keys(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut input_focus: ResMut<InputFocus>,
    mut input_query: Query<&mut TextInput, With<ChatRoomInputField>>,
    chat_room_state: Res<ChatRoomState>,
    mut send_messages: MessageWriter<SendChatMessageRequest>,
) {
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut input) = input_query.get_mut(focused) else {
        return;
    };

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Enter => send_chat_message(&chat_room_state, &mut input, &mut send_messages),
            Key::Escape => input_focus.clear(),
            _ => {}
        }
    }
}

/// 发送输入框里的消息并清空（发送按钮与 Enter 共用）
///
/// 未连接或内容为空时不发送，也不清空输入 —— 重连后内容还在。
fn send_chat_message(
    chat_room_state: &ChatRoomState,
    input: &mut TextInput,
    send_messages: &mut MessageWriter<SendChatMessageRequest>,
) {
    let message = input.value.trim().to_string();
    if message.is_empty() || !chat_room_state.is_connected {
        return;
    }

    send_messages.write(SendChatMessageRequest {
        room_id: chat_room_state.room_id.clone(),
        message,
    });
    input.set_value("");
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
