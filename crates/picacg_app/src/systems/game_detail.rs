//! 游戏详情系统
//!
//! 实现游戏详情页面的 UI 和交互

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        ui_common::truncate_text,
        widgets::ButtonStyle,
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 游戏详情根节点
#[derive(Component, Default, Clone)]
pub struct GameDetailRoot;

/// 游戏详情滚动容器
#[derive(Component, Default, Clone)]
pub struct GameDetailScrollContainer;

/// 游戏详情返回按钮
#[derive(Component, Default, Clone)]
pub struct GameDetailBackButton;

/// 游戏详情图标（图标与截图共用，图片就绪后就地换成 `ImageNode`）
#[derive(Component, Default, Clone)]
pub struct GameDetailIcon {
    pub url: String,
}

// ==================== 场景函数 ====================

/// 游戏详情页面场景（供 setup 和 refresh 共用）
fn game_detail_page(
    game_detail_state: &GameDetailState,
    image_cache: &ImageCache,
) -> impl Scene + use<> {
    // 滚动区内边距（右侧额外让出滚动条宽度）
    let scroll_padding = UiRect {
        left: Val::Px(20.0),
        right: Val::Px(20.0 + SCROLLBAR_WIDTH),
        top: Val::Px(20.0),
        bottom: Val::Px(20.0),
    };

    // 滚动区内容：加载中 / 加载失败 / 游戏详情 / 暂无数据
    let content: Box<dyn SceneList> = if game_detail_state.is_loading {
        Box::new(bsn_list![(
            LoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else if let Some(ref error) = game_detail_state.error {
        let error_text = format!("加载失败: {}", error);
        Box::new(bsn_list![(
            ErrorMessage
            Text({error_text})
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::ERROR)
        )])
    } else if let Some(ref game) = game_detail_state.game {
        game_detail_content(game, image_cache)
    } else {
        Box::new(bsn_list![(
            Text("暂无数据")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    };

    bsn! {
        GameDetailRoot
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
                    column_gap: Val::Px(10.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // 返回按钮
                        GameDetailBackButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_CHEVRON_LEFT)
                                TextFont { font_size: FontSize::Px(20.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        Text("游戏详情")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
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
                        // 可滚动内容区域
                        #GameDetailScroll
                        GameDetailScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: {scroll_padding},
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [
                            {content},
                            (
                                // 底部间距
                                Node {
                                    height: Val::Px(30.0),
                                    min_height: Val::Px(30.0),
                                }
                            ),
                        ]
                    ),
                    // 滚动条
                    scrollbar(#GameDetailScroll),
                ]
            ),
        ]
    }
}

/// 游戏详情内容（滚动区内的并列区块）
fn game_detail_content(
    game: &picacg_api::models::Game,
    image_cache: &ImageCache,
) -> Box<dyn SceneList> {
    let mut blocks: Vec<Box<dyn Scene>> = Vec::new();

    // 基本信息区域（图标 + 基本信息）
    blocks.push(Box::new(game_basic_info(game, image_cache)));

    // 分隔线
    blocks.push(Box::new(divider()));

    // 描述区域
    blocks.push(Box::new(section("简介", &game.description)));

    // 更新内容
    if let Some(ref update_content) = game.update_content
        && !update_content.is_empty()
    {
        blocks.push(Box::new(section("更新内容", update_content)));
    }

    // 下载链接区域
    let has_android = game.android_link.is_some()
        || game
            .android_links
            .as_ref()
            .is_some_and(|links| !links.is_empty());
    let has_ios = game.ios_link.is_some()
        || game
            .ios_links
            .as_ref()
            .is_some_and(|links| !links.is_empty());

    if has_android || has_ios {
        blocks.push(Box::new(divider()));
        blocks.push(Box::new(download_links_section(game)));
    }

    // 截图区域
    if let Some(ref screenshots) = game.screenshots
        && !screenshots.is_empty()
    {
        blocks.push(Box::new(divider()));
        blocks.push(Box::new(screenshots_section(screenshots, image_cache)));
    }

    Box::new(blocks)
}

/// 基本信息区域场景（图标 + 标题 / 开发者 / 版本 / 互动数据）
fn game_basic_info(
    game: &picacg_api::models::Game,
    image_cache: &ImageCache,
) -> impl Scene + use<> {
    let icon_url = game.icon.url();
    let title = game.title.clone();

    // 发布者 / 版本
    let mut rows: Vec<Box<dyn Scene>> = Vec::new();
    if let Some(ref publisher) = game.publisher {
        rows.push(Box::new(info_row("开发者", publisher)));
    }
    if let Some(ref version) = game.version {
        rows.push(Box::new(info_row("版本", version)));
    }

    // 互动数据
    let mut stats: Vec<Box<dyn Scene>> = Vec::new();
    if let Some(likes) = game.likes_count {
        stats.push(Box::new(stat_badge(ICON_HEART, &format!("{}", likes))));
    }
    if let Some(comments) = game.comments_count {
        stats.push(Box::new(stat_badge(ICON_CHAT, &format!("{}", comments))));
    }

    bsn! {
        Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(20.0)),
        }
        Children [
            // 游戏图标
            game_icon(icon_url, image_cache),
            (
                // 基本信息
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(8.0),
                }
                Children [
                    (
                        // 标题
                        Text({title})
                        TextFont { font_size: FontSize::Px(22.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    {rows},
                    (
                        // 互动数据
                        Node {
                            column_gap: Val::Px(15.0),
                            margin: UiRect::top(Val::Px(4.0)),
                        }
                        Children [ {stats} ]
                    ),
                ]
            ),
        ]
    }
}

/// 游戏图标场景（缓存命中显示图片，否则显示占位图标）
fn game_icon(url: String, image_cache: &ImageCache) -> impl Scene + use<> {
    // 尝试从缓存加载图标
    let inner: Box<dyn SceneList> = match image_cache.get(&url) {
        Some(handle) => {
            let handle = handle.clone();
            Box::new(bsn_list![(
                ImageNode { image: {handle} }
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(20.0)),
                }
            )])
        }
        None => Box::new(bsn_list![(
            Text(ICON_GAMEPAD)
            TextFont { font_size: FontSize::Px(48.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )]),
    };

    bsn! {
        GameDetailIcon { url: {url} }
        Node {
            width: Val::Px(120.0),
            height: Val::Px(120.0),
            min_width: Val::Px(120.0),
            min_height: Val::Px(120.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(20.0)),
        }
        BackgroundColor(AppColors::SURFACE)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [ {inner} ]
    }
}

/// 分隔线场景
fn divider() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(10.0)),
        }
        BackgroundColor(AppColors::BORDER)
    }
}

/// 信息行场景（标签 + 值）
fn info_row(label: &str, value: &str) -> impl Scene + use<> {
    let label = format!("{}:", label);
    let value = value.to_string();

    bsn! {
        Node {
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
        }
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                Text({value})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            ),
        ]
    }
}

