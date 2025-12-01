//! 导航系统

use bevy::prelude::*;

use crate::resources::AppRoute;

/// 导航历史栈
#[derive(Resource, Default)]
pub struct NavigationHistory {
    pub stack: Vec<AppRoute>,
}

/// 处理返回导航
pub fn handle_back_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_route: Res<State<AppRoute>>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // 从历史栈弹出上一个路由
        if let Some(prev_route) = history.stack.pop() {
            next_route.set(prev_route);
        } else {
            // 如果历史栈为空，根据当前路由决定返回目标
            match current_route.get() {
                AppRoute::ComicDetail => next_route.set(AppRoute::ComicsList),
                AppRoute::ComicsList => next_route.set(AppRoute::Categories),
                AppRoute::ReadView => next_route.set(AppRoute::ComicDetail),
                AppRoute::ProxySettings => next_route.set(AppRoute::Login),
                _ => {}
            }
        }
    }
}
