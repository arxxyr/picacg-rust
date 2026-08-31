//! 主布局系统
//!
//! 实现侧边栏 + 内容区域的主布局，模仿 Python 版本的界面结构

use bevy::prelude::*;

use crate::{
    components::*,
    resources::*,
    systems::{
        login::AppColors, navigation::NavigationHistory, scrollbar::ScrollArea,
        widgets::ButtonStyle,
    },
    utils::{i18n::I18n, icons::*},
};

/// 侧边栏宽度
pub const SIDEBAR_WIDTH: f32 = 260.0;

/// 侧边栏头像图片标记（用于登录后替换为用户头像）
#[derive(Component, Default, Clone)]
pub struct SidebarAvatarImage {
    pub url: String,
}

/// 下载中计数徽章
#[derive(Component, Default, Clone)]
pub struct DownloadingCountBadge;

/// 排队中计数徽章
#[derive(Component, Default, Clone)]
pub struct QueuedCountBadge;

/// 侧边栏菜单滚动区域标记
#[derive(Component, Default, Clone)]
pub struct SidebarMenuArea;

/// 侧边栏按钮分组
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    User,       // 用户
    Navigation, // 导航
    Other,      // 其他
}

/// 侧边栏按钮配置
struct SidebarButtonConfig {
    route: SidebarRoute,
    label: &'static str,
    icon: &'static str,
    section: SidebarSection,
}

const SIDEBAR_BUTTONS: &[SidebarButtonConfig] = &[
    // 用户分组
    SidebarButtonConfig {
        route: SidebarRoute::Favorites,
        label: "sidebar.favorites",
        icon: ICON_HEART,
        section: SidebarSection::User,
    },
    SidebarButtonConfig {
        route: SidebarRoute::History,
        label: "sidebar.history",
        icon: ICON_HISTORY,
        section: SidebarSection::User,
    },
    SidebarButtonConfig {
        route: SidebarRoute::LikeRecords,
        label: "sidebar.like_records",
        icon: ICON_THUMB_UP,
        section: SidebarSection::User,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Profile,
        label: "sidebar.profile",
        icon: ICON_USER,
        section: SidebarSection::User,
    },
    // 导航分组
    SidebarButtonConfig {
        route: SidebarRoute::Home,
        label: "sidebar.home",
        icon: ICON_HOME,
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Categories,
        label: "sidebar.categories",
        icon: ICON_BOOKSHELF,
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Search,
        label: "sidebar.search",
        icon: ICON_SEARCH,
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Rankings,
        label: "sidebar.rankings",
        icon: ICON_TROPHY,
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Games,
        label: "sidebar.games",
        icon: ICON_GAMEPAD,
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Fried,
        label: "sidebar.fried",
        icon: ICON_FORUM,
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Chat,
        label: "sidebar.chat",
        icon: ICON_CHAT,
        section: SidebarSection::Navigation,
    },
    // 其他分组
    SidebarButtonConfig {
        route: SidebarRoute::Downloads,
        label: "sidebar.downloads",
        icon: ICON_DOWNLOAD,
        section: SidebarSection::Other,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Settings,
        label: "sidebar.settings",
        icon: ICON_COG,
        section: SidebarSection::Other,
    },
];

/// 创建主布局
pub fn setup_main_layout(mut commands: Commands, _asset_server: Res<AssetServer>, i18n: Res<I18n>) {
    commands.spawn_scene(main_layout_page(&i18n));
}

/// 主布局场景：根节点横向排列（侧边栏 + 内容区域）
fn main_layout_page(i18n: &I18n) -> impl Scene + use<> {
    bsn! {
        MainLayoutRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            // 侧边栏
            sidebar(i18n),
            (
                // 内容区域
                ContentArea
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                }
                BackgroundColor(AppColors::BACKGROUND)
            ),
        ]
    }
}

