//! 点赞记录系统
//!
//! 实现点赞记录页面，展示用户点赞过的漫画

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

/// 点赞记录页面根节点
#[derive(Component, Default, Clone)]
pub struct LikeRecordsRoot;

/// 点赞记录滚动容器
#[derive(Component, Default, Clone)]
pub struct LikeRecordsScrollContainer;

/// 点赞记录卡片
#[derive(Component, Default, Clone)]
pub struct LikeRecordCard {
    pub comic_id: String,
}

/// 点赞记录删除按钮（取消点赞）
#[derive(Component, Default, Clone)]
pub struct LikeRecordDeleteButton {
    pub comic_id: String,
}

/// 点赞记录封面缩略图（占位与成图共用，`url` 是替换图片的唯一依据）
#[derive(Component, Default, Clone)]
pub struct LikeRecordThumbnail {
    pub url: String,
}

/// 点赞记录空状态提示
#[derive(Component, Default, Clone)]
pub struct LikeRecordsEmptyHint;

// ==================== 布局常量 ====================

mod like_records_layout {
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

/// 创建点赞记录界面（如果已存在则只显示）
pub fn setup_like_records_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    like_records_state: Res<LikeRecordsState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_messages: MessageWriter<LoadLikeRecordsRequest>,
    mut existing_query: Query<&mut Node, With<LikeRecordsRoot>>,
) {
    // 如果 LikeRecordsRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if like_records_state.records.is_empty() && !like_records_state.is_loading {
            load_messages.write(LoadLikeRecordsRequest);
        }
        return;
    }

    let content_area = content_area_query.single().ok();

    let like_records_root = commands
        .spawn_scene(like_records_page(&like_records_state))
        .id();

    // 如果有 ContentArea，将点赞记录作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(like_records_root);
    }

    // 发送加载请求
    if like_records_state.records.is_empty() && !like_records_state.is_loading {
        load_messages.write(LoadLikeRecordsRequest);
    }

    tracing::info!("点赞记录页面 UI 已创建");
}

/// 点赞记录页面场景
fn like_records_page(like_records_state: &LikeRecordsState) -> impl Scene + use<> {
    // 列表初始占位内容：加载指示器 / 空状态提示 / 两者皆无
    let list_placeholder: Box<dyn SceneList> = if like_records_state.is_loading {
        Box::new(bsn_list![(
            LoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT)
        )])
    } else if like_records_state.records.is_empty() && like_records_state.error.is_none() {
        Box::new(bsn_list![empty_hint()])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        LikeRecordsRoot
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
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                        }
                        Children [
                            (
                                Text(ICON_THUMB_UP)
                                TextFont { font_size: FontSize::Px(18.0) }
                                TextColor(AppColors::PRIMARY)
                            ),
                            (
                                Text("点赞记录")
                                TextFont { font_size: FontSize::Px(18.0) }
                                TextColor(AppColors::TEXT)
                            ),
                        ]
                    )
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
                        // 列表（可滚动）
                        #LikeRecordsScroll
                        LikeRecordsScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::new(
                                Val::Px(like_records_layout::PADDING_LEFT),
                                Val::Px(like_records_layout::PADDING_RIGHT),
                                Val::Px(like_records_layout::PADDING_TOP),
                                Val::Px(like_records_layout::PADDING_BOTTOM),
                            ),
                            row_gap: Val::Px(like_records_layout::CARD_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [ {list_placeholder} ]
                    ),
                    // 创建滚动条
                    scrollbar(#LikeRecordsScroll),
                ]
            ),
        ]
    }
}

/// 封面缩略图场景（图片已缓存时使用）
fn thumbnail_image(url: String, handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        LikeRecordThumbnail { url: {url} }
        ImageNode { image: {handle} }
        Node {
            width: Val::Px(like_records_layout::THUMB_WIDTH),
            height: Val::Px(like_records_layout::THUMB_HEIGHT),
            flex_shrink: 0.0,
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
    }
}

