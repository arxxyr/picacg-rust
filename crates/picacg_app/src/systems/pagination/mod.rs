//! 通用分页组件模块
//!
//! 提供可复用的分页 UI 组件和辅助函数。
//!
//! ## 使用方法
//!
//! 1. 定义页面特定的标记类型：
//! ```ignore
//! pub struct FavoritesPage;
//! pub struct ComicsPage;
//! ```
//!
//! 2. 使用 `spawn_pagination_controls` 创建分页 UI：
//! ```ignore
//! spawn_pagination_controls::<FavoritesPage>(
//!     parent,
//!     &font,
//!     &theme,
//!     current_page,
//!     total_pages,
//! );
//! ```
//!
//! 3. 使用 `update_pagination_display` 更新分页显示：
//! ```ignore
//! update_pagination_display::<FavoritesPage>(
//!     &mut page_text_query,
//!     &mut prev_btn_query,
//!     &mut next_btn_query,
//!     &theme,
//!     current_page,
//!     total_pages,
//! );
//! ```

mod components;
mod scenes;
mod systems;

pub use components::*;
pub use scenes::*;
pub use systems::*;
