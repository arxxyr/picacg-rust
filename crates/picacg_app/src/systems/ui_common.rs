//! 通用 UI 构建函数
//!
//! 提取各页面共享的 UI 构建逻辑，避免代码重复。

use bevy::{
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
    window::PrimaryWindow,
};

use crate::{
    components::ContextMenuTarget,
    events::DownloadComicRequest,
    resources::{DownloadBadgeState, DownloadedComicsIndex},
    systems::{login::AppColors, scrollbar::scrollbar_config::*},
    utils::icons::{ICON_CHECK, ICON_SYNC},
};

// ==================== 标签徽章 ====================

/// 标签颜色类型
#[derive(Clone, Copy)]
pub enum TagColor {
    /// 分类（蓝色）
    Category,
    /// 标签（绿色）- 用于收藏和排行榜
    Tag,
}

impl TagColor {
    /// 获取背景色和文字颜色
    #[must_use]
    pub fn colors(self) -> (Color, Color) {
        match self {
            Self::Category => (Color::srgba(0.2, 0.4, 0.8, 0.3), Color::srgb(0.6, 0.8, 1.0)),
            Self::Tag => (Color::srgba(0.2, 0.6, 0.4, 0.3), Color::srgb(0.5, 0.9, 0.7)),
        }
    }
}

// ==================== 标签徽章（BSN 场景版） ====================

/// 标签徽章场景
pub fn tag_badge(text: &str, color_type: TagColor) -> impl Scene + use<> {
    let (bg_color, text_color) = color_type.colors();
    let text = text.to_string();

    // 单实体徽章：Text 节点自带 padding/圆角/底色
    bsn! {
        Text({text})
        TextFont { font_size: FontSize::Px(10.0) }
        TextColor(text_color)
        Node {
            padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(1.0), Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
        }
        BackgroundColor(bg_color)
    }
}

