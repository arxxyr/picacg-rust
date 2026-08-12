//! 首页系统
//!
//! 实现首页推荐漫画展示

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        ui_common::{TagColor, comic_time_info, tag_badge},
        widgets::ButtonStyle,
    },
};

/// 首页卡片布局常量
mod home_layout {
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
#[derive(Component, Default, Clone)]
pub struct HomeRoot;

/// 首页滚动容器标记
#[derive(Component, Default, Clone)]
pub struct HomeScrollContainer;

/// 首页漫画卡片标记
#[derive(Component, Default, Clone)]
pub struct HomeComicCard {
    pub comic_id: String,
}

/// 首页卡片缩略图标记（占位符与实际图片共用，`url` 供替换系统直接取用）
#[derive(Component, Default, Clone)]
pub struct HomeThumbnail {
    /// 图片 URL
    pub url: String,
}

/// 刷新按钮标记
#[derive(Component, Default, Clone)]
pub struct HomeRefreshButton;

/// 首页加载指示器
#[derive(Component, Default, Clone)]
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
}

impl HomeCardCreationState {
    /// 开始预创建模式
    pub fn start_precreate(&mut self, total: usize) {
        self.is_creating = true;
        self.total_cards = total;
        self.visible_count = 0;
        self.cards_per_frame = 3;
    }

    /// 清空状态
    pub fn clear(&mut self) {
        self.is_creating = false;
        self.total_cards = 0;
        self.visible_count = 0;
    }
}

/// 创建首页界面（如果已存在则只显示）
pub fn setup_home_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    home_state: Res<HomeState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<HomeCardCreationState>,
    mut load_recommendations: MessageWriter<LoadRecommendationsRequest>,
    mut existing_query: Query<&mut Node, With<HomeRoot>>,
) {
    // 如果 HomeRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        // 仍然触发加载（刷新数据）
        if home_state.recommendations.is_empty() && !home_state.is_loading {
            load_recommendations.write(LoadRecommendationsRequest);
        }
        return;
    }

    // 清空之前的创建状态
    creation_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    let home_root = commands.spawn_scene(home_page(&home_state)).id();

    // 如果有 ContentArea，将首页作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(home_root);
    }

    // 如果推荐列表为空且没有在加载，发送加载请求
    if home_state.recommendations.is_empty() && !home_state.is_loading {
        load_recommendations.write(LoadRecommendationsRequest);
    } else if !home_state.recommendations.is_empty() && !home_state.is_loading {
        // 启动预创建模式
        creation_state.start_precreate(home_state.recommendations.len());
    }

    tracing::info!("首页 UI 已创建");
}

/// 首页页面场景
fn home_page(home_state: &HomeState) -> impl Scene + use<> {
    // 内容网格内边距（右侧额外让出滚动条宽度）
    let grid_padding = UiRect {
        left: Val::Px(home_layout::PADDING_LEFT),
        right: Val::Px(home_layout::PADDING_RIGHT),
        top: Val::Px(home_layout::PADDING_TOP),
        bottom: Val::Px(home_layout::PADDING_BOTTOM),
    };

    // 网格初始占位内容：加载中指示器（仅加载态）
    let loading_indicator: Box<dyn SceneList> = if home_state.is_loading {
        Box::new(bsn_list![(
            HomeLoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT)
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        HomeRoot
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
                        // 标题
                        Text("推荐漫画")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 刷新按钮
                        HomeRefreshButton
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(12.0),
                                Val::Px(6.0),
                                Val::Px(6.0),
                            ),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        Children [
                            (
                                Text("换一批")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            )
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
                        // 内容网格（可滚动）
                        #HomeScroll
                        HomeScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: {grid_padding},
                            column_gap: Val::Px(home_layout::COLUMN_GAP),
                            row_gap: Val::Px(home_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [ {loading_indicator} ]
                    ),
                    // 创建滚动条
                    scrollbar(#HomeScroll),
                ]
            ),
        ]
    }
}