/// 统计徽章场景
fn stat_badge(icon: &str, value: &str) -> impl Scene + use<> {
    let icon = icon.to_string();
    let value = value.to_string();

    bsn! {
        Node {
            column_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
        }
        Children [
            (
                Text({icon})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::PRIMARY)
            ),
            (
                Text({value})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// 内容段落场景（标题 + 正文）
fn section(title: &str, body: &str) -> impl Scene + use<> {
    let title = title.to_string();
    let body = body.to_string();

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            row_gap: Val::Px(8.0),
            margin: UiRect::bottom(Val::Px(15.0)),
        }
        Children [
            (
                Text({title})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                Text({body})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// 下载链接区域场景
fn download_links_section(game: &picacg_api::models::Game) -> impl Scene + use<> {
    let mut rows: Vec<Box<dyn Scene>> = Vec::new();

    // Android 链接
    if let Some(ref link) = game.android_link {
        rows.push(Box::new(link_row("Android", link)));
    }
    if let Some(ref links) = game.android_links {
        for (i, link) in links.iter().enumerate() {
            let label = if links.len() > 1 {
                format!("Android {}", i + 1)
            } else {
                "Android".to_string()
            };
            rows.push(Box::new(link_row(&label, link)));
        }
    }

    // iOS 链接
    if let Some(ref link) = game.ios_link {
        rows.push(Box::new(link_row("iOS", link)));
    }
    if let Some(ref links) = game.ios_links {
        for (i, link) in links.iter().enumerate() {
            let label = if links.len() > 1 {
                format!("iOS {}", i + 1)
            } else {
                "iOS".to_string()
            };
            rows.push(Box::new(link_row(&label, link)));
        }
    }

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            row_gap: Val::Px(8.0),
            margin: UiRect::bottom(Val::Px(15.0)),
        }
        Children [
            (
                Text("下载链接")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::bottom(Val::Px(8.0)) }
            ),
            {rows},
        ]
    }
}

/// 链接行场景
fn link_row(platform: &str, url: &str) -> impl Scene + use<> {
    let platform = platform.to_string();
    // URL（截断显示；按字符计，避免非 ASCII URL 切在字节边界上 panic）
    let display_url = truncate_text(url, 50);

    bsn! {
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(8.0)),
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        BackgroundColor(AppColors::SURFACE_SUNKEN)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 平台标签
                Node {
                    padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(2.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(AppColors::PRIMARY)
                Children [
                    (
                        Text({platform})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(Color::WHITE)
                    )
                ]
            ),
            (
                Text({display_url})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { flex_shrink: 1.0 }
            ),
        ]
    }
}

/// 截图区域场景
fn screenshots_section(
    screenshots: &[picacg_api::models::ImageInfo],
    image_cache: &ImageCache,
) -> impl Scene + use<> {
    let title = format!("截图 ({})", screenshots.len());
    let items: Vec<_> = screenshots
        .iter()
        .map(|screenshot| screenshot_item(screenshot.url(), image_cache))
        .collect();

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                Text({title})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::bottom(Val::Px(8.0)) }
            ),
            (
                // 截图列表（横向滚动）
                Node {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(10.0),
                    overflow: Overflow::scroll_x(),
                }
                Children [ {items} ]
            ),
        ]
    }
}

