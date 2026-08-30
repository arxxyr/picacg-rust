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
        ui_common::{
            BadgeAnchor, DownloadStatusBadge, LoadingShimmer, TagColor, download_status_badge,
            format_api_date,
        },
        widgets::ButtonStyle,
    },
    utils::{content_filter::CompiledFilter, icons::ICON_CHECK},
};

/// 面包屑"分类"按钮，点击返回分类页
#[derive(Component, Default, Clone)]
pub struct BreadcrumbBackToCategories;

// ==================== 批量选择组件 ====================

/// 选择模式开关按钮（"选择" / "退出选择"）
#[derive(Component, Default, Clone)]
pub struct ComicsSelectModeButton;

/// 「全选」按钮（选中当前已加载且未被屏蔽的全部漫画）
#[derive(Component, Default, Clone)]
pub struct ComicsSelectAllButton;

/// 「清空」按钮
#[derive(Component, Default, Clone)]
pub struct ComicsClearSelectionButton;

/// 「下载选中」按钮
#[derive(Component, Default, Clone)]
pub struct ComicsDownloadSelectedButton;

/// 选择工具栏容器（非选择模式时 display:none）
#[derive(Component, Default, Clone)]
pub struct ComicsSelectionBar;

/// 选择计数文本
#[derive(Component, Default, Clone)]
pub struct ComicsSelectionCountText;

/// 卡片上的选中标记（勾选圈，未选中时 Visibility::Hidden）
#[derive(Component, Default, Clone)]
pub struct ComicSelectionMark {
    pub comic_id: String,
}

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
    /// 待重绑定的卡片：(卡片实体, 漫画在 `ComicsListState.comics` 里的下标)
    ///
    /// 滚动时被复用的节点排进这里，由 `comics_rebind_cards` 下一步改内容。
    /// 分成两步是为了让滚动系统不必持有一大堆改 UI 的可变查询。
    pending_rebind: Vec<(Entity, usize)>,
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
        self.pending_rebind.clear();
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
    mut selection: ResMut<ComicsSelectionState>,
    existing_query: Query<Entity, With<ComicsListRoot>>,
) {
    // 参数化页面：每次进入可能是不同分类，直接 despawn 重建
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    // 换分类等于换数据集，上一轮的选中项不该带过来
    selection.exit();

    // 旧窗口实体已随根节点销毁，清空虚拟滚动状态待重建
    virtual_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    let comics_root = commands
        .spawn_scene(comics_list_page(&comics_state, &selection))
        .id();

    // 如果有 ContentArea，将漫画列表作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(comics_root);
    }
}