/// 带截断的标签徽章场景
pub fn tag_badge_truncated(
    text: &str,
    color_type: TagColor,
    max_chars: usize,
) -> impl Scene + use<> {
    let display_text = truncate_text(text, max_chars);
    let (bg_color, text_color) = color_type.colors();

    // 单实体徽章：Text 节点自带 padding/圆角/底色
    bsn! {
        Text({display_text})
        TextFont { font_size: FontSize::Px(9.0) }
        TextColor(text_color)
        Node {
            padding: UiRect::new(Val::Px(3.0), Val::Px(3.0), Val::Px(1.0), Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
        }
        BackgroundColor(bg_color)
    }
}

// ==================== 占位块骨架屏微光 ====================

/// 微光脉动的角频率（rad/s）——约 1.6 秒一个来回
const SHIMMER_SPEED: f32 = 4.0;
/// 高光相对底色的提亮量
const SHIMMER_LIFT: f32 = 0.09;
/// 每个占位块按实体号错开的相位量
///
/// 不错开的话整屏占位块会同频闪烁，像坏了而不像在加载
const SHIMMER_PHASE_STEP: f32 = 0.6;

/// 图片占位块的加载动画标记
///
/// 挂在**占位节点自身**上，不额外建子实体：占位块要么被 `ImageNode` 就地覆盖
/// （漫画列表的节点复用要求封面是单实体），要么整个换掉，多一个子节点就得跟着
/// 处理生命周期。动画只改 `BackgroundColor`，不碰布局。
#[derive(Component, Clone)]
pub struct LoadingShimmer {
    /// 静息底色
    pub base: Color,
    /// 脉动到顶时的高光色
    pub highlight: Color,
}

impl LoadingShimmer {
    /// 由底色推出高光色（各通道提亮后钳位）
    #[must_use]
    pub fn new(base: Color) -> Self {
        let srgba = base.to_srgba();
        let highlight = Color::srgba(
            (srgba.red + SHIMMER_LIFT).min(1.0),
            (srgba.green + SHIMMER_LIFT).min(1.0),
            (srgba.blue + SHIMMER_LIFT).min(1.0),
            srgba.alpha,
        );
        Self { base, highlight }
    }
}

impl Default for LoadingShimmer {
    fn default() -> Self {
        Self::new(AppColors::SURFACE_HOVER)
    }
}

/// 占位块微光动画（全局注册一次）
///
/// `Without<ImageNode>` 是关键：图片一就位就地插上 `ImageNode`，节点自动退出
/// 本查询，不需要任何清理代码。加载**终局失败**的节点由各页的图片系统摘掉
/// `LoadingShimmer`，免得失败的框一直在那儿"假装还在加载"。
pub fn animate_loading_shimmer(
    time: Res<Time>,
    mut query: Query<(Entity, &LoadingShimmer, &mut BackgroundColor), Without<ImageNode>>,
) {
    let elapsed = time.elapsed_secs();

    for (entity, shimmer, mut bg_color) in query.iter_mut() {
        // Entity → 稳定的相位偏移（0.19 的 EntityIndex 不能直接 as，走 to_bits）
        let phase = (entity.to_bits() % 64) as f32 * SHIMMER_PHASE_STEP;
        // sin → 0..1
        let k = (elapsed * SHIMMER_SPEED + phase).sin() * 0.5 + 0.5;
        let (base, high) = (shimmer.base.to_srgba(), shimmer.highlight.to_srgba());
        let lerp = |a: f32, b: f32| a + (b - a) * k;
        *bg_color = BackgroundColor(Color::srgba(
            lerp(base.red, high.red),
            lerp(base.green, high.green),
            lerp(base.blue, high.blue),
            lerp(base.alpha, high.alpha),
        ));
    }
}

// ==================== 封面下载角标 ====================

/// 角标锚定方式
///
/// 角标一律 `PositionType::Absolute`，定位参照是**直接父节点的 padding box**。
/// 两种挂法对应两套偏移，卡片结构决定用哪种。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BadgeAnchor {
    /// 角标是卡片根节点的直接子节点，封面为 164×220 且卡片
    /// `padding: 8px` / `border: 1px`（comics / favorites / home 三种卡片）
    CardCover,
    /// 角标挂在封面容器内部，直接贴容器右下角（search / rankings）
    CoverContainer,
}

impl BadgeAnchor {
    /// (right, top, bottom) 三个绝对定位偏移
    ///
    /// `CardCover` 的推导：卡片 border 1px → padding box 宽 178；封面左上角在
    /// padding box 的 (8, 8)，尺寸 164×220，故右边缘 172、下边缘 228。
    /// 角标 `BADGE_SIZE` 见方、离封面边缘 `BADGE_INSET`：
    /// right = 178 - (172 - 4) = 10，top = 228 - 4 - 20 = 204。
    const fn offsets(self) -> (Val, Val, Val) {
        match self {
            Self::CardCover => (Val::Px(10.0), Val::Px(204.0), Val::Auto),
            Self::CoverContainer => (Val::Px(BADGE_INSET), Val::Auto, Val::Px(BADGE_INSET)),
        }
    }
}

/// 角标边长（正方形，便于精确锚定）
const BADGE_SIZE: f32 = 20.0;
/// 角标离封面边缘的间距
const BADGE_INSET: f32 = 4.0;
/// 角标图标字号
///
/// 上限由行高定：Sarasa 的 hhea 行高 1.25em，字号 15 → 行盒 18.75px，
/// 在 20px 的徽章里还剩 1.25px 余量；再大行盒就顶满甚至被裁。
const BADGE_FONT_SIZE: f32 = 15.0;
/// 已下载（绿）
const BADGE_DOWNLOADED_COLOR: Color = Color::srgba(0.16, 0.65, 0.31, 0.92);
/// 有新章节（橙）
const BADGE_UPDATE_COLOR: Color = Color::srgba(0.92, 0.58, 0.13, 0.94);

