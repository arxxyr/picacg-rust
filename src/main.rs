//! PicACG 漫画客户端 - Rust Bevy 版
//!
//! 使用 Bevy 0.17 ECS 架构重写

mod api;
mod components;
mod config;
mod db;
mod download;
mod error;
mod events;
mod plugins;
mod resources;
mod systems;

use bevy::{asset::AssetPlugin, prelude::*};
use bevy_tokio_tasks::TokioTasksPlugin;
use plugins::{ApiPlugin, UiPlugin};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, reload, util::SubscriberInitExt};

fn main() {
    // 加载配置（在初始化日志前）
    let settings = config::settings::AppSettings::global().read();
    let log_level_str = settings.log_level.as_str();

    // 创建可重载的日志过滤器
    let filter = EnvFilter::new(log_level_str);
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    // 初始化日志（使用可重载的过滤器）
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_target(false))
        .init();

    // 保存 reload handle 供后续动态更新使用
    config::settings::set_log_level_handle(reload_handle);

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

    // 获取数据库路径
    let db_path = settings.get_database_path();
    drop(settings);

    // 初始化数据库
    tracing::info!("正在初始化数据库: {:?}", db_path);
    db::database::db_runtime().block_on(async {
        if let Err(e) = db::database::Database::init(db_path).await {
            tracing::error!("数据库初始化失败: {}", e);
        }
    });

    // 配置 assets 路径 (确保能找到字体)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let assets_path = std::path::Path::new(manifest_dir).join("assets");
    tracing::info!("Assets 路径: {}", assets_path.display());

    // 创建并运行 Bevy 应用
    App::new()
        // Bevy 默认插件
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: assets_path.to_string_lossy().to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "PicACG - Rust Bevy 版".to_string(),
                        resolution: (1024u32, 768u32).into(),
                        ime_enabled: false, // 输入法在输入框获得焦点时动态启用
                        ..default()
                    }),
                    ..default()
                }),
        )
        // Tokio 运行时集成
        .add_plugins(TokioTasksPlugin::default())
        // 自定义插件
        .add_plugins((UiPlugin, ApiPlugin))
        .run();
}
