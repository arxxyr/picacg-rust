//! 导航系统
//!
//! 支持浏览器风格的前进/后退导航

#![allow(dead_code)]

use bevy::prelude::*;

use crate::{
    events::*,
    resources::{AppRoute, ComicDetailState, ComicsListState},
};

/// 导航历史记录（支持前进后退）
#[derive(Resource, Default)]
pub struct NavigationHistory {
    /// 后退栈：存储已访问过的页面
    pub back_stack: Vec<AppRoute>,
    /// 前进栈：存储后退后的页面
    pub forward_stack: Vec<AppRoute>,
}

impl NavigationHistory {
    /// 导航到新页面（清空前进栈）
    pub fn push(&mut self, route: AppRoute) {
        self.back_stack.push(route);
        self.forward_stack.clear();
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

    /// 能否后退
    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    /// 能否前进
    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }
}

/// 处理导航消息
pub fn handle_navigation_messages(
    current_route: Res<State<AppRoute>>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
    mut comics_state: ResMut<ComicsListState>,
    mut detail_state: ResMut<ComicDetailState>,
    // 导航消息
    mut categories_events: MessageReader<NavigateToCategoriesEvent>,
    mut comics_events: MessageReader<NavigateToComicsListEvent>,
    mut detail_events: MessageReader<NavigateToComicDetailEvent>,
    mut reader_events: MessageReader<NavigateToReaderEvent>,
    mut proxy_events: MessageReader<NavigateToProxySettingsEvent>,
    mut back_events: MessageReader<NavigateBackEvent>,
    mut forward_events: MessageReader<NavigateForwardEvent>,
    mut login_events: MessageReader<NavigateToLoginEvent>,
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
        comics_state.error = None;
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
        // TODO: 设置阅读器状态
        let _ = event;
        next_route.set(AppRoute::ReadView);
    }

    // 处理导航到代理设置
    for _ in proxy_events.read() {
        history.push(current.clone());
        next_route.set(AppRoute::ProxySettings);
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
