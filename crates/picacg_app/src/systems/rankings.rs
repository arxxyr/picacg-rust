//! 排行榜页面系统

use bevy::prelude::*;
use picacg_api::endpoints::{RankTimeType, rank::KnightUser};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        ui_common::{
            BadgeAnchor, TagColor, comic_time_info, download_status_badge, format_number,
            tag_badge_truncated, truncate_text,
        },
        waterfall::{RankingsCardCreationState, RankingsContext},
        widgets::ButtonStyle,
    },
    utils::{content_filter::CompiledFilter, icons::*},
};

// ==================== 组件 ====================

/// 排行榜页面根标记
#[derive(Component, Default, Clone)]
pub struct RankingsRoot;

/// 排行榜滚动容器标记
#[derive(Component, Default, Clone)]
pub struct RankingsScrollContainer;

/// 排行榜内容容器（预留）
#[derive(Component, Default, Clone)]
#[allow(dead_code)]
pub struct RankingsContentContainer;

/// Tab 按钮标记
#[derive(Component, Clone)]
pub struct RankingsTabButton {
    pub tab_type: RankingsTabType,
}

/// `Default` 仅用于满足 BSN 模板的 `Default + Clone` 约束（`RankingsTabType`
/// 本身没有 `Default`），实际值总是由场景函数显式指定。
impl Default for RankingsTabButton {
    fn default() -> Self {
        Self {
            tab_type: RankingsTabType::Comics(RankTimeType::H24),
        }
    }
}

/// 骑士榜卡片标记
#[derive(Component, Default, Clone)]
pub struct KnightRankCard {
    #[allow(dead_code)]
    pub user_id: String,
}

/// 骑士榜列表容器标记
#[derive(Component, Default, Clone)]
pub struct KnightListContainer;

/// 排行榜卡片标记
#[derive(Component, Default, Clone)]
pub struct RankingsComicCard {
    pub comic_id: String,
    /// 排名（用于显示）
    #[allow(dead_code)]
    pub rank: usize,
}

/// 排行榜封面图片占位标记
///
/// 直接存封面 URL（建卡时本就算过一次），图片替换系统据此取缓存，
/// 无需反查当前排行榜列表。
#[derive(Component, Default, Clone)]
pub struct RankingsComicImage {
    pub url: String,
}

/// 排名标签标记
#[derive(Component, Default, Clone)]
pub struct RankBadge;

/// 加载中指示器标记
#[derive(Component, Default, Clone)]
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

/// 创建排行榜 UI（如果已存在则只显示）
pub fn setup_rankings_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
    rankings_state: Res<RankingsState>,
    mut existing_query: Query<&mut Node, With<RankingsRoot>>,
) {
    // 如果 RankingsRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        return;
    }

    let content_area = match content_area_query.iter().next() {
        Some(entity) => entity,
        None => {
            tracing::warn!("排行榜页：找不到内容区域");
            return;
        }
    };

    let rankings_root = commands.spawn_scene(rankings_page(&rankings_state)).id();
    commands.entity(content_area).add_child(rankings_root);

    tracing::info!("排行榜 UI 已创建");
}

/// 排行榜页面场景
fn rankings_page(state: &RankingsState) -> impl Scene + use<> {
    // 滚动区内边距（右侧额外让出滚动条宽度）
    let scroll_padding = UiRect {
        left: Val::Px(layout::PADDING_LEFT),
        right: Val::Px(layout::PADDING_RIGHT),
        top: Val::Px(layout::PADDING_TOP),
        bottom: Val::Px(layout::PADDING_BOTTOM),
    };

    // 加载中时显示指示器，否则等待瀑布式系统建卡
    let loading_placeholder: Box<dyn SceneList> = if state.is_loading {
        Box::new(bsn_list![loading_indicator()])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        RankingsRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            // Tab 栏
            tab_bar(state),
            (
                // 滚动区域包装器（与收藏/分类一致的结构）
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
                        // 滚动容器（直接使用 Wrap，不嵌套 ContentContainer）
                        #RankingsScroll
                        RankingsScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: {scroll_padding},
                            column_gap: Val::Px(layout::COLUMN_GAP),
                            row_gap: Val::Px(layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        ScrollPosition
                        Children [ {loading_placeholder} ]
                    ),
                    // 滚动条
                    scrollbar(#RankingsScroll),
                ]
            ),
        ]
    }
}

