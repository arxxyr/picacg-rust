//! 主布局系统
//!
//! 实现侧边栏 + 内容区域的主布局，模仿 Python 版本的界面结构

#![allow(dead_code)]

use bevy::prelude::*;

use super::font_loader::get_font;
use crate::{
    components::*,
    resources::*,
    systems::{login::AppColors, navigation::NavigationHistory, ui_common::Scrollable},
    utils::{i18n::I18n, icons::*},
};

/// 侧边栏宽度
pub const SIDEBAR_WIDTH: f32 = 260.0;

/// 侧边栏头像图片标记（用于登录后替换为用户头像）
#[derive(Component)]
pub struct SidebarAvatarImage {
    pub url: String,
}

/// 下载中计数徽章
#[derive(Component)]
pub struct DownloadingCountBadge;

/// 排队中计数徽章
#[derive(Component)]
pub struct QueuedCountBadge;

/// 侧边栏菜单滚动区域标记
#[derive(Component)]
pub struct SidebarMenuArea;

/// 侧边栏按钮分组
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    User,       // 用户
    Navigation, // 导航
    Tools,      // 工具
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
    let font: Handle<Font> = get_font();

    // 主布局根节点：横向排列（侧边栏 + 内容区域）
    commands
        .spawn((
            MainLayoutRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|parent| {
            // 侧边栏
            spawn_sidebar(parent, &font, &i18n);

            // 内容区域
            parent.spawn((
                ContentArea,
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(AppColors::BACKGROUND),
            ));
        });
}

