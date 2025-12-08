//! 导航消息
//!
//! 定义页面导航相关的消息 (Bevy 0.17 使用 Message)

use bevy::prelude::*;

/// 导航到分类页面
#[derive(Message)]
pub struct NavigateToCategoriesEvent;

/// 导航到漫画列表
#[derive(Message)]
pub struct NavigateToComicsListEvent {
    pub category: String,
}

/// 导航到漫画详情
#[derive(Message)]
pub struct NavigateToComicDetailEvent {
    pub comic_id: String,
}

/// 导航到阅读界面
#[derive(Message)]
pub struct NavigateToReaderEvent {
    pub comic_id: String,
    pub episode_order: i32,
}

/// 导航到代理设置
#[derive(Message)]
pub struct NavigateToProxySettingsEvent;

/// 返回上一页（后退）
#[derive(Message)]
pub struct NavigateBackEvent;

/// 前进到下一页
#[derive(Message)]
pub struct NavigateForwardEvent;

/// 返回登录页
#[derive(Message)]
pub struct NavigateToLoginEvent;
