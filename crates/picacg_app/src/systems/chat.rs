//! 聊天大厅系统
//!
//! 展示聊天房间列表，点击进入聊天室

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        navigation::NavigationHistory,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        widgets::ButtonStyle,
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 聊天大厅根节点
#[derive(Component, Default, Clone)]
pub struct ChatRoot;

/// 聊天大厅滚动容器
#[derive(Component, Default, Clone)]
pub struct ChatScrollContainer;

/// 聊天房间卡片
#[derive(Component, Default, Clone)]
pub struct ChatRoomCard {
    pub room_id: String,
    pub room_title: String,
}

/// 聊天房间图标
#[derive(Component, Default, Clone)]
pub struct ChatRoomIcon {
    pub url: String,
}

/// 刷新按钮
#[derive(Component, Default, Clone)]
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

    let content_area = content_area_query.single().ok();

    let chat_root = commands.spawn_scene(chat_page(&chat_state)).id();

    // 挂载到内容区域
    if let Some(content) = content_area {
        commands.entity(content).add_children(&[chat_root]);
    }

    // 如果没有房间数据，自动加载
    if chat_state.rooms.is_empty() && !chat_state.is_loading {
        load_rooms_messages.write(LoadChatRoomsRequest);
    }
}

/// 聊天大厅页面场景
fn chat_page(chat_state: &ChatState) -> impl Scene + use<> {
    let content = chat_content(chat_state);

    bsn! {
        ChatRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 标题栏
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // 图标
                        Text(ICON_CHAT)
                        TextFont { font_size: FontSize::Px(20.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                    (
                        // 标题
                        Text("聊天室")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 弹性空间
                        Node { flex_grow: 1.0 }
                    ),
                    (
                        // 刷新按钮
                        ChatRefreshButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.0),
                        }
                        template_value(BorderColor::all(AppColors::BORDER))
                        // 静息底色与 ButtonStyle::ghost() 的 None 态一致
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_REFRESH)
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            ),
                            (
                                Text("刷新")
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::TEXT)
                            ),
                        ]
                    ),
                ]
            ),
            (
                // 内容区域（可滚动）
                Node {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                }
                Children [
                    (
                        #ChatScroll
                        ChatScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            overflow: Overflow::scroll_y(),
                            flex_direction: FlexDirection::Column,
                            padding: {UiRect {
                                left: Val::Px(chat_layout::PADDING_LEFT),
                                right: Val::Px(chat_layout::PADDING_RIGHT),
                                top: Val::Px(chat_layout::PADDING_TOP),
                                bottom: Val::Px(chat_layout::PADDING_BOTTOM),
                            }},
                            row_gap: Val::Px(chat_layout::CARD_GAP),
                        }
                        ScrollArea
                        Children [ {content} ]
                    ),
                    // 滚动条
                    scrollbar(#ChatScroll),
                ]
            ),
        ]
    }
}

/// 滚动容器内容（加载中 / 错误 / 空状态 / 房间列表）
fn chat_content(chat_state: &ChatState) -> Vec<Box<dyn Scene>> {
    // 加载中提示
    if chat_state.is_loading {
        vec![Box::new(loading_indicator()) as Box<dyn Scene>]
    } else if let Some(ref error) = chat_state.error {
        vec![Box::new(error_message(error)) as Box<dyn Scene>]
    } else if chat_state.rooms.is_empty() {
        vec![Box::new(empty_hint("点击刷新加载聊天房间列表")) as Box<dyn Scene>]
    } else {
        // 渲染房间列表
        chat_state
            .rooms
            .iter()
            .map(|room| Box::new(room_card(room)) as Box<dyn Scene>)
            .collect()
    }
}

