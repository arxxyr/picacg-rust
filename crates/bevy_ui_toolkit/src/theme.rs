//! 主题/颜色系统
//!
//! 提供可配置的主题系统，用于统一应用的颜色风格。
//!
//! ## 使用方法
//!
//! ```ignore
//! // 使用默认深色主题
//! app.add_plugins(BevyUiToolkitPlugin::default());
//!
//! // 或者自定义主题
//! app.add_plugins(BevyUiToolkitPlugin {
//!     theme: Some(Theme::light()),
//! });
//!
//! // 在系统中访问主题
//! fn my_system(theme: Res<CurrentTheme>) {
//!     let bg_color = theme.background;
//! }
//! ```

use bevy::prelude::*;

/// 应用主题配置
///
/// 包含所有 UI 元素使用的颜色定义
#[derive(Clone, Debug)]
pub struct Theme {
    /// 背景色
    pub background: Color,
    /// 表面/卡片背景色
    pub surface: Color,
    /// 卡片背景色（更浅）
    pub card_bg: Color,
    /// 主色调（按钮、链接等）
    pub primary: Color,
    /// 主色调悬停状态
    pub primary_hover: Color,
    /// 主色调按下状态
    pub primary_pressed: Color,
    /// 次要色调
    pub secondary: Color,
    /// 次要色调悬停状态
    pub secondary_hover: Color,
    /// 主文本颜色
    pub text: Color,
    /// 次要文本颜色
    pub text_secondary: Color,
    /// 弱化文本颜色
    pub text_muted: Color,
    /// 错误颜色
    pub error: Color,
    /// 成功颜色
    pub success: Color,
    /// 警告颜色
    pub warning: Color,
    /// 边框颜色
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// 深色主题（默认）
    ///
    /// 适用于深色背景的应用界面
    pub fn dark() -> Self {
        Self {
            background: Color::srgb(0.1, 0.1, 0.15),
            surface: Color::srgb(0.15, 0.15, 0.2),
            card_bg: Color::srgb(0.18, 0.18, 0.25),
            primary: Color::srgb(0.2, 0.4, 0.8),
            primary_hover: Color::srgb(0.25, 0.45, 0.85),
            primary_pressed: Color::srgb(0.15, 0.35, 0.7),
            secondary: Color::srgb(0.3, 0.3, 0.4),
            secondary_hover: Color::srgb(0.35, 0.35, 0.45),
            text: Color::WHITE,
            text_secondary: Color::srgb(0.6, 0.6, 0.7),
            text_muted: Color::srgb(0.5, 0.5, 0.6),
            error: Color::srgb(1.0, 0.3, 0.3),
            success: Color::srgb(0.3, 0.8, 0.3),
            warning: Color::srgb(1.0, 0.7, 0.2),
            border: Color::srgb(0.3, 0.3, 0.4),
        }
    }

    /// 浅色主题
    ///
    /// 适用于浅色背景的应用界面
    pub fn light() -> Self {
        Self {
            background: Color::srgb(0.95, 0.95, 0.97),
            surface: Color::srgb(1.0, 1.0, 1.0),
            card_bg: Color::srgb(0.98, 0.98, 1.0),
            primary: Color::srgb(0.2, 0.4, 0.8),
            primary_hover: Color::srgb(0.25, 0.45, 0.85),
            primary_pressed: Color::srgb(0.15, 0.35, 0.7),
            secondary: Color::srgb(0.7, 0.7, 0.75),
            secondary_hover: Color::srgb(0.65, 0.65, 0.7),
            text: Color::srgb(0.1, 0.1, 0.15),
            text_secondary: Color::srgb(0.4, 0.4, 0.45),
            text_muted: Color::srgb(0.5, 0.5, 0.55),
            error: Color::srgb(0.9, 0.2, 0.2),
            success: Color::srgb(0.2, 0.7, 0.2),
            warning: Color::srgb(0.9, 0.6, 0.1),
            border: Color::srgb(0.8, 0.8, 0.85),
        }
    }
}

/// 当前主题资源
///
/// 在系统中通过 `Res<CurrentTheme>` 访问
#[derive(Resource, Deref, DerefMut)]
pub struct CurrentTheme(pub Theme);

impl Default for CurrentTheme {
    fn default() -> Self {
        Self(Theme::default())
    }
}
