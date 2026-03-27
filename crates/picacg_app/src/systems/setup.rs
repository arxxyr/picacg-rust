//! 初始化系统
//!
//! 应用启动时的初始化逻辑，以及全局窗口管理系统

use bevy::{
    prelude::*,
    window::{ClosingWindow, PrimaryWindow, WindowCloseRequested},
};
use picacg_config::{AppSettings, CloseBehavior};

/// 设置 2D 相机
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 窗口位置保存计时器资源
#[derive(Resource)]
pub struct WindowPositionSaveTimer {
    timer: Timer,
}

impl Default for WindowPositionSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

/// 处理窗口关闭事件（替代 Bevy 默认的 close_when_requested）
///
/// 根据用户设置的关闭行为执行不同操作：
/// - Close / Ask: 正常关闭窗口（先保存窗口位置）
/// - Minimize: 阻止关闭，改为最小化窗口
pub fn handle_window_close(
    mut commands: Commands,
    mut close_events: MessageReader<WindowCloseRequested>,
    closing: Query<Entity, With<ClosingWindow>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    // 先处理上一帧标记为 ClosingWindow 的窗口（与默认行为一致）
    for window in closing.iter() {
        commands.entity(window).despawn();
    }

    // 读取关闭行为设置
    let close_behavior = AppSettings::global().read().close_behavior;

    for event in close_events.read() {
        match close_behavior {
            CloseBehavior::Close | CloseBehavior::Ask => {
                // 关闭前保存窗口位置和大小
                if let Ok(window) = window_query.get(event.window) {
                    save_window_geometry_to_config(window);
                }
                // 标记窗口为正在关闭，下一帧执行 despawn
                commands.entity(event.window).try_insert(ClosingWindow);
            }
            CloseBehavior::Minimize => {
                // 最小化窗口而非关闭
                if let Ok(mut window) = window_query.get_mut(event.window) {
                    window.set_minimized(true);
                    tracing::debug!("窗口已最小化（关闭行为设置为 Minimize）");
                }
            }
        }
    }
}

/// 定期保存窗口位置和大小到配置文件（每 5 秒检查一次）
pub fn save_window_position(
    time: Res<Time>,
    mut timer: ResMut<WindowPositionSaveTimer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    timer.timer.tick(time.delta());
    if !timer.timer.just_finished() {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    save_window_geometry_to_config(window);
}

/// 将窗口几何信息保存到全局配置（内部辅助函数）
fn save_window_geometry_to_config(window: &Window) {
    let mut settings = AppSettings::global().write();

    // 保存窗口大小（逻辑像素）
    let width = window.width();
    let height = window.height();
    let size_changed =
        settings.window_width != Some(width) || settings.window_height != Some(height);

    if size_changed {
        settings.window_width = Some(width);
        settings.window_height = Some(height);
    }

    // 保存窗口位置（物理像素，从 WindowPosition::At 读取）
    let position_changed = match &window.position {
        bevy::window::WindowPosition::At(pos) => {
            let changed =
                settings.window_x != Some(pos.x as f32) || settings.window_y != Some(pos.y as f32);
            if changed {
                settings.window_x = Some(pos.x as f32);
                settings.window_y = Some(pos.y as f32);
            }
            changed
        }
        _ => false,
    };

    // 只在有变化时写入磁盘
    if size_changed || position_changed {
        if let Err(e) = settings.save() {
            tracing::error!("保存窗口位置失败: {}", e);
        } else {
            tracing::trace!(
                "窗口位置已保存: pos={:?},{:?} size={:?}x{:?}",
                settings.window_x,
                settings.window_y,
                settings.window_width,
                settings.window_height
            );
        }
    }
}
