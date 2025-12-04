//! 瀑布式显示系统
//!
//! 预创建隐藏的 UI 元素，然后分批显示，避免布局重计算导致卡顿

use std::marker::PhantomData;

use bevy::{prelude::*, time::Timer};

/// 默认每批显示的数量
pub const DEFAULT_BATCH_SIZE: usize = 4;

/// 默认显示间隔（毫秒）
pub const DEFAULT_INTERVAL_MS: u64 = 64; // 约 60fps，每帧显示一批

/// 瀑布式显示状态（泛型，通过 Marker 类型区分不同页面，支持额外上下文）
#[derive(Resource)]
pub struct WaterfallState<T, C = ()>
where
    T: Send + Sync + 'static,
    C: Send + Sync + Default + 'static,
{
    /// 待显示的实体列表（预创建时设置为 Hidden）
    pub pending_entities: Vec<Entity>,
    /// 字体句柄缓存
    pub font_handle: Option<Handle<Font>>,
    /// 是否正在显示中
    pub is_creating: bool,
    /// 显示间隔定时器
    pub timer: Timer,
    /// 是否是第一批（第一批立即显示，不等待定时器）
    pub first_batch: bool,
    /// 每批显示数量
    pub batch_size: usize,
    /// 需要预创建的数量（>0 表示需要预创建）
    pub precreate_count: usize,
    /// 额外上下文数据（用于特殊需求，如 rankings 的 current_type）
    pub context: C,
    /// 类型标记
    _marker: PhantomData<T>,
}

impl<T, C> Default for WaterfallState<T, C>
where
    T: Send + Sync + 'static,
    C: Send + Sync + Default + 'static,
{
    fn default() -> Self {
        Self {
            pending_entities: Vec::new(),
            font_handle: None,
            is_creating: false,
            timer: Timer::from_seconds(DEFAULT_INTERVAL_MS as f32 / 1000.0, TimerMode::Repeating),
            first_batch: true,
            batch_size: DEFAULT_BATCH_SIZE,
            precreate_count: 0,
            context: C::default(),
            _marker: PhantomData,
        }
    }
}

impl<T, C> WaterfallState<T, C>
where
    T: Send + Sync + 'static,
    C: Send + Sync + Default + 'static,
{
    /// 开始预创建模式（在系统中一次性创建所有隐藏卡片，然后瀑布式显示）
    pub fn start_precreate(&mut self, count: usize, font: Handle<Font>) {
        self.precreate_count = count;
        self.pending_entities.clear();
        self.font_handle = Some(font);
        self.is_creating = true;
        self.timer.reset();
        self.first_batch = true;
    }

    /// 开始预创建模式（带上下文）
    pub fn start_precreate_with_context(&mut self, count: usize, font: Handle<Font>, context: C) {
        self.start_precreate(count, font);
        self.context = context;
    }

    /// 检查是否需要预创建
    pub fn needs_precreate(&self) -> bool {
        self.precreate_count > 0 && self.pending_entities.is_empty()
    }

    /// 设置预创建完成后的实体列表
    pub fn set_precreated_entities(&mut self, entities: Vec<Entity>) {
        self.pending_entities = entities;
        self.precreate_count = 0;
    }

    /// 获取需要预创建的数量
    pub fn get_precreate_count(&self) -> usize {
        self.precreate_count
    }

    /// 获取下一批要显示的实体
    pub fn take_batch(&mut self) -> Vec<Entity> {
        let batch_size = self.batch_size.min(self.pending_entities.len());
        self.pending_entities.drain(0..batch_size).collect()
    }

    /// 是否还有待显示的实体
    pub fn has_pending(&self) -> bool {
        !self.pending_entities.is_empty()
    }

    /// 清空状态
    pub fn clear(&mut self) {
        self.pending_entities.clear();
        self.font_handle = None;
        self.is_creating = false;
        self.first_batch = true;
        self.precreate_count = 0;
        self.context = C::default();
    }

    /// 检查是否应该显示下一批（处理定时器逻辑）
    /// 返回 true 表示应该显示，false 表示等待
    pub fn should_show_batch(&mut self, time_delta: std::time::Duration) -> bool {
        if !self.has_pending() {
            return false;
        }

        // 第一批立即显示
        if self.first_batch {
            self.first_batch = false;
            self.timer.reset();
            return true;
        }

        // 后续批次使用定时器
        self.timer.tick(time_delta);
        self.timer.just_finished()
    }

    /// 标记显示完成
    pub fn finish(&mut self) {
        self.is_creating = false;
    }
}

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

use crate::api::endpoints::RankTimeType;

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
