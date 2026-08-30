//! 阅读器系统
//!
//! 核心设计：
//! - 单页模式：三实体预加载池（SlotPrev / SlotCurrent /
//!   SlotNext），翻页时旋转，零延迟切换
//! - 条漫模式：窗口化加载，只维护当前可见区域 ± PRELOAD_RANGE
//!   张图片，远离窗口的释放纹理
//! - 共享：图片加载本地优先 → ImageCache → 网络请求，章节末尾自动切换下一章

use std::path::PathBuf;

use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::{
    events::*,
    resources::{ComicDetailState, ImageCache, ReadMode, ReaderState},
    systems::{
        downloads::get_download_base_path, login::AppColors, ui_common::LoadingShimmer,
        widgets::ButtonStyle,
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 阅读器根节点
#[derive(Component, Default, Clone)]
pub struct ReaderRoot;

/// 阅读器图片容器（单页模式的三 slot 或条漫滚动容器的父级）
#[derive(Component, Default, Clone)]
pub struct ReaderImageContainer;

/// 加载指示器
#[derive(Component, Default, Clone)]
pub struct ReaderLoadingIndicator;

/// 错误提示
#[derive(Component, Default, Clone)]
pub struct ReaderErrorText;

/// 图片加载中指示器（通用，等待 ImageCache 回调）
#[derive(Component)]
pub struct ReaderImageLoading {
    pub url: String,
}

// ---------- 工具栏 ----------

/// 阅读器顶部工具栏
#[derive(Component, Default, Clone)]
pub struct ReaderToolbar;

/// 阅读器底部信息栏
#[derive(Component, Default, Clone)]
pub struct ReaderBottomBar;

/// 返回按钮
#[derive(Component, Default, Clone)]
pub struct ReaderBackButton;

/// 上一页按钮
#[derive(Component, Default, Clone)]
pub struct ReaderPrevButton;

/// 下一页按钮
#[derive(Component, Default, Clone)]
pub struct ReaderNextButton;

/// 页码显示文本
#[derive(Component, Default, Clone)]
pub struct ReaderPageText;

/// 章节标题文本
#[derive(Component, Default, Clone)]
pub struct ReaderEpisodeText;

/// 模式切换按钮
#[derive(Component, Default, Clone)]
pub struct ReaderModeButton;

/// 缩放显示文本
#[derive(Component, Default, Clone)]
pub struct ReaderScaleText;

/// 缩放按钮动作
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ZoomAction {
    /// 放大一档
    #[default]
    In,
    /// 缩小一档
    Out,
    /// 复位到 100%
    Reset,
}

/// 工具栏缩放按钮
#[derive(Component, Default, Clone)]
pub struct ReaderZoomButton {
    pub action: ZoomAction,
}

// ---------- 单页模式三 slot ----------

/// 单页模式的图片槽位类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotType {
    /// 前一页预加载（Display::None）
    Prev,
    /// 当前显示
    Current,
    /// 后一页预加载（Display::None）
    Next,
}

/// 单页模式的图片槽位
#[derive(Component)]
pub struct ImageSlot {
    pub slot_type: SlotType,
    /// 该 slot 对应的页码（0-indexed），None 表示空 slot
    pub page_index: Option<usize>,
}

// ---------- 条漫模式 ----------

/// Webtoon 模式滚动容器
#[derive(Component, Default, Clone)]
pub struct WebtoonScrollContainer;

/// 条漫模式槽位（每页一个实体，图片按需懒加载）
#[derive(Component, Default, Clone)]
pub struct WebtoonSlot {
    /// 绑定的全局页码（None = 空槽）
    pub page_index: Option<usize>,
}

// ==================== 常量 ====================

mod consts {
    use bevy::prelude::Color;

    pub const TOOLBAR_HEIGHT: f32 = 50.0;
    pub const BOTTOM_BAR_HEIGHT: f32 = 40.0;
    /// 最小缩放比例
    pub const MIN_SCALE: f32 = 0.5;
    /// 最大缩放比例
    pub const MAX_SCALE: f32 = 3.0;
    /// 缩放步长
    pub const SCALE_STEP: f32 = 0.1;
    /// 条漫模式滚动速度（每行滚动像素）
    pub const WEBTOON_SCROLL_SPEED: f32 = 60.0;
    /// 条漫模式图片宽度百分比
    pub const WEBTOON_IMAGE_WIDTH_PERCENT: f32 = 80.0;
    /// 条漫模式预加载范围（可见区域上下各预加载多少张）
    pub const WEBTOON_PRELOAD_RANGE: usize = 3;
    /// 条漫模式占位高度（未加载图片的默认高度）
    pub const WEBTOON_PLACEHOLDER_HEIGHT: f32 = 1000.0;
    /// 条漫占位槽底色（骨架屏微光的静息色）
    pub const SLOT_PLACEHOLDER: Color = Color::srgb(0.05, 0.05, 0.08);
    /// 条漫模式图片间距
    pub const WEBTOON_GAP: f32 = 8.0;
}

// ==================== 本地文件加载 ====================

/// 清理文件名中的非法字符（与 api_plugin.rs 中的 sanitize_filename 保持一致）
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// 尝试获取本地图片路径
///
/// 返回 Some(path) 如果本地文件存在，否则返回 None
fn try_get_local_image_path(
    comic_title: &str,
    episode_order: i32,
    original_name: &str,
    page_index: usize,
) -> Option<PathBuf> {
    let base_path = get_download_base_path();
    let sanitized_title = sanitize_filename(comic_title);
    let ep_folder = base_path
        .join(&sanitized_title)
        .join(format!("第{}章", episode_order));

    // 尝试原始文件名
    if !original_name.is_empty() {
        let path = ep_folder.join(original_name);
        if path.exists() {
            tracing::debug!("找到本地文件: {}", path.display());
            return Some(path);
        }
    }

    // 尝试序号文件名 (如: 0001.jpg)
    let numbered_name = format!("{:04}.jpg", page_index + 1);
    let path = ep_folder.join(&numbered_name);
    if path.exists() {
        tracing::debug!("找到本地文件: {}", path.display());
        return Some(path);
    }

    None
}

// ==================== 图片样式 ====================

/// 单页模式图片节点样式（根据缩放比例）
fn single_page_image_style(scale: f32) -> Node {
    let scale_percent = scale * 100.0;
    Node {
        width: Val::Auto,
        height: Val::Percent(scale_percent),
        max_width: Val::Percent(scale_percent),
        ..default()
    }
}

/// 工具栏缩放按钮场景
fn zoom_button(action: ZoomAction, icon: &'static str) -> impl Scene + use<> {
    let marker = ReaderZoomButton { action };
    bsn! {
        template_value(marker)
        Button
        template_value(ButtonStyle::ghost())
        Node {
            width: Val::Px(26.0),
            height: Val::Px(26.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::NONE)
        Children [
            (
                Text(icon)
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::WHITE)
            )
        ]
    }
}

/// 应用新的缩放比例并同步工具栏百分比文字
///
/// 滚轮、键盘、按钮三条入口共用，避免各写一份钳位与文本刷新后走样。
/// 返回是否真的改变了（用于跳过无谓的日志/刷新）。
fn apply_scale(
    reader_state: &mut ReaderState,
    new_scale: f32,
    scale_text_query: &mut Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
) -> bool {
    let clamped = new_scale.clamp(consts::MIN_SCALE, consts::MAX_SCALE);
    if (clamped - reader_state.scale).abs() < 0.001 {
        return false;
    }
    reader_state.scale = clamped;
    let label = format!("{}%", (clamped * 100.0) as i32);
    for mut text in scale_text_query.iter_mut() {
        **text = label.clone();
    }
    true
}

// ==================== 条漫滚动：锚点 ↔ 像素 ====================
//
// 条漫的滚动位置以**锚点**（第几页 + 页内偏移）为唯一真相，每帧换算成
// `ScrollPosition` 写下去。图片真实高度陆续就位时，锚定页**上方**的高度变化
// 会被换算自然吸收——锚定页在屏幕上的位置纹丝不动，且与用户是否正在滚动无关。
//
// 反过来（把 ScrollPosition 当真相、事后纠正）就是旧实现，它的补偿只在
// 「非用户滚动帧」执行，用户一路拖到底时每帧都是用户滚动帧，补偿全被跳过 →
// 新图一加载就错位。这是本次重设计要根除的东西。
//
// 下面四个函数是纯函数，配有单测——滚动错位靠肉眼复现代价太高。

/// 某页顶边的累计偏移（含页间距）
fn page_top(heights: &[f32], page: usize) -> f32 {
    heights
        .iter()
        .take(page.min(heights.len()))
        .map(|h| h + consts::WEBTOON_GAP)
        .sum()
}

/// 内容总高度（末页不计尾部间距）
fn content_height(heights: &[f32]) -> f32 {
    match heights.len() {
        0 => 0.0,
        n => page_top(heights, n) - consts::WEBTOON_GAP,
    }
}

/// 锚点 → 滚动像素
fn anchor_to_scroll(heights: &[f32], anchor: (usize, f32)) -> f32 {
    (page_top(heights, anchor.0) + anchor.1).max(0.0)
}

/// 滚动像素 → 锚点
///
/// 取「视口顶边落在哪一页」：从头累加直到跨过 `scroll`。页间距算进前一页的
/// 尾巴，落在间距里时归到前一页的末尾，避免锚点在间距处来回抖。
fn scroll_to_anchor(heights: &[f32], scroll: f32) -> (usize, f32) {
    let scroll = scroll.max(0.0);
    let mut cumulative = 0.0_f32;
    for (page, height) in heights.iter().enumerate() {
        let next = cumulative + height + consts::WEBTOON_GAP;
        if scroll < next {
            return (page, scroll - cumulative);
        }
        cumulative = next;
    }
    // 超出末页：钉在最后一页尾部
    match heights.len() {
        0 => (0, 0.0),
        n => (n - 1, scroll - page_top(heights, n - 1)),
    }
}

/// 条漫模式图片宽度百分比（基准 80% × 缩放比例）
fn webtoon_width_percent(scale: f32) -> f32 {
    consts::WEBTOON_IMAGE_WIDTH_PERCENT * scale
}

/// 条漫模式图片节点样式（宽度随缩放，高度自动保持比例，底部间距）
fn webtoon_image_style(scale: f32) -> Node {
    Node {
        width: Val::Percent(webtoon_width_percent(scale)),
        height: Val::Auto,
        margin: UiRect::bottom(Val::Px(consts::WEBTOON_GAP)),
        ..default()
    }
}

// ==================== 图片加载辅助 ====================

/// 加载单张图片到指定实体的子节点（本地优先 → 缓存 → 网络）
///
/// 返回 true 表示图片已立即可用（本地或缓存命中），false 表示需要异步加载
fn load_image_for_slot(
    commands: &mut Commands,
    parent_entity: Entity,
    url: &str,
    original_name: &str,
    comic_title: &str,
    episode_order: i32,
    page_index: usize,
    node_style: Node,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) -> bool {
    // 1. 检查本地文件
    if let Some(local_path) =
        try_get_local_image_path(comic_title, episode_order, original_name, page_index)
    {
        let handle: Handle<Image> = asset_server.load(local_path);
        commands.entity(parent_entity).with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: handle,
                    ..default()
                },
                node_style,
            ));
        });
        return true;
    }

    // 2. 检查内存缓存
    if let Some(handle) = image_cache.get(url) {
        commands.entity(parent_entity).with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: handle.clone(),
                    ..default()
                },
                node_style,
            ));
        });
        return true;
    }

    // 3. 网络请求 + 加载中占位
    load_image_messages.write(LoadImageRequest {
        url: url.to_string(),
    });
    commands.entity(parent_entity).with_children(|parent| {
        parent.spawn((
            ReaderImageLoading {
                url: url.to_string(),
            },
            Node {
                width: Val::Percent(50.0),
                height: Val::Px(400.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
        ));
    });
    false
}

// ==================== Setup / Cleanup ====================

/// 创建阅读器 UI
pub fn setup_reader_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    reader_state: Res<ReaderState>,
) {
    commands.spawn_scene(reader_page(&reader_state));

    tracing::info!(
        "阅读器 UI 初始化: comic_id={}, episode={}",
        reader_state.comic_id,
        reader_state.episode_order
    );
}