/// 清理聊天大厅界面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_chat_ui(mut query: Query<&mut Node, With<ChatRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 房间卡片场景
fn room_card(room: &picacg_api::endpoints::chat::ChatRoom) -> impl Scene + use<> {
    let is_available = room.is_available;
    let text_color = if is_available {
        AppColors::TEXT
    } else {
        AppColors::TEXT_SECONDARY
    };
    let room_id = room.id.clone();
    let room_title = room.title.clone();
    let title = room.title.clone();
    let level_label = format!("LV.{} 以上", room.min_level);

    // 房间图标：有 URL 时标记为待加载（由 update_chat_room_icons 填充图片），
    // 否则显示默认图标
    let icon_content: Box<dyn SceneList> = if room.icon.is_empty() {
        Box::new(bsn_list![(
            Text(ICON_CHAT)
            TextFont { font_size: FontSize::Px(24.0) }
            TextColor(AppColors::PRIMARY)
        )])
    } else {
        let url = room.icon.clone();
        Box::new(bsn_list![(
            ChatRoomIcon { url: {url} }
            Node {
                width: Val::Px(60.0),
                height: Val::Px(60.0),
                border_radius: BorderRadius::all(Val::Px(8.0)),
            }
        )])
    };

    // 房间描述
    let description: Box<dyn SceneList> = if room.description.is_empty() {
        Box::new(bsn_list![])
    } else {
        let desc = room.description.clone();
        Box::new(bsn_list![(
            Text({desc})
            TextFont { font_size: FontSize::Px(13.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    };

    // 注册天数要求
    let register_days: Box<dyn SceneList> = if room.min_register_days > 0 {
        let days_label = format!("注册 {} 天以上", room.min_register_days);
        Box::new(bsn_list![(
            Text({days_label})
            TextFont { font_size: FontSize::Px(11.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 可用状态
    let unavailable: Box<dyn SceneList> = if is_available {
        Box::new(bsn_list![])
    } else {
        Box::new(bsn_list![(
            Text("(不可用)")
            TextFont { font_size: FontSize::Px(11.0) }
            TextColor(AppColors::ERROR)
        )])
    };

    // 右箭头
    let chevron: Box<dyn SceneList> = if is_available {
        Box::new(bsn_list![(
            Text(ICON_CHEVRON_RIGHT)
            TextFont { font_size: FontSize::Px(18.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        ChatRoomCard { room_id: {room_id}, room_title: {room_title} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        // 静息底色与 ButtonStyle::card() 的 None 态一致
        BackgroundColor(AppColors::SURFACE)
        Children [
            (
                // 房间图标占位
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                }
                BackgroundColor(Color::srgb(0.15, 0.15, 0.22))
                Children [ {icon_content} ]
            ),
            (
                // 房间信息
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(6.0),
                }
                Children [
                    (
                        // 房间标题
                        Text({title})
                        TextFont { font_size: FontSize::Px(16.0) }
                        TextColor(text_color)
                    ),
                    {description},
                    (
                        // 准入条件
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                        }
                        Children [
                            (
                                // 等级要求
                                Text({level_label})
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                            {register_days},
                            {unavailable},
                        ]
                    ),
                ]
            ),
            {chevron},
        ]
    }
}

/// 加载中提示场景
fn loading_indicator() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        Children [
            (
                Text("加载中...")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            )
        ]
    }
}

/// 错误提示场景
fn error_message(error: &str) -> impl Scene + use<> {
    let error = error.to_string();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
        }
        Children [
            (
                Text("加载失败")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::ERROR)
            ),
            (
                Text({error})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// 空状态提示场景
fn empty_hint(hint: &str) -> impl Scene + use<> {
    let hint = hint.to_string();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        Children [
            (
                Text({hint})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            )
        ]
    }
}

// ==================== 交互系统 ====================

/// 房间卡片交互：点击进入聊天室
pub fn chat_room_card_interaction(
    interaction_query: Query<(&Interaction, &ChatRoomCard), Changed<Interaction>>,
    chat_state: Res<ChatState>,
    mut chat_room_state: ResMut<ChatRoomState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
    current_route: Res<State<AppRoute>>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 未登录聊天服务时不进房
        if chat_state.chat_token.is_none() {
            tracing::warn!("聊天服务未登录，无法进入聊天室");
            continue;
        }

        // 设置聊天室状态
        chat_room_state.room_id = card.room_id.clone();
        chat_room_state.room_title = card.room_title.clone();
        chat_room_state.messages.clear();
        chat_room_state.online_count = 0;
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
    }
}

/// 刷新按钮交互
pub fn chat_refresh_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ChatRefreshButton>)>,
    mut load_rooms_messages: MessageWriter<LoadChatRoomsRequest>,
    chat_state: Res<ChatState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed && !chat_state.is_loading {
            load_rooms_messages.write(LoadChatRoomsRequest);
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
    for scene in chat_content(&chat_state) {
        commands.spawn_scene(scene).insert(ChildOf(scroll_entity));
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
        match image_cache.get(&icon.url) {
            Some(handle) => {
                commands.entity(entity).insert(ImageNode {
                    image: handle.clone(),
                    ..default()
                });
            }
            // 加载失败：摘掉标记退出每帧扫描集，避免永久残留
            None if image_cache.is_failed(&icon.url) => {
                commands.entity(entity).remove::<ChatRoomIcon>();
            }
            // 仅对从未请求过的 URL 发起加载；失败的不再无限重试
            None if !image_cache.is_known(&icon.url) => {
                image_messages.write(LoadImageRequest {
                    url: icon.url.clone(),
                });
            }
            None => {}
        }
    }
}
