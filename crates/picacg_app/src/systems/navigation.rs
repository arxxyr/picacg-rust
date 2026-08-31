//! 导航系统
//!
//! 支持浏览器风格的前进/后退导航

use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    components::SidebarRoute,
    events::*,
    resources::{AppRoute, ComicDetailState, ComicsListState, GameDetailState, ReaderState},
};

/// 导航历史记录（支持前进后退）
#[derive(Resource, Default)]
pub struct NavigationHistory {
    /// 后退栈：存储已访问过的页面
    pub back_stack: Vec<AppRoute>,
    /// 前进栈：存储后退后的页面
    pub forward_stack: Vec<AppRoute>,
    /// 每个侧边栏分区最后访问的路由
    pub section_last_routes: HashMap<SidebarRoute, AppRoute>,
}

/// 获取 AppRoute 对应的 SidebarRoute
///
/// 注意：ComicDetail 和 ReadView 返回 None，因为它们可以从多个入口进入
/// （搜索、收藏、排行榜、首页、分类等），不应该归属于任何固定的侧边栏分区
pub fn get_sidebar_route(route: &AppRoute) -> Option<SidebarRoute> {
    match route {
        AppRoute::Home => Some(SidebarRoute::Home),
        AppRoute::Categories | AppRoute::ComicsList => Some(SidebarRoute::Categories),
        AppRoute::Search => Some(SidebarRoute::Search),
        AppRoute::Rankings => Some(SidebarRoute::Rankings),
        AppRoute::Games | AppRoute::GameDetail => Some(SidebarRoute::Games),
        AppRoute::Fried => Some(SidebarRoute::Fried),
        AppRoute::Favorites => Some(SidebarRoute::Favorites),
        AppRoute::History => Some(SidebarRoute::History),
        AppRoute::LikeRecords => Some(SidebarRoute::LikeRecords),
        AppRoute::Profile => Some(SidebarRoute::Profile),
        AppRoute::LocalRead => Some(SidebarRoute::LocalRead),
        AppRoute::Downloads => Some(SidebarRoute::Downloads),
        AppRoute::Settings => Some(SidebarRoute::Settings),
        AppRoute::ImageConvert => Some(SidebarRoute::ImageConvert),
        AppRoute::Waifu2x => Some(SidebarRoute::Waifu2x),
        AppRoute::Chat | AppRoute::ChatRoom => Some(SidebarRoute::Chat),
        AppRoute::Nas => Some(SidebarRoute::Nas),
        // ComicDetail/ReadView 可从多个入口进入，不归属于任何分区
        // Login/Register/ProxySettings 不属于主导航
        AppRoute::ComicDetail
        | AppRoute::ReadView
        | AppRoute::Login
        | AppRoute::Register
        | AppRoute::ProxySettings
        | AppRoute::Comments
        | AppRoute::ForgotPassword => None,
    }
}

/// 获取 SidebarRoute 的默认 AppRoute
pub fn get_default_route(sidebar_route: SidebarRoute) -> AppRoute {
    match sidebar_route {
        SidebarRoute::Home => AppRoute::Home,
        SidebarRoute::Categories => AppRoute::Categories,
        SidebarRoute::Search => AppRoute::Search,
        SidebarRoute::Rankings => AppRoute::Rankings,
        SidebarRoute::Games => AppRoute::Games,
        SidebarRoute::Favorites => AppRoute::Favorites,
        SidebarRoute::History => AppRoute::History,
        SidebarRoute::Profile => AppRoute::Profile,
        SidebarRoute::LocalRead => AppRoute::LocalRead,
        SidebarRoute::LikeRecords => AppRoute::LikeRecords,
        SidebarRoute::Downloads => AppRoute::Downloads,
        SidebarRoute::Settings => AppRoute::Settings,
        SidebarRoute::Fried => AppRoute::Fried,
        SidebarRoute::ImageConvert => AppRoute::ImageConvert,
        SidebarRoute::Waifu2x => AppRoute::Waifu2x,
        SidebarRoute::Chat => AppRoute::Chat,
        SidebarRoute::Nas => AppRoute::Nas,
    }
}

impl NavigationHistory {
    /// 导航到新页面（清空前进栈）
    pub fn push(&mut self, route: AppRoute) {
        self.back_stack.push(route);
        self.forward_stack.clear();
    }

    /// 更新某个侧边栏分区的最后访问路由
    pub fn update_section_route(&mut self, route: &AppRoute) {
        if let Some(sidebar_route) = get_sidebar_route(route) {
            self.section_last_routes
                .insert(sidebar_route, route.clone());
        }
    }

    /// 获取某个侧边栏分区的最后访问路由
    pub fn get_section_route(&self, sidebar_route: SidebarRoute) -> AppRoute {
        self.section_last_routes
            .get(&sidebar_route)
            .cloned()
            .unwrap_or_else(|| get_default_route(sidebar_route))
    }

