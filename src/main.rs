// use mimalloc::MiMalloc;

// #[global_allocator]
// static GLOBAL: MiMalloc = MiMalloc;

mod api;
mod config;
mod db;
mod download;
mod error;
mod ui;

use tracing::Level;
use tracing_subscriber;

fn main() -> iced::Result {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("PicACG Rust 客户端启动");

    // 加载并打印配置
    let settings = config::settings::AppSettings::global().read();
    if settings.proxy.enabled {
        tracing::info!("代理已启用: {:?} {}:{}",
            settings.proxy.proxy_type,
            settings.proxy.host,
            settings.proxy.port
        );
    } else {
        tracing::info!("代理未启用");
    }

    // 启动 GUI 应用
    ui::run()
}