/// 封面下载角标标记
///
/// `remote_episodes` 随卡片一起烘进组件，索引变化时刷新系统无需回查漫画列表。
#[derive(Component, Default, Clone)]
pub struct DownloadStatusBadge {
    pub comic_id: String,
    /// 服务端章节数（`Comic::eps_count`，0 表示接口未给出）
    pub remote_episodes: i32,
}

/// 角标里的图标文本子节点（刷新系统据此改字形，父节点只改底色/可见性）
#[derive(Component, Default, Clone)]
pub struct DownloadStatusBadgeIcon;

/// 角标状态 → (图标, 底色)
fn badge_appearance(state: DownloadBadgeState) -> (&'static str, Color) {
    match state {
        DownloadBadgeState::Downloaded => (ICON_CHECK, BADGE_DOWNLOADED_COLOR),
        DownloadBadgeState::UpdateAvailable => (ICON_SYNC, BADGE_UPDATE_COLOR),
    }
}

/// 封面右下角的下载状态角标场景
///
/// **无论是否已下载都会创建**：未下载时 `Visibility::Hidden`，等
/// `refresh_download_status_badges` 在索引变化时点亮。若按需创建，
/// 下载完成后回到列表页得等整页重建才看得到角标。
///
/// 结构是「定尺寸圆底容器 + 文本子节点」两个实体，不是一个 Text 顶着背景色：
/// Bevy 把文本画在 content box 的**左上角**（`bevy_ui_render/text.rs` 用
/// `content_box().min`），既不水平也不垂直居中，靠 padding 手调对不准——
/// 字形的 advance 宽度与行高由字体度量决定，换个图标就偏。交给 flex 的
/// `justify_content` / `align_items` 居中文本节点才是稳的。
pub fn download_status_badge(
    comic_id: &str,
    remote_episodes: i32,
    index: &DownloadedComicsIndex,
    anchor: BadgeAnchor,
) -> impl Scene + use<> {
    let state = index.badge_state(comic_id, remote_episodes);
    let (icon, bg_color) = match state {
        Some(state) => badge_appearance(state),
        // 未下载：内容随便给一个，靠 Visibility 藏起来
        None => (ICON_CHECK, BADGE_DOWNLOADED_COLOR),
    };
    let visibility = if state.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let (right, top, bottom) = anchor.offsets();
    let marker = DownloadStatusBadge {
        comic_id: comic_id.to_string(),
        remote_episodes,
    };

    bsn! {
        template_value(marker)
        Node {
            position_type: PositionType::Absolute,
            right: {right},
            top: {top},
            bottom: {bottom},
            width: {Val::Px(BADGE_SIZE)},
            height: {Val::Px(BADGE_SIZE)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(BADGE_SIZE / 2.0)),
        }
        BackgroundColor(bg_color)
        template_value(visibility)
        ZIndex(2)
        Children [
            (
                DownloadStatusBadgeIcon
                Text({icon})
                TextFont { font_size: {FontSize::Px(BADGE_FONT_SIZE)} }
                TextColor(Color::WHITE)
            )
        ]
    }
}

/// 索引变化时刷新所有在场角标（全局注册一次）
///
/// 下载完成 / 删除记录都只改索引，角标自己跟着变——卡片不必重建。
pub fn refresh_download_status_badges(
    index: Res<DownloadedComicsIndex>,
    mut badge_query: Query<(
        Ref<DownloadStatusBadge>,
        &mut Visibility,
        &mut BackgroundColor,
        &Children,
    )>,
    mut icon_query: Query<&mut Text, With<DownloadStatusBadgeIcon>>,
) {
    // 索引变了要全刷；索引没变也可能有个别角标刚被节点复用改绑（comics 列表），
    // 那种情况只刷改动的那几个
    let index_changed = index.is_changed();
    if !index_changed && badge_query.is_empty() {
        return;
    }

    for (badge, mut visibility, mut bg_color, children) in badge_query.iter_mut() {
        if !index_changed && !badge.is_changed() {
            continue;
        }
        let Some(state) = index.badge_state(&badge.comic_id, badge.remote_episodes) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let (icon, color) = badge_appearance(state);
        // 比较后写：避免每次索引变动都把全场角标标脏
        if *visibility != Visibility::Inherited {
            *visibility = Visibility::Inherited;
        }
        if bg_color.0 != color {
            *bg_color = BackgroundColor(color);
        }
        for child in children.iter() {
            if let Ok(mut text) = icon_query.get_mut(child)
                && text.as_str() != icon
            {
                **text = icon.to_string();
            }
        }
    }
}

