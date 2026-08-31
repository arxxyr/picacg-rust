//! PicACG 漫画客户端 - Rust Bevy 版
//!
//! 使用 Bevy 0.18 ECS 架构重写

#![windows_subsystem = "windows"]

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod components;
mod events;
mod plugins;
mod resources;
mod systems;
mod utils;

use bevy::{
    asset::{AssetPlugin, UnapprovedPathMode},
    prelude::*,
    window::{ExitCondition, WindowPosition},
};
use picacg_config::{AppSettings, set_log_level_handle};
use picacg_db::{Database, db_runtime};
use plugins::{ApiPlugin, UiPlugin};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, reload, util::SubscriberInitExt};
use utils::TokioTasksPlugin;

fn main() {
    // 加载配置（在初始化日志前）
    let settings = AppSettings::global().read();
    let log_level_str = settings.log_level.as_str();

    // 创建可重载的日志过滤器
    let filter = EnvFilter::new(log_level_str);
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    // 系统耗时统计（--features profiling 才编进来，见 utils::profiling）
    let profiling_wanted = utils::profiling::wants_profiling();

    // 初始化日志（使用可重载的过滤器）
    let fmt_layer = fmt::layer()
        .with_target(false)
        // 日志时间戳用系统本地时区（默认是 UTC，排查问题时对不上表）
        .with_timer(fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        ));
    let profiler_layer = profiling_wanted.then(utils::profiling::layer);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(profiler_layer)
        .init();

    // 保存 reload handle 供后续动态更新使用
    set_log_level_handle(reload_handle);

    tracing::info!("PicACG Rust 客户端启动 (Bevy 版)");
    tracing::info!("日志等级: {} (支持动态更新)", log_level_str);
    if settings.proxy.enabled {
        tracing::info!(
            "使用代理: {:?}://{}:{}",
            settings.proxy.proxy_type,
            settings.proxy.host,
            settings.proxy.port
        );
    } else {
        tracing::info!("代理未启用");
    }

    // 读取窗口位置和大小（在 drop settings 之前）
    let saved_window_x = settings.window_x;
    let saved_window_y = settings.window_y;
    let saved_window_width = settings.window_width;
    let saved_window_height = settings.window_height;

    // 获取数据库路径
    let db_path = settings.get_database_path();
    drop(settings);

    // 初始化数据库
    tracing::info!("正在初始化数据库: {:?}", db_path);
    db_runtime().block_on(async {
        if let Err(e) = Database::init(db_path).await {
            tracing::error!("数据库初始化失败: {}", e);
        }
    });

    // 配置 assets 路径
    // 注意：在 workspace 结构中，assets
    // 在项目根目录（字体已改用系统字体，此路径主要用于图片等资源）
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let assets_path = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("assets"))
        .unwrap_or_else(|| std::path::PathBuf::from("assets"));
    tracing::info!("Assets 路径: {}", assets_path.display());

    // 创建并运行 Bevy 应用
    App::new()
        // Bevy 默认插件
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: assets_path.to_string_lossy().to_string(),
                    // 允许加载 assets 目录之外的文件（如下载目录的图片）
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "PicACG - Rust Bevy 版".to_string(),
                        resolution: {
                            // 恢复保存的窗口大小，默认 1024x768
                            let w = saved_window_width.unwrap_or(1024.0);
                            let h = saved_window_height.unwrap_or(768.0);
                            (w as u32, h as u32).into()
                        },
                        position: {
                            // 恢复保存的窗口位置
                            match (saved_window_x, saved_window_y) {
                                (Some(x), Some(y)) => {
                                    WindowPosition::At(IVec2::new(x as i32, y as i32))
                                }
                                _ => WindowPosition::default(),
                            }
                        },
                        ime_enabled: false, // 输入法在输入框获得焦点时动态启用
                        ..default()
                    }),
                    // 禁用默认关闭行为，由 handle_window_close 系统自行处理
                    close_when_requested: false,
                    // 窗口 ≠ 应用生命周期：合盖等系统事件销毁窗口时应用继续运行
                    // （下载不中断），由 ensure_primary_window 自动重建窗口；
                    // 用户主动关闭走 handle_window_close 显式发 AppExit
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                // 已在上方用 tracing_subscriber 初始化日志，禁用 Bevy 内置的 LogPlugin 避免冲突
                .disable::<bevy::log::LogPlugin>(),
        )
        // Tokio 运行时集成
        .add_plugins(TokioTasksPlugin::default())
        // 窗口图标 + macOS dock 图标（窗口重建后会自动补贴）
        .add_systems(Update, systems::app_icon::apply_app_icon)
        // 性能追踪：F3 叠加层（FPS/帧时间/实体数）+ F4 系统耗时榜
        .add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
        ))
        .init_resource::<systems::perf_overlay::PerfOverlayState>()
        .add_systems(Startup, systems::perf_overlay::setup_perf_overlay)
        .add_systems(
            Update,
            (
                systems::perf_overlay::toggle_perf_overlay,
                systems::perf_overlay::update_perf_overlay,
                systems::perf_overlay::print_system_timings,
                systems::perf_overlay::auto_report_slow_frames,
            ),
        )
        // 自定义插件
        .add_plugins((UiPlugin, ApiPlugin))
        // 设置全局 panic handler 为 warn（防止 text_system 等内部系统 panic 导致崩溃）
        .set_error_handler(bevy::ecs::error::warn)
        .run();
}
