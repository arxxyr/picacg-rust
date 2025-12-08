//! 瀑布式显示系统 - 业务特定类型
//!
//! 通用的 `WaterfallState` 由 `bevy_ui_toolkit` crate 提供，
//! 此模块定义业务特定的页面标记和上下文类型。

// 重新导出 crate 的通用类型
pub use bevy_ui_toolkit::WaterfallState;
use picacg_api::endpoints::RankTimeType;

// ==================== 页面标记类型 ====================

/// 排行榜页面标记
#[derive(Default)]
pub struct RankingsWaterfall;

/// 分类页面标记
#[derive(Default)]
pub struct CategoriesWaterfall;

/// 漫画列表页面标记
#[derive(Default)]
pub struct ComicsWaterfall;

/// 搜索页面标记
#[derive(Default)]
pub struct SearchWaterfall;

// ==================== 上下文类型 ====================

/// 排行榜上下文（记录当前排行榜类型，用于验证）
#[derive(Default, Clone, Copy)]
pub struct RankingsContext {
    pub current_type: Option<RankTimeType>,
}

// ==================== 类型别名 ====================

/// 排行榜瀑布状态（带上下文）
pub type RankingsCardCreationState = WaterfallState<RankingsWaterfall, RankingsContext>;

/// 分类瀑布状态
pub type CategoriesCardCreationState = WaterfallState<CategoriesWaterfall>;

/// 漫画列表瀑布状态
pub type ComicsCardCreationState = WaterfallState<ComicsWaterfall>;

/// 搜索瀑布状态
pub type SearchCardCreationState = WaterfallState<SearchWaterfall>;