/// 阅读器页面场景（根节点 - 全屏黑色背景）
fn reader_page(reader_state: &ReaderState) -> impl Scene + use<> {
    bsn! {
        ReaderRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
        }
        BackgroundColor(Color::BLACK)
        Children [
            // 图片显示区域（底层，全屏）
            reader_image_area(),
            // 顶部工具栏（浮动层）
            reader_toolbar(reader_state),
            // 底部信息栏（浮动层）
            reader_bottom_bar(reader_state),
        ]
    }
}

/// 顶部工具栏场景
fn reader_toolbar(reader_state: &ReaderState) -> impl Scene + use<> {
    // 中间：章节标题
    let episode_title = reader_state
        .episodes
        .get(reader_state.current_episode_idx)
        .map(|ep| ep.title.as_str())
        .unwrap_or("未知章节");
    let episode_label = format!("第 {} 章 - {}", reader_state.episode_order, episode_title);
    // 右侧：缩放显示
    let scale_label = format!("{}%", (reader_state.scale * 100.0) as i32);
    // 模式切换按钮文本
    let mode_label = match reader_state.read_mode {
        ReadMode::SinglePage => "单页",
        ReadMode::Webtoon => "条漫",
    };

    bsn! {
        ReaderToolbar
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            height: Val::Px(consts::TOOLBAR_HEIGHT),
            padding: UiRect::horizontal(Val::Px(15.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8))
        ZIndex(10)
        Children [
            (
                // 左侧：返回按钮
                ReaderBackButton
                Button
                template_value(ButtonStyle::ghost())
                Node { padding: UiRect::all(Val::Px(8.0)) }
                BackgroundColor(Color::NONE)
                Children [
                    (
                        Text(ICON_CHEVRON_LEFT)
                        TextFont { font_size: FontSize::Px(24.0) }
                        TextColor(Color::WHITE)
                    )
                ]
            ),
            (
                // 中间：章节标题
                ReaderEpisodeText
                Text({episode_label})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(Color::WHITE)
            ),
            (
                // 右侧：缩放显示 + 模式切换
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                }
                Children [
                    (
                        // 缩放控件组：− / 百分比 / + / 复位
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                        }
                        Children [
                            zoom_button(ZoomAction::Out, ICON_MINUS),
                            (
                                // 缩放显示（键盘 +/-/0 与 Ctrl+滚轮也会改它）
                                ReaderScaleText
                                Text({scale_label})
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(Color::srgb(0.8, 0.8, 0.8))
                                Node {
                                    min_width: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                }
                            ),
                            zoom_button(ZoomAction::In, ICON_PLUS),
                            zoom_button(ZoomAction::Reset, ICON_REFRESH),
                        ]
                    ),
                    (
                        // 模式切换按钮（芯片状，保持有底色 → secondary）
                        ReaderModeButton
                        Button
                        template_value(ButtonStyle::secondary())
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SECONDARY)
                        template_value(BorderColor::all(Color::srgb(0.5, 0.5, 0.5)))
                        Children [
                            (
                                Text({mode_label})
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

/// 图片显示区域场景
fn reader_image_area() -> impl Scene {
    bsn! {
        ReaderImageContainer
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
        }
        BackgroundColor(Color::BLACK)
        Children [
            (
                // 加载指示器（默认显示）
                ReaderLoadingIndicator
                Text("加载中...")
                TextFont { font_size: FontSize::Px(18.0) }
                TextColor(Color::srgb(0.7, 0.7, 0.7))
            )
        ]
    }
}

/// 底部信息栏场景
fn reader_bottom_bar(reader_state: &ReaderState) -> impl Scene + use<> {
    let prev_label = format!("{ICON_CHEVRON_LEFT} 上一页");
    let next_label = format!("下一页 {ICON_CHEVRON_RIGHT}");
    // 页码显示（1-indexed 展示）
    let display_page = reader_state.current_page + 1;
    let page_label = format!("{} / {}", display_page, reader_state.total_pages);

    bsn! {
        ReaderBottomBar
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            height: Val::Px(consts::BOTTOM_BAR_HEIGHT),
            padding: UiRect::horizontal(Val::Px(15.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8))
        ZIndex(10)
        Children [
            (
                // 上一页按钮
                ReaderPrevButton
                Button
                template_value(ButtonStyle::ghost())
                Node { padding: UiRect::all(Val::Px(8.0)) }
                BackgroundColor(Color::NONE)
                Children [
                    (
                        Text({prev_label})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(Color::WHITE)
                    )
                ]
            ),
            (
                // 页码显示
                ReaderPageText
                Text({page_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::WHITE)
            ),
            (
                // 下一页按钮
                ReaderNextButton
                Button
                template_value(ButtonStyle::ghost())
                Node { padding: UiRect::all(Val::Px(8.0)) }
                BackgroundColor(Color::NONE)
                Children [
                    (
                        Text({next_label})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(Color::WHITE)
                    )
                ]
            ),
        ]
    }
}

/// 清理阅读器 UI
pub fn cleanup_reader_ui(mut commands: Commands, query: Query<Entity, With<ReaderRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ==================== 图片加载 ====================

/// 触发加载图片（进入阅读器时自动执行）
///
/// - 条漫模式 Phase 1：先加载当前章节（秒级），立即显示
/// - 条漫模式 Phase 2：由 handle_pictures_loaded 触发后台加载全章节
/// - 单页模式：发送 LoadPicturesRequest，只加载当前章节
pub fn trigger_load_pictures(
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
) {
    if reader_state.comic_id.is_empty()
        || !reader_state.pictures.is_empty()
        || reader_state.is_loading
        || reader_state.is_loading_all_chapters
    {
        return;
    }

    // 条漫和单页模式都先加载当前章节
    tracing::info!(
        "触发加载第 {} 章图片（模式={:?}）",
        reader_state.episode_order,
        reader_state.read_mode,
    );
    reader_state.is_loading = true;
    load_messages.write(LoadPicturesRequest {
        comic_id: reader_state.comic_id.clone(),
        episode_order: reader_state.episode_order,
        page: 1,
    });
}

/// 处理单章节图片加载完成
///
/// - 单页模式：直接创建视图
/// - 条漫模式 Phase 1：用当前章节创建视图，然后触发 Phase 2（后台加载全章节）
pub fn handle_pictures_loaded(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    mut pictures_events: MessageReader<PicturesLoadedEvent>,
    loading_query: Query<Entity, With<ReaderLoadingIndicator>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    asset_server: Res<AssetServer>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    image_cache: Res<ImageCache>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    mut load_all_messages: MessageWriter<LoadAllChapterPicturesRequest>,
) {
    for event in pictures_events.read() {
        // 单页模式的下一章预加载
        if reader_state.is_loading_next_chapter {
            reader_state.is_loading_next_chapter = false;
            reader_state.next_chapter_pictures = event.pictures.clone();
            tracing::info!(
                "下一章图片预加载完成: {} 张",
                reader_state.next_chapter_pictures.len()
            );
            continue;
        }

        // 全章节加载结果走 handle_all_pictures_loaded
        if reader_state.is_loading_all_chapters {
            continue;
        }

        reader_state.is_loading = false;

        // 构建当前章节的 page_metas
        let ep_order = reader_state.episode_order;
        let metas: Vec<_> = (0..event.pictures.len())
            .map(|i| crate::events::WebtoonPageMeta {
                episode_order: ep_order,
                page_in_chapter: i,
            })
            .collect();

        reader_state.pictures = event.pictures.clone();
        reader_state.page_metas = metas;
        reader_state.total_pages = reader_state.pictures.len();
        reader_state.current_page = 0;

        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }
        update_page_text(&reader_state, &mut page_text_query);

        let Ok(container) = container_query.single() else {
            continue;
        };

        match reader_state.read_mode {
            ReadMode::SinglePage => {
                create_single_page_slots(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
                );
            }
            ReadMode::Webtoon => {
                tracing::info!(
                    "条漫 Phase 1：当前章节 {} 张图片已就绪，创建视图",
                    reader_state.total_pages
                );
                reader_state.webtoon_anchor = (reader_state.current_page, 0.0);
                // 页高表随视图一起重置：换漫画时总页数可能恰好相同，
                // 只靠 sync 里的长度判断会把上一本的高度留下来
                reader_state.webtoon_page_heights =
                    vec![consts::WEBTOON_PLACEHOLDER_HEIGHT; reader_state.total_pages];
                create_webtoon_view(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
                );

                // Phase 2：后台加载全章节（DB 缓存优先）
                if reader_state.episodes.len() > 1 {
                    tracing::info!(
                        "条漫 Phase 2：后台加载全部 {} 个章节图片列表",
                        reader_state.episodes.len()
                    );
                    reader_state.is_loading_all_chapters = true;
                    load_all_messages.write(LoadAllChapterPicturesRequest {
                        comic_id: reader_state.comic_id.clone(),
                        episodes: reader_state.episodes.clone(),
                    });
                }
            }
        }
    }
}

/// 处理全章节图片列表加载完成（条漫模式 Phase 2）
///
/// 销毁当前视图，用完整列表重建，保持在对应的全局页码位置
pub fn handle_all_pictures_loaded(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    mut events: MessageReader<AllChapterPicturesLoadedEvent>,
    loading_query: Query<Entity, With<ReaderLoadingIndicator>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    webtoon_container_query: Query<Entity, With<WebtoonScrollContainer>>,
    asset_server: Res<AssetServer>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    image_cache: Res<ImageCache>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in events.read() {
        reader_state.is_loading_all_chapters = false;

        // 计算用户选择章节的起始页码 + 当前在章节内的偏移
        let target_order = reader_state.episode_order;
        let chapter_start = event
            .page_metas
            .iter()
            .position(|m| m.episode_order == target_order)
            .unwrap_or(0);
        // 保持当前阅读偏移（Phase 1 时用户可能已经翻了几页）
        let global_page = chapter_start + reader_state.current_page;

        reader_state.pictures = event.pictures.clone();
        reader_state.page_metas = event.page_metas.clone();
        reader_state.total_pages = reader_state.pictures.len();
        reader_state.current_page = global_page.min(reader_state.total_pages.saturating_sub(1));

        tracing::info!(
            "条漫 Phase 2 完成：共 {} 张，当前全局页={}（第 {} 章偏移 {}）",
            reader_state.total_pages,
            reader_state.current_page,
            target_order,
            reader_state.current_page - chapter_start,
        );

        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }
        // 销毁旧的条漫视图
        for entity in webtoon_container_query.iter() {
            commands.entity(entity).despawn();
        }

        update_page_text(&reader_state, &mut page_text_query);

        let Ok(container) = container_query.single() else {
            continue;
        };

        // 用完整列表重建条漫视图
        reader_state.webtoon_anchor = (reader_state.current_page, 0.0);
        // 页高表随视图一起重置：换漫画时总页数可能恰好相同，
        // 只靠 sync 里的长度判断会把上一本的高度留下来
        reader_state.webtoon_page_heights =
            vec![consts::WEBTOON_PLACEHOLDER_HEIGHT; reader_state.total_pages];
        create_webtoon_view(
            &mut commands,
            container,
            &reader_state,
            &image_cache,
            &asset_server,
            &mut load_image_messages,
        );
    }
}

/// 处理图片加载失败
pub fn handle_pictures_load_failed(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    mut error_events: MessageReader<PicturesLoadFailedEvent>,
    loading_query: Query<Entity, With<ReaderLoadingIndicator>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
) {
    for event in error_events.read() {
        tracing::error!("图片加载失败: {}", event.error);

        reader_state.is_loading = false;
        reader_state.is_loading_next_chapter = false;
        reader_state.error = Some(event.error.clone());

        // 移除加载指示器
        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }

        // 显示错误信息
        if let Ok(container) = container_query.single() {
            let error_label = format!("加载失败: {}", event.error);
            commands
                .spawn_scene(bsn! {
                    ReaderErrorText
                    Text({error_label})
                    TextFont { font_size: FontSize::Px(16.0) }
                    TextColor(AppColors::ERROR)
                })
                .insert(ChildOf(container));
        }
    }
}

// ==================== 单页模式：三 slot 预加载池 ====================

/// 创建三个图片 slot（prev / current / next）
fn create_single_page_slots(
    commands: &mut Commands,
    container: Entity,
    reader_state: &ReaderState,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) {
    if reader_state.pictures.is_empty() {
        return;
    }

    let current_page = reader_state.current_page;
    let total = reader_state.total_pages;

    // Prev slot（隐藏）
    let prev_page = if current_page > 0 {
        Some(current_page - 1)
    } else {
        None
    };
    spawn_image_slot(
        commands,
        container,
        SlotType::Prev,
        prev_page,
        reader_state,
        image_cache,
        asset_server,
        load_image_messages,
        true, // hidden
    );

    // Current slot（可见）
    spawn_image_slot(
        commands,
        container,
        SlotType::Current,
        Some(current_page),
        reader_state,
        image_cache,
        asset_server,
        load_image_messages,
        false,
    );

    // Next slot（隐藏）
    let next_page = if current_page + 1 < total {
        Some(current_page + 1)
    } else {
        None
    };
    spawn_image_slot(
        commands,
        container,
        SlotType::Next,
        next_page,
        reader_state,
        image_cache,
        asset_server,
        load_image_messages,
        true, // hidden
    );

    tracing::info!(
        "单页模式：创建三 slot, prev={:?}, current={}, next={:?}",
        prev_page,
        current_page,
        next_page
    );
}

/// 创建单个图片 slot 实体
fn spawn_image_slot(
    commands: &mut Commands,
    container: Entity,
    slot_type: SlotType,
    page_index: Option<usize>,
    reader_state: &ReaderState,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
    hidden: bool,
) {
    let display = if hidden { Display::None } else { Display::Flex };

    // 先 spawn slot 实体并设为 container 的子节点
    let slot_entity = commands
        .spawn((
            ImageSlot {
                slot_type,
                page_index,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display,
                ..default()
            },
        ))
        .id();
    commands.entity(container).add_child(slot_entity);

    // 如果有图片数据，加载图片到 slot 子节点
    if let Some(idx) = page_index
        && let Some(picture) = reader_state.pictures.get(idx)
    {
        let url = picture.media.url();
        load_image_for_slot(
            commands,
            slot_entity,
            &url,
            &picture.media.original_name,
            &reader_state.comic_title,
            reader_state.episode_order,
            idx,
            single_page_image_style(reader_state.scale),
            image_cache,
            asset_server,
            load_image_messages,
        );
    }
}

/// 单页模式翻页：旋转三个 slot
///
/// `direction`: 正数 = 下一页, 负数 = 上一页
fn rotate_slots(
    commands: &mut Commands,
    reader_state: &ReaderState,
    slot_query: &Query<(Entity, &ImageSlot, &Children)>,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) {
    let current_page = reader_state.current_page;
    let total = reader_state.total_pages;

    // 收集现有 slot 信息
    let mut slots: Vec<(Entity, SlotType, Option<usize>)> = slot_query
        .iter()
        .map(|(e, slot, _)| (e, slot.slot_type, slot.page_index))
        .collect();

    // 按类型排序确保顺序一致
    slots.sort_by_key(|(_, t, _)| match t {
        SlotType::Prev => 0,
        SlotType::Current => 1,
        SlotType::Next => 2,
    });

    // 更新每个 slot 的角色和内容
    for (entity, old_type, old_page) in &slots {
        let (new_type, new_page) = match old_type {
            SlotType::Prev => {
                let prev = if current_page > 0 {
                    Some(current_page - 1)
                } else {
                    None
                };
                (SlotType::Prev, prev)
            }
            SlotType::Current => (SlotType::Current, Some(current_page)),
            SlotType::Next => {
                let next = if current_page + 1 < total {
                    Some(current_page + 1)
                } else {
                    None
                };
                (SlotType::Next, next)
            }
        };

        // 更新 slot 组件
        let is_hidden = new_type != SlotType::Current;
        commands.entity(*entity).insert((
            ImageSlot {
                slot_type: new_type,
                page_index: new_page,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: if is_hidden {
                    Display::None
                } else {
                    Display::Flex
                },
                ..default()
            },
        ));

        // 如果页码没变，不需要重新加载图片
        if new_page == *old_page {
            continue;
        }

        // 清空旧子节点
        // 获取 slot 的 children，通过 query 已有
        if let Ok((_, _, children)) = slot_query.get(*entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 加载新图片
        if let Some(idx) = new_page
            && let Some(picture) = reader_state.pictures.get(idx)
        {
            let url = picture.media.url();
            load_image_for_slot(
                commands,
                *entity,
                &url,
                &picture.media.original_name,
                &reader_state.comic_title,
                reader_state.episode_order,
                idx,
                single_page_image_style(reader_state.scale),
                image_cache,
                asset_server,
                load_image_messages,
            );
        }
    }
}

/// 更新单页模式三 slot（ReaderState 变化时调用）
pub fn update_single_page_slots(
    mut commands: Commands,
    reader_state: Res<ReaderState>,
    slot_query: Query<(Entity, &ImageSlot, &Children)>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
) {
    if !reader_state.is_changed()
        || reader_state.read_mode != ReadMode::SinglePage
        || reader_state.pictures.is_empty()
    {
        return;
    }

    // 如果没有 slot 存在（首次或模式切换后），不在此处创建
    if slot_query.is_empty() {
        return;
    }

    rotate_slots(
        &mut commands,
        &reader_state,
        &slot_query,
        &image_cache,
        &asset_server,
        &mut load_image_messages,
    );
}

// ==================== 条漫模式：窗口化加载 ====================

/// 创建条漫视图（为每页创建一个槽位，图片按需懒加载）
fn create_webtoon_view(
    commands: &mut Commands,
    container: Entity,
    reader_state: &ReaderState,
    _image_cache: &ImageCache,
    _asset_server: &AssetServer,
    _load_image_messages: &mut MessageWriter<LoadImageRequest>,
) {
    let total = reader_state.total_pages;
    if total == 0 {
        return;
    }

    let current = reader_state.current_page;

    // 初始位置只需设锚点，像素值由 sync_webtoon_scroll 每帧算出来。
    // 旧实现在这里按占位高度硬算一个 scroll_y，图片一加载高度就对不上，
    // 得靠补偿慢慢"校正到位"——现在不存在这个过程。
    let initial_scroll_y =
        current as f32 * (consts::WEBTOON_PLACEHOLDER_HEIGHT + consts::WEBTOON_GAP);

    commands
        .spawn_scene(webtoon_view(total, initial_scroll_y, reader_state.scale))
        .insert(ChildOf(container));

    // 图片加载由 update_webtoon_window 每帧自动处理（±3 范围）

    tracing::info!(
        "条漫模式：创建 {} 个槽位，从第 {} 页开始（scroll_y={}）",
        total,
        current + 1,
        initial_scroll_y
    );
}

/// 条漫视图场景（滚动容器 + 每页一个占位槽位）
fn webtoon_view(total: usize, initial_scroll_y: f32, scale: f32) -> impl Scene + use<> {
    // 上下留出工具栏高度 + 10px 的余量
    let scroll_padding = UiRect::vertical(Val::Px(consts::TOOLBAR_HEIGHT + 10.0));
    let initial_scroll = Vec2::new(0.0, initial_scroll_y);
    let slots: Vec<_> = (0..total).map(|page| webtoon_slot(page, scale)).collect();

    bsn! {
        WebtoonScrollContainer
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            // 双向滚动：缩放 > 125% 时图片宽于视口，需要横向平移（Shift+滚轮）
            overflow: Overflow::scroll(),
            padding: {scroll_padding},
        }
        BackgroundColor(Color::BLACK)
        // 不加 ScrollArea：滚轮由 reader_mouse_wheel_control 模态处理
        //（单页=翻页 / 条漫=滚动），避免上游派发与业务逻辑重复滚动
        ScrollPosition({initial_scroll})
        Children [ {slots} ]
    }
}

/// 条漫单页槽位场景（占位背景，图片按需懒加载）
fn webtoon_slot(page: usize, scale: f32) -> impl Scene + use<> {
    let width = Val::Percent(webtoon_width_percent(scale));
    bsn! {
        WebtoonSlot { page_index: {Some(page)} }
        template_value(LoadingShimmer::new(consts::SLOT_PLACEHOLDER))
        Node {
            width: {width},
            height: Val::Px(consts::WEBTOON_PLACEHOLDER_HEIGHT),
            margin: UiRect::bottom(Val::Px(consts::WEBTOON_GAP)),
        }
        BackgroundColor(consts::SLOT_PLACEHOLDER)
    }
}

/// 条漫滚动同步：实测页高 → 由锚点算出 `ScrollPosition`
///
/// 每帧无条件执行，**不读 `ScrollPosition` 作输入**——本容器没有挂
/// `ScrollArea`，上游不会写它，所以这里是唯一写者，可以放心地把它当成
/// 「锚点的投影」而非状态。
///
/// 这就是修掉「拉到底加载新图会错位」的关键：锚定页上方的高度一变，
/// 换算结果同步变，视觉位置自然不动；旧实现要在「非用户滚动帧」才补偿，
/// 而一路拖动时每帧都是用户滚动帧，补偿永远轮不到。
pub fn sync_webtoon_scroll(
    mut reader_state: ResMut<ReaderState>,
    slot_query: Query<(&WebtoonSlot, &ComputedNode, Has<ImageNode>)>,
    mut scroll_query: Query<(&mut ScrollPosition, &ComputedNode), With<WebtoonScrollContainer>>,
) {
    if reader_state.read_mode != ReadMode::Webtoon || reader_state.pictures.is_empty() {
        return;
    }
    let Ok((mut scroll_pos, container)) = scroll_query.single_mut() else {
        return;
    };

    // 页高表按总页数对齐（换章/换漫画后 pictures 变了）
    let total = reader_state.total_pages;
    if reader_state.webtoon_page_heights.len() != total {
        reader_state.webtoon_page_heights = vec![consts::WEBTOON_PLACEHOLDER_HEIGHT; total];
    }

    // 实测已加载图片的真实高度，覆盖占位值
    //
    // 只认带 ImageNode 的槽位：没加载的槽位高度就是占位值本身，
    // 回写它没有意义，还会把首帧 ComputedNode 尚为 0 的噪声写进去。
    for (slot, computed, has_image) in slot_query.iter() {
        if !has_image {
            continue;
        }
        let Some(page) = slot.page_index else {
            continue;
        };
        let measured = computed.size().y * computed.inverse_scale_factor;
        if measured <= 1.0 {
            continue;
        }
        if let Some(height) = reader_state.webtoon_page_heights.get_mut(page)
            && (*height - measured).abs() > 0.5
        {
            *height = measured;
        }
    }

    // 上界优先用引擎实测的内容尺寸——padding、间距的口径以引擎为准，
    // 两边各算一套迟早对不上，钳位处就会互相打架。
    //
    // ⚠️ 但**首帧 ComputedNode 还没布局，content_size 是 0**，直接拿它算上界
    // 会得到 max_scroll = 0，把「恢复上次阅读页」的锚点当场钳回第 1 页。
    // 引擎值不可信时退回页高表自算的总高。
    let inv = container.inverse_scale_factor;
    let viewport_h = container.size().y * inv;
    let engine_content_h = container.content_size().y * inv;
    let content_h = if engine_content_h > 1.0 {
        engine_content_h
    } else {
        content_height(&reader_state.webtoon_page_heights)
    };
    let max_scroll = (content_h - viewport_h).max(0.0);

    let raw = anchor_to_scroll(
        &reader_state.webtoon_page_heights,
        reader_state.webtoon_anchor,
    );
    let desired = raw.min(max_scroll);

    // 被上界钳住时把锚点拉回来，免得锚点越飘越远、反向滚动要空转一段才动
    if raw - desired > 0.5 {
        reader_state.webtoon_anchor = scroll_to_anchor(&reader_state.webtoon_page_heights, desired);
    }

    // 比较后写：避免每帧把 ScrollPosition 标脏，拖累变更检测
    if (scroll_pos.y - desired).abs() > 0.5 {
        scroll_pos.y = desired;
    }
}

/// 条漫模式：按锚点懒加载当前页 ±3 范围内的图片
///
/// 滚动位置本身由 `sync_webtoon_scroll` 负责，这里只管"加载哪些图"。
/// 两件事拆开之后，本系统不再碰 `ScrollPosition`，也就不存在
/// 「补偿写入被误判成用户滚动」那类时序问题。
///
/// 所有章节的图片列表已在初始化时全部获取完毕（仅 URL），
/// 此系统只负责按需下载实际图片数据。不做卸载、不做锚点补偿。
pub fn update_webtoon_window(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    slot_query: Query<(
        Entity,
        &WebtoonSlot,
        Option<&ImageNode>,
        Option<&ReaderImageLoading>,
    )>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
) {
    if reader_state.read_mode != ReadMode::Webtoon || reader_state.pictures.is_empty() {
        return;
    }

    // 当前页直接取锚点——锚点就是真相，不必再从滚动像素反推
    let current_page = reader_state.webtoon_anchor.0;
    if reader_state.current_page != current_page {
        reader_state.current_page = current_page;
    }

    // 加载范围：当前页 ± PRELOAD_RANGE
    let load_start = current_page.saturating_sub(consts::WEBTOON_PRELOAD_RANGE);
    let load_end = (current_page + consts::WEBTOON_PRELOAD_RANGE + 1).min(reader_state.total_pages);

    // 只遍历需要加载的槽位
    for (entity, slot, existing_img, loading_marker) in slot_query.iter() {
        let Some(page) = slot.page_index else {
            continue;
        };
        if page < load_start || page >= load_end {
            continue;
        }
        // 已有图片或已在等待远程加载的槽位不再重复探测本地文件
        // （此前每帧对每个待加载槽位做两次 stat 系统调用）
        if existing_img.is_some() || loading_marker.is_some() {
            continue;
        }

        let Some(picture) = reader_state.pictures.get(page) else {
            continue;
        };
        let url = picture.media.url();

        // 终局失败：停掉微光，留一个静止的暗块——一直脉动等于在骗用户"还在加载"
        if image_cache.is_failed(&url) {
            commands.entity(entity).remove::<LoadingShimmer>();
            continue;
        }

        // 用 page_metas 获取正确的章节 order 和章内页码
        let (ep_order, page_in_chapter) = reader_state
            .page_metas
            .get(page)
            .map(|m| (m.episode_order, m.page_in_chapter))
            .unwrap_or((reader_state.episode_order, page));

        if let Some(local_path) = try_get_local_image_path(
            &reader_state.comic_title,
            ep_order,
            &picture.media.original_name,
            page_in_chapter,
        ) {
            let handle: Handle<Image> = asset_server.load(local_path);
            commands.entity(entity).remove::<BackgroundColor>();
            commands.entity(entity).insert((
                ImageNode {
                    image: handle,
                    ..default()
                },
                webtoon_image_style(reader_state.scale),
            ));
        } else if let Some(handle) = image_cache.get(&url) {
            commands.entity(entity).remove::<BackgroundColor>();
            commands.entity(entity).insert((
                ImageNode {
                    image: handle.clone(),
                    ..default()
                },
                webtoon_image_style(reader_state.scale),
            ));
        } else if !image_cache.is_loading(&url) {
            load_image_messages.write(LoadImageRequest { url: url.clone() });
            commands.entity(entity).insert(ReaderImageLoading { url });
        }
    }
}

/// 条漫模式：ImageCache 加载回调，为等待中的槽位填入图片
pub fn update_webtoon_images_from_cache(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    reader_state: Res<ReaderState>,
    loading_query: Query<(Entity, &WebtoonSlot, &ReaderImageLoading)>,
) {
    if reader_state.read_mode != ReadMode::Webtoon || !image_cache.is_changed() {
        return;
    }

    for (entity, _slot, loading) in loading_query.iter() {
        if let Some(handle) = image_cache.get(&loading.url) {
            commands.entity(entity).remove::<ReaderImageLoading>();
            commands.entity(entity).remove::<BackgroundColor>();
            commands.entity(entity).insert((
                ImageNode {
                    image: handle.clone(),
                    ..default()
                },
                webtoon_image_style(reader_state.scale),
            ));
        }
    }
}

/// 条漫模式缩放更新
pub fn update_webtoon_scale(
    reader_state: Res<ReaderState>,
    mut webtoon_images_query: Query<(&mut Node, &WebtoonSlot, Option<&ImageNode>)>,
) {
    if !reader_state.is_changed() || reader_state.read_mode != ReadMode::Webtoon {
        return;
    }

    // 图片按缩放比例改宽度；未加载的槽位也要跟着改，否则放大后
    // 已载入的图和占位槽宽度不一致，滚动条与锚定都会跳
    let width = Val::Percent(webtoon_width_percent(reader_state.scale));
    for (mut node, _slot, img) in webtoon_images_query.iter_mut() {
        if node.width != width {
            node.width = width;
        }
        if img.is_some() && node.height != Val::Auto {
            node.height = Val::Auto;
        }
    }
}

// ==================== 单页模式：缓存回调 ====================

/// 单页模式：检查 ImageCache 更新 slot 中的加载指示器
pub fn update_reader_image_from_cache(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    reader_state: Res<ReaderState>,
    image_loading_query: Query<(Entity, &ReaderImageLoading, &ChildOf)>,
) {
    if reader_state.read_mode != ReadMode::SinglePage {
        return;
    }

    for (entity, loading, child_of) in image_loading_query.iter() {
        if let Some(handle) = image_cache.get(&loading.url) {
            let parent = child_of.parent();

            // 移除加载指示器
            commands.entity(entity).despawn();

            // 添加图片到 slot
            commands.entity(parent).with_children(|p| {
                p.spawn((
                    ImageNode {
                        image: handle.clone(),
                        ..default()
                    },
                    single_page_image_style(reader_state.scale),
                ));
            });
        }
    }
}

// ==================== 翻页与导航 ====================

/// 翻页方向
enum PageDirection {
    Prev,
    Next,
}

/// 执行翻页操作（核心翻页逻辑）
///
/// 处理单页模式和条漫模式的翻页，支持跨章节切换
fn navigate_page(
    reader_state: &mut ResMut<ReaderState>,
    direction: PageDirection,
    load_messages: &mut MessageWriter<LoadPicturesRequest>,
) -> bool {
    if reader_state.pictures.is_empty() || reader_state.is_loading {
        return false;
    }

    match direction {
        PageDirection::Next => {
            if reader_state.current_page + 1 < reader_state.total_pages {
                // 同章节下一页
                reader_state.current_page += 1;
                true
            } else {
                // 章节末尾 → 尝试切换下一章
                try_switch_chapter(reader_state, true, load_messages)
            }
        }
        PageDirection::Prev => {
            if reader_state.current_page > 0 {
                // 同章节上一页
                reader_state.current_page -= 1;
                true
            } else {
                // 章节开头 → 尝试切换上一章
                try_switch_chapter(reader_state, false, load_messages)
            }
        }
    }
}

/// 尝试切换到下一/上一章
///
/// 返回 true 表示成功发起切换，false 表示没有更多章节
fn try_switch_chapter(
    reader_state: &mut ResMut<ReaderState>,
    is_next: bool,
    load_messages: &mut MessageWriter<LoadPicturesRequest>,
) -> bool {
    let target_idx = if is_next {
        if reader_state.current_episode_idx + 1 >= reader_state.episodes.len() {
            tracing::info!("已到最后一章，无法前进");
            return false;
        }
        reader_state.current_episode_idx + 1
    } else {
        if reader_state.current_episode_idx == 0 {
            tracing::info!("已到第一章，无法后退");
            return false;
        }
        reader_state.current_episode_idx - 1
    };

    let episode = reader_state.episodes[target_idx].clone();
    tracing::info!(
        "切换到{}: 第 {} 章 ({})",
        if is_next { "下一章" } else { "上一章" },
        episode.order,
        episode.title
    );

    // 更新状态
    reader_state.current_episode_idx = target_idx;
    reader_state.episode_order = episode.order;
    reader_state.pictures.clear();
    reader_state.page_metas.clear();
    reader_state.total_pages = 0;
    reader_state.current_page = 0;
    reader_state.is_loading = true;
    reader_state.error = None;
    reader_state.next_chapter_pictures.clear();

    // 如果是上一章，从最后一页开始（标记，在 pictures 加载完后处理）
    // 这里先设置 current_page = usize::MAX 作为标记
    if !is_next {
        reader_state.current_page = usize::MAX; // 标记：加载完后跳到最后一页
    }

    // 发起图片加载
    load_messages.write(LoadPicturesRequest {
        comic_id: reader_state.comic_id.clone(),
        episode_order: episode.order,
        page: 1,
    });

    true
}

// ==================== 交互处理 ====================

/// 返回按钮交互
pub fn reader_back_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderBackButton>)>,
    mut back_events: MessageWriter<NavigateBackEvent>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            back_events.write(NavigateBackEvent);
        }
    }
}

