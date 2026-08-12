//! 漫画列表系统

use bevy::prelude::*;

use super::font_loader::get_font;
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
    utils::content_filter::CompiledFilter,
};

/// 面包屑"分类"按钮，点击返回分类页
#[derive(Component, Default, Clone)]
pub struct BreadcrumbBackToCategories;

/// 虚拟滚动：顶部占位实体（撑起窗口上方被跳过的行）
#[derive(Component, Default, Clone)]
pub struct ComicsTopSpacer;

/// 虚拟滚动：底部占位实体
#[derive(Component, Default, Clone)]
pub struct ComicsBottomSpacer;

/// 漫画列表虚拟滚动状态
///
/// 只为可见窗口 ±2 行维持卡片实体；上下用 spacer 撑出正确的内容总高，
/// 上游滚动条与 `ComputedNode::content_size()` 因此天然正确。
/// 取代原瀑布流分帧建卡（实体数从"无限累积"钉到窗口常数）。
#[derive(Resource, Default)]
pub struct ComicsVirtualState {
    /// 过滤后的数据索引缓存（列表或屏蔽词变化时重建）
    filtered: Vec<usize>,
    /// 缓存对应的列表长度（用于检测数据变化）
    filtered_for_len: usize,
    /// 实测卡片高度（逻辑像素；0 = 未测量，测得后驱动 spacer 计算）
    card_height: f32,
    /// 当前列数
    columns: usize,
    /// 当前窗口行区间 [start_row, end_row)（半开）
    window: Option<(usize, usize)>,
    /// 窗口内卡片实体（与窗口数据索引一一对应，按序）
    cards: Vec<Entity>,
}

impl ComicsVirtualState {
    /// 清空（换分类/退出页面时调用）
    pub fn clear(&mut self) {
        self.filtered.clear();
        self.filtered_for_len = 0;
        self.card_height = 0.0;
        self.columns = 0;
        self.window = None;
        self.cards.clear();
    }
}

/// 漫画卡片布局常量
mod comic_layout {
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

/// 卡片固定宽度（与 comic_card 场景一致）
const CARD_WIDTH: f32 = 180.0;
/// 卡片高度估算值（首帧未实测时兜底；实测后被覆盖）
const CARD_FALLBACK_HEIGHT: f32 = 330.0;

/// 创建漫画列表界面（如果已存在则只显示）
pub fn setup_comics_list_ui(
    mut commands: Commands,
    comics_state: Res<ComicsListState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut virtual_state: ResMut<ComicsVirtualState>,
    existing_query: Query<Entity, With<ComicsListRoot>>,
) {
    // 参数化页面：每次进入可能是不同分类，直接 despawn 重建
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    // 旧窗口实体已随根节点销毁，清空虚拟滚动状态待重建
    virtual_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    let comics_root = commands.spawn_scene(comics_list_page(&comics_state)).id();

    // 如果有 ContentArea，将漫画列表作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(comics_root);
    }
}

/// 漫画列表页面场景
fn comics_list_page(state: &ComicsListState) -> impl Scene + use<> {
    let category = state.category.clone();
    // 恢复上次退出时保存的滚动位置
    let scroll_offset = Vec2::new(0.0, state.scroll_y);
    // 网格内边距（右侧额外让出滚动条宽度）
    let grid_padding = UiRect {
        left: Val::Px(comic_layout::PADDING_LEFT),
        right: Val::Px(comic_layout::PADDING_RIGHT),
        top: Val::Px(comic_layout::PADDING_TOP),
        bottom: Val::Px(comic_layout::PADDING_BOTTOM),
    };

    // 加载中时显示指示器；漫画卡片通过瀑布式创建系统添加
    let loading_placeholder: Box<dyn SceneList> = if state.is_loading {
        Box::new(bsn_list![(
            LoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT)
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        ComicsListRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 标题栏（包含面包屑导航）
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
                        // 面包屑: 分类 > 当前分类名（"分类"可点击返回）
                        BreadcrumbBackToCategories
                        Button
                        template_value(ButtonStyle::ghost())
                        Node
                        // 静息底色与 ButtonStyle::ghost() 的 None 态一致
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text("分类")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    (
                        Text(">")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        Text({category})
                        TextFont { font_size: FontSize::Px(16.0) }
                        TextColor(AppColors::TEXT)
                    ),
                ]
            ),
            (
                // 滚动区域包装器（用于放置滚动条）
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
                        // 漫画网格（可滚动）
                        #ComicsScroll
                        ComicsScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: {grid_padding},
                            column_gap: Val::Px(comic_layout::COLUMN_GAP),
                            row_gap: Val::Px(comic_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        ScrollPosition({scroll_offset})
                        Children [
                            (
                                // 虚拟滚动上占位（width:100% 独占整行）
                                ComicsTopSpacer
                                Node { width: Val::Percent(100.0), height: Val::Px(0.0) }
                            ),
                            (
                                // 虚拟滚动下占位
                                ComicsBottomSpacer
                                Node { width: Val::Percent(100.0), height: Val::Px(0.0) }
                            ),
                            {loading_placeholder},
                        ]
                    ),
                    // 创建滚动条
                    scrollbar(#ComicsScroll),
                ]
            ),
            // 无限滚动不再需要分页控件
        ]
    }
}

