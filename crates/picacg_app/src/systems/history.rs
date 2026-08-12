//! 阅读历史系统
//!
//! 实现阅读历史页面，展示用户的漫画阅读记录

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        ui_common::format_timestamp,
        widgets::ButtonStyle,
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 历史页面根节点
#[derive(Component, Default, Clone)]
pub struct HistoryRoot;

/// 历史记录滚动容器
#[derive(Component, Default, Clone)]
pub struct HistoryScrollContainer;

/// 历史记录卡片
#[derive(Component, Default, Clone)]
pub struct HistoryItemCard {
    pub comic_id: String,
}

/// 历史记录删除按钮
#[derive(Component, Default, Clone)]
pub struct HistoryDeleteButton {
    pub comic_id: String,
}

/// 清空所有历史按钮
#[derive(Component, Default, Clone)]
pub struct ClearAllHistoryButton;

/// 历史记录封面缩略图（占位符与实际图片共用，`url` 供替换系统直接取用）
#[derive(Component, Default, Clone)]
pub struct HistoryThumbnail {
    /// 图片 URL
    pub url: String,
}

/// 历史空状态提示
#[derive(Component, Default, Clone)]
pub struct HistoryEmptyHint;

// ==================== 布局常量 ====================

mod history_layout {
    /// 卡片高度
    pub const CARD_HEIGHT: f32 = 90.0;
    /// 卡片间距
    pub const CARD_GAP: f32 = 8.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 15.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
    /// 封面宽度
    pub const THUMB_WIDTH: f32 = 60.0;
    /// 封面高度
    pub const THUMB_HEIGHT: f32 = 75.0;
}

// ==================== 系统函数 ====================

/// 创建历史记录界面（如果已存在则只显示）
pub fn setup_history_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    history_state: Res<HistoryState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_history_messages: MessageWriter<LoadHistoryRequest>,
    mut existing_query: Query<&mut Node, With<HistoryRoot>>,
) {
    // 如果 HistoryRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        // 仍然触发加载（刷新数据）
        if history_state.records.is_empty() && !history_state.is_loading {
            load_history_messages.write(LoadHistoryRequest);
        }
        return;
    }

    let content_area = content_area_query.single().ok();

    let history_root = commands.spawn_scene(history_page(&history_state)).id();

    // 如果有 ContentArea，将历史列表作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(history_root);
    }

    // 发送加载请求
    if history_state.records.is_empty() && !history_state.is_loading {
        load_history_messages.write(LoadHistoryRequest);
    }

    tracing::info!("阅读历史页面 UI 已创建");
}

/// 历史页面场景
fn history_page(history_state: &HistoryState) -> impl Scene + use<> {
    let list_padding = UiRect {
        left: Val::Px(history_layout::PADDING_LEFT),
        right: Val::Px(history_layout::PADDING_RIGHT),
        top: Val::Px(history_layout::PADDING_TOP),
        bottom: Val::Px(history_layout::PADDING_BOTTOM),
    };

    // 列表初始内容：加载中 / 空状态提示（两者都不满足时为空列表）
    let list_hint: Box<dyn SceneList> = if history_state.is_loading {
        Box::new(bsn_list![loading_indicator()])
    } else if history_state.records.is_empty() && history_state.error.is_none() {
        Box::new(bsn_list![empty_hint()])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        HistoryRoot
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
                    justify_content: JustifyContent::SpaceBetween,
                    border: UiRect::bottom(Val::Px(1.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // 左侧标题
                        Text("阅读历史")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 右侧清空按钮
                        ClearAllHistoryButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(12.0),
                                Val::Px(6.0),
                                Val::Px(6.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                        }
                        template_value(BorderColor::all(AppColors::BORDER))
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_DELETE)
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::ERROR)
                            ),
                            (
                                Text("清空")
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::ERROR)
                            ),
                        ]
                    ),
                ]
            ),
            (
                // 滚动区域包装器
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                }
                Children [
                    (
                        // 历史列表（可滚动）
                        #HistoryScroll
                        HistoryScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: {list_padding},
                            row_gap: Val::Px(history_layout::CARD_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [ {list_hint} ]
                    ),
                    // 创建滚动条
                    scrollbar(#HistoryScroll),
                ]
            ),
        ]
    }
}

/// 加载指示器场景
fn loading_indicator() -> impl Scene {
    bsn! {
        LoadingIndicator
        Text("加载中...")
        TextFont { font_size: FontSize::Px(16.0) }
        TextColor(AppColors::TEXT)
    }
}

/// 空状态提示场景
fn empty_hint() -> impl Scene {
    bsn! {
        HistoryEmptyHint
        Text("暂无阅读记录")
        TextFont { font_size: FontSize::Px(16.0) }
        TextColor(AppColors::TEXT_SECONDARY)
    }
}