/// 侧边栏场景
fn sidebar(i18n: &I18n) -> impl Scene + use<> {
    let version_label = format!("v{}", env!("CARGO_PKG_VERSION"));

    bsn! {
        Sidebar
        Node {
            width: Val::Px(SIDEBAR_WIDTH),
            min_width: Val::Px(SIDEBAR_WIDTH),
            max_width: Val::Px(SIDEBAR_WIDTH),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0, // 不收缩
            border: UiRect::right(Val::Px(1.0)),
        }
        BackgroundColor(AppColors::HEADER_BG)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            // 用户信息区
            user_info_area(i18n),
            (
                // 分隔线
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    flex_shrink: 0.0,
                    margin: UiRect::vertical(Val::Px(10.0)),
                }
                BackgroundColor(AppColors::BORDER)
            ),
            // 菜单区域（可滚动，占据剩余空间）
            menu_area(i18n),
            (
                // 底部版本信息（固定显示）
                Text({version_label})
                TextFont { font_size: FontSize::Px(10.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node {
                    flex_shrink: 0.0,
                    margin: UiRect::all(Val::Px(10.0)),
                }
            ),
        ]
    }
}

/// 用户信息区场景
fn user_info_area(i18n: &I18n) -> impl Scene + use<> {
    let subtitle = i18n.t("sidebar.subtitle").to_string();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(15.0)),
            flex_shrink: 0.0, // 不被菜单区压缩
        }
        Children [
            (
                // 头像占位符 (100x100)
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    overflow: Overflow::clip(),
                }
                BackgroundColor(AppColors::SURFACE)
                template_value(BorderColor::all(AppColors::PRIMARY))
                Children [
                    (
                        SidebarAvatarImage
                        Text("👤")
                        TextFont { font_size: FontSize::Px(32.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    )
                ]
            ),
            (
                // 应用标题
                Text("PicACG")
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(AppColors::PRIMARY)
            ),
            (
                // 副标题
                Text({subtitle})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { margin: UiRect::top(Val::Px(5.0)) }
            ),
        ]
    }
}

/// 菜单区域场景（可滚动）
fn menu_area(i18n: &I18n) -> impl Scene + use<> {
    // 三个分组的按钮列表（数据驱动）
    let user_buttons: Vec<_> = SIDEBAR_BUTTONS
        .iter()
        .filter(|c| c.section == SidebarSection::User)
        .map(|config| sidebar_button(config, i18n))
        .collect();
    let navigation_buttons: Vec<_> = SIDEBAR_BUTTONS
        .iter()
        .filter(|c| c.section == SidebarSection::Navigation)
        .map(|config| sidebar_button(config, i18n))
        .collect();
    let other_buttons: Vec<_> = SIDEBAR_BUTTONS
        .iter()
        .filter(|c| c.section == SidebarSection::Other)
        .map(|config| sidebar_button(config, i18n))
        .collect();

    bsn! {
        SidebarMenuArea
        // 侧边栏菜单滚动：上游 ScrollArea 按悬停派发滚轮，无需光标分区路由
        ScrollArea
        Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: Val::Px(0.0),
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::horizontal(Val::Px(10.0)),
            overflow: Overflow::scroll_y(),
        }
        Children [
            // 用户分组
            section_header(i18n.t("sidebar.user")),
            {user_buttons},
            // 分隔线
            section_separator(),
            // 导航分组
            section_header(i18n.t("sidebar.navigation")),
            {navigation_buttons},
            // 分隔线
            section_separator(),
            // 其他分组
            section_header(i18n.t("sidebar.other")),
            {other_buttons},
        ]
    }
}

/// 分组标题场景
fn section_header(title: &str) -> impl Scene + use<> {
    let title = title.to_string();
    // 标题外边距：上 6 / 下 2 / 左 5 / 右 0
    let title_margin = UiRect {
        top: Val::Px(6.0),
        bottom: Val::Px(2.0),
        left: Val::Px(5.0),
        right: Val::Px(0.0),
    };

    bsn! {
        Text({title})
        TextFont { font_size: FontSize::Px(12.0) }
        TextColor(AppColors::TEXT_SECONDARY)
        Node { margin: {title_margin} }
    }
}

/// 分组分隔线场景
fn section_separator() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(4.0)),
        }
        BackgroundColor(AppColors::BORDER)
    }
}