/// 单张截图场景（缓存命中显示图片，否则显示等待图标）
fn screenshot_item(url: String, image_cache: &ImageCache) -> impl Scene + use<> {
    let inner: Box<dyn SceneList> = match image_cache.get(&url) {
        Some(handle) => {
            let handle = handle.clone();
            Box::new(bsn_list![(
                ImageNode { image: {handle} }
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                }
            )])
        }
        None => Box::new(bsn_list![(
            Text(ICON_TIMER_SAND)
            TextFont { font_size: FontSize::Px(24.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )]),
    };

    bsn! {
        GameDetailIcon { url: {url} }
        Node {
            width: Val::Px(200.0),
            height: Val::Px(150.0),
            min_width: Val::Px(200.0),
            min_height: Val::Px(150.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(AppColors::SURFACE)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [ {inner} ]
    }
}

// ==================== 系统函数 ====================

/// 创建游戏详情界面
pub fn setup_game_detail_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    game_detail_state: Res<GameDetailState>,
    image_cache: Res<ImageCache>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_detail_messages: MessageWriter<LoadGameDetailRequest>,
    existing_query: Query<Entity, With<GameDetailRoot>>,
) {
    // 每次进入游戏详情页面都销毁旧的重建（不同游戏的数据不同，不适合缓存）
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    let content_area = content_area_query.single().ok();

    let detail_root = commands
        .spawn_scene(game_detail_page(&game_detail_state, &image_cache))
        .id();

    // 挂载到内容区域
    if let Some(content_area) = content_area {
        commands.entity(content_area).add_child(detail_root);
    }

    // 如果数据未加载，触发加载
    if !game_detail_state.game_id.is_empty()
        && game_detail_state.game.is_none()
        && !game_detail_state.is_loading
    {
        load_detail_messages.write(LoadGameDetailRequest {
            game_id: game_detail_state.game_id.clone(),
        });
    }
}

/// 清理游戏详情界面（销毁 UI，参数化页面不适合缓存）
pub fn cleanup_game_detail_ui(mut commands: Commands, query: Query<Entity, With<GameDetailRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 刷新游戏详情 UI（despawn 旧根节点并重建）
pub fn refresh_game_detail_ui(
    mut commands: Commands,
    game_detail_state: Res<GameDetailState>,
    detail_root_query: Query<Entity, With<GameDetailRoot>>,
    content_area_query: Query<Entity, With<ContentArea>>,
    _asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    mut load_detail_messages: MessageWriter<LoadGameDetailRequest>,
) {
    if !game_detail_state.is_changed() {
        return;
    }

    // 如果正在加载且没数据也没错误，暂不重建
    if game_detail_state.is_loading
        && game_detail_state.game.is_none()
        && game_detail_state.error.is_none()
    {
        return;
    }

    // 删除旧 UI
    for entity in detail_root_query.iter() {
        commands.entity(entity).despawn();
    }

    // 重建（直接内联创建逻辑，不调用 setup 避免 deferred despawn 导致
    // existing_query 误判）
    let content_area = content_area_query.single().ok();
    let detail_root = commands
        .spawn_scene(game_detail_page(&game_detail_state, &image_cache))
        .id();

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(detail_root);
    }

    // 如果数据未加载，触发加载
    if !game_detail_state.game_id.is_empty()
        && game_detail_state.game.is_none()
        && !game_detail_state.is_loading
    {
        load_detail_messages.write(LoadGameDetailRequest {
            game_id: game_detail_state.game_id.clone(),
        });
    }
}

/// 返回按钮交互
pub fn game_detail_back_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<GameDetailBackButton>)>,
    mut back_messages: MessageWriter<NavigateBackEvent>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            back_messages.write(NavigateBackEvent);
        }
    }
}