/// Tab 栏场景
fn tab_bar(state: &RankingsState) -> impl Scene + use<> {
    let title = format!("{ICON_TROPHY} 排行榜");

    let mut tabs = Vec::with_capacity(4);

    // 漫画排行 Tab 按钮
    for time_type in [RankTimeType::H24, RankTimeType::D7, RankTimeType::D30] {
        let tab_type = RankingsTabType::Comics(time_type);
        let is_active = state.current_tab == tab_type;
        tabs.push(tab_button(tab_type, is_active));
    }

    // 骑士榜 Tab 按钮
    let is_active = state.current_tab.is_knight();
    tabs.push(tab_button(RankingsTabType::Knight, is_active));

    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            padding: UiRect::horizontal(Val::Px(layout::PADDING_LEFT)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            border: UiRect::bottom(Val::Px(1.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        BackgroundColor(AppColors::CARD_BG)
        Children [
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(18.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::right(Val::Px(20.0)) }
            ),
            {tabs},
        ]
    }
}

/// Tab 按钮场景
///
/// 单选组：选中态由 `ButtonStyle.selected` 钉在 primary，未选中走 Segment
/// 三态（下沉表面 + 悬停浮起）。
fn tab_button(tab_type: RankingsTabType, is_active: bool) -> impl Scene {
    let style = ButtonStyle::segment(is_active);
    // 静息底色与 style 解析结果一致，避免首帧闪烁
    let bg_color = if is_active {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let label = tab_type.display_name().to_string();

    bsn! {
        RankingsTabButton { tab_type: {tab_type} }
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(bg_color)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 清理排行榜 UI（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_rankings_ui(
    mut query: Query<&mut Node, With<RankingsRoot>>,
    mut creation_state: ResMut<RankingsCardCreationState>,
) {
    // 清空瀑布式创建状态（防止对已隐藏的 Entity 操作）
    creation_state.clear();

    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

// ==================== 交互系统 ====================

/// Tab 按钮交互（配色由全局 `apply_button_interaction` 统一处理，
/// 选中态由 `refresh_rankings_ui` 写回 `ButtonStyle.selected`）
pub fn rankings_tab_interaction(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &RankingsTabButton), Changed<Interaction>>,
    mut rankings_state: ResMut<RankingsState>,
    mut load_messages: MessageWriter<LoadRankingsRequest>,
    mut load_knight_messages: MessageWriter<LoadKnightRankingsRequest>,
    mut creation_state: ResMut<RankingsCardCreationState>,
    comic_card_query: Query<Entity, With<RankingsComicCard>>,
    knight_card_query: Query<Entity, With<KnightRankCard>>,
    knight_container_query: Query<Entity, With<KnightListContainer>>,
    mut scroll_query: Query<&mut ScrollPosition, With<RankingsScrollContainer>>,
) {
    for (interaction, tab) in interaction_query.iter() {
        // 只处理点击；点当前已激活的页签是空操作
        if *interaction != Interaction::Pressed || rankings_state.current_tab == tab.tab_type {
            continue;
        }

        let start = std::time::Instant::now();

        // 立即清除旧漫画卡片
        for entity in comic_card_query.iter() {
            commands.entity(entity).despawn();
        }

        // 清除骑士榜卡片和容器
        for entity in knight_card_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in knight_container_query.iter() {
            commands.entity(entity).despawn();
        }

        // 清除瀑布流状态
        creation_state.clear();

        // 重置滚动位置
        for mut scroll_pos in scroll_query.iter_mut() {
            scroll_pos.y = 0.0;
        }

        // 切换当前标签类型
        rankings_state.current_tab = tab.tab_type;

        match tab.tab_type {
            RankingsTabType::Comics(time_type) => {
                // 更新漫画排行类型
                rankings_state.current_type = time_type;

                // 如果该类型还没有加载数据，发送加载请求
                if !rankings_state.is_loaded(time_type) {
                    rankings_state.is_loading = true;
                    load_messages.write(LoadRankingsRequest { time_type });
                    tracing::info!(
                        "切换到 {} 榜（需要加载）: {:?}",
                        time_type.display_name(),
                        start.elapsed()
                    );
                } else {
                    tracing::info!(
                        "切换到 {} 榜（使用缓存）: {:?}",
                        time_type.display_name(),
                        start.elapsed()
                    );
                }
            }
            RankingsTabType::Knight => {
                // 如果骑士榜数据还没有加载，发送加载请求
                if !rankings_state.is_knight_loaded() {
                    rankings_state.knight_loading = true;
                    load_knight_messages.write(LoadKnightRankingsRequest);
                    tracing::info!("切换到骑士榜（需要加载）: {:?}", start.elapsed());
                } else {
                    tracing::info!("切换到骑士榜（使用缓存）: {:?}", start.elapsed());
                }
            }
        }
    }
}

/// 漫画卡片点击交互（配色由全局 `apply_button_interaction` 统一处理）
pub fn rankings_card_interaction(
    interaction_query: Query<(&Interaction, &RankingsComicCard), Changed<Interaction>>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // 通过导航消息跳转到详情页（保留导航历史）
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
            tracing::info!("点击排行榜漫画: {}", card.comic_id);
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
    mut tab_query: Query<(&RankingsTabButton, &mut ButtonStyle)>,
    card_query: Query<&RankingsComicCard>,
) {
    if !rankings_state.is_changed() {
        return;
    }

    let start = std::time::Instant::now();
    tracing::debug!("refresh_rankings_ui 开始");

    // 更新 Tab 按钮选中态（配色交给全局 apply_button_interaction）
    for (tab, mut style) in tab_query.iter_mut() {
        let is_active = rankings_state.current_tab == tab.tab_type;
        if style.selected != is_active {
            style.selected = is_active;
        }
    }

    // 骑士榜模式下跳过漫画排行的刷新逻辑，由 refresh_knight_rankings_ui 处理
    if rankings_state.current_tab.is_knight() {
        tracing::debug!(
            "refresh_rankings_ui 跳过（当前为骑士榜）: {:?}",
            start.elapsed()
        );
        return;
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
        commands
            .spawn_scene(loading_indicator())
            .insert(ChildOf(container_entity));
    } else {
        // 显示空状态
        commands
            .spawn_scene(empty_state("点击上方标签加载排行榜"))
            .insert(ChildOf(container_entity));
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
    downloaded: Res<DownloadedComicsIndex>,
    time: Res<Time>,
    _asset_server: Res<AssetServer>,
) {
    // 骑士榜模式下跳过漫画卡片创建
    if rankings_state.current_tab.is_knight() {
        return;
    }

    // 如果数据已加载但 creation_state 未启动，主动启动预创建
    // （解决系统执行顺序导致 is_changed() 检测失败的问题）
    if !creation_state.is_creating
        && !rankings_state.is_loading
        && let Ok((container_entity, children)) = scroll_container_query.single()
    {
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

        // 惰性过滤：稳定态（已有卡片且类型匹配）什么都不做，不必每帧全量 zhconv
        // 扫描
        if !has_cards || !type_matches {
            let comics = rankings_state.current_comics();
            let filtered_indices = CompiledFilter::from_settings().filter_comic_indices(comics);

            if !filtered_indices.is_empty() {
                // 如果有卡片但类型不匹配，需要清除旧卡片
                if has_cards {
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
                creation_state.start_precreate_with_context(filtered_indices.len(), font, context);
                tracing::debug!(
                    "自动启动排行榜卡片预创建: {} 个（过滤后，{:?}）",
                    filtered_indices.len(),
                    rankings_state.current_type
                );
            }
        }
        let _ = container_entity; // suppress warning
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

        // 字体句柄只作为"预创建已启动"的门闸，BSN 场景统一走默认字体句柄
        if creation_state.font_handle.is_none() {
            return;
        }

        let comics = rankings_state.current_comics();
        // 惰性过滤：预创建是单帧一次性事件，与上面的启动检测各算各的
        let filtered_indices = CompiledFilter::from_settings().filter_comic_indices(comics);
        let count = creation_state.get_precreate_count();

        if filtered_indices.is_empty() || count == 0 {
            creation_state.clear();
            return;
        }

        // 一次性创建所有隐藏卡片（使用过滤后的索引，保留原始排名号）
        let mut entities = Vec::with_capacity(count);
        for i in 0..count {
            if let Some(&original_index) = filtered_indices.get(i)
                && let Some(comic) = comics.get(original_index)
            {
                // 排名号使用原始索引 +1，保留真实排名
                let entity = commands
                    .spawn_scene(comic_card(comic, original_index + 1, &downloaded, true))
                    .insert(ChildOf(container_entity))
                    .id();
                entities.push(entity);
            }
        }

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

/// 加载指示器场景
fn loading_indicator() -> impl Scene {
    bsn! {
        RankingsLoadingIndicator
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
                Text(ICON_TIMER_SAND)
                TextFont { font_size: FontSize::Px(48.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                Text("加载中...")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// 空状态场景
fn empty_state(message: &str) -> impl Scene + use<> {
    let message = message.to_string();

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
                Text(ICON_INBOX)
                TextFont { font_size: FontSize::Px(48.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                Text({message})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// 错误状态场景（与空态区分：红字，不是灰字）
///
/// 刻意不挂 `ErrorMessage` 标记——该标记被漫画列表/分类页的刷新系统全局查询，
/// 排行榜挂上会让缓存页面的残留实体误抑制别的页面的错误提示。
fn error_state(message: &str) -> impl Scene + use<> {
    let message = message.to_string();

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
                Text(ICON_CLOSE)
                TextFont { font_size: FontSize::Px(48.0) }
                TextColor(AppColors::ERROR)
            ),
            (
                Text({message})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::ERROR)
            ),
        ]
    }
}

/// 漫画卡片场景（`hidden` 用于瀑布式预创建，先隐藏后分批显示）
fn comic_card(
    comic: &picacg_api::models::Comic,
    rank: usize,
    downloaded: &DownloadedComicsIndex,
    hidden: bool,
) -> impl Scene + use<> {
    let card_comic_id = comic.id.clone();
    let menu_comic_id = comic.id.clone();
    let menu_comic_title = comic.title.clone();
    let menu_eps_count = comic.eps_count;
    let image_url = comic.thumb.url();
    let title = truncate_text(&comic.title, 12);
    let author = truncate_text(&comic.author, 10);
    let likes_label = format!("❤️ {}", format_number(comic.likes_count));
    let rank_label = format!("#{}", rank);

    let visibility = if hidden {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    // 排名标签配色
    let (badge_color, badge_text_color) = match rank {
        1 => (Color::srgb(1.0, 0.84, 0.0), Color::BLACK), // 金色
        2 => (Color::srgb(0.75, 0.75, 0.75), Color::BLACK), // 银色
        3 => (Color::srgb(0.8, 0.5, 0.2), Color::WHITE),  // 铜色
        _ => (Color::srgba(0.0, 0.0, 0.0, 0.7), Color::WHITE),
    };

    // 排名标签内边距（左右 8 / 上下 4）
    let badge_padding = UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0));

    // 分类和标签容器（两者都为空时不创建）
    let tags_container: Box<dyn SceneList> =
        if !comic.categories.is_empty() || !comic.tags.is_empty() {
            // 分类（蓝色）+ 标签（绿色）
            let badges: Vec<_> = comic
                .categories
                .iter()
                .take(2)
                .map(|category| tag_badge_truncated(category, TagColor::Category, 6))
                .chain(
                    comic
                        .tags
                        .iter()
                        .take(2)
                        .map(|tag| tag_badge_truncated(tag, TagColor::Tag, 6)),
                )
                .collect();

            Box::new(bsn_list![(
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(3.0),
                    row_gap: Val::Px(2.0),
                    max_width: {Val::Px(layout::CARD_WIDTH - 16.0)},
                    overflow: Overflow::clip(),
                }
                Children [ {badges} ]
            )])
        } else {
            Box::new(bsn_list![])
        };

    // 创建/更新时间
    let time_info = comic_time_info(comic.created_at.as_deref(), comic.updated_at.as_deref());

    // 封面右下角下载角标（挂在封面区域内，与左上角排名标签互不遮挡）
    let badge: Box<dyn SceneList> = Box::new(bsn_list![download_status_badge(
        &comic.id,
        comic.eps_count,
        downloaded,
        BadgeAnchor::CoverContainer
    )]);

    bsn! {
        RankingsComicCard { comic_id: {card_comic_id}, rank: {rank} }
        ContextMenuTarget { comic_id: {menu_comic_id}, comic_title: {menu_comic_title}, eps_count: {menu_eps_count} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Px(layout::CARD_WIDTH),
            height: Val::Px(layout::CARD_HEIGHT),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        // 静息底色与 ButtonStyle::card() 的 None 态一致，避免首帧闪烁
        BackgroundColor(AppColors::SURFACE)
        template_value(BorderColor::all(AppColors::BORDER))
        template_value(visibility)
        Children [
            (
                // 封面区域（带排名标签）
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(layout::COVER_HEIGHT),
                    position_type: PositionType::Relative,
                }
                Children [
                    (
                        // 封面图片占位
                        RankingsComicImage { url: {image_url} }
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::top(Val::Px(8.0)),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        Children [
                            (
                                // 加载中文字
                                Text(ICON_BOOK)
                                TextFont { font_size: FontSize::Px(32.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    (
                        // 排名标签
                        RankBadge
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(8.0),
                            left: Val::Px(8.0),
                            padding: {badge_padding},
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(badge_color)
                        Children [
                            (
                                Text({rank_label})
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(badge_text_color)
                            )
                        ]
                    ),
                    // 下载状态角标（右下角）
                    {badge},
                ]
            ),
            (
                // 信息区域
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(4.0),
                }
                Children [
                    (
                        // 标题
                        Text({title})
                        TextFont { font_size: FontSize::Px(13.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 作者
                        Text({author})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        // 点赞数
                        Text({likes_label})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    // 分类和标签容器
                    {tags_container},
                    // 创建/更新时间
                    {time_info},
                ]
            ),
        ]
    }
}

// ==================== 图片加载 ====================

/// 更新排行榜图片
///
/// 不使用 `is_changed()` 检查，因为系统执行顺序可能导致检测失败。图片填好
/// （或确认加载失败）后摘掉 `RankingsComicImage` 标记，实体即退出每帧扫描集，
/// 无需再逐个查子节点确认是否已有 `ImageNode`。
pub fn update_rankings_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    image_query: Query<(Entity, &RankingsComicImage, Option<&Children>)>,
) {
    for (entity, placeholder, children) in image_query.iter() {
        // 加载失败：摘掉占位标记，保留占位图标，但不再每帧重扫
        if image_cache.is_failed(&placeholder.url) {
            commands.entity(entity).remove::<RankingsComicImage>();
            continue;
        }

        // 检查缓存中是否有图片
        let Some(handle) = image_cache.get(&placeholder.url) else {
            continue;
        };

        // 清除占位内容（文字等）
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 添加图片，并摘掉占位标记
        commands
            .spawn_scene(cover_image(handle.clone()))
            .insert(ChildOf(entity));
        commands.entity(entity).remove::<RankingsComicImage>();
        // 图片加载请求已在 handle_rankings_response 中发送，无需重复请求
    }
}

/// 封面图片场景（替换占位文字）
fn cover_image(handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        ImageNode { image: {handle} }
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border_radius: BorderRadius::top(Val::Px(8.0)),
        }
    }
}

// ==================== 进入页面时触发加载 ====================

/// 进入排行榜页面时触发加载
pub fn trigger_load_rankings(
    rankings_state: Res<RankingsState>,
    mut load_messages: MessageWriter<LoadRankingsRequest>,
    mut load_knight_messages: MessageWriter<LoadKnightRankingsRequest>,
) {
    match rankings_state.current_tab {
        RankingsTabType::Comics(time_type) => {
            // 如果漫画排行数据还没有加载，发送加载请求
            if !rankings_state.is_loaded(time_type) && !rankings_state.is_loading {
                load_messages.write(LoadRankingsRequest { time_type });
                tracing::info!("自动加载 {} 榜", time_type.display_name());
            }
        }
        RankingsTabType::Knight => {
            // 如果骑士榜数据还没有加载，发送加载请求
            if !rankings_state.is_knight_loaded() && !rankings_state.knight_loading {
                load_knight_messages.write(LoadKnightRankingsRequest);
                tracing::info!("自动加载骑士榜");
            }
        }
    }
}

// ==================== 骑士榜系统 ====================

/// 刷新骑士榜 UI
pub fn refresh_knight_rankings_ui(
    mut commands: Commands,
    rankings_state: Res<RankingsState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<RankingsScrollContainer>>,
    knight_container_query: Query<Entity, With<KnightListContainer>>,
) {
    if !rankings_state.is_changed() {
        return;
    }

    // 只在骑士榜标签激活时处理
    if !rankings_state.current_tab.is_knight() {
        return;
    }

    let Ok((container_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 如果已有骑士榜容器且数据未变化，跳过
    if !knight_container_query.is_empty() && !rankings_state.knight_users.is_empty() {
        return;
    }

    // 清除现有内容
    if let Some(children) = children {
        for child in children.iter() {
            if let Ok(mut entity_commands) = commands.get_entity(child) {
                entity_commands.despawn();
            }
        }
    }

    if rankings_state.knight_loading {
        // 显示加载中
        commands
            .spawn_scene(loading_indicator())
            .insert(ChildOf(container_entity));
    } else if let Some(ref error) = rankings_state.knight_error {
        // 显示错误（红字，与灰色空态区分）
        let error_msg = format!("加载失败: {}", error);
        commands
            .spawn_scene(error_state(&error_msg))
            .insert(ChildOf(container_entity));
    } else if rankings_state.knight_users.is_empty() {
        // 显示空状态
        commands
            .spawn_scene(empty_state("暂无骑士榜数据"))
            .insert(ChildOf(container_entity));
    } else {
        // 渲染骑士榜用户列表
        // 骑士榜卡片直接作为滚动容器的子节点
        // 滚动容器已有 padding，无需额外 padding
        commands
            .spawn_scene(knight_list(&rankings_state.knight_users))
            .insert(ChildOf(container_entity));

        tracing::info!(
            "骑士榜 UI 已渲染: {} 位骑士",
            rankings_state.knight_users.len()
        );
    }
}

/// 骑士榜列表场景
fn knight_list(users: &[KnightUser]) -> impl Scene + use<> {
    let cards: Vec<_> = users
        .iter()
        .enumerate()
        .map(|(index, user)| knight_card(user, index + 1))
        .collect();

    bsn! {
        KnightListContainer
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [ {cards} ]
    }
}

/// 骑士榜用户卡片场景
fn knight_card(user: &KnightUser, rank: usize) -> impl Scene + use<> {
    // 排名颜色
    let (badge_color, badge_text_color) = match rank {
        1 => (Color::srgb(1.0, 0.84, 0.0), Color::BLACK), // 金色
        2 => (Color::srgb(0.75, 0.75, 0.75), Color::BLACK), // 银色
        3 => (Color::srgb(0.8, 0.5, 0.2), Color::WHITE),  // 铜色
        _ => (Color::srgba(0.3, 0.3, 0.35, 0.9), Color::WHITE), // 普通
    };

    let user_id = user.id.clone();
    let rank_label = format!("#{}", rank);
    // 用首字母作为头像占位
    let initial = user.name.chars().next().unwrap_or('?').to_string();
    let name = truncate_text(&user.name, 20);
    let level_label = format!("Lv.{}", user.level);
    let uploaded_label = format!("上传: {}", format_number(user.comics_uploaded as i64));

    // 称号标签（无称号时不创建）
    let title_badge: Box<dyn SceneList> = if !user.title.is_empty() {
        let title = truncate_text(&user.title, 10);
        Box::new(bsn_list![(
            Node {
                padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(2.0), Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
            }
            BackgroundColor(Color::srgba(0.6, 0.4, 0.0, 0.3))
            Children [
                (
                    Text({title})
                    TextFont { font_size: FontSize::Px(11.0) }
                    TextColor(Color::srgb(1.0, 0.84, 0.0))
                )
            ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        KnightRankCard { user_id: {user_id} }
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(12.0)),
            column_gap: Val::Px(16.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(AppColors::CARD_BG)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 排名标签
                Node {
                    min_width: Val::Px(40.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                }
                BackgroundColor(badge_color)
                Children [
                    (
                        Text({rank_label})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(badge_text_color)
                    )
                ]
            ),
            (
                // 用户头像占位（圆形）
                Node {
                    width: Val::Px(48.0),
                    height: Val::Px(48.0),
                    border_radius: BorderRadius::all(Val::Px(24.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor(Color::srgba(0.25, 0.25, 0.3, 1.0))
                Children [
                    (
                        Text({initial})
                        TextFont { font_size: FontSize::Px(20.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
            (
                // 用户信息区域
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(4.0),
                }
                Children [
                    (
                        // 用户名 + 称号
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                        }
                        Children [
                            (
                                Text({name})
                                TextFont { font_size: FontSize::Px(15.0) }
                                TextColor(AppColors::TEXT)
                            ),
                            {title_badge},
                        ]
                    ),
                    (
                        // 等级 + 上传数
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(16.0),
                        }
                        Children [
                            (
                                Text({level_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::PRIMARY)
                            ),
                            (
                                Text({uploaded_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
}