/// 空状态提示场景
fn empty_hint() -> impl Scene {
    bsn! {
        LikeRecordsEmptyHint
        Text("暂无点赞记录")
        TextFont { font_size: FontSize::Px(16.0) }
        TextColor(AppColors::TEXT_SECONDARY)
    }
}

/// 清理点赞记录页面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_like_records_ui(mut query: Query<&mut Node, With<LikeRecordsRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新点赞记录列表 UI（响应数据变化）
pub fn refresh_like_records_ui(
    mut commands: Commands,
    like_records_state: Res<LikeRecordsState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<LikeRecordsScrollContainer>>,
    card_query: Query<&LikeRecordCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    empty_hint_query: Query<Entity, With<LikeRecordsEmptyHint>>,
    image_cache: Res<ImageCache>,
) {
    if !like_records_state.is_changed() {
        return;
    }

    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 如果有错误，显示错误信息
    if let Some(ref error) = like_records_state.error {
        // 删除加载指示器
        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in empty_hint_query.iter() {
            commands.entity(entity).despawn();
        }

        let error_text = format!("加载失败: {}", error);
        commands
            .spawn_scene(bsn! {
                ErrorMessage
                Text({error_text})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::ERROR)
            })
            .insert(ChildOf(scroll_entity));
        return;
    }

    // 检查是否已有卡片
    let has_cards = children
        .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
        .unwrap_or(false);

    // 如果数据存在或已有卡片，不重建
    if has_cards || like_records_state.records.is_empty() {
        // 如果记录为空且没有空状态提示，添加空状态提示
        if like_records_state.records.is_empty()
            && !like_records_state.is_loading
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

    // 创建所有点赞记录卡片
    for record in like_records_state.records.iter() {
        commands
            .spawn_scene(like_record_card(record, &image_cache))
            .insert(ChildOf(scroll_entity));
    }
}