/// 更新游戏详情图标和截图图片（图片就绪后就地挂 `ImageNode`）
///
/// `Without<ImageNode>` 让换过图的实体永久退出扫描集，取代原先"背景色 !=
/// NONE"的占位哨兵；加载失败的图标摘掉标记组件，同样不再每帧重扫。
/// 图片直接挂在占位实体上，圆角随各自容器（图标 20px / 截图 8px），
/// 与首次渲染路径一致。
pub fn update_game_detail_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    icon_query: Query<(Entity, &GameDetailIcon, Option<&Children>), Without<ImageNode>>,
) {
    for (entity, icon, children) in icon_query.iter() {
        if image_cache.is_failed(&icon.url) {
            commands.entity(entity).remove::<GameDetailIcon>();
            continue;
        }

        let Some(handle) = image_cache.get(&icon.url) else {
            continue;
        };

        // 清除占位子节点（占位图标文字）
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        commands.entity(entity).insert((
            ImageNode {
                image: handle.clone(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));
    }
}

/// 处理游戏详情加载完成事件
pub fn handle_game_detail_loaded(
    mut loaded_messages: MessageReader<GameDetailLoadedEvent>,
    mut game_detail_state: ResMut<GameDetailState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
    image_cache: Res<ImageCache>,
) {
    for event in loaded_messages.read() {
        tracing::info!("游戏详情加载完成: {}", event.game.title);

        // 触发加载图标（已有状态的 URL 不再重复请求）
        let icon_url = event.game.icon.url();
        if !image_cache.is_known(&icon_url) {
            image_messages.write(LoadImageRequest { url: icon_url });
        }

        // 触发加载截图
        if let Some(ref screenshots) = event.game.screenshots {
            for screenshot in screenshots {
                let url = screenshot.url();
                if !image_cache.is_known(&url) {
                    image_messages.write(LoadImageRequest { url });
                }
            }
        }

        game_detail_state.game = Some(event.game.clone());
        game_detail_state.is_loading = false;
        game_detail_state.error = None;
    }
}

/// 处理游戏详情加载失败事件
pub fn handle_game_detail_load_failed(
    mut failed_messages: MessageReader<GameDetailLoadFailedEvent>,
    mut game_detail_state: ResMut<GameDetailState>,
) {
    for event in failed_messages.read() {
        game_detail_state.is_loading = false;
        game_detail_state.error = Some(event.error.clone());
        tracing::warn!("游戏详情加载失败: {}", event.error);
    }
}
