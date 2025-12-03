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

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("PicACG Rust 客户端启动 (Bevy 版)");

    // 加载并打印配置
    let settings = config::settings::AppSettings::global().read();
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
                        ime_enabled: true, // 启用输入法支持
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
