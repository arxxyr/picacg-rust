//! 初始化系统
//!
//! 应用启动时的初始化逻辑

use bevy::prelude::*;

/// 设置 2D 相机
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