/// 漫画封面缩略图场景（图片已缓存时使用）
fn comic_thumbnail(url: String, handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        ComicThumbnail { url: {url} }
        ImageNode { image: {handle} }
        Node {
            width: Val::Px(164.0),
            height: Val::Px(220.0),
        }
    }
}

/// 漫画标签徽章场景（漫画列表专用紫色配色，与 `ui_common` 的绿色标签不同）
fn comic_tag_badge(text: &str) -> impl Scene + use<> {
    let text = text.to_string();

    // 单实体徽章：Text 节点自带 padding/圆角/底色（原 Node 套 Text 两实体，虚拟滚动
    // 窗口内实体数减半的主要来源之一）
    bsn! {
        Text({text})
        TextFont { font_size: FontSize::Px(10.0) }
        TextColor(Color::srgb(0.9, 0.7, 0.9))
        Node {
            padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(1.0), Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
        }
        BackgroundColor(Color::srgba(0.6, 0.3, 0.6, 0.3))
    }
}

/// 漫画卡片场景
fn comic_card(comic: &picacg_api::models::Comic, image_cache: &ImageCache) -> impl Scene + use<> {
    let card_comic_id = comic.id.clone();
    let menu_comic_id = comic.id.clone();
    let menu_comic_title = comic.title.clone();
    let title = comic.title.clone();
    let author = comic.author.clone();

    // 封面图片（已缓存直接显示，否则先放占位符）
    let thumb_url = comic.thumb.url();
    let cover: Box<dyn SceneList> = match image_cache.get(&thumb_url) {
        Some(handle) => Box::new(bsn_list![comic_thumbnail(
            thumb_url.clone(),
            handle.clone()
        )]),
        // 占位符自带 URL，图片替换系统据此直接取缓存，无需反查漫画列表
        None => {
            let placeholder_url = thumb_url.clone();
            Box::new(bsn_list![(
                PlaceholderImage
                ComicThumbnail { url: {placeholder_url} }
                Node {
                    width: Val::Px(164.0),
                    height: Val::Px(220.0),
                }
                BackgroundColor(AppColors::SURFACE_HOVER)
            )])
        }
    };

    // 分类标签容器（为空时不创建）
    let categories_container: Box<dyn SceneList> = if !comic.categories.is_empty() {
        // 最多显示 3 个分类
        let badges: Vec<_> = comic
            .categories
            .iter()
            .take(3)
            .map(|category| tag_badge(category, TagColor::Category))
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

    // 标签容器（为空时不创建）
    let tags_container: Box<dyn SceneList> = if !comic.tags.is_empty() {
        // 最多显示 3 个标签
        let badges: Vec<_> = comic
            .tags
            .iter()
            .take(3)
            .map(|tag| comic_tag_badge(tag.as_str()))
            .collect();

        Box::new(bsn_list![(
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(2.0),
                max_width: Val::Px(164.0),
                margin: UiRect::top(Val::Px(2.0)),
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
        ComicCard { comic_id: {card_comic_id} }
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
        // 静息底色与 ButtonStyle::card() 的 None 态一致，避免首帧闪烁
        BackgroundColor(AppColors::SURFACE)
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
                Node { margin: UiRect::bottom(Val::Px(4.0)) }
            ),
            // 分类标签容器
            {categories_container},
            // 标签容器
            {tags_container},
            // 创建/更新时间
            {time_info},
        ]
    }
}

/// 清理漫画列表界面（退出时保存滚动位置）
pub fn cleanup_comics_list_ui(
    mut commands: Commands,
    query: Query<Entity, With<ComicsListRoot>>,
    mut virtual_state: ResMut<ComicsVirtualState>,
    scroll_query: Query<&ScrollPosition, With<ComicsScrollContainer>>,
    mut comics_state: ResMut<ComicsListState>,
) {
    // 保存滚动位置
    if let Ok(scroll_pos) = scroll_query.single() {
        comics_state.scroll_y = scroll_pos.y;
    }
    virtual_state.clear();
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 漫画卡片交互系统（配色由全局 `apply_button_interaction` 统一处理）
pub fn comic_card_interaction(
    interaction_query: Query<(&Interaction, &ComicCard), Changed<Interaction>>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // 通过导航消息跳转到详情页（保留导航历史）
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
        }
    }
}

/// 无限滚动自动加载更多漫画
pub fn auto_load_more_comics(
    scroll_query: Query<(&ScrollPosition, &ComputedNode), With<ComicsScrollContainer>>,
    mut comics_state: ResMut<ComicsListState>,
    mut load_messages: MessageWriter<LoadComicsRequest>,
) {
    let Ok((scroll_pos, computed)) = scroll_query.single() else {
        return;
    };

    // 内容/视口尺寸由引擎布局输出（物理像素），换算成 ScrollPosition 所用的逻辑像素
    let content_height = computed.content_size().y * computed.inverse_scale_factor;
    let viewport_height = computed.size().y * computed.inverse_scale_factor;

    // 视口或内容高度为 0 时不触发
    if viewport_height <= 0.0 || content_height <= 0.0 {
        return;
    }

    let remaining = content_height - viewport_height - scroll_pos.y;

    // 距底部 200px 时触发加载下一页
    if remaining < 200.0
        && !comics_state.is_loading
        && !comics_state.is_loading_more
        && comics_state.page < comics_state.total_pages
    {
        comics_state.page += 1;
        comics_state.is_loading_more = true;
        load_messages.write(LoadComicsRequest {
            category: comics_state.category.clone(),
            page: comics_state.page,
            sort: comics_state.sort.clone(),
        });
        tracing::debug!(
            "无限滚动：加载第 {}/{} 页",
            comics_state.page,
            comics_state.total_pages
        );
    }
}

/// 刷新漫画列表界面（只处理错误状态，卡片由瀑布式系统创建）
///
/// 注意：这个函数**不应该**在数据加载完成后重建整个
/// UI，否则会覆盖瀑布式系统创建的卡片。 它只在出现错误时处理错误显示。
pub fn refresh_comics_list_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    comics_state: Res<ComicsListState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ComicsScrollContainer>>,
    error_query: Query<Entity, With<ErrorMessage>>,
) {
    // 只在状态变化时检查
    if !comics_state.is_changed() {
        return;
    }

    // 如果有错误，显示错误信息
    if let Some(ref error) = comics_state.error {
        // 如果还没有错误信息 UI，添加它
        if error_query.is_empty()
            && let Ok((container_entity, _)) = scroll_container_query.single()
        {
            let error_text = format!("加载失败: {}", error);
            commands
                .spawn_scene(bsn! {
                    ErrorMessage
                    Text({error_text})
                    TextFont { font_size: FontSize::Px(16.0) }
                    TextColor(AppColors::ERROR)
                })
                .insert(ChildOf(container_entity));
        }
    }

    // 如果数据存在或已有卡片，让瀑布式系统处理，不干涉
    // 数据为空且没有卡片则保持加载中状态
}

