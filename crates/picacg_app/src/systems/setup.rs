//! 初始化系统
//!
//! 应用启动时的初始化逻辑，以及全局窗口管理系统

use bevy::{
    prelude::*,
    window::{ClosingWindow, PrimaryWindow, WindowCloseRequested, WindowPosition},
};
use picacg_config::{AppSettings, CloseBehavior};

/// 设置 2D 相机
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 用户显式关闭标记
///
/// 区分「用户点关闭」与「系统销毁窗口」（合盖导致显示器移除时，winit 会关掉
/// 该显示器上的窗口）。前者应退出应用；后者应用继续运行（下载不中断），
/// 由 `ensure_primary_window` 自动重建窗口。
#[derive(Resource, Default)]
pub struct ExplicitWindowClose(pub bool);

/// 主窗口保活：意外销毁自动重建；显式关闭则退出应用
///
/// 配合 `ExitCondition::DontExit`（main.rs）：窗口不再绑定应用生命周期。
/// 合盖（无外接显示器且未休眠）时 macOS 移除内置显示器 → winit 销毁窗口 →
/// 此前 `OnAllClosed` 直接退出进程、下载全断。现在应用存活并按保存的几何
/// 信息重建窗口；创建失败（如暂时无任何显示器）约 2 秒后重试。
pub fn ensure_primary_window(
    mut commands: Commands,
    windows: Query<(), With<PrimaryWindow>>,
    explicit_close: Res<ExplicitWindowClose>,
    mut exit_messages: MessageWriter<AppExit>,
    mut retry_countdown: Local<u32>,
) {
    if !windows.is_empty() {
        *retry_countdown = 0;
        return;
    }

    // 用户主动关闭：窗口销毁完成后显式退出
    if explicit_close.0 {
        exit_messages.write(AppExit::Success);
        return;
    }

    if *retry_countdown > 0 {
        *retry_countdown -= 1;
        return;
    }
    *retry_countdown = 120;

    tracing::warn!("主窗口被系统销毁（如合盖导致显示器移除），自动重建以保住下载任务");
    let settings = AppSettings::global().read();
    let mut window = Window {
        title: "PicACG - Rust Bevy 版".to_string(),
        ..default()
    };
    if let (Some(width), Some(height)) = (settings.window_width, settings.window_height) {
        window.resolution = (width as u32, height as u32).into();
    }
    if let (Some(x), Some(y)) = (settings.window_x, settings.window_y) {
        window.position = WindowPosition::At(IVec2::new(x as i32, y as i32));
    }
    commands.spawn((window, PrimaryWindow));
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
    mut explicit_close: ResMut<ExplicitWindowClose>,
) {
    // 先处理上一帧标记为 ClosingWindow 的窗口（与默认行为一致）
    for window in closing.iter() {
        commands.entity(window).despawn();
    }

    for event in close_events.read() {
        // 读取关闭行为设置（挪进循环：关闭事件一生最多一次，不必每帧取锁）
        let close_behavior = AppSettings::global().read().close_behavior;
        match close_behavior {
            CloseBehavior::Close | CloseBehavior::Ask => {
                // 关闭前保存窗口位置和大小
                if let Ok(window) = window_query.get(event.window) {
                    save_window_geometry_to_config(window);
                }
                // 用户显式关闭：窗口销毁后由 ensure_primary_window 发 AppExit
                explicit_close.0 = true;
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