/// 封面缩略图场景（图片已缓存时使用）
fn thumbnail_image(url: String, handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        HomeThumbnail { url: {url} }
        ImageNode { image: {handle} }
        Node {
            width: Val::Px(164.0),
            height: Val::Px(220.0),
        }
    }
}

/// 漫画卡片场景（`hidden` 用于瀑布式预创建，先隐藏后分批显示）
fn home_comic_card(
    comic: &picacg_api::models::Comic,
    image_cache: &ImageCache,
    hidden: bool,
) -> impl Scene + use<> {
    let card_comic_id = comic.id.clone();
    let menu_comic_id = comic.id.clone();
    let menu_comic_title = comic.title.clone();
    let title = comic.title.clone();
    let author = comic.author.clone();

    let visibility = if hidden {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    // 封面图片（缓存命中用图片，否则占位符）
    let thumb_url = comic.thumb.url();
    let cover: Box<dyn SceneList> = if let Some(handle) = image_cache.get(&thumb_url) {
        Box::new(bsn_list![thumbnail_image(
            thumb_url.clone(),
            handle.clone()
        )])
    } else {
        // 占位符自带 URL：图片就绪时无需回查 HomeState
        let placeholder_url = thumb_url.clone();
        Box::new(bsn_list![(
            PlaceholderImage
            HomeThumbnail { url: {placeholder_url} }
            Node {
                width: Val::Px(164.0),
                height: Val::Px(220.0),
            }
            BackgroundColor(AppColors::SURFACE_HOVER)
        )])
    };

    // 分类和标签容器（两者都为空时不创建）
    let tags_container: Box<dyn SceneList> =
        if !comic.categories.is_empty() || !comic.tags.is_empty() {
            // 分类（蓝色）+ 标签（绿色）
            let badges: Vec<_> = comic
                .categories
                .iter()
                .take(2)
                .map(|category| tag_badge(category, TagColor::Category))
                .chain(
                    comic
                        .tags
                        .iter()
                        .take(2)
                        .map(|tag| tag_badge(tag, TagColor::Tag)),
                )
                .collect();

            Box::new(bsn_list![(
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                }
                Children [ {badges} ]
            )])
        } else {
            Box::new(bsn_list![])
        };

    // 创建/更新时间
    let time_info = comic_time_info(comic.created_at.as_deref(), comic.updated_at.as_deref());

    bsn! {
        HomeComicCard { comic_id: {card_comic_id} }
        ContextMenuTarget { comic_id: {menu_comic_id}, comic_title: {menu_comic_title} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Px(180.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        BackgroundColor(AppColors::SURFACE)
        template_value(visibility)
        Children [
            // 封面图片
            {cover},
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                }
            ),
            (
                // 作者
                Text({author})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                }
            ),
            // 分类和标签容器
            {tags_container},
            // 创建/更新时间
            {time_info},
        ]
    }
}

/// 清理首页
pub fn cleanup_home_ui(
    mut query: Query<&mut Node, With<HomeRoot>>,
    mut creation_state: ResMut<HomeCardCreationState>,
) {
    creation_state.clear();

    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 首页卡片交互系统（配色由 `apply_button_interaction` 统一接管）
pub fn home_card_interaction(
    interaction_query: Query<(&Interaction, &HomeComicCard), Changed<Interaction>>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // 通过导航消息跳转到详情页（保留导航历史）
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
        }
    }
}

