//! 瀑布式显示系统
//!
//! 预创建隐藏的 UI 元素，然后分批显示，避免布局重计算导致卡顿。
//!
//! ## 使用方法
//!
//! 1. 定义页面标记类型：
//! ```ignore
//! pub struct CategoriesWaterfall;
//! ```
//!
//! 2. 注册资源：
//! ```ignore
//! app.init_resource::<WaterfallState<CategoriesWaterfall>>();
//! ```
//!
//! 3. 在系统中使用：
//! ```ignore
//! fn waterfall_create_cards(
//!     mut creation_state: ResMut<WaterfallState<CategoriesWaterfall>>,
//!     time: Res<Time>,
//!     // ...
//! ) {
//!     // 检查是否需要启动预创建
//!     if !creation_state.is_creating && data_ready {
//!         creation_state.start_precreate(item_count, font);
//!     }
//!
//!     // 预创建阶段：一次性创建所有隐藏卡片
//!     if creation_state.needs_precreate() {
//!         let entities = create_hidden_cards(...);
//!         creation_state.set_precreated_entities(entities);
//!     }
//!
//!     // 显示阶段：分批显示卡片
//!     if creation_state.should_show_batch(time.delta()) {
//!         let batch = creation_state.take_batch();
//!         for entity in batch {
//!             // 显示卡片（移除 Hidden 组件或设置 Visibility）
//!         }
//!
//!         if !creation_state.has_pending() {
//!             creation_state.finish();
//!         }
//!     }
//! }
//! ```

mod state;

pub use state::*;