/// 漫画列表页面场景
fn comics_list_page(
    state: &ComicsListState,
    selection: &ComicsSelectionState,
) -> impl Scene + use<> {
    let category = state.category.clone();
    let selection_bar_display = if selection.active {
        Display::Flex
    } else {
        Display::None
    };
    let select_mode_label = if selection.active {
        "退出选择"
    } else {
        "选择"
    };
    let selection_count_label = format!("已选 {}", selection.selected.len());
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
                    (
                        // 撑开，把选择控件推到右侧
                        Node { flex_grow: 1.0 }
                    ),
                    (
                        // 选择工具条（仅选择模式可见）
                        ComicsSelectionBar
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            display: {selection_bar_display},
                        }
                        Children [
                            (
                                ComicsSelectionCountText
                                Text({selection_count_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                            toolbar_button(ComicsSelectAllButton, "全选", AppColors::BORDER),
                            toolbar_button(ComicsClearSelectionButton, "清空", AppColors::BORDER),
                            toolbar_button(
                                ComicsDownloadSelectedButton,
                                "下载选中",
                                AppColors::PRIMARY,
                            ),
                        ]
                    ),
                    toolbar_button(
                        ComicsSelectModeButton,
                        select_mode_label,
                        AppColors::BORDER,
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

/// 标题栏小按钮（选择模式的几个操作共用）
fn toolbar_button<T: Component + Default + Clone + Unpin>(
    marker: T,
    label: &str,
    border: Color,
) -> impl Scene + use<T> {
    let label = label.to_string();
    bsn! {
        template_value(marker)
        Button
        template_value(ButtonStyle::card())
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(AppColors::SURFACE)
        template_value(BorderColor::all(border))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 卡片内可重绑定的部件标记
///
/// 卡片改成**固定形态**：徽章、时间行都常驻，多余的槽位 `Display::None`。
/// 形态固定后，滚动时不必销毁重建，只要把这些节点的文字/颜色/可见性改掉
/// 就能"换一本漫画"——这是节点复用（`comics_virtual_scroll` 的 recycle 路径）
/// 成立的前提。
#[derive(Component, Default, Clone)]
pub struct CardTitle;

/// 卡片作者行
#[derive(Component, Default, Clone)]
pub struct CardAuthor;

/// 卡片徽章槽位（分类槽在前、标签槽在后，见 `CARD_CATEGORY_SLOTS`）
#[derive(Component, Default, Clone)]
pub struct CardBadgeSlot {
    pub index: usize,
}

/// 卡片徽章行容器（整行没内容时 `Display::None`）
#[derive(Component, Default, Clone)]
pub struct CardBadgeRow {
    /// true = 分类行，false = 标签行
    pub is_category: bool,
}

/// 卡片时间行（0 = 更新时间，1 = 创建时间）
#[derive(Component, Default, Clone)]
pub struct CardTimeSlot {
    pub index: usize,
}

/// 卡片时间容器
#[derive(Component, Default, Clone)]
pub struct CardTimeRow;

/// 分类徽章槽位数（与标签槽位数相同）
const CARD_CATEGORY_SLOTS: usize = 3;
/// 标签徽章槽位数
const CARD_TAG_SLOTS: usize = 3;
/// 时间行槽位数（更新 / 创建）
const CARD_TIME_SLOTS: usize = 2;

/// 漫画卡片场景（固定形态，可被 `bind_comic_card` 重绑定到任意一本漫画）
fn comic_card(
    comic: &picacg_api::models::Comic,
    image_cache: &ImageCache,
    downloaded: &DownloadedComicsIndex,
    selection: &ComicsSelectionState,
) -> impl Scene + use<> {
    let card_comic_id = comic.id.clone();
    let menu_comic_id = comic.id.clone();
    let menu_comic_title = comic.title.clone();
    let menu_eps_count = comic.eps_count;
    let title = comic.title.clone();
    let author = comic.author.clone();

    // 封面：**单实体**，缓存命中就直接带 ImageNode，否则留灰底等
    // update_comics_images 就地补 ImageNode（不再销毁占位再建图片实体）
    let thumb_url = comic.thumb.url();
    let cover: Box<dyn SceneList> = match image_cache.get(&thumb_url) {
        Some(handle) => Box::new(bsn_list![comic_cover_loaded(
            thumb_url.clone(),
            handle.clone()
        )]),
        None => Box::new(bsn_list![comic_cover_pending(thumb_url.clone())]),
    };

    // 徽章槽位：分类在前、标签在后，一次性建满，按数据决定显隐
    let category_slots: Vec<_> = (0..CARD_CATEGORY_SLOTS)
        .map(|i| card_badge_slot(i, comic.categories.get(i).map(String::as_str), true))
        .collect();
    let tag_slots: Vec<_> = (0..CARD_TAG_SLOTS)
        .map(|i| {
            card_badge_slot(
                CARD_CATEGORY_SLOTS + i,
                comic.tags.get(i).map(String::as_str),
                false,
            )
        })
        .collect();
    let time_slots: Vec<_> = (0..CARD_TIME_SLOTS)
        .map(|i| card_time_slot(i, card_time_label(comic, i).as_deref()))
        .collect();

    let category_display = row_display(!comic.categories.is_empty());
    let tag_display = row_display(!comic.tags.is_empty());
    let time_display = row_display(comic.created_at.is_some() || comic.updated_at.is_some());

    // 封面右下角下载角标（绝对定位，与封面同为卡片直接子节点）
    let badge: Box<dyn SceneList> = Box::new(bsn_list![download_status_badge(
        &comic.id,
        comic.eps_count,
        downloaded,
        BadgeAnchor::CardCover
    )]);

    // 封面左上角选中标记（与右下角下载角标错开，互不遮挡）
    let mark: Box<dyn SceneList> = Box::new(bsn_list![selection_mark(
        &comic.id,
        selection.selected.contains(&comic.id)
    )]);

    bsn! {
        ComicCard { comic_id: {card_comic_id} }
        ContextMenuTarget { comic_id: {menu_comic_id}, comic_title: {menu_comic_title}, eps_count: {menu_eps_count} }
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
                CardTitle
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
                CardAuthor
                Text({author})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { margin: UiRect::bottom(Val::Px(4.0)) }
            ),
            (
                // 分类徽章行
                CardBadgeRow { is_category: true }
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    display: {category_display},
                }
                Children [ {category_slots} ]
            ),
            (
                // 标签徽章行
                CardBadgeRow
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    margin: UiRect::top(Val::Px(2.0)),
                    overflow: Overflow::clip(),
                    display: {tag_display},
                }
                Children [ {tag_slots} ]
            ),
            (
                // 创建/更新时间
                CardTimeRow
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(2.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    display: {time_display},
                }
                Children [ {time_slots} ]
            ),
            // 下载状态角标（绝对定位，不参与列布局）
            {badge},
            // 批量选择标记（同上）
            {mark},
        ]
    }
}

/// 行容器显隐
fn row_display(has_content: bool) -> Display {
    if has_content {
        Display::Flex
    } else {
        Display::None
    }
}

/// 时间槽位文案（0 = 更新时间，1 = 创建时间）
fn card_time_label(comic: &picacg_api::models::Comic, index: usize) -> Option<String> {
    let raw = match index {
        0 => comic.updated_at.as_deref(),
        _ => comic.created_at.as_deref(),
    }?;
    let prefix = if index == 0 { "更新" } else { "创建" };
    Some(format!("{prefix} {}", format_api_date(raw)))
}

/// 徽章槽位场景（`value` 为 None 时建出来但隐藏，供复用时再点亮）
fn card_badge_slot(index: usize, value: Option<&str>, is_category: bool) -> impl Scene + use<> {
    let text = value.unwrap_or_default().to_string();
    let display = row_display(value.is_some());
    let (bg_color, text_color) = if is_category {
        TagColor::Category.colors()
    } else {
        // 漫画列表专用紫色标签，与 ui_common 的绿色标签区分
        (Color::srgba(0.6, 0.3, 0.6, 0.3), Color::srgb(0.9, 0.7, 0.9))
    };

    bsn! {
        CardBadgeSlot { index: {index} }
        Text({text})
        TextFont { font_size: FontSize::Px(10.0) }
        TextColor(text_color)
        Node {
            padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(1.0), Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
            display: {display},
        }
        BackgroundColor(bg_color)
    }
}

/// 时间槽位场景
fn card_time_slot(index: usize, label: Option<&str>) -> impl Scene + use<> {
    let text = label.unwrap_or_default().to_string();
    let display = row_display(label.is_some());

    bsn! {
        CardTimeSlot { index: {index} }
        Text({text})
        TextFont { font_size: FontSize::Px(9.0) }
        TextColor(AppColors::TEXT_SECONDARY)
        Node { display: {display} }
    }
}

/// 封面（图片已就绪）
fn comic_cover_loaded(url: String, handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        ComicThumbnail { url: {url} }
        ImageNode { image: {handle} }
        Node {
            width: Val::Px(164.0),
            height: Val::Px(220.0),
        }
    }
}

/// 封面（图片未就绪，等 `update_comics_images` 就地补 `ImageNode`）
fn comic_cover_pending(url: String) -> impl Scene + use<> {
    bsn! {
        PlaceholderImage
        ComicThumbnail { url: {url} }
        template_value(LoadingShimmer::new(AppColors::SURFACE_HOVER))
        Node {
            width: Val::Px(164.0),
            height: Val::Px(220.0),
        }
        BackgroundColor(AppColors::SURFACE_HOVER)
    }
}

/// 卡片左上角的选中标记
///
/// 与下载角标一样**始终创建**、靠 `Visibility` 控制显隐：选择状态变化时由
/// `refresh_comic_selection_marks` 就地点亮，不必重建卡片。
/// 定位口径同 `BadgeAnchor::CardCover`——卡片 padding 8 / border 1，
/// 封面左上角在 padding box 的 (8, 8)，向内缩 4px。
fn selection_mark(comic_id: &str, selected: bool) -> impl Scene + use<> {
    let marker = ComicSelectionMark {
        comic_id: comic_id.to_string(),
    };
    let visibility = if selected {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    bsn! {
        template_value(marker)
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            width: Val::Px(20.0),
            height: Val::Px(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(AppColors::PRIMARY)
        template_value(visibility)
        ZIndex(2)
        Children [
            (
                Text(ICON_CHECK)
                TextFont { font_size: FontSize::Px(15.0) }
                TextColor(Color::WHITE)
            )
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
    mut selection: ResMut<ComicsSelectionState>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 选择模式下点击 = 勾选/取消，不跳详情
        if selection.active {
            let now_selected = selection.toggle(&card.comic_id);
            tracing::debug!(
                "{}选中: {}",
                if now_selected { "" } else { "取消" },
                card.comic_id
            );
        } else {
            // 通过导航消息跳转到详情页（保留导航历史）
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
        }
    }
}

// ==================== 批量选择交互 ====================

/// 「选择 / 退出选择」开关
pub fn comics_select_mode_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ComicsSelectModeButton>)>,
    mut selection: ResMut<ComicsSelectionState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if selection.active {
            selection.exit();
        } else {
            selection.active = true;
        }
        tracing::info!("漫画列表选择模式: {}", selection.active);
    }
}

/// 「全选」：选中当前已加载且未被屏蔽的全部漫画
///
/// 以过滤后的列表为准——屏蔽掉的漫画根本没建卡，不该被"全选"捎带上。
pub fn comics_select_all_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ComicsSelectAllButton>)>,
    comics_state: Res<ComicsListState>,
    virtual_state: Res<ComicsVirtualState>,
    mut selection: ResMut<ComicsSelectionState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for &index in &virtual_state.filtered {
            if let Some(comic) = comics_state.comics.get(index) {
                selection.selected.insert(comic.id.clone());
            }
        }
        tracing::info!("全选：共 {} 本", selection.selected.len());
    }
}