/// 刷新按钮交互（配色由 `apply_button_interaction` 统一接管）
pub fn home_refresh_button_interaction(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<HomeRefreshButton>)>,
    mut home_state: ResMut<HomeState>,
    mut load_recommendations: MessageWriter<LoadRecommendationsRequest>,
    mut creation_state: ResMut<HomeCardCreationState>,
    card_query: Query<Entity, With<HomeComicCard>>,
    mut scroll_query: Query<&mut ScrollPosition, With<HomeScrollContainer>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

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
    _asset_server: Res<AssetServer>,
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

            creation_state.start_precreate(home_state.recommendations.len());
        }
    }

    // 如果不在创建模式，退出
    if !creation_state.is_creating {
        return;
    }

    // 阶段1：预创建所有卡片（隐藏状态）
    let has_cards = children
        .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
        .unwrap_or(false);

    if !has_cards && creation_state.visible_count == 0 {
        // 一次性创建所有卡片（隐藏）
        for comic in home_state.recommendations.iter() {
            commands
                .spawn_scene(home_comic_card(comic, &image_cache, true))
                .insert(ChildOf(scroll_entity));
        }
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

/// 更新首页封面图片
///
/// 扫描集只含"仍是占位符"的实体：已替换的带 `ImageNode`，加载失败的会被摘掉
/// `PlaceholderImage` 标记，两者都不再进入每帧遍历。
pub fn update_home_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    placeholder_query: Query<
        (Entity, &ChildOf, &HomeThumbnail),
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

        let Some(handle) = image_cache.get(&thumbnail.url) else {
            continue;
        };

        let parent_entity: Entity = child_of.parent();
        commands.entity(placeholder_entity).despawn();
        let image_entity = commands
            .spawn_scene(thumbnail_image(thumbnail.url.clone(), handle.clone()))
            .id();

        commands
            .entity(parent_entity)
            .insert_children(0, &[image_entity]);
        replaced_count += 1;
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

// ==================== 签到 Toast 通知 ====================

/// 签到 Toast 通知标记
#[derive(Component, Default, Clone)]
pub struct PunchInToast;

/// 签到 Toast 计时器
#[derive(Resource)]
pub struct PunchInToastTimer(pub Timer);

/// 签到成功背景色
const PUNCH_IN_SUCCESS_COLOR: Color = Color::srgb(0.2, 0.6, 0.3);
/// 签到失败背景色
const PUNCH_IN_ERROR_COLOR: Color = Color::srgb(0.7, 0.2, 0.2);

/// 显示签到 Toast 通知
///
/// 监听 `PunchInState` 变化，在首页内容区域顶部创建 Toast 通知条。
pub fn display_punch_in_toast(
    mut commands: Commands,
    punch_in_state: Res<PunchInState>,
    toast_query: Query<Entity, With<PunchInToast>>,
    home_root_query: Query<Entity, With<HomeRoot>>,
) {
    if !punch_in_state.is_changed() {
        return;
    }

    let Some(ref message) = punch_in_state.message else {
        return;
    };

    // 如果已有 Toast，先移除旧的
    for entity in toast_query.iter() {
        commands.entity(entity).despawn();
    }
    // 同时移除旧的计时器
    commands.remove_resource::<PunchInToastTimer>();

    let Ok(home_root) = home_root_query.single() else {
        return;
    };

    let bg_color = if punch_in_state.is_success {
        PUNCH_IN_SUCCESS_COLOR
    } else {
        PUNCH_IN_ERROR_COLOR
    };

    // 创建 Toast 通知条
    let toast = commands
        .spawn_scene(punch_in_toast(message.as_str(), bg_color))
        .id();

    // 将 Toast 插入到 HomeRoot 的第一个子节点位置
    commands.entity(home_root).insert_children(0, &[toast]);

    // 插入 3 秒自动消失计时器
    commands.insert_resource(PunchInToastTimer(Timer::from_seconds(3.0, TimerMode::Once)));

    tracing::debug!("显示签到 Toast: {}", message);
}

/// 签到 Toast 通知条场景
fn punch_in_toast(message: &str, bg_color: Color) -> impl Scene + use<> {
    let message = message.to_string();

    bsn! {
        PunchInToast
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        BackgroundColor(bg_color)
        ZIndex(100)
        Children [
            (
                Text({message})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::WHITE)
            )
        ]
    }
}

/// 自动隐藏签到 Toast 通知
///
/// 计时器到期后移除 Toast 实体和计时器资源。
pub fn auto_hide_punch_in_toast(
    mut commands: Commands,
    time: Res<Time>,
    timer: Option<ResMut<PunchInToastTimer>>,
    toast_query: Query<Entity, With<PunchInToast>>,
) {
    let Some(mut timer) = timer else {
        return;
    };

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        for entity in toast_query.iter() {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<PunchInToastTimer>();
    }
}
