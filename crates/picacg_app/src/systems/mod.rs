//! ECS 系统模块
//!
//! 定义应用中使用的所有系统

// Bevy ECS 系统函数通常需要多个参数进行依赖注入，这是框架设计模式
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unused_imports)]

mod categories;
mod comics;
mod detail;
mod downloads;
mod favorites;
mod home;
pub mod login;
mod main_layout;
mod navigation;
pub mod pagination;
mod placeholder;
mod proxy_settings;
mod rankings;
mod reader;
mod register;
mod scrollbar;
mod search;
mod settings;
mod setup;
pub mod ui_common;
pub mod waterfall;

pub use categories::*;
pub use comics::*;
pub use detail::*;
pub use downloads::*;
pub use favorites::*;
pub use home::*;
pub use login::*;
pub use main_layout::*;
pub use navigation::*;
pub use pagination::*;
pub use placeholder::*;
pub use proxy_settings::*;
pub use rankings::*;
pub use reader::*;
pub use register::*;
pub use scrollbar::*;
pub use search::*;
pub use settings::*;
pub use setup::*;
pub use ui_common::*;
pub use waterfall::*;
