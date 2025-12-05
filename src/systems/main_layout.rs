//! 主布局系统
//!
//! 实现侧边栏 + 内容区域的主布局，模仿 Python 版本的界面结构

#![allow(dead_code)]

use bevy::prelude::*;

use crate::{
    components::*,
    resources::*,
    systems::{
        login::{AppColors, FONT_PATH},
        navigation::NavigationHistory,
    },
};

/// 侧边栏宽度
pub const SIDEBAR_WIDTH: f32 = 260.0;

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
        label: "我的收藏",
        icon: "⭐",
        section: SidebarSection::User,
    },
    // 导航分组
    SidebarButtonConfig {
        route: SidebarRoute::Home,
        label: "首页",
        icon: "🏠",
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Categories,
        label: "分类",
        icon: "📚",
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Search,
        label: "搜索",
        icon: "🔍",
        section: SidebarSection::Navigation,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Rankings,
        label: "排行榜",
        icon: "🏆",
        section: SidebarSection::Navigation,
    },
    // 其他分组
    SidebarButtonConfig {
        route: SidebarRoute::Downloads,
        label: "下载",
        icon: "📥",
        section: SidebarSection::Other,
    },
    SidebarButtonConfig {
        route: SidebarRoute::Settings,
        label: "设置",
        icon: "⚙️",
        section: SidebarSection::Other,
    },
];

/// 创建主布局
pub fn setup_main_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

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
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|parent| {
            // 侧边栏
            spawn_sidebar(parent, &font);

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
                Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
            ));
        });
}

/// 创建侧边栏
fn spawn_sidebar(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
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
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|sidebar| {
            // 用户信息区
            spawn_user_info_area(sidebar, font);

            // 分隔线
            sidebar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(AppColors::BORDER),
            ));

            // 菜单区域（可滚动）
            spawn_menu_area(sidebar, font);

            // 底部版本信息
            sidebar.spawn((
                Text::new("Bevy 0.17.3 版"),
                TextFont {
                    font: font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
}

/// 创建用户信息区
fn spawn_user_info_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
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
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                BorderColor::all(AppColors::PRIMARY),
                BorderRadius::all(Val::Percent(50.0)),
            ))
            .with_children(|avatar| {
                avatar.spawn((
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
                Text::new("漫画客户端"),
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
fn spawn_menu_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::horizontal(Val::Px(10.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|menu| {
            // 用户分组
            spawn_section_header(menu, "用户", font);
            for config in SIDEBAR_BUTTONS
                .iter()
                .filter(|c| c.section == SidebarSection::User)
            {
                spawn_sidebar_button(menu, font, config);
            }

            // 分隔线
            spawn_section_separator(menu);

            // 导航分组
            spawn_section_header(menu, "导航", font);
            for config in SIDEBAR_BUTTONS
                .iter()
                .filter(|c| c.section == SidebarSection::Navigation)
            {
                spawn_sidebar_button(menu, font, config);
            }

            // 分隔线
            spawn_section_separator(menu);

            // 其他分组
            spawn_section_header(menu, "其他", font);
            for config in SIDEBAR_BUTTONS
                .iter()
                .filter(|c| c.section == SidebarSection::Other)
            {
                spawn_sidebar_button(menu, font, config);
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
                top: Val::Px(10.0),
                bottom: Val::Px(5.0),
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
            margin: UiRect::vertical(Val::Px(8.0)),
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
) {
    parent
        .spawn((
            SidebarButton {
                route: config.route,
            },
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                margin: UiRect::bottom(Val::Px(3.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(6.0)),
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

            // 标签
            btn.spawn((
                Text::new(config.label),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
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
        let is_active = match (sidebar_btn.route, current_route.get()) {
            (SidebarRoute::Home, AppRoute::Home) => true,
            (SidebarRoute::Categories, AppRoute::Categories | AppRoute::ComicsList) => true,
            (SidebarRoute::Search, AppRoute::Search) => true,
            (SidebarRoute::Rankings, AppRoute::Rankings) => true,
            (SidebarRoute::Favorites, AppRoute::Favorites) => true,
            (SidebarRoute::Downloads, AppRoute::Downloads) => true,
            (SidebarRoute::Settings, AppRoute::Settings) => true,
            _ => false,
        };

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
        let is_active = match (sidebar_btn.route, current_route.get()) {
            (SidebarRoute::Home, AppRoute::Home) => true,
            (SidebarRoute::Categories, AppRoute::Categories | AppRoute::ComicsList) => true,
            (SidebarRoute::Search, AppRoute::Search) => true,
            (SidebarRoute::Rankings, AppRoute::Rankings) => true,
            (SidebarRoute::Favorites, AppRoute::Favorites) => true,
            (SidebarRoute::Downloads, AppRoute::Downloads) => true,
            (SidebarRoute::Settings, AppRoute::Settings) => true,
            _ => false,
        };

        if is_active {
            *bg_color = BackgroundColor(AppColors::PRIMARY);
            *border_color = BorderColor::all(AppColors::PRIMARY);
        } else {
            *bg_color = BackgroundColor(Color::NONE);
            *border_color = BorderColor::all(Color::NONE);
        }
    }
}