/// 虚拟滚动窗口维护（取代瀑布流分帧建卡）
///
/// 只为可见窗口 ±2 行维持卡片实体，上下 spacer 撑起总高度。
/// 滚动跨行边界时按行增量 spawn/despawn；数据或列数变化时全量重建。
/// 200 张卡片时在场实体从 ~4200 钉到 ~300。
#[allow(clippy::too_many_arguments)]
pub fn comics_virtual_scroll(
    mut commands: Commands,
    comics_state: Res<ComicsListState>,
    mut virtual_state: ResMut<ComicsVirtualState>,
    image_cache: Res<ImageCache>,
    scroll_query: Query<(Entity, &ScrollPosition, &ComputedNode), With<ComicsScrollContainer>>,
    scroll_changed: Query<
        (),
        (
            With<ComicsScrollContainer>,
            Or<(Changed<ScrollPosition>, Changed<ComputedNode>)>,
        ),
    >,
    mut top_spacer: Query<&mut Node, (With<ComicsTopSpacer>, Without<ComicsBottomSpacer>)>,
    mut bottom_spacer: Query<&mut Node, (With<ComicsBottomSpacer>, Without<ComicsTopSpacer>)>,
    card_computed: Query<&ComputedNode, With<ComicCard>>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
) {
    // 触发条件：滚动/布局变化、数据变化、或窗口未初始化；其余帧零开销
    if scroll_changed.is_empty() && !comics_state.is_changed() && virtual_state.window.is_some() {
        return;
    }
    let Ok((container, scroll_pos, computed)) = scroll_query.single() else {
        return;
    };

    // 数据变化（加载/追加/换分类）→ 重建过滤缓存并作废窗口。
    // 屏蔽词过滤只在这条低频路径执行。
    if comics_state.is_changed() || virtual_state.filtered_for_len != comics_state.comics.len() {
        let filter = crate::utils::content_filter::CompiledFilter::from_settings();
        virtual_state.filtered = filter.filter_comic_indices(&comics_state.comics);
        virtual_state.filtered_for_len = comics_state.comics.len();
        let stale: Vec<Entity> = std::mem::take(&mut virtual_state.cards);
        for entity in stale {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
        virtual_state.window = None;
        // 数据就绪后移除加载指示器
        if !comics_state.comics.is_empty() {
            for entity in loading_query.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                }
            }
        }
    }

    // 几何参数
    let inv = computed.inverse_scale_factor;
    let inner_width =
        computed.size().x * inv - comic_layout::PADDING_LEFT - comic_layout::PADDING_RIGHT;
    let viewport_height = computed.size().y * inv;
    if inner_width <= 0.0 || viewport_height <= 0.0 {
        return;
    }
    let columns = ((inner_width + comic_layout::COLUMN_GAP)
        / (CARD_WIDTH + comic_layout::COLUMN_GAP))
        .floor()
        .max(1.0) as usize;

    // 行高：优先实测在场卡片（图片加载后高度可能变化），未测量用估算值兜底
    if let Some(card_node) = card_computed.iter().next() {
        let measured = card_node.size().y * card_node.inverse_scale_factor;
        if measured > 1.0 {
            virtual_state.card_height = measured;
        }
    }
    let card_height = if virtual_state.card_height > 1.0 {
        virtual_state.card_height
    } else {
        CARD_FALLBACK_HEIGHT
    };
    let row_pitch = card_height + comic_layout::ROW_GAP;

    let total = virtual_state.filtered.len();
    if total == 0 {
        // 空列表：spacer 归零即可
        set_spacer_height(&mut top_spacer, 0.0);
        set_spacer_height(&mut bottom_spacer, 0.0);
        virtual_state.window = Some((0, 0));
        return;
    }
    let total_rows = total.div_ceil(columns);

    // 目标窗口（可见行 ±2，半开区间）
    let scrolled = (scroll_pos.y - comic_layout::PADDING_TOP).max(0.0);
    let first_visible_row = (scrolled / row_pitch).floor() as usize;
    let last_visible_row = ((scrolled + viewport_height) / row_pitch).ceil() as usize;
    let new_start = first_visible_row.saturating_sub(2).min(total_rows);
    let new_end = (last_visible_row + 2).min(total_rows);

    // 列数变化 → 行映射全变，作废窗口
    if virtual_state.columns != columns {
        virtual_state.columns = columns;
        let stale: Vec<Entity> = std::mem::take(&mut virtual_state.cards);
        for entity in stale {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
        virtual_state.window = None;
    }

    // 行 → 过滤索引区间（末行可能不满）
    let row_index_range = |row: usize| -> std::ops::Range<usize> {
        (row * columns).min(total)..((row + 1) * columns).min(total)
    };
    // 按行区间建卡（数据序）
    let spawn_rows = |commands: &mut Commands,
                      filtered: &[usize],
                      rows: std::ops::Range<usize>|
     -> Vec<Entity> {
        let mut entities = Vec::new();
        for row in rows {
            for &comic_index in &filtered[row_index_range(row)] {
                if let Some(comic) = comics_state.comics.get(comic_index) {
                    entities.push(commands.spawn_scene(comic_card(comic, &image_cache)).id());
                }
            }
        }
        entities
    };

    let old_window = virtual_state.window;
    match old_window {
        // 窗口重叠：行级增量
        Some((old_start, old_end))
            if new_start < old_end && old_start < new_end && !virtual_state.cards.is_empty() =>
        {
            if (old_start, old_end) != (new_start, new_end) {
                // 顶部移出窗口的行
                if new_start > old_start {
                    let remove =
                        row_index_range(new_start).start - row_index_range(old_start).start;
                    let cards_len = virtual_state.cards.len();
                    let removed: Vec<Entity> =
                        virtual_state.cards.drain(..remove.min(cards_len)).collect();
                    for entity in removed {
                        if let Ok(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.despawn();
                        }
                    }
                }
                // 底部移出窗口的行
                if new_end < old_end {
                    let keep = row_index_range(new_end).start - row_index_range(new_start).start;
                    let cards_len = virtual_state.cards.len();
                    let removed = virtual_state.cards.split_off(keep.min(cards_len));
                    for entity in removed {
                        if let Ok(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.despawn();
                        }
                    }
                }
                // 顶部移入的行（插到 TopSpacer 之后 = 子索引 1）
                if new_start < old_start {
                    let fresh =
                        spawn_rows(&mut commands, &virtual_state.filtered, new_start..old_start);
                    commands.entity(container).insert_children(1, &fresh);
                    let mut merged = fresh;
                    merged.append(&mut virtual_state.cards);
                    virtual_state.cards = merged;
                }
                // 底部移入的行（插到 BottomSpacer 之前）
                if new_end > old_end {
                    let fresh =
                        spawn_rows(&mut commands, &virtual_state.filtered, old_end..new_end);
                    commands
                        .entity(container)
                        .insert_children(1 + virtual_state.cards.len(), &fresh);
                    virtual_state.cards.extend(fresh);
                }
            }
        }
        // 无重叠/首建：全量重建窗口
        _ => {
            let stale: Vec<Entity> = std::mem::take(&mut virtual_state.cards);
            for entity in stale {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                }
            }
            let fresh = spawn_rows(&mut commands, &virtual_state.filtered, new_start..new_end);
            commands.entity(container).insert_children(1, &fresh);
            virtual_state.cards = fresh;
        }
    }
    virtual_state.window = Some((new_start, new_end));

    // spacer 高度 = 窗口外行数 × 行距（近似含行间隙，误差 < 1 gap 不可感知）
    set_spacer_height(&mut top_spacer, new_start as f32 * row_pitch);
    set_spacer_height(
        &mut bottom_spacer,
        (total_rows - new_end) as f32 * row_pitch,
    );
}