/// 「清空」：只清选中项，保持在选择模式里
pub fn comics_clear_selection_interaction(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<ComicsClearSelectionButton>),
    >,
    mut selection: ResMut<ComicsSelectionState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed && !selection.selected.is_empty() {
            selection.selected.clear();
        }
    }
}

/// 「下载选中」：逐本发下载请求，随后退出选择模式
///
/// 并发上限由 `download_queue_manager` 管，这里只管把请求排进去；
/// `remote_eps_count` 顺手带上，下完就有更新基准（见封面角标一节）。
pub fn comics_download_selected_interaction(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<ComicsDownloadSelectedButton>),
    >,
    comics_state: Res<ComicsListState>,
    mut selection: ResMut<ComicsSelectionState>,
    mut download_messages: MessageWriter<DownloadComicRequest>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if selection.selected.is_empty() {
            tracing::warn!("未选中任何漫画");
            continue;
        }

        let mut count = 0;
        for comic in &comics_state.comics {
            if !selection.selected.contains(&comic.id) {
                continue;
            }
            download_messages.write(DownloadComicRequest {
                comic_id: comic.id.clone(),
                comic_title: comic.title.clone(),
                episodes: vec![], // 空 = 下载全部章节
                remote_eps_count: (comic.eps_count > 0).then_some(comic.eps_count),
            });
            count += 1;
        }

        tracing::info!("批量下载：已提交 {} 本漫画", count);
        selection.exit();
    }
}