    /// 后退导航
    pub fn go_back(&mut self, current: AppRoute) -> Option<AppRoute> {
        if let Some(prev) = self.back_stack.pop() {
            self.forward_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    /// 前进导航
    pub fn go_forward(&mut self, current: AppRoute) -> Option<AppRoute> {
        if let Some(next) = self.forward_stack.pop() {
            self.back_stack.push(current);
            Some(next)
        } else {
            None
        }
    }
}

/// 处理导航消息
pub fn handle_navigation_messages(
    current_route: Res<State<AppRoute>>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
    mut comics_state: ResMut<ComicsListState>,
    mut detail_state: ResMut<ComicDetailState>,
    mut reader_state: ResMut<ReaderState>,
    mut game_detail_state: ResMut<GameDetailState>,
    // 导航消息
    mut categories_events: MessageReader<NavigateToCategoriesEvent>,
    mut comics_events: MessageReader<NavigateToComicsListEvent>,
    mut detail_events: MessageReader<NavigateToComicDetailEvent>,
    mut reader_events: MessageReader<NavigateToReaderEvent>,
    mut proxy_events: MessageReader<NavigateToProxySettingsEvent>,
    mut back_events: MessageReader<NavigateBackEvent>,
    mut forward_events: MessageReader<NavigateForwardEvent>,
    mut login_events: MessageReader<NavigateToLoginEvent>,
    mut game_detail_events: MessageReader<NavigateToGameDetailEvent>,
) {
    let current = current_route.get().clone();

    // 处理导航到分类页面
    for _ in categories_events.read() {
        if current != AppRoute::Categories {
            history.push(current.clone());
            next_route.set(AppRoute::Categories);
        }
    }

    // 处理导航到漫画列表
    for event in comics_events.read() {
        history.push(current.clone());
        comics_state.category = event.category.clone();
        comics_state.comics.clear();
        comics_state.page = 1;
        comics_state.total_pages = 0;
        comics_state.is_loading = false;
        comics_state.is_loading_more = false;
        comics_state.error = None;
        comics_state.scroll_y = 0.0;
        next_route.set(AppRoute::ComicsList);
    }

    // 处理导航到漫画详情
    for event in detail_events.read() {
        history.push(current.clone());
        detail_state.comic_id = event.comic_id.clone();
        detail_state.comic = None;
        detail_state.episodes.clear();
        detail_state.is_loading = false;
        detail_state.error = None;
        next_route.set(AppRoute::ComicDetail);
    }

    // 处理导航到阅读界面
    for event in reader_events.read() {
        history.push(current.clone());

        // 从 ComicDetailState 获取漫画标题和章节列表
        let comic_title = detail_state
            .comic
            .as_ref()
            .map(|c| c.title.clone())
            .unwrap_or_default();

        // 复制章节列表并按 order 升序排序
        let mut episodes = detail_state.episodes.clone();
        episodes.sort_by_key(|ep| ep.order);

        // 查找当前章节在排序后列表中的索引
        let current_episode_idx = episodes
            .iter()
            .position(|ep| ep.order == event.episode_order)
            .unwrap_or(0);

        // 设置阅读器状态
        reader_state.comic_id = event.comic_id.clone();
        reader_state.comic_title = comic_title;
        reader_state.episodes = episodes;
        reader_state.current_episode_idx = current_episode_idx;
        reader_state.episode_order = event.episode_order;
        reader_state.current_page = 0;
        reader_state.total_pages = 0;
        reader_state.pictures.clear();
        reader_state.page_metas.clear();
        reader_state.is_loading = false;
        reader_state.error = None;
        reader_state.is_loading_next_chapter = false;
        reader_state.next_chapter_pictures.clear();
        reader_state.is_loading_all_chapters = false;
        next_route.set(AppRoute::ReadView);
    }

    // 处理导航到代理设置
    for _ in proxy_events.read() {
        history.push(current.clone());
        next_route.set(AppRoute::ProxySettings);
    }

    // 处理导航到游戏详情
    for event in game_detail_events.read() {
        history.push(current.clone());
        game_detail_state.game_id = event.game_id.clone();
        game_detail_state.game = None;
        game_detail_state.is_loading = false;
        game_detail_state.error = None;
        next_route.set(AppRoute::GameDetail);
    }

    // 处理后退导航
    for _ in back_events.read() {
        if let Some(prev) = history.go_back(current.clone()) {
            next_route.set(prev);
        } else {
            // 后退栈为空，使用默认返回逻辑
            match &current {
                AppRoute::ComicDetail => next_route.set(AppRoute::ComicsList),
                AppRoute::ComicsList => next_route.set(AppRoute::Categories),
                AppRoute::ReadView => next_route.set(AppRoute::ComicDetail),
                AppRoute::ProxySettings => next_route.set(AppRoute::Login),
                AppRoute::GameDetail => next_route.set(AppRoute::Games),
                _ => {}
            }
        }
    }

    // 处理前进导航
    for _ in forward_events.read() {
        if let Some(next) = history.go_forward(current.clone()) {
            next_route.set(next);
        }
    }

    // 处理返回登录页
    for _ in login_events.read() {
        // 返回登录页时清空导航历史
        history.back_stack.clear();
        history.forward_stack.clear();
        next_route.set(AppRoute::Login);
    }
}

/// 处理键盘导航快捷键
pub fn handle_back_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut back_events: MessageWriter<NavigateBackEvent>,
    mut forward_events: MessageWriter<NavigateForwardEvent>,
) {
    // Escape 键后退
    if keyboard_input.just_pressed(KeyCode::Escape) {
        back_events.write(NavigateBackEvent);
    }

    // Alt+Left 后退，Alt+Right 前进（类似浏览器）
    let alt_pressed =
        keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight);

    if alt_pressed {
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            back_events.write(NavigateBackEvent);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            forward_events.write(NavigateForwardEvent);
        }
    }
}

/// 追踪路由变化，更新各分区的最后访问路由
pub fn track_route_changes(
    current_route: Res<State<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
) {
    if current_route.is_changed() {
        history.update_section_route(current_route.get());
    }
}
