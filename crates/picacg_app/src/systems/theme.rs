//! 设计令牌（Design Tokens）——颜色与尺度的唯一权威源
//!
//! 盘点结论（rv-widgets）：此前存在两套数值逐字段相同的平行色板
//! （Theme 与 AppColors），另有 481 处硬编码颜色绕过两者
//! （59 处重新硬编码 SURFACE、54 处未命名下沉表面色、29 处未命名 hover 色）。
//! 本文件收敛为单一体系：
//! - `Theme`：颜色令牌（含此前散落的 3 个未命名高频角色）
//! - `Scale`：字号/圆角/间距（17 种字号 → 6 档，9 种圆角 → 3 档）
//!
//! `login.rs` 的 `AppColors` 是 `Theme::dark()` 的常量视图（1000+
//! 处存量引用）， 数值以本文件为准；新代码优先用 `AppColors`/`Scale`
//! 常量，硬编码 srgb 一律不许。

use bevy::prelude::*;

/// 应用主题配置（颜色令牌全集）
#[derive(Clone, Debug)]
pub struct Theme {
    /// 页面背景
    pub background: Color,
    /// 表面/容器背景
    pub surface: Color,
    /// 下沉表面（输入框、未选中分段按钮）——此前 54 处裸值
    /// `srgb(0.12,0.12,0.16)`
    pub surface_sunken: Color,
    /// 通用悬停背景——此前 29 处裸值 `srgb(0.2,0.2,0.25)`
    pub surface_hover: Color,
    /// 主色调（按钮、链接、选中态）
    pub primary: Color,
    /// 主色调悬停
    pub primary_hover: Color,
    /// 主色调按下
    pub primary_pressed: Color,
    /// 次要色调（禁用态、次要按钮）
    pub secondary: Color,
    /// 次要色调悬停
    pub secondary_hover: Color,
    /// 主文本
    pub text: Color,
    /// 错误（文本与危险按钮共用此系）
    pub error: Color,
    /// 边框
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// 深色主题（当前唯一主题）
    pub fn dark() -> Self {
        Self {
            background: Color::srgb(0.1, 0.1, 0.15),
            surface: Color::srgb(0.15, 0.15, 0.2),
            surface_sunken: Color::srgb(0.12, 0.12, 0.16),
            surface_hover: Color::srgb(0.2, 0.2, 0.25),
            primary: Color::srgb(0.2, 0.4, 0.8),
            primary_hover: Color::srgb(0.25, 0.45, 0.85),
            primary_pressed: Color::srgb(0.15, 0.35, 0.7),
            secondary: Color::srgb(0.3, 0.3, 0.4),
            secondary_hover: Color::srgb(0.35, 0.35, 0.45),
            text: Color::WHITE,
            error: Color::srgb(1.0, 0.3, 0.3),
            border: Color::srgb(0.3, 0.3, 0.4),
        }
    }
}

/// 尺度令牌（字号/圆角/间距）
///
/// 收敛映射已定（17 种字号→6 档、9 种圆角→3 档），全库替换属全局视觉变更，
/// 留待专项波次执行（见终轮清单顺延项）；新代码直接用这些档位。
#[allow(dead_code)]
pub struct Scale;

#[allow(dead_code)]
impl Scale {
    // ---- 字号：17 种取值收敛为 6 档 ----
    /// 徽章/时间/次要说明（收编 9/10/11）
    pub const CAPTION: f32 = 11.0;
    /// 小正文（收编 12/13）
    pub const BODY_SM: f32 = 12.0;
    /// 默认正文（现 133 处的事实标准；收编 13/15）
    pub const BODY: f32 = 14.0;
    /// 小标题/按钮
    pub const TITLE: f32 = 16.0;
    /// 页面标题（收编 20/22）
    pub const HEADER: f32 = 18.0;
    /// 空态大图标（收编 24/28/32/40/64）
    pub const DISPLAY: f32 = 48.0;

    // ---- 圆角：9 种取值收敛为 3 档 ----
    /// 按钮/输入框/徽章（现 73 处的事实标准；收编 2/3）
    pub const R_SM: f32 = 4.0;
    /// 卡片（收编 6/9/12）
    pub const R_MD: f32 = 8.0;
    /// 胶囊
    pub const R_PILL: f32 = 999.0;

    // ---- 间距：11 种取值收敛为 5 档 ----
    /// 紧凑
    pub const SP_XS: f32 = 4.0;
    /// 小
    pub const SP_SM: f32 = 8.0;
    /// 中（默认）
    pub const SP_MD: f32 = 12.0;
    /// 大
    pub const SP_LG: f32 = 16.0;
    /// 页面级
    pub const SP_XL: f32 = 20.0;
}