/// 上一页按钮交互
pub fn reader_prev_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderPrevButton>)>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            navigate_page(&mut reader_state, PageDirection::Prev, &mut load_messages);
        }
    }
}

/// 下一页按钮交互
pub fn reader_next_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderNextButton>)>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            navigate_page(&mut reader_state, PageDirection::Next, &mut load_messages);
        }
    }
}

/// 键盘控制
pub fn reader_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
    mut back_events: MessageWriter<NavigateBackEvent>,
) {
    // 左方向键 / A 键 - 上一页
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) || keyboard_input.just_pressed(KeyCode::KeyA)
    {
        navigate_page(&mut reader_state, PageDirection::Prev, &mut load_messages);
    }

    // 右方向键 / D 键 / 空格键 - 下一页
    if keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::KeyD)
        || keyboard_input.just_pressed(KeyCode::Space)
    {
        navigate_page(&mut reader_state, PageDirection::Next, &mut load_messages);
    }

    // Escape 键 - 返回
    if keyboard_input.just_pressed(KeyCode::Escape) {
        back_events.write(NavigateBackEvent);
    }
}

/// 鼠标滚轮控制（单页翻页 / 条漫滚动 / Ctrl+缩放）
pub fn reader_mouse_wheel_control(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
    mut scale_text_query: Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
    mut webtoon_scroll_query: Query<&mut ScrollPosition, With<WebtoonScrollContainer>>,
) {
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        // macOS 上 ⌘ 更顺手，且不会被系统缩放手势抢走
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);
    let shift_pressed =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    for event in mouse_wheel_events.read() {
        let scroll_delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y / 40.0,
        };

        if ctrl_pressed {
            // Ctrl + 滚轮：缩放
            //
            // ⚠️ macOS 会把
            // Ctrl+滚轮吃掉当系统缩放手势，触控板上多半到不了这里，
            // 故工具栏另配了 − / + / ⟲ 按钮和键盘 +/-/0 两条等价入口
            let step = if scroll_delta > 0.0 {
                consts::SCALE_STEP
            } else {
                -consts::SCALE_STEP
            };
            let target = reader_state.scale + step;
            apply_scale(&mut reader_state, target, &mut scale_text_query);
        } else if shift_pressed && reader_state.read_mode == ReadMode::Webtoon {
            // Shift + 滚轮：条漫横向平移（放大到宽于视口时用）
            for mut scroll_pos in webtoon_scroll_query.iter_mut() {
                scroll_pos.x =
                    (scroll_pos.x - scroll_delta * consts::WEBTOON_SCROLL_SPEED).max(0.0);
            }
        } else {
            match reader_state.read_mode {
                ReadMode::SinglePage => {
                    if scroll_delta < 0.0 {
                        navigate_page(&mut reader_state, PageDirection::Next, &mut load_messages);
                    } else if scroll_delta > 0.0 {
                        navigate_page(&mut reader_state, PageDirection::Prev, &mut load_messages);
                    }
                }
                ReadMode::Webtoon => {
                    // 滚轮改的是**锚点**，不是 ScrollPosition：
                    // 先按当前页高把锚点换算成像素，加上滚动量，再换算回锚点。
                    // 换算全在同一帧、用同一份页高完成，所以是自洽的；
                    // 跨帧的高度变化由 sync_webtoon_scroll 吸收。
                    let heights = &reader_state.webtoon_page_heights;
                    if !heights.is_empty() {
                        let scroll_amount = -scroll_delta * consts::WEBTOON_SCROLL_SPEED;
                        let current = anchor_to_scroll(heights, reader_state.webtoon_anchor);
                        let target = (current + scroll_amount).max(0.0);
                        // 上界交给 sync_webtoon_scroll 用引擎实测内容高度钳，
                        // 这里只挡负数，避免两处各算一套上界
                        reader_state.webtoon_anchor = scroll_to_anchor(heights, target);
                    }
                }
            }
        }
    }
}