/// 单个点赞记录卡片场景
fn like_record_card(
    record: &picacg_db::DbLikeRecord,
    image_cache: &ImageCache,
) -> impl Scene + use<> {
    let time_str = format_timestamp(record.liked_at);
    let card_comic_id = record.comic_id.clone();
    let menu_comic_id = record.comic_id.clone();
    let menu_comic_title = record.comic_title.clone();
    let delete_comic_id = record.comic_id.clone();
    let comic_title = record.comic_title.clone();
    let liked_label = format!("点赞于 {}", time_str);

    // 封面缩略图
    let thumb_url = record.thumb_url.as_deref().unwrap_or("");
    let thumbnail: Box<dyn SceneList> = if !thumb_url.is_empty() {
        if let Some(handle) = image_cache.get(thumb_url) {
            Box::new(bsn_list![thumbnail_image(
                thumb_url.to_string(),
                handle.clone()
            )])
        } else {
            // 占位符：自带 URL，图片就绪后由 update_like_records_images 直接替换
            let placeholder_url = thumb_url.to_string();
            Box::new(bsn_list![(
                PlaceholderImage
                LikeRecordThumbnail { url: {placeholder_url} }
                Node {
                    width: Val::Px(like_records_layout::THUMB_WIDTH),
                    height: Val::Px(like_records_layout::THUMB_HEIGHT),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(AppColors::SURFACE_HOVER)
            )])
        }
    } else {
        // 无封面占位符
        Box::new(bsn_list![(
            Node {
                width: Val::Px(like_records_layout::THUMB_WIDTH),
                height: Val::Px(like_records_layout::THUMB_HEIGHT),
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
        )])
    };

    bsn! {
        LikeRecordCard { comic_id: {card_comic_id} }
        // 点赞记录来自本地库，没有服务端章节数，eps_count 留 0 = 未知
        ContextMenuTarget { comic_id: {menu_comic_id}, comic_title: {menu_comic_title} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(like_records_layout::CARD_HEIGHT),
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
                        Text({comic_title})
                        TextFont { font_size: FontSize::Px(15.0) }
                        TextColor(AppColors::TEXT)
                        Node {
                            max_width: Val::Percent(100.0),
                            overflow: Overflow::clip(),
                        }
                    ),
                    (
                        // 点赞时间
                        Text({liked_label})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            (
                // 右侧取消点赞按钮
                LikeRecordDeleteButton { comic_id: {delete_comic_id} }
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

/// 点赞记录卡片点击交互（跳转到漫画详情；配色由 `ButtonStyle` 统一接管）
pub fn like_record_card_interaction(
    interaction_query: Query<(&Interaction, &LikeRecordCard), Changed<Interaction>>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction == Interaction::Pressed {
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
        }
    }
}

/// 点赞记录删除按钮交互（取消点赞；配色由 `ButtonStyle` 统一接管）
pub fn like_record_delete_interaction(
    interaction_query: Query<(&Interaction, &LikeRecordDeleteButton), Changed<Interaction>>,
    mut delete_messages: MessageWriter<DeleteLikeRecordRequest>,
) {
    for (interaction, btn) in &interaction_query {
        if *interaction == Interaction::Pressed {
            delete_messages.write(DeleteLikeRecordRequest {
                comic_id: btn.comic_id.clone(),
            });
            tracing::info!("取消点赞: {}", btn.comic_id);
        }
    }
}

/// 处理点赞记录数据加载完成
pub fn handle_like_records_loaded(
    mut like_records_state: ResMut<LikeRecordsState>,
    mut messages: MessageReader<LikeRecordsLoadedEvent>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    image_cache: Res<ImageCache>,
) {
    for event in messages.read() {
        like_records_state.records = event.records.clone();
        like_records_state.total_count = event.total_count;
        like_records_state.is_loading = false;
        like_records_state.error = None;

        // 预加载封面图片（已有状态的 URL 不再重复请求，含加载中与失败）
        for record in &like_records_state.records {
            if let Some(ref url) = record.thumb_url
                && !url.is_empty()
                && !image_cache.is_known(url)
            {
                load_image_messages.write(LoadImageRequest { url: url.clone() });
            }
        }

        tracing::info!(
            "点赞记录加载完成: {} 条记录",
            like_records_state.records.len()
        );
    }
}

/// 处理点赞记录数据加载失败
pub fn handle_like_records_load_failed(
    mut like_records_state: ResMut<LikeRecordsState>,
    mut messages: MessageReader<LikeRecordsLoadFailedEvent>,
) {
    for event in messages.read() {
        like_records_state.is_loading = false;
        like_records_state.error = Some(event.error.clone());
        tracing::warn!("点赞记录加载失败: {}", event.error);
    }
}

/// 更新点赞记录封面图片（当图片加载完成时替换占位符）
///
/// 占位实体自带 URL，无需反查 `LikeRecordsState`；失败的图片摘掉占位标记
/// 退出扫描集（此前失败图片永久留在集合里被每帧重扫）。
pub fn update_like_records_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    placeholder_query: Query<
        (Entity, &LikeRecordThumbnail, &ChildOf),
        (With<PlaceholderImage>, Without<ImageNode>),
    >,
) {
    let mut replaced_count = 0;
    for (placeholder_entity, thumb, child_of) in placeholder_query.iter() {
        if image_cache.is_failed(&thumb.url) {
            commands
                .entity(placeholder_entity)
                .remove::<PlaceholderImage>();
            continue;
        }

        let Some(handle) = image_cache.get(&thumb.url) else {
            continue;
        };

        commands.entity(placeholder_entity).despawn();
        let image_entity = commands
            .spawn_scene(thumbnail_image(thumb.url.clone(), handle.clone()))
            .id();

        // 插入到第一个位置（在信息区域之前）
        commands
            .entity(child_of.parent())
            .insert_children(0, &[image_entity]);
        replaced_count += 1;
    }

    if replaced_count > 0 {
        tracing::trace!("[LikeRecords] 替换了 {} 个封面图片", replaced_count);
    }
}