/// 选择状态变化 → 刷新工具条显隐、计数文本、卡片选中标记
pub fn refresh_comics_selection_ui(
    selection: Res<ComicsSelectionState>,
    mut bar_query: Query<&mut Node, With<ComicsSelectionBar>>,
    mut count_query: Query<&mut Text, With<ComicsSelectionCountText>>,
    mut mark_query: Query<(Ref<ComicSelectionMark>, &mut Visibility)>,
    mode_btn_query: Query<&Children, With<ComicsSelectModeButton>>,
    mut mode_text_query: Query<&mut Text, Without<ComicsSelectionCountText>>,
) {
    // 选择状态没变时，仍要照顾刚被节点复用改绑的标记（它们的 comic_id 变了）
    let selection_changed = selection.is_changed();
    if !selection_changed {
        for (mark, mut visibility) in mark_query.iter_mut() {
            if !mark.is_changed() {
                continue;
            }
            let visible = selection.active && selection.selected.contains(&mark.comic_id);
            let target = if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *visibility != target {
                *visibility = target;
            }
        }
        return;
    }

    let display = if selection.active {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in bar_query.iter_mut() {
        if node.display != display {
            node.display = display;
        }
    }

    let count_label = format!("已选 {}", selection.selected.len());
    for mut text in count_query.iter_mut() {
        if text.as_str() != count_label {
            **text = count_label.clone();
        }
    }

    let mode_label = if selection.active {
        "退出选择"
    } else {
        "选择"
    };
    for children in mode_btn_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = mode_text_query.get_mut(child)
                && text.as_str() != mode_label
            {
                **text = mode_label.to_string();
            }
        }
    }

    // 退出选择模式后标记一律熄灭
    for (mark, mut visibility) in mark_query.iter_mut() {
        let visible = selection.active && selection.selected.contains(&mark.comic_id);
        let target = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != target {
            *visibility = target;
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

    // 内容/视口尺寸由引擎布局输出（物理像素），换算成 ScrollPosition
    // 所用的逻辑像素
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
    downloaded: Res<DownloadedComicsIndex>,
    selection: Res<ComicsSelectionState>,
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

    // ==================== 节点复用 ====================
    //
    // 不再"移出窗口就 despawn、移入窗口就 spawn"。移出的卡片实体直接留作
    // 空闲池，改绑到移入位置的数据上——实体数恒定，滚动路径上零 spawn/despawn。
    // 卡片能被改绑的前提是它是**固定形态**（徽章/时间槽位常驻，见
    // comic_card）。
    let new_lo = row_index_range(new_start).start;
    let new_hi = row_index_range(new_end).start;
    let needed = new_hi.saturating_sub(new_lo);

    let old_cards = std::mem::take(&mut virtual_state.cards);
    // 旧池只有在「区间长度与实体数对得上」时才可信；对不上说明中途出过岔子，
    // 整池作废重建，别拿错位的实体去改绑
    let old_span = virtual_state
        .window
        .map(|(s, e)| (row_index_range(s).start, row_index_range(e).start));
    let pool_valid = matches!(old_span, Some((lo, hi)) if hi.saturating_sub(lo) == old_cards.len())
        && !old_cards.is_empty();

    let mut slots: Vec<Option<Entity>> = vec![None; needed];
    let mut free: Vec<Entity> = Vec::new();

    if pool_valid {
        let (old_lo, old_hi) = old_span.expect("pool_valid 已保证 old_span 非空");
        let (keep, spare) = plan_recycle(old_lo..old_hi, new_lo..new_hi);
        for (offset, reused) in keep.iter().enumerate() {
            if let Some(old_offset) = reused {
                slots[offset] = Some(old_cards[*old_offset]);
            }
        }
        free.extend(spare.into_iter().map(|offset| old_cards[offset]));
    } else {
        for entity in old_cards {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
    }

    // 空位优先吃空闲池（改绑），池空了才新建
    for (offset, slot) in slots.iter_mut().enumerate() {
        if slot.is_some() {
            continue;
        }
        let Some(&comic_index) = virtual_state.filtered.get(new_lo + offset) else {
            continue;
        };
        let Some(comic) = comics_state.comics.get(comic_index) else {
            continue;
        };
        if let Some(entity) = free.pop() {
            virtual_state.pending_rebind.push((entity, comic_index));
            *slot = Some(entity);
        } else {
            *slot = Some(
                commands
                    .spawn_scene(comic_card(comic, &image_cache, &downloaded, &selection))
                    .id(),
            );
        }
    }

    // 窗口变小（缩窗/列数变化）才会有富余，此时才销毁
    for entity in free {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    let cards: Vec<Entity> = slots.into_iter().flatten().collect();
    // 子节点顺序 = 数据顺序（flex-wrap 按子序排版），复用后必须重排；
    // 索引 1 = TopSpacer 之后
    commands.entity(container).insert_children(1, &cards);
    virtual_state.cards = cards;

    virtual_state.window = Some((new_start, new_end));

    // spacer 高度 = 窗口外行数 × 行距（近似含行间隙，误差 < 1 gap 不可感知）
    set_spacer_height(&mut top_spacer, new_start as f32 * row_pitch);
    set_spacer_height(
        &mut bottom_spacer,
        (total_rows - new_end) as f32 * row_pitch,
    );
}

/// 复用规划：旧窗口的卡片如何落到新窗口的槽位
///
/// 两个区间都是「过滤后列表里的数据位置」，且都是连续区间（窗口按行切）。
/// 返回 `(keep, spare)`：
/// - `keep[i]` = 新窗口第 i 个槽位可以直接沿用的旧卡片下标（None =
///   需要改绑或新建）
/// - `spare`   = 移出新窗口、可拿去改绑的旧卡片下标
///
/// 抽成纯函数是为了能单测——复用路径只在滚动时触发，跑起来靠肉眼很难覆盖到
/// 「窗口缩小」「完全不重叠」这些边界。
fn plan_recycle(
    old_range: std::ops::Range<usize>,
    new_range: std::ops::Range<usize>,
) -> (Vec<Option<usize>>, Vec<usize>) {
    let mut keep = vec![None; new_range.len()];
    let mut spare = Vec::new();

    for (offset, pos) in old_range.clone().enumerate() {
        if new_range.contains(&pos) {
            keep[pos - new_range.start] = Some(offset);
        } else {
            spare.push(offset);
        }
    }

    (keep, spare)
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
    cover_query: Query<(Entity, &ComicThumbnail), (With<PlaceholderImage>, Without<ImageNode>)>,
) {
    let mut bound = 0;
    for (entity, thumb) in cover_query.iter() {
        // 加载失败：摘掉占位标记与微光，保留灰底方块，但不再每帧重扫
        //（微光必须停——一直脉动等于在骗用户"还在加载"）
        if image_cache.is_failed(&thumb.url) {
            commands
                .entity(entity)
                .remove::<PlaceholderImage>()
                .remove::<LoadingShimmer>();
            continue;
        }

        let Some(handle) = image_cache.get(&thumb.url) else {
            continue;
        };

        // 就地补 ImageNode——不再"销毁占位实体 + 新建图片实体 +
        // insert_children"， 那套会打乱卡片子节点顺序，
        // 也和节点复用冲突（复用要求封面实体身份稳定）
        commands
            .entity(entity)
            .remove::<PlaceholderImage>()
            .insert(ImageNode {
                image: handle.clone(),
                ..default()
            });
        bound += 1;
    }

    if bound > 0 {
        tracing::trace!("[Comics] 填入了 {} 个封面图片", bound);
    }
}

/// 把复用来的卡片节点改绑到另一本漫画
///
/// 只改内容、不动结构：文字、颜色、显隐、封面 URL、角标绑定。卡片是固定形态
/// （徽章/时间槽位常驻），所以"换一本漫画"退化成一串就地赋值。
///
/// **查询为什么长成这样**：卡片子节点全都有 `Node`，于是把 `Text`/`TextColor`/
/// `BackgroundColor` 全做成 `Option<&mut>` 挤进**同一个** `node_query`——
/// 拆成多个 `&mut Node` / `&mut Text` 查询会直接撞 B0001（同系统同组件多写），
/// 而想证明不相交就得给每个查询挂一串 `Without`，可读性更差。
pub fn comics_rebind_cards(
    mut commands: Commands,
    mut virtual_state: ResMut<ComicsVirtualState>,
    comics_state: Res<ComicsListState>,
    image_cache: Res<ImageCache>,
    mut card_query: Query<(&mut ComicCard, &mut ContextMenuTarget, &Children)>,
    children_query: Query<&Children>,
    mut node_query: Query<(
        &mut Node,
        Option<&mut Text>,
        Option<&mut TextColor>,
        Option<&mut BackgroundColor>,
        Option<&CardBadgeSlot>,
        Option<&CardTimeSlot>,
        Option<&CardBadgeRow>,
        Has<CardTimeRow>,
        Has<CardTitle>,
        Has<CardAuthor>,
    )>,
    mut cover_query: Query<(&mut ComicThumbnail, Has<ImageNode>)>,
    mut badge_query: Query<&mut DownloadStatusBadge>,
    mut mark_query: Query<&mut ComicSelectionMark>,
) {
    if virtual_state.pending_rebind.is_empty() {
        return;
    }

    let pending = std::mem::take(&mut virtual_state.pending_rebind);
    for (entity, comic_index) in pending {
        let Some(comic) = comics_state.comics.get(comic_index) else {
            continue;
        };
        let Ok((mut card, mut menu_target, children)) = card_query.get_mut(entity) else {
            continue;
        };

        card.comic_id = comic.id.clone();
        menu_target.comic_id = comic.id.clone();
        menu_target.comic_title = comic.title.clone();
        menu_target.eps_count = comic.eps_count;

        let child_list: Vec<Entity> = children.iter().collect();
        for child in child_list {
            // 封面：换 URL，并把旧图摘掉重新进入"等加载"状态
            if let Ok((mut thumb, has_image)) = cover_query.get_mut(child) {
                let url = comic.thumb.url();
                if thumb.url != url {
                    thumb.url = url.clone();
                    match image_cache.get(&url) {
                        Some(handle) => {
                            commands
                                .entity(child)
                                .remove::<PlaceholderImage>()
                                .insert(ImageNode {
                                    image: handle.clone(),
                                    ..default()
                                });
                        }
                        None => {
                            // 新 URL 还没缓存：撤掉旧图，挂回占位标记等
                            // update_comics_images
                            if has_image {
                                commands.entity(child).remove::<ImageNode>();
                            }
                            commands.entity(child).insert((
                                PlaceholderImage,
                                BackgroundColor(AppColors::SURFACE_HOVER),
                            ));
                        }
                    }
                }
                continue;
            }

            // 下载角标：改绑后由 refresh_download_status_badges 按 Changed
            // 刷外观
            if let Ok(mut badge) = badge_query.get_mut(child) {
                badge.comic_id = comic.id.clone();
                badge.remote_episodes = comic.eps_count;
                continue;
            }

            // 选中标记：同上，由 refresh_comics_selection_ui 刷显隐
            if let Ok(mut mark) = mark_query.get_mut(child) {
                mark.comic_id = comic.id.clone();
                continue;
            }

            rebind_card_node(child, comic, &mut node_query);

            // 徽章行 / 时间行的槽位在孙节点上
            if let Ok(grandchildren) = children_query.get(child) {
                let grandchild_list: Vec<Entity> = grandchildren.iter().collect();
                for grandchild in grandchild_list {
                    rebind_card_node(grandchild, comic, &mut node_query);
                }
            }
        }
    }
}

/// 就地重绑一个卡片子节点（标题/作者/徽章槽/时间槽/行容器，认不出的原样跳过）
fn rebind_card_node(
    entity: Entity,
    comic: &picacg_api::models::Comic,
    node_query: &mut Query<(
        &mut Node,
        Option<&mut Text>,
        Option<&mut TextColor>,
        Option<&mut BackgroundColor>,
        Option<&CardBadgeSlot>,
        Option<&CardTimeSlot>,
        Option<&CardBadgeRow>,
        Has<CardTimeRow>,
        Has<CardTitle>,
        Has<CardAuthor>,
    )>,
) {
    let Ok((
        mut node,
        text,
        _color,
        _bg,
        badge_slot,
        time_slot,
        badge_row,
        is_time_row,
        is_title,
        is_author,
    )) = node_query.get_mut(entity)
    else {
        return;
    };

    // 行容器：只管显隐
    if let Some(row) = badge_row {
        let has_content = if row.is_category {
            !comic.categories.is_empty()
        } else {
            !comic.tags.is_empty()
        };
        set_display(&mut node, has_content);
        return;
    }
    if is_time_row {
        set_display(
            &mut node,
            comic.created_at.is_some() || comic.updated_at.is_some(),
        );
        return;
    }

    let Some(mut text) = text else {
        return;
    };

    if is_title {
        set_text(&mut text, &comic.title);
        return;
    }
    if is_author {
        set_text(&mut text, &comic.author);
        return;
    }
    if let Some(slot) = badge_slot {
        // 前 CARD_CATEGORY_SLOTS 个是分类槽，其后是标签槽
        let value = if slot.index < CARD_CATEGORY_SLOTS {
            comic.categories.get(slot.index)
        } else {
            comic.tags.get(slot.index - CARD_CATEGORY_SLOTS)
        };
        set_text(&mut text, value.map(String::as_str).unwrap_or_default());
        set_display(&mut node, value.is_some());
        return;
    }
    if let Some(slot) = time_slot {
        let label = card_time_label(comic, slot.index);
        set_text(&mut text, label.as_deref().unwrap_or_default());
        set_display(&mut node, label.is_some());
    }
}

/// 比较后写显隐（避免无谓的布局标脏）
fn set_display(node: &mut Node, visible: bool) {
    let target = row_display(visible);
    if node.display != target {
        node.display = target;
    }
}

/// 比较后写文本
fn set_text(text: &mut Text, value: &str) {
    if text.as_str() != value {
        **text = value.to_string();
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

#[cfg(test)]
mod tests {
    use super::plan_recycle;

    /// 向下滚一行（4 列）：顶部一行让位给底部一行，实体总数不变
    #[test]
    fn recycle_scroll_down_one_row() {
        let (keep, spare) = plan_recycle(0..12, 4..16);
        // 新窗口前 8 个槽位沿用旧的第 4..12 个
        assert_eq!(
            keep,
            vec![
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(10),
                Some(11),
                None,
                None,
                None,
                None,
            ]
        );
        // 移出的顶部一行拿去改绑
        assert_eq!(spare, vec![0, 1, 2, 3]);
    }

    /// 向上滚一行：底部一行让位给顶部一行
    #[test]
    fn recycle_scroll_up_one_row() {
        let (keep, spare) = plan_recycle(4..16, 0..12);
        assert_eq!(keep[..4], [None, None, None, None]);
        assert_eq!(
            keep[4..],
            [
                Some(0),
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7)
            ]
        );
        assert_eq!(spare, vec![8, 9, 10, 11]);
    }

    /// 完全不重叠（拖滚动条跳跃）：旧卡片全部转为空闲，全靠改绑
    #[test]
    fn recycle_disjoint_windows() {
        let (keep, spare) = plan_recycle(0..8, 40..48);
        assert!(keep.iter().all(Option::is_none));
        assert_eq!(spare, (0..8).collect::<Vec<_>>());
    }

    /// 窗口变小（缩窗口/列数变化）：多出来的旧卡片进空闲池，由调用方销毁
    #[test]
    fn recycle_window_shrinks() {
        let (keep, spare) = plan_recycle(0..12, 0..6);
        assert_eq!(keep.len(), 6);
        assert!(keep.iter().all(Option::is_some));
        assert_eq!(spare, vec![6, 7, 8, 9, 10, 11]);
    }

    /// 窗口变大：沿用全部旧卡片，新增槽位留空待新建
    #[test]
    fn recycle_window_grows() {
        let (keep, spare) = plan_recycle(0..6, 0..12);
        assert_eq!(keep.len(), 12);
        assert!(keep[..6].iter().all(Option::is_some));
        assert!(keep[6..].iter().all(Option::is_none));
        assert!(spare.is_empty());
    }

    /// 窗口不动：全部沿用，零空闲
    #[test]
    fn recycle_no_movement() {
        let (keep, spare) = plan_recycle(8..20, 8..20);
        assert_eq!(keep, (0..12).map(Some).collect::<Vec<_>>());
        assert!(spare.is_empty());
    }
}