/// 键盘 +/- 缩放控制
pub fn reader_zoom_keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    mut scale_text_query: Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
) {
    let target = if keyboard_input.just_pressed(KeyCode::Equal)
        || keyboard_input.just_pressed(KeyCode::NumpadAdd)
    {
        Some(reader_state.scale + consts::SCALE_STEP)
    } else if keyboard_input.just_pressed(KeyCode::Minus)
        || keyboard_input.just_pressed(KeyCode::NumpadSubtract)
    {
        Some(reader_state.scale - consts::SCALE_STEP)
    } else if keyboard_input.just_pressed(KeyCode::Digit0)
        || keyboard_input.just_pressed(KeyCode::Numpad0)
    {
        Some(1.0)
    } else {
        None
    };

    if let Some(target) = target {
        apply_scale(&mut reader_state, target, &mut scale_text_query);
    }
}

/// 工具栏缩放按钮交互（− / + / 复位）
pub fn reader_zoom_button_interaction(
    interaction_query: Query<(&Interaction, &ReaderZoomButton), Changed<Interaction>>,
    mut reader_state: ResMut<ReaderState>,
    mut scale_text_query: Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let target = match button.action {
            ZoomAction::In => reader_state.scale + consts::SCALE_STEP,
            ZoomAction::Out => reader_state.scale - consts::SCALE_STEP,
            ZoomAction::Reset => 1.0,
        };
        if apply_scale(&mut reader_state, target, &mut scale_text_query) {
            tracing::debug!("缩放: {}%", (reader_state.scale * 100.0) as i32);
        }
    }
}

