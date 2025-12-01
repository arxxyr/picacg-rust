//! 初始化系统
//!
//! 应用启动时的初始化逻辑

use bevy::prelude::*;

/// 设置 2D 相机
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 设置字体资源
pub fn setup_fonts(asset_server: Res<AssetServer>, mut app_font: ResMut<AppFont>) {
    // 加载中文字体 (路径相对于 assets 目录)
    let font_handle: Handle<Font> =
        asset_server.load("fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf");
    app_font.0 = font_handle;
}

/// 应用字体资源
#[derive(Resource, Clone)]
pub struct AppFont(pub Handle<Font>);

impl Default for AppFont {
    fn default() -> Self {
        Self(Handle::default())
    }
}