/// 创建侧边栏
fn spawn_sidebar(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, i18n: &I18n) {
    parent
        .spawn((
            Sidebar,
            Node {
                width: Val::Px(SIDEBAR_WIDTH),
                min_width: Val::Px(SIDEBAR_WIDTH),
                max_width: Val::Px(SIDEBAR_WIDTH),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0, // 不收缩
                border: UiRect::right(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|sidebar| {
            // 用户信息区
            spawn_user_info_area(sidebar, font, i18n);

            // 分隔线
            sidebar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    flex_shrink: 0.0,
                    margin: UiRect::vertical(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(AppColors::BORDER),
            ));

            // 菜单区域（可滚动，占据剩余空间）
            spawn_menu_area(sidebar, font, i18n);

            // 底部版本信息（固定显示）
            sidebar.spawn((
                Text::new(format!("v{}", env!("CARGO_PKG_VERSION"))),
                TextFont {
                    font: font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    flex_shrink: 0.0,
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
}

/// 创建用户信息区
fn spawn_user_info_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, i18n: &I18n) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(15.0)),
            flex_shrink: 0.0, // 不被菜单区压缩
            ..default()
        },))
        .with_children(|area| {
            // 头像占位符 (100x100)
            area.spawn((
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                BorderColor::all(AppColors::PRIMARY),
            ))
            .with_children(|avatar| {
                avatar.spawn((
                    SidebarAvatarImage { url: String::new() },
                    Text::new("👤"),
                    TextFont {
                        font: font.clone(),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });

            // 应用标题
            area.spawn((
                Text::new("PicACG"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(AppColors::PRIMARY),
            ));

            // 副标题
            area.spawn((
                Text::new(i18n.t("sidebar.subtitle")),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
            ));
        });
}

/// 创建菜单区域
fn spawn_menu_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, i18n: &I18n) {
    parent
        .spawn((
            SidebarMenuArea,
            Scrollable,
            ScrollPosition::default(),
            Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::horizontal(Val::Px(10.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .with_children(|menu| {
            // 用户分组
            spawn_section_header(menu, i18n.t("sidebar.user"), font);
            for config in SIDEBAR_BUTTONS
                .iter()
                .filter(|c| c.section == SidebarSection::User)
            {
                spawn_sidebar_button(menu, font, config, i18n);
            }

            // 分隔线
            spawn_section_separator(menu);

            // 导航分组
            spawn_section_header(menu, i18n.t("sidebar.navigation"), font);
            for config in SIDEBAR_BUTTONS
                .iter()
                .filter(|c| c.section == SidebarSection::Navigation)
            {
                spawn_sidebar_button(menu, font, config, i18n);
            }

            // 分隔线
            spawn_section_separator(menu);

            // 其他分组
            spawn_section_header(menu, i18n.t("sidebar.other"), font);
            for config in SIDEBAR_BUTTONS
                .iter()
                .filter(|c| c.section == SidebarSection::Other)
            {
                spawn_sidebar_button(menu, font, config, i18n);
            }
        });
}

/// 创建分组标题
fn spawn_section_header(parent: &mut ChildSpawnerCommands, title: &str, font: &Handle<Font>) {
    parent.spawn((
        Text::new(title),
        TextFont {
            font: font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(AppColors::TEXT_SECONDARY),
        Node {
            margin: UiRect {
                top: Val::Px(6.0),
                bottom: Val::Px(2.0),
                left: Val::Px(5.0),
                right: Val::Px(0.0),
            },
            ..default()
        },
    ));
}

/// 创建分组分隔线
fn spawn_section_separator(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(AppColors::BORDER),
    ));
}

/// 创建侧边栏按钮
fn spawn_sidebar_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    config: &SidebarButtonConfig,
    i18n: &I18n,
) {
    parent
        .spawn((
            SidebarButton {
                route: config.route,
            },
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(1.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(Color::NONE),
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            // 图标
            btn.spawn((
                Text::new(config.icon),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 标签（通过 i18n 翻译）
            btn.spawn((
                Text::new(i18n.t(config.label)),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 下载按钮添加计数徽章（下载中 + 排队中）
            if config.route == SidebarRoute::Downloads {
                // 下载中徽章（蓝色）
                spawn_download_badge(btn, font, DownloadingCountBadge, AppColors::PRIMARY);
                // 排队中徽章（橙色）
                spawn_download_badge(btn, font, QueuedCountBadge, Color::srgb(0.9, 0.6, 0.2));
            }
        });
}

/// 创建下载计数徽章
fn spawn_download_badge(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    marker: impl Component,
    color: Color,
) {
    parent
        .spawn((
            marker,
            Node {
                display: Display::None,
                min_width: Val::Px(18.0),
                height: Val::Px(18.0),
                padding: UiRect::horizontal(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(9.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
        ))
        .with_children(|badge| {
            badge.spawn((
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 清理主布局
pub fn cleanup_main_layout(mut commands: Commands, query: Query<Entity, With<MainLayoutRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 侧边栏按钮交互系统
pub fn sidebar_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &SidebarButton,
        ),
        Changed<Interaction>,
    >,
    current_route: Res<State<AppRoute>>,
    mut next_route: ResMut<NextState<AppRoute>>,
    history: Res<NavigationHistory>,
) {
    for (interaction, mut bg_color, mut border_color, sidebar_btn) in &mut interaction_query {
        // 判断是否为当前激活的路由
        // 注意：ComicDetail/ReadView 不高亮任何按钮，因为可从多个入口进入
        let is_active = matches!(
            (sidebar_btn.route, current_route.get()),
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
        );

        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 使用保存的最后访问路由，而不是默认路由
                let target_route = history.get_section_route(sidebar_btn.route);
                next_route.set(target_route);
            }
            Interaction::Hovered => {
                if is_active {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                    *border_color = BorderColor::all(Color::NONE);
                }
            }
            Interaction::None => {
                if is_active {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                } else {
                    *bg_color = BackgroundColor(Color::NONE);
                    *border_color = BorderColor::all(Color::NONE);
                }
            }
        }
    }
}

/// 更新侧边栏按钮的激活状态（路由变化时）
pub fn update_sidebar_active_state(
    current_route: Res<State<AppRoute>>,
    mut button_query: Query<(&SidebarButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    // 只在路由变化时触发
    if !current_route.is_changed() {
        return;
    }

    for (sidebar_btn, mut bg_color, mut border_color) in &mut button_query {
        // 注意：ComicDetail/ReadView 不高亮任何按钮，因为可从多个入口进入
        let is_active = matches!(
            (sidebar_btn.route, current_route.get()),
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
        );

        if is_active {
            *bg_color = BackgroundColor(AppColors::PRIMARY);
            *border_color = BorderColor::all(AppColors::PRIMARY);
        } else {
            *bg_color = BackgroundColor(Color::NONE);
            *border_color = BorderColor::all(Color::NONE);
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
        if let Some(handle) = image_cache.handles.get(&avatar.url) {
            // 隐藏占位符文本（不删除，避免 bevy_text panic）
            if let Ok(mut node) = text_node_query.get_mut(entity) {
                node.display = Display::None;
            }
            // 在头像容器（父节点）中添加图片子节点
            commands.entity(child_of.parent()).with_children(|parent| {
                parent.spawn((
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(76.0),
                        height: Val::Px(76.0),
                        ..default()
                    },
                ));
            });
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
    let tasks = download_state.active_tasks();
    let downloading = tasks
        .iter()
        .filter(|t| t.meta.state.is_downloading())
        .count();
    let queued = tasks
        .iter()
        .filter(|t| matches!(t.meta.state, crate::resources::DownloadState::Queued))
        .count();

    // 更新下载中徽章
    for (mut node, children) in downloading_badge.iter_mut() {
        if downloading == 0 {
            node.display = Display::None;
        } else {
            node.display = Display::Flex;
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = format!("{}", downloading);
                }
            }
        }
    }

    // 更新排队中徽章
    for (mut node, children) in queued_badge.iter_mut() {
        if queued == 0 {
            node.display = Display::None;
        } else {
            node.display = Display::Flex;
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = format!("{}", queued);
                }
            }
        }
    }
}