/// 模式切换按钮交互
pub fn reader_mode_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderModeButton>)>,
    mut reader_state: ResMut<ReaderState>,
    mode_btn_children_query: Query<&Children, With<ReaderModeButton>>,
    mut all_text_query: Query<&mut Text, Without<ReaderModeButton>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            reader_state.read_mode = match reader_state.read_mode {
                ReadMode::SinglePage => ReadMode::Webtoon,
                ReadMode::Webtoon => ReadMode::SinglePage,
            };

            let new_label = match reader_state.read_mode {
                ReadMode::SinglePage => "单页",
                ReadMode::Webtoon => "条漫",
            };

            for children in mode_btn_children_query.iter() {
                for child in children.iter() {
                    if let Ok(mut text) = all_text_query.get_mut(child) {
                        **text = new_label.to_string();
                    }
                }
            }

            tracing::info!("切换阅读模式: {:?}", reader_state.read_mode);
        }
    }
}

/// 处理阅读模式变化，重建图片视图
pub fn handle_read_mode_change(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    slot_query: Query<Entity, With<ImageSlot>>,
    webtoon_container_query: Query<Entity, With<WebtoonScrollContainer>>,
    image_loading_query: Query<Entity, With<ReaderImageLoading>>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    mut previous_mode: Local<Option<ReadMode>>,
) {
    let current_mode = reader_state.read_mode;

    let mode_changed = previous_mode
        .map(|prev| prev != current_mode)
        .unwrap_or(false);
    *previous_mode = Some(current_mode);

    if !mode_changed || reader_state.pictures.is_empty() {
        return;
    }

    tracing::info!("阅读模式切换: {:?}", current_mode);

    let Ok(container) = container_query.single() else {
        return;
    };

    match current_mode {
        ReadMode::SinglePage => {
            // 清除条漫视图
            for entity in webtoon_container_query.iter() {
                commands.entity(entity).despawn();
            }

            // 创建三 slot
            if slot_query.is_empty() {
                create_single_page_slots(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
                );
            }
        }
        ReadMode::Webtoon => {
            // 清除单页 slot
            for entity in slot_query.iter() {
                commands.entity(entity).despawn();
            }
            // 清除残留的加载指示器
            for entity in image_loading_query.iter() {
                commands.entity(entity).despawn();
            }

            // 如果没有条漫容器，创建
            if webtoon_container_query.is_empty() {
                reader_state.webtoon_anchor = (reader_state.current_page, 0.0);
                // 页高表随视图一起重置：换漫画时总页数可能恰好相同，
                // 只靠 sync 里的长度判断会把上一本的高度留下来
                reader_state.webtoon_page_heights =
                    vec![consts::WEBTOON_PLACEHOLDER_HEIGHT; reader_state.total_pages];
                create_webtoon_view(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
                );
            }
        }
    }
}

