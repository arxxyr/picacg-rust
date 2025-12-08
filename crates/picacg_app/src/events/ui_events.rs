//! UI 交互消息
//!
//! 定义用户界面交互相关的消息 (Bevy 0.17 使用 Message)

use bevy::prelude::*;

use crate::resources::ReadMode;

// ==================== 表单输入消息 ====================

/// 用户名输入变化
#[derive(Message)]
pub struct EmailChangedEvent(pub String);

/// 密码输入变化
#[derive(Message)]
pub struct PasswordChangedEvent(pub String);

// ==================== 分页消息 ====================

/// 上一页
#[derive(Message)]
pub struct PrevPageEvent;

/// 下一页
#[derive(Message)]
pub struct NextPageEvent;

/// 跳转到指定页
#[derive(Message)]
pub struct GoToPageEvent(pub i32);

// ==================== 阅读器消息 ====================

/// 上一张图片
#[derive(Message)]
pub struct PrevPictureEvent;

/// 下一张图片
#[derive(Message)]
pub struct NextPictureEvent;

/// 上一章节
#[derive(Message)]
pub struct PrevEpisodeEvent;

/// 下一章节
#[derive(Message)]
pub struct NextEpisodeEvent;

/// 放大
#[derive(Message)]
pub struct ZoomInEvent;

/// 缩小
#[derive(Message)]
pub struct ZoomOutEvent;

/// 重置缩放
#[derive(Message)]
pub struct ResetZoomEvent;

/// 切换阅读模式
#[derive(Message)]
pub struct ChangeReadModeEvent(pub ReadMode);

// ==================== 代理设置消息 ====================

/// 代理启用切换
#[derive(Message)]
pub struct ProxyEnabledToggleEvent(pub bool);

/// 保存代理设置
#[derive(Message)]
pub struct SaveProxySettingsEvent;

/// 测试代理连接
#[derive(Message)]
pub struct TestProxyConnectionEvent;

/// 代理测试结果
#[derive(Message)]
pub struct ProxyTestResultEvent {
    pub success: bool,
    pub message: String,
}

// ==================== 消息提示消息 ====================

/// 显示错误消息
#[derive(Message)]
pub struct ShowErrorEvent(pub String);

/// 显示成功消息
#[derive(Message)]
pub struct ShowSuccessEvent(pub String);

/// 清除消息
#[derive(Message)]
pub struct ClearMessagesEvent;
