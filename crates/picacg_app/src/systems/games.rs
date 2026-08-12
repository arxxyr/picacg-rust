//! 游戏列表系统
//!
//! 实现游戏区页面，展示可用的游戏列表

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        pagination::{Pagination, PaginationControl, pagination_controls},
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        ui_common::truncate_text,
        widgets::ButtonStyle,
    },
    utils::icons::*,
};

/// 游戏页面标记类型（用于分页组件的泛型参数）
pub struct GamesPage;

// ==================== 组件定义 ====================

/// 游戏列表根节点
#[derive(Component, Default, Clone)]
pub struct GamesRoot;

/// 游戏列表滚动容器
#[derive(Component, Default, Clone)]
pub struct GamesScrollContainer;

/// 游戏卡片
#[derive(Component, Default, Clone)]
pub struct GameCard {
    pub game_id: String,
}

/// 游戏图标缩略图（占位实体自带 URL，图片就绪后就地换成 `ImageNode`）
#[derive(Component, Default, Clone)]
pub struct GameIconThumbnail {
    pub url: String,
}

// ==================== 布局常量 ====================

mod games_layout {
    /// 卡片高度
    pub const CARD_HEIGHT: f32 = 100.0;
    /// 卡片间距
    pub const CARD_GAP: f32 = 10.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 15.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
    /// 图标宽度
    pub const ICON_SIZE: f32 = 72.0;
}

// ==================== 系统函数 ====================

/// 创建游戏列表界面（如果已存在则只显示）
pub fn setup_games_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    games_state: Res<GamesState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_games_messages: MessageWriter<LoadGamesRequest>,
    mut existing_query: Query<&mut Node, With<GamesRoot>>,
) {
    // 如果 GamesRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if games_state.games.is_empty() && !games_state.is_loading && games_state.error.is_none() {
            load_games_messages.write(LoadGamesRequest {
                page: games_state.page.max(1),
            });
        }
        return;
    }

    let content_area = content_area_query.single().ok();

    let games_root = commands.spawn_scene(games_page(&games_state)).id();

    // 挂载到内容区域
    if let Some(content_area) = content_area {
        commands.entity(content_area).add_child(games_root);
    }

    // 如果数据未加载，触发加载
    if games_state.games.is_empty() && !games_state.is_loading && games_state.error.is_none() {
        load_games_messages.write(LoadGamesRequest {
            page: games_state.page.max(1),
        });
    }
}