// ==================== 滚动条 ====================

// ==================== 滚动处理 ====================

/// 时间戳 → 本地时间字符串（history/like_records 共用；原两份字节级相同的拷贝）
/// 格式化时间戳为可读字符串
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::{Local, TimeZone};

    if timestamp == 0 {
        return "未知时间".to_string();
    }

    match Local.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => "未知时间".to_string(),
    }
}

/// 截断文本
#[must_use]
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        format!("{}...", text.chars().take(max_chars).collect::<String>())
    } else {
        text.to_string()
    }
}

/// 格式化数字（支持万和k）
#[must_use]
pub fn format_number(n: i64) -> String {
    if n >= 10000 {
        format!("{:.1}万", n as f64 / 10000.0)
    } else if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

/// 格式化 API 返回的 ISO 8601 时间字符串为日期
///
/// `"2023-01-01T12:00:00.000Z"` → `"2023-01-01"`
#[must_use]
pub fn format_api_date(iso_str: &str) -> &str {
    iso_str.split('T').next().unwrap_or(iso_str)
}

/// 漫画卡片时间信息场景（两者皆 None 时返回空列表）
pub fn comic_time_info(created_at: Option<&str>, updated_at: Option<&str>) -> Box<dyn SceneList> {
    if created_at.is_none() && updated_at.is_none() {
        return Box::new(bsn_list![]);
    }

    let mut rows: Vec<Box<dyn Scene>> = Vec::new();
    if let Some(updated) = updated_at {
        let label = format!("更新 {}", format_api_date(updated));
        rows.push(Box::new(bsn! {
            Text({label})
            TextFont { font_size: FontSize::Px(9.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    }
    if let Some(created) = created_at {
        let label = format!("创建 {}", format_api_date(created));
        rows.push(Box::new(bsn! {
            Text({label})
            TextFont { font_size: FontSize::Px(9.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    }

    Box::new(bsn_list![(
        Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(Val::Px(2.0)),
            max_width: Val::Px(164.0),
            overflow: Overflow::clip(),
        }
        Children [ {rows} ]
    )])
}

// ==================== 全局右键菜单系统 ====================

/// 右键菜单根节点
#[derive(Component, Default, Clone)]
pub struct ComicContextMenu;

/// 右键菜单项类型
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextMenuAction {
    #[default]
    Download,
    Block,
}

/// 右键菜单项
#[derive(Component, Default, Clone)]
pub struct ComicContextMenuItem {
    pub action: ContextMenuAction,
    pub comic_id: String,
    pub comic_title: String,
    /// 由 `ContextMenuTarget` 透传的服务端章节数（0 = 未知）
    pub eps_count: i32,
}

/// 检测漫画卡片上的右键点击，弹出上下文菜单（全局，作用于所有带
/// ContextMenuTarget 的卡片）
pub fn comic_card_context_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&ContextMenuTarget, &Interaction)>,
    existing_menu: Query<Entity, With<ComicContextMenu>>,
) {
    // 右键刚按下
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    // 关闭已有菜单
    for entity in existing_menu.iter() {
        commands.entity(entity).despawn();
    }

    // 找到悬停中的卡片
    let hovered_card = card_query
        .iter()
        .find(|(_, interaction)| **interaction == Interaction::Hovered);

    let Some((target, _)) = hovered_card else {
        return;
    };

    // 获取光标位置
    let Some(window) = window_query.single().ok() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let comic_id = target.comic_id.clone();
    let comic_title = target.comic_title.clone();
    let eps_count = target.eps_count;

    // 创建菜单
    commands.spawn_scene(context_menu(cursor, &comic_id, &comic_title, eps_count));
}

/// 右键菜单场景
fn context_menu(
    cursor: Vec2,
    comic_id: &str,
    comic_title: &str,
    eps_count: i32,
) -> impl Scene + use<> {
    let download_label = format!("{} 下载", crate::utils::icons::ICON_DOWNLOAD);
    let block_label = format!("{} 屏蔽", crate::utils::icons::ICON_EYE_OFF);
    let download_item = ComicContextMenuItem {
        action: ContextMenuAction::Download,
        comic_id: comic_id.to_string(),
        comic_title: comic_title.to_string(),
        eps_count,
    };
    let block_item = ComicContextMenuItem {
        action: ContextMenuAction::Block,
        comic_id: comic_id.to_string(),
        comic_title: comic_title.to_string(),
        eps_count,
    };
    let (x, y) = (cursor.x, cursor.y);

    bsn! {
        ComicContextMenu
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(y),
            min_width: Val::Px(140.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        GlobalZIndex(100)
        BackgroundColor(Color::srgb(0.12, 0.12, 0.16))
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            // 下载按钮
            context_menu_item(download_label, download_item),
            (
                // 分割线
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(3.0)),
                }
                BackgroundColor(AppColors::BORDER)
            ),
            // 屏蔽按钮
            context_menu_item(block_label, block_item),
        ]
    }
}

/// 菜单项场景
fn context_menu_item(label: String, item: ComicContextMenuItem) -> impl Scene + use<> {
    bsn! {
        template_value(item)
        Button
        Interaction
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::NONE)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 创建菜单项/// 处理右键菜单项点击
pub fn comic_context_menu_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ComicContextMenuItem),
        Changed<Interaction>,
    >,
    menu_query: Query<Entity, With<ComicContextMenu>>,
    mut download_messages: MessageWriter<DownloadComicRequest>,
) {
    for (interaction, mut bg_color, item) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match item.action {
                    ContextMenuAction::Download => {
                        download_messages.write(DownloadComicRequest {
                            comic_id: item.comic_id.clone(),
                            comic_title: item.comic_title.clone(),
                            episodes: vec![], // 空 = 下载全部
                            remote_eps_count: (item.eps_count > 0).then_some(item.eps_count),
                        });
                        tracing::info!("右键菜单：下载漫画 {}", item.comic_title);
                    }
                    ContextMenuAction::Block => {
                        // 将标题添加到屏蔽词
                        let title = item.comic_title.clone();
                        if !title.is_empty() {
                            let settings = picacg_config::AppSettings::global();
                            let mut s = settings.write();
                            if !s.filter.blocked_keywords.contains(&title) {
                                s.filter.blocked_keywords.push(title.clone());
                                if let Err(e) = s.save() {
                                    tracing::error!("保存屏蔽设置失败: {}", e);
                                } else {
                                    tracing::info!("右键菜单：已屏蔽「{}」", title);
                                }
                            }
                        }
                    }
                }
                // 关闭菜单
                for entity in menu_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 点击菜单外区域关闭菜单
pub fn dismiss_context_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    menu_query: Query<Entity, With<ComicContextMenu>>,
    menu_item_query: Query<&Interaction, With<ComicContextMenuItem>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    // 如果有菜单项被悬停/按下，说明点的是菜单内部，不关闭
    let hovering_menu = menu_item_query.iter().any(|i| *i != Interaction::None);
    if hovering_menu {
        return;
    }
    // 关闭所有菜单
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}