/// 侧边栏按钮场景
fn sidebar_button(config: &SidebarButtonConfig, i18n: &I18n) -> impl Scene + use<> {
    let route = config.route;
    let icon = config.icon;
    let label = i18n.t(config.label).to_string();

    // 下载按钮添加计数徽章（下载中 + 排队中）
    let badges: Box<dyn SceneList> = if config.route == SidebarRoute::Downloads {
        Box::new(bsn_list![
            // 下载中徽章（蓝色）
            download_badge(DownloadingCountBadge, AppColors::PRIMARY),
            // 排队中徽章（橙色）
            download_badge(QueuedCountBadge, Color::srgb(0.9, 0.6, 0.2)),
        ])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        SidebarButton { route: {route} }
        Button
        // 侧边栏是单选组：未选中走 Segment（surface_sunken），选中态由
        // update_sidebar_active_state 写 selected 钉在 primary
        template_value(ButtonStyle::segment(false))
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(32.0),
            padding: UiRect::horizontal(Val::Px(10.0)),
            margin: UiRect::bottom(Val::Px(1.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        template_value(BorderColor::all(Color::NONE))
        BackgroundColor(AppColors::SURFACE_SUNKEN)
        Children [
            (
                // 图标
                Text({icon})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 标签（通过 i18n 翻译）
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            {badges},
        ]
    }
}

/// 下载计数徽章场景（marker 区分下载中 / 排队中）
fn download_badge<M: Component + Default + Clone + Unpin>(
    marker: M,
    color: Color,
) -> impl Scene + use<M> {
    bsn! {
        template_value(marker)
        Node {
            display: Display::None,
            min_width: Val::Px(18.0),
            height: Val::Px(18.0),
            padding: UiRect::horizontal(Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(9.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor({color})
        Children [
            (
                Text("")
                TextFont { font_size: FontSize::Px(10.0) }
                TextColor(Color::WHITE)
            )
        ]
    }
}

/// 侧边栏头像图片场景（图片缓存就绪后插入头像容器）
fn sidebar_avatar_image(handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        ImageNode { image: {handle} }
        Node {
            width: Val::Px(76.0),
            height: Val::Px(76.0),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
        }
    }
}

/// 侧边栏按钮是否对应当前路由
///
/// 注意：ComicDetail/ReadView 不高亮任何按钮，因为可从多个入口进入。
fn is_sidebar_route_active(sidebar_route: SidebarRoute, app_route: &AppRoute) -> bool {
    matches!(
        (sidebar_route, app_route),
        (SidebarRoute::Home, AppRoute::Home)
            | (
                SidebarRoute::Categories,
                AppRoute::Categories | AppRoute::ComicsList
            )
            | (SidebarRoute::Search, AppRoute::Search)
            | (SidebarRoute::Rankings, AppRoute::Rankings)
            | (SidebarRoute::Games, AppRoute::Games | AppRoute::GameDetail)
            | (SidebarRoute::Fried, AppRoute::Fried)
            | (SidebarRoute::Favorites, AppRoute::Favorites)
            | (SidebarRoute::History, AppRoute::History)
            | (SidebarRoute::LikeRecords, AppRoute::LikeRecords)
            | (SidebarRoute::Profile, AppRoute::Profile)
            | (SidebarRoute::LocalRead, AppRoute::LocalRead)
            | (SidebarRoute::Downloads, AppRoute::Downloads)
            | (SidebarRoute::Settings, AppRoute::Settings)
            | (SidebarRoute::ImageConvert, AppRoute::ImageConvert)
            | (SidebarRoute::Waifu2x, AppRoute::Waifu2x)
            | (SidebarRoute::Nas, AppRoute::Nas)
            | (SidebarRoute::Chat, AppRoute::Chat | AppRoute::ChatRoom)
    )
}

/// 侧边栏按钮交互系统（只管跳转，配色由 apply_button_interaction 统一处理）
pub fn sidebar_button_interaction(
    interaction_query: Query<(&Interaction, &SidebarButton), Changed<Interaction>>,
    mut next_route: ResMut<NextState<AppRoute>>,
    history: Res<NavigationHistory>,
) {
    for (interaction, sidebar_btn) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // 使用保存的最后访问路由，而不是默认路由
            let target_route = history.get_section_route(sidebar_btn.route);
            next_route.set(target_route);
        }
    }
}

/// 更新侧边栏按钮的激活状态（路由变化 / 侧边栏刚建好时）
pub fn update_sidebar_active_state(
    current_route: Res<State<AppRoute>>,
    mut button_query: Query<(&SidebarButton, &mut ButtonStyle)>,
    new_buttons: Query<(), Added<SidebarButton>>,
) {
    // 路由未变且没有新建按钮时零开销；
    // 后者保证主布局首次建好的那一帧也能点亮当前页
    if !current_route.is_changed() && new_buttons.is_empty() {
        return;
    }

    for (sidebar_btn, mut style) in &mut button_query {
        let is_active = is_sidebar_route_active(sidebar_btn.route, current_route.get());
        if style.selected != is_active {
            style.selected = is_active;
        }
    }
}

/// 主布局创建后自动加载用户资料（获取头像 URL）
/// 主布局创建后自动预加载用户资料和分类数据
pub fn auto_load_user_profile(
    mut has_loaded: Local<bool>,
    mut load_profile: MessageWriter<crate::events::LoadUserProfileRequest>,
    mut load_categories: MessageWriter<crate::events::LoadCategoriesRequest>,
) {
    if !*has_loaded {
        *has_loaded = true;
        load_profile.write(crate::events::LoadUserProfileRequest);
        load_categories.write(crate::events::LoadCategoriesRequest);
    }
}

/// 监听用户资料加载完成，更新侧边栏头像 URL 并请求加载图片
pub fn update_sidebar_avatar_url(
    mut loaded_messages: MessageReader<crate::events::UserProfileLoadedEvent>,
    mut avatar_query: Query<&mut SidebarAvatarImage>,
    mut image_messages: MessageWriter<crate::events::LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        let avatar_url = event
            .user
            .avatar
            .as_ref()
            .map(|a| a.url())
            .unwrap_or_default();
        if avatar_url.is_empty() {
            continue;
        }
        for mut avatar in avatar_query.iter_mut() {
            if avatar.url != avatar_url {
                avatar.url = avatar_url.clone();
                image_messages.write(crate::events::LoadImageRequest {
                    url: avatar_url.clone(),
                });
            }
        }
    }
}

/// 当头像图片缓存就绪后，隐藏占位符文本并在父容器中添加图片
pub fn update_sidebar_avatar_image(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    avatar_query: Query<(Entity, &SidebarAvatarImage, &ChildOf), Without<ImageNode>>,
    mut text_node_query: Query<&mut Node>,
) {
    for (entity, avatar, child_of) in avatar_query.iter() {
        if avatar.url.is_empty() {
            continue;
        }
        // 加载失败的头像退出扫描集，否则每帧重查一次且永远等不到
        if image_cache.is_failed(&avatar.url) {
            commands.entity(entity).remove::<SidebarAvatarImage>();
            continue;
        }
        if let Some(handle) = image_cache.get(&avatar.url) {
            // 隐藏占位符文本（不删除，避免 bevy_text panic）
            if let Ok(mut node) = text_node_query.get_mut(entity) {
                node.display = Display::None;
            }
            // 在头像容器（父节点）中添加图片子节点
            commands
                .spawn_scene(sidebar_avatar_image(handle.clone()))
                .insert(ChildOf(child_of.parent()));
            // 移除 SidebarAvatarImage 防止重复添加
            commands.entity(entity).remove::<SidebarAvatarImage>();
        }
    }
}

/// 更新下载计数徽章
pub fn update_download_count_badge(
    download_state: Res<DownloadManagerState>,
    mut downloading_badge: Query<
        (&mut Node, &Children),
        (With<DownloadingCountBadge>, Without<QueuedCountBadge>),
    >,
    mut queued_badge: Query<
        (&mut Node, &Children),
        (With<QueuedCountBadge>, Without<DownloadingCountBadge>),
    >,
    mut text_query: Query<&mut Text>,
) {
    // 下载状态未变化时零开销（此前每帧无条件写
    // Node/Text，全应用范围强制布局与文字整形）
    if !download_state.is_changed() {
        return;
    }
    let tasks = download_state.active_tasks();
    let downloading = tasks
        .iter()
        .filter(|t| t.meta.state.is_downloading())
        .count();
    let queued = tasks
        .iter()
        .filter(|t| matches!(t.meta.state, crate::resources::DownloadState::Queued))
        .count();

    // 更新下载中徽章（比较后写，避免无谓的布局标脏）
    for (mut node, children) in downloading_badge.iter_mut() {
        let target = if downloading == 0 {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != target {
            node.display = target;
        }
        if downloading > 0 {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    let label = downloading.to_string();
                    if **text != label {
                        **text = label;
                    }
                }
            }
        }
    }

    // 更新排队中徽章
    for (mut node, children) in queued_badge.iter_mut() {
        let target = if queued == 0 {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != target {
            node.display = target;
        }
        if queued > 0 {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    let label = queued.to_string();
                    if **text != label {
                        **text = label;
                    }
                }
            }
        }
    }
}