// ==================== 章节切换时重建视图 ====================

/// 处理章节切换后图片加载完成 —— 重建当前模式的视图
///
/// 监听 ReaderState 的 pictures 变化：
/// - 当 is_loading 从 true 变为 false 且 pictures 非空时触发
/// - 清除旧视图，根据当前模式重建
pub fn handle_chapter_switch(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    slot_query: Query<Entity, With<ImageSlot>>,
    webtoon_container_query: Query<Entity, With<WebtoonScrollContainer>>,
    loading_indicator_query: Query<Entity, With<ReaderLoadingIndicator>>,
    error_text_query: Query<Entity, With<ReaderErrorText>>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    mut episode_text_query: Query<&mut Text, (With<ReaderEpisodeText>, Without<ReaderPageText>)>,
    mut last_episode_order: Local<i32>,
) {
    if !reader_state.is_changed() {
        return;
    }

    // 检测章节切换：episode_order 变了且图片已加载
    let episode_changed = *last_episode_order != 0
        && *last_episode_order != reader_state.episode_order
        && !reader_state.pictures.is_empty()
        && !reader_state.is_loading;

    *last_episode_order = reader_state.episode_order;

    if !episode_changed {
        // 处理上一章跳转到末页的标记
        if reader_state.current_page == usize::MAX && !reader_state.pictures.is_empty() {
            reader_state.current_page = reader_state.total_pages.saturating_sub(1);
        }
        return;
    }

    // 处理上一章跳转到末页的标记
    if reader_state.current_page == usize::MAX {
        reader_state.current_page = reader_state.total_pages.saturating_sub(1);
    }

    tracing::info!(
        "章节切换完成: 第 {} 章, {} 张图片, 页码={}",
        reader_state.episode_order,
        reader_state.total_pages,
        reader_state.current_page
    );

    let Ok(container) = container_query.single() else {
        return;
    };

    // 清除旧视图
    for entity in slot_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in webtoon_container_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in loading_indicator_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in error_text_query.iter() {
        commands.entity(entity).despawn();
    }

    // 根据当前模式重建
    match reader_state.read_mode {
        ReadMode::SinglePage => {
            create_single_page_slots(
                &mut commands,
                container,
                &reader_state,
                &image_cache,
                &asset_server,
                &mut load_image_messages,
            );
        }
        ReadMode::Webtoon => {
            create_webtoon_view(
                &mut commands,
                container,
                &reader_state,
                &image_cache,
                &asset_server,
                &mut load_image_messages,
            );
        }
    }

    // 更新章节标题
    let episode_title = reader_state
        .episodes
        .get(reader_state.current_episode_idx)
        .map(|ep| ep.title.as_str())
        .unwrap_or("未知章节");
    for mut text in episode_text_query.iter_mut() {
        **text = format!("第 {} 章 - {}", reader_state.episode_order, episode_title);
    }

    // 更新页码
    update_page_text(&reader_state, &mut page_text_query);
}