/// 加载失败提示场景
fn error_message(error: &str) -> impl Scene + use<> {
    let label = format!("加载失败: {}", error);

    bsn! {
        ErrorMessage
        Text({label})
        TextFont { font_size: FontSize::Px(14.0) }
        TextColor(AppColors::ERROR)
    }
}

/// 清理历史页面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_history_ui(mut query: Query<&mut Node, With<HistoryRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新历史列表 UI（响应数据变化）
pub fn refresh_history_ui(
    mut commands: Commands,
    history_state: Res<HistoryState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<HistoryScrollContainer>>,
    card_query: Query<&HistoryItemCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    empty_hint_query: Query<Entity, With<HistoryEmptyHint>>,
    image_cache: Res<ImageCache>,
) {
    if !history_state.is_changed() {
        return;
    }

    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 如果有错误，显示错误信息
    if let Some(ref error) = history_state.error {
        // 删除加载指示器
        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in empty_hint_query.iter() {
            commands.entity(entity).despawn();
        }

        commands
            .spawn_scene(error_message(error))
            .insert(ChildOf(scroll_entity));
        return;
    }

    // 检查是否已有卡片
    let has_cards = children
        .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
        .unwrap_or(false);

    // 如果数据存在或已有卡片，不重建
    if has_cards || history_state.records.is_empty() {
        // 如果记录为空且没有空状态提示，添加空状态提示
        if history_state.records.is_empty()
            && !history_state.is_loading
            && !has_cards
            && empty_hint_query.is_empty()
            && loading_query.is_empty()
        {
            commands
                .spawn_scene(empty_hint())
                .insert(ChildOf(scroll_entity));
        }
        return;
    }

    // 删除加载指示器和空状态提示
    for entity in loading_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in empty_hint_query.iter() {
        commands.entity(entity).despawn();
    }

    // 创建所有历史卡片
    for record in history_state.records.iter() {
        commands
            .spawn_scene(history_card(record, &image_cache))
            .insert(ChildOf(scroll_entity));
    }
}

/// 单个历史记录卡片场景
fn history_card(record: &picacg_db::DbHistory, image_cache: &ImageCache) -> impl Scene + use<> {
    let comic_title = record.comic_title.as_deref().unwrap_or("未知漫画");
    let eps_fallback = format!("第{}章", record.last_eps);
    let eps_title = record.last_eps_title.as_deref().unwrap_or(&eps_fallback);
    let time_str = format_timestamp(record.last_read);

    let card = HistoryItemCard {
        comic_id: record.book_id.clone(),
    };
    let menu_target = ContextMenuTarget {
        comic_id: record.book_id.clone(),
        comic_title: comic_title.to_string(),
    };
    let delete_button = HistoryDeleteButton {
        comic_id: record.book_id.clone(),
    };
    let title = comic_title.to_string();
    let progress = format!("上次看到：{} 第{}页", eps_title, record.last_page);

    // 封面缩略图
    let thumb_url = record.thumb_url.as_deref().unwrap_or("");
    let thumbnail: Box<dyn SceneList> = match (thumb_url.is_empty(), image_cache.get(thumb_url)) {
        (false, Some(handle)) => Box::new(bsn_list![history_thumbnail(thumb_url, handle.clone())]),
        // 占位符
        (false, None) => {
            // 占位符自带 URL：图片就绪时无需回查 HistoryState
            let placeholder_url = thumb_url.to_string();
            Box::new(bsn_list![(
                PlaceholderImage
                HistoryThumbnail { url: {placeholder_url} }
                Node {
                    width: Val::Px(history_layout::THUMB_WIDTH),
                    height: Val::Px(history_layout::THUMB_HEIGHT),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(AppColors::SURFACE_HOVER)
            )])
        }
        // 无封面占位符
        (true, _) => Box::new(bsn_list![(
            Node {
                width: Val::Px(history_layout::THUMB_WIDTH),
                height: Val::Px(history_layout::THUMB_HEIGHT),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            BackgroundColor(AppColors::SURFACE_HOVER)
            Children [
                (
                    Text(ICON_BOOK)
                    TextFont { font_size: FontSize::Px(20.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                )
            ]
        )]),
    };

    bsn! {
        template_value(card)
        template_value(menu_target)
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(history_layout::CARD_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            column_gap: Val::Px(12.0),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        BackgroundColor(AppColors::SURFACE)
        Children [
            {thumbnail},
            (
                // 中间信息区域
                Node {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(4.0),
                    overflow: Overflow::clip(),
                }
                Children [
                    (
                        // 漫画标题
                        Text({title})
                        TextFont { font_size: FontSize::Px(15.0) }
                        TextColor(AppColors::TEXT)
                        Node {
                            max_width: Val::Percent(100.0),
                            overflow: Overflow::clip(),
                        }
                    ),
                    (
                        // 上次阅读进度
                        Text({progress})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        // 时间
                        Text({time_str})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(Color::srgb(0.4, 0.4, 0.45))
                    ),
                ]
            ),
            (
                // 右侧删除按钮
                template_value(delete_button)
                Button
                template_value(ButtonStyle::ghost())
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(Color::NONE)
                Children [
                    (
                        Text(ICON_CLOSE)
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(Color::srgb(0.5, 0.5, 0.55))
                    )
                ]
            ),
        ]
    }
}

/// 封面缩略图场景（卡片创建与图片加载完成后的替换共用）
fn history_thumbnail(url: &str, image: Handle<Image>) -> impl Scene + use<> {
    let thumbnail = HistoryThumbnail {
        url: url.to_string(),
    };

    bsn! {
        template_value(thumbnail)
        // visual_box 必须显式写：补丁基于 ImageNodeTemplate 的字段级默认值
        //（PaddingBox），而非 ImageNode::default() 的 ContentBox
        ImageNode {
            image: {image},
            visual_box: VisualBox::ContentBox,
        }
        Node {
            width: Val::Px(history_layout::THUMB_WIDTH),
            height: Val::Px(history_layout::THUMB_HEIGHT),
            flex_shrink: 0.0,
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
    }
}

/// 历史卡片点击交互（跳转到漫画详情；配色由 `apply_button_interaction`
/// 统一接管）
pub fn history_card_interaction(
    interaction_query: Query<(&Interaction, &HistoryItemCard), Changed<Interaction>>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
        }
    }
}

/// 历史记录删除按钮交互（配色由 `apply_button_interaction` 统一接管）
pub fn history_delete_interaction(
    interaction_query: Query<(&Interaction, &HistoryDeleteButton), Changed<Interaction>>,
    mut delete_messages: MessageWriter<DeleteHistoryRequest>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            delete_messages.write(DeleteHistoryRequest {
                comic_id: btn.comic_id.clone(),
            });
            tracing::info!("删除历史记录: {}", btn.comic_id);
        }
    }
}

/// 清空所有历史按钮交互
pub fn clear_all_history_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClearAllHistoryButton>)>,
    mut clear_messages: MessageWriter<ClearAllHistoryRequest>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            clear_messages.write(ClearAllHistoryRequest);
            tracing::info!("清空所有阅读历史");
        }
    }
}