/// 游戏列表页面场景
fn games_page(games_state: &GamesState) -> impl Scene + use<> {
    let scroll_padding = UiRect {
        left: Val::Px(games_layout::PADDING_LEFT),
        right: Val::Px(games_layout::PADDING_RIGHT),
        top: Val::Px(games_layout::PADDING_TOP),
        bottom: Val::Px(games_layout::PADDING_BOTTOM),
    };
    let content = games_content(games_state);

    bsn! {
        GamesRoot
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
                        // 图标
                        Text(ICON_GAMEPAD)
                        TextFont { font_size: FontSize::Px(20.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                    (
                        Text("游戏区")
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
                        #GamesScroll
                        GamesScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: {scroll_padding},
                            row_gap: Val::Px(games_layout::CARD_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [ {content} ]
                    ),
                    // 滚动条
                    scrollbar(#GamesScroll),
                ]
            ),
        ]
    }
}

/// 滚动容器内的内容（加载中 / 错误 / 空 / 游戏列表 + 分页，末尾附底部间距）
///
/// `setup_games_ui` 与 `refresh_games_ui` 共用，保证首次创建与刷新结构一致。
fn games_content(games_state: &GamesState) -> Vec<Box<dyn Scene>> {
    let mut items: Vec<Box<dyn Scene>> = Vec::new();

    if games_state.is_loading {
        items.push(Box::new(bsn! {
            LoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    } else if let Some(ref error) = games_state.error {
        let message = format!("加载失败: {}", error);
        items.push(Box::new(bsn! {
            ErrorMessage
            Text({message})
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::ERROR)
        }));
    } else if games_state.games.is_empty() {
        items.push(Box::new(bsn! {
            Text("暂无游戏")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    } else {
        // 显示游戏列表
        for game in &games_state.games {
            items.push(Box::new(game_card(game)));
        }

        // 分页控件（共享控件，翻页行为内联在控件观察者里）
        // page 默认为 0（首屏实际展示的是第 1 页），这里统一按 1 起算
        if games_state.total_pages > 1 {
            items.push(Box::new(pagination_controls::<GamesPage>(
                games_state.page.max(1) as u32,
                games_state.total_pages.max(0) as u32,
            )));
        }
    }

    // 底部间距
    items.push(Box::new(bsn! {
        Node {
            height: Val::Px(30.0),
            min_height: Val::Px(30.0),
        }
    }));

    items
}

/// 单个游戏卡片场景
fn game_card(game: &picacg_api::models::Game) -> impl Scene + use<> {
    let game_id = game.id.clone();
    let icon_url = game.icon.url();
    let title = game.title.clone();

    // 描述（截取前 60 个字符）
    let desc = truncate_text(&game.description, 60);

    // 发布者信息（没有发布者时为空列表）
    let publisher_row: Box<dyn SceneList> = match game.publisher {
        Some(ref publisher) => {
            let publisher = publisher.clone();
            Box::new(bsn_list![(
                Node {
                    column_gap: Val::Px(6.0),
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        Text("开发者:")
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        Text({publisher})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                ]
            )])
        }
        None => Box::new(bsn_list![]),
    };

    bsn! {
        GameCard { game_id: {game_id} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(games_layout::CARD_HEIGHT),
            padding: UiRect::all(Val::Px(12.0)),
            column_gap: Val::Px(12.0),
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(AppColors::SURFACE)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 游戏图标占位
                GameIconThumbnail { url: {icon_url} }
                Node {
                    width: Val::Px(games_layout::ICON_SIZE),
                    height: Val::Px(games_layout::ICON_SIZE),
                    min_width: Val::Px(games_layout::ICON_SIZE),
                    min_height: Val::Px(games_layout::ICON_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                }
                BackgroundColor(Color::srgb(0.18, 0.18, 0.22))
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        Text(ICON_GAMEPAD)
                        TextFont { font_size: FontSize::Px(28.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    )
                ]
            ),
            (
                // 文字信息区域
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(4.0),
                    overflow: Overflow::clip(),
                }
                Children [
                    (
                        // 标题
                        Text({title})
                        TextFont { font_size: FontSize::Px(15.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 描述
                        Text({desc})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    {publisher_row},
                ]
            ),
            (
                // 右侧箭头
                Text(ICON_CHEVRON_RIGHT)
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// 清理游戏列表界面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_games_ui(mut query: Query<&mut Node, With<GamesRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新游戏列表 UI（数据变化时重建滚动容器内容）
pub fn refresh_games_ui(
    mut commands: Commands,
    games_state: Res<GamesState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<GamesScrollContainer>>,
) {
    if !games_state.is_changed() {
        return;
    }

    // 只在有数据/错误变化时才重建（跳过仅 is_loading 变化的场景）
    let has_data = !games_state.games.is_empty();
    let has_error = games_state.error.is_some();
    let is_loading = games_state.is_loading;

    if is_loading && !has_data && !has_error {
        return;
    }

    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 清除滚动容器内的所有子元素
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // 重建滚动容器内容
    for scene in games_content(&games_state) {
        commands.spawn_scene(scene).insert(ChildOf(scroll_entity));
    }
}

/// 游戏卡片点击交互：导航到游戏详情
pub fn game_card_interaction(
    interaction_query: Query<(&Interaction, &GameCard), Changed<Interaction>>,
    mut navigate_messages: MessageWriter<NavigateToGameDetailEvent>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction == Interaction::Pressed {
            tracing::info!("点击游戏卡片: game_id={}", card.game_id);
            navigate_messages.write(NavigateToGameDetailEvent {
                game_id: card.game_id.clone(),
            });
        }
    }
}

/// 消费分页控件状态变化（翻页边界与按钮行为已内联在控件观察者里）
pub fn games_pagination_changed(
    pagination_query: Query<&Pagination, (With<PaginationControl<GamesPage>>, Changed<Pagination>)>,
    mut games_state: ResMut<GamesState>,
    mut load_messages: MessageWriter<LoadGamesRequest>,
) {
    let Ok(pagination) = pagination_query.single() else {
        return;
    };
    // 只响应真实翻页（控件重建后的同值回填在此被过滤）
    let new_page = pagination.current_page as i32;
    if new_page == games_state.page.max(1) {
        return;
    }

    games_state.page = new_page;
    games_state.games.clear();
    games_state.is_loading = true;
    games_state.error = None;
    load_messages.write(LoadGamesRequest { page: new_page });

    tracing::debug!("切换到游戏列表第 {} 页", new_page);
}

/// 更新游戏图标图片（图片就绪后就地挂 `ImageNode`）
///
/// `Without<ImageNode>` 让换过图的实体永久退出扫描集，取代原先"背景色 !=
/// NONE"的占位哨兵；加载失败的图标摘掉标记组件，同样不再每帧重扫。
pub fn update_games_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    icon_query: Query<(Entity, &GameIconThumbnail, Option<&Children>), Without<ImageNode>>,
) {
    for (entity, thumb, children) in icon_query.iter() {
        if image_cache.is_failed(&thumb.url) {
            commands.entity(entity).remove::<GameIconThumbnail>();
            continue;
        }

        let Some(handle) = image_cache.get(&thumb.url) else {
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

/// 处理游戏列表加载完成事件
pub fn handle_games_loaded(
    mut loaded_messages: MessageReader<GamesLoadedEvent>,
    mut games_state: ResMut<GamesState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
    image_cache: Res<ImageCache>,
) {
    for event in loaded_messages.read() {
        games_state.games = event.games.clone();
        games_state.total_pages = event.total_pages;
        games_state.is_loading = false;
        games_state.error = None;
        tracing::info!(
            "游戏列表加载完成: {} 个, 共 {} 页",
            games_state.games.len(),
            games_state.total_pages
        );

        // 触发加载游戏图标（已有状态的 URL 不再重复请求）
        for game in &games_state.games {
            let url = game.icon.url();
            if !image_cache.is_known(&url) {
                image_messages.write(LoadImageRequest { url });
            }
        }
    }
}

/// 处理游戏列表加载失败事件
pub fn handle_games_load_failed(
    mut failed_messages: MessageReader<GamesLoadFailedEvent>,
    mut games_state: ResMut<GamesState>,
) {
    for event in failed_messages.read() {
        games_state.is_loading = false;
        games_state.error = Some(event.error.clone());
        tracing::warn!("游戏列表加载失败: {}", event.error);
    }
}