// ==================== 页码更新 ====================

/// 更新底部栏页码/章节信息
pub fn update_page_info(
    reader_state: Res<ReaderState>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
) {
    if !reader_state.is_changed() || reader_state.pictures.is_empty() {
        return;
    }

    update_page_text(&reader_state, &mut page_text_query);
}

/// 辅助函数：设置页码文本
fn update_page_text(
    reader_state: &ReaderState,
    page_text_query: &mut Query<&mut Text, With<ReaderPageText>>,
) {
    let display_page = reader_state.current_page + 1;
    let total = reader_state.total_pages;
    for mut text in page_text_query.iter_mut() {
        **text = format!("{} / {}", display_page, total);
    }
}

// ==================== 阅读历史保存 ====================

/// 自动保存阅读历史（监听 ReaderState 变化）
pub fn save_reading_history(
    reader_state: Res<ReaderState>,
    comic_detail_state: Res<ComicDetailState>,
    mut save_messages: MessageWriter<SaveHistoryRequest>,
    mut last_saved: Local<(i32, usize)>,
) {
    if !reader_state.is_changed() {
        return;
    }

    if reader_state.comic_id.is_empty() || reader_state.pictures.is_empty() {
        return;
    }

    // 跳过 usize::MAX 标记页（上一章切换中间状态）
    if reader_state.current_page == usize::MAX {
        return;
    }

    let current = (reader_state.episode_order, reader_state.current_page);
    if *last_saved == current {
        return;
    }
    *last_saved = current;

    // 获取漫画信息
    let Some(comic) = &comic_detail_state.comic else {
        return;
    };

    // 获取章节标题
    let eps_title = reader_state
        .episodes
        .get(reader_state.current_episode_idx)
        .map(|ep| ep.title.clone())
        .unwrap_or_else(|| format!("第{}章", reader_state.episode_order));

    // current_page 为 0-indexed，SaveHistoryRequest.last_page 为 1-indexed
    save_messages.write(SaveHistoryRequest {
        comic_id: reader_state.comic_id.clone(),
        comic_title: comic.title.clone(),
        thumb_url: comic.thumb.url(),
        last_eps_order: reader_state.episode_order,
        last_eps_title: eps_title,
        last_page: (reader_state.current_page + 1) as i32,
    });
}