/// 处理历史数据加载完成
pub fn handle_history_loaded(
    mut history_state: ResMut<HistoryState>,
    mut messages: MessageReader<HistoryLoadedEvent>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    image_cache: Res<ImageCache>,
) {
    for event in messages.read() {
        history_state.records = event.records.clone();
        history_state.total_count = event.total_count;
        history_state.is_loading = false;
        history_state.error = None;

        // 预加载封面图片（已入队/加载中/失败的 URL 不重复请求）
        for record in &history_state.records {
            if let Some(ref url) = record.thumb_url
                && !url.is_empty()
                && !image_cache.is_known(url)
            {
                load_image_messages.write(LoadImageRequest { url: url.clone() });
            }
        }

        tracing::info!("阅读历史加载完成: {} 条记录", history_state.records.len());
    }
}

/// 处理历史数据加载失败
pub fn handle_history_load_failed(
    mut history_state: ResMut<HistoryState>,
    mut messages: MessageReader<HistoryLoadFailedEvent>,
) {
    for event in messages.read() {
        history_state.is_loading = false;
        history_state.error = Some(event.error.clone());
        tracing::warn!("阅读历史加载失败: {}", event.error);
    }
}

/// 更新历史封面图片（当图片加载完成时替换占位符）
///
/// 扫描集只含"仍是占位符"的实体：已替换的带 `ImageNode`，加载失败的会被摘掉
/// `PlaceholderImage` 标记，两者都不再进入每帧遍历。
pub fn update_history_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    placeholder_query: Query<
        (Entity, &ChildOf, &HistoryThumbnail),
        (With<PlaceholderImage>, Without<ImageNode>),
    >,
) {
    let mut replaced_count = 0;
    for (placeholder_entity, child_of, thumbnail) in placeholder_query.iter() {
        // 加载失败：摘掉占位标记（灰底保留），让它退出扫描集
        if image_cache.is_failed(&thumbnail.url) {
            commands
                .entity(placeholder_entity)
                .remove::<PlaceholderImage>();
            continue;
        }

        // 检查图片是否已加载
        let Some(handle) = image_cache.get(&thumbnail.url) else {
            continue;
        };

        let parent_entity: Entity = child_of.parent();
        commands.entity(placeholder_entity).despawn();
        let image_entity = commands
            .spawn_scene(history_thumbnail(&thumbnail.url, handle.clone()))
            .id();

        // 插入到第一个位置（在信息区域之前）
        commands
            .entity(parent_entity)
            .insert_children(0, &[image_entity]);
        replaced_count += 1;
    }

    if replaced_count > 0 {
        tracing::trace!("[History] 替换了 {} 个封面图片", replaced_count);
    }
}
