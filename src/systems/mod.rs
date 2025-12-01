//! ECS 系统模块
//!
//! 定义应用中使用的所有系统

mod categories;
mod comics;
mod detail;
pub mod login;
mod main_layout;
mod navigation;
mod proxy_settings;
mod reader;
mod scrollbar;
mod setup;

pub use categories::*;
pub use comics::*;
pub use detail::*;
pub use login::*;
pub use main_layout::*;
pub use navigation::*;
pub use proxy_settings::*;
pub use reader::*;
pub use scrollbar::*;
pub use setup::*;