#[cfg(test)]
mod tests {
    use super::{anchor_to_scroll, consts, content_height, page_top, scroll_to_anchor};

    const GAP: f32 = consts::WEBTOON_GAP;

    /// 页顶偏移 = 前面所有页的高度 + 间距
    #[test]
    fn page_top_accumulates_with_gaps() {
        let heights = [100.0, 200.0, 300.0];
        assert_eq!(page_top(&heights, 0), 0.0);
        assert_eq!(page_top(&heights, 1), 100.0 + GAP);
        assert_eq!(page_top(&heights, 2), 300.0 + 2.0 * GAP);
    }

    /// 总高度不含末页尾部间距
    #[test]
    fn content_height_excludes_trailing_gap() {
        assert_eq!(content_height(&[]), 0.0);
        assert_eq!(content_height(&[100.0]), 100.0);
        assert_eq!(content_height(&[100.0, 200.0]), 300.0 + GAP);
    }

    /// 锚点 ↔ 像素 往返一致
    #[test]
    fn anchor_scroll_round_trip() {
        let heights = [100.0, 200.0, 300.0, 400.0];
        for (page, offset) in [(0, 0.0), (0, 50.0), (1, 0.0), (2, 150.0), (3, 399.0)] {
            let scroll = anchor_to_scroll(&heights, (page, offset));
            let back = scroll_to_anchor(&heights, scroll);
            assert_eq!(back.0, page, "page 往返不一致 @ ({page}, {offset})");
            assert!(
                (back.1 - offset).abs() < 0.001,
                "offset 往返不一致 @ ({page}, {offset}) -> {back:?}"
            );
        }
    }

    /// **本次重设计的核心不变量**：锚定页**上方**的页高变化，不改变
    /// 「锚定页顶边相对滚动位置的距离」——即视觉位置不动。
    ///
    /// 旧实现在用户持续滚动时会跳过补偿，正是这条不变量被破坏，
    /// 表现为"拉到底加载新图就错位"。
    #[test]
    fn anchor_absorbs_height_growth_above() {
        let before = [1000.0, 1000.0, 1000.0, 1000.0];
        // 第 0、1 页图片加载完，高度由占位 1000 变成真实值
        let after = [1450.0, 780.0, 1000.0, 1000.0];

        let anchor = (2, 120.0);
        let scroll_before = anchor_to_scroll(&before, anchor);
        let scroll_after = anchor_to_scroll(&after, anchor);

        // 滚动像素本身变了（上方内容长高/变矮了）
        assert_ne!(scroll_before, scroll_after);
        // 但锚定页顶边到视口顶边的距离没变 —— 视觉上纹丝不动
        assert!((scroll_before - page_top(&before, 2) - 120.0).abs() < 0.001);
        assert!((scroll_after - page_top(&after, 2) - 120.0).abs() < 0.001);
    }

    /// 落在页间距里时归到前一页尾部，不会在间距处左右横跳
    #[test]
    fn scroll_in_gap_sticks_to_previous_page() {
        let heights = [100.0, 200.0];
        let in_gap = 100.0 + GAP / 2.0;
        let (page, offset) = scroll_to_anchor(&heights, in_gap);
        assert_eq!(page, 0);
        assert!(offset > 100.0 && offset < 100.0 + GAP);
    }

    /// 越过末页时钉在最后一页，不会跑出数组
    #[test]
    fn scroll_past_end_clamps_to_last_page() {
        let heights = [100.0, 200.0];
        let (page, offset) = scroll_to_anchor(&heights, 99_999.0);
        assert_eq!(page, 1);
        assert!(offset > 0.0);
    }

    /// 负数滚动归零，空列表不 panic
    #[test]
    fn scroll_edges_are_safe() {
        assert_eq!(scroll_to_anchor(&[100.0], -50.0), (0, 0.0));
        assert_eq!(scroll_to_anchor(&[], 10.0), (0, 0.0));
        assert_eq!(anchor_to_scroll(&[], (5, 10.0)), 10.0);
    }

    /// 滚动量叠加：连续滚动应跨页推进，而不是卡在同一页
    #[test]
    fn repeated_scroll_advances_pages() {
        let heights = [100.0, 100.0, 100.0, 100.0];
        let mut anchor = (0, 0.0);
        for _ in 0..3 {
            let scroll = anchor_to_scroll(&heights, anchor) + (100.0 + GAP);
            anchor = scroll_to_anchor(&heights, scroll);
        }
        assert_eq!(anchor.0, 3);
        assert!(anchor.1.abs() < 0.001);
    }
}