/// 比较后写 spacer 高度（避免无谓布局标脏）
fn set_spacer_height<F: bevy::ecs::query::QueryFilter>(
    query: &mut Query<&mut Node, F>,
    height: f32,
) {
    for mut node in query.iter_mut() {
        let target = Val::Px(height);
        if node.height != target {
            node.height = target;
        }
    }
}

/// 每帧扫描占位符（不仅在 `image_cache` 变化时），因为占位符可能在缓存
/// 变化之后的帧才创建。已换成图片的实体不带 `PlaceholderImage`，加载失败的
/// 实体会被摘掉该标记，两者都自然退出扫描集。
pub fn update_comics_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    placeholder_query: Query<
        (Entity, &ChildOf, &ComicThumbnail),
        (With<PlaceholderImage>, Without<ImageNode>),
    >,
) {
    let mut replaced_count = 0;
    for (placeholder_entity, child_of, thumb) in placeholder_query.iter() {
        // 加载失败：摘掉占位标记，保留灰底方块，但不再每帧重扫
        if image_cache.is_failed(&thumb.url) {
            commands
                .entity(placeholder_entity)
                .remove::<PlaceholderImage>();
            continue;
        }

        // 检查图片是否已加载
        let Some(handle) = image_cache.get(&thumb.url) else {
            continue;
        };

        // 删除占位符，添加实际图片
        let parent_entity: Entity = child_of.parent();
        commands.entity(placeholder_entity).despawn();
        // 创建新的图片实体并插入到父卡片的第一个位置（在标题之前）
        let image_entity = commands
            .spawn_scene(comic_thumbnail(thumb.url.clone(), handle.clone()))
            .id();
        commands
            .entity(parent_entity)
            .insert_children(0, &[image_entity]);
        replaced_count += 1;
    }

    if replaced_count > 0 {
        tracing::trace!("[Comics] 替换了 {} 个封面图片", replaced_count);
    }
}

/// 面包屑"分类"按钮交互：点击返回分类列表页
pub fn breadcrumb_back_to_categories(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<BreadcrumbBackToCategories>),
    >,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_route.set(AppRoute::Categories);
        }
    }
}
