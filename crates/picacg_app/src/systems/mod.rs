//! ECS 系统模块
//!
//! 定义应用中使用的所有系统

// Bevy ECS 系统函数通常需要多个参数进行依赖注入，这是框架设计模式
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unused_imports)]

pub mod app_icon;
mod categories;
mod chat;
mod chat_room;
mod comics;
mod comments;
mod detail;
mod downloads;
mod favorites;
pub mod font_loader;
mod forgot_password;
mod fried;
mod game_detail;
mod games;
mod history;
mod home;
mod image_convert;
mod like_records;
mod local_read;
pub mod login;
mod main_layout;
mod nas;
mod navigation;
pub mod pagination;
pub mod perf_overlay;
mod placeholder;
mod profile;
mod proxy_settings;
mod rankings;
mod reader;
mod register;
pub mod scrollbar;
mod search;
mod settings;
mod setup;
pub mod theme;
pub mod ui_common;
mod waifu2x;
pub mod waterfall;
pub mod widgets;

pub use app_icon::*;
pub use categories::*;
pub use chat::*;
pub use chat_room::*;
pub use comics::*;
pub use comments::*;
pub use detail::*;
pub use downloads::*;
pub use favorites::*;
pub use font_loader::*;
pub use forgot_password::*;
pub use fried::*;
pub use game_detail::*;
pub use games::*;
pub use history::*;
pub use home::*;
pub use image_convert::*;
pub use like_records::*;
pub use local_read::*;
pub use login::*;
pub use main_layout::*;
pub use nas::*;
pub use navigation::*;
pub use pagination::*;
pub use perf_overlay::*;
pub use placeholder::*;
pub use profile::*;
pub use proxy_settings::*;
pub use rankings::*;
pub use reader::*;
pub use register::*;
pub use scrollbar::*;
pub use search::*;
pub use settings::*;
pub use setup::*;
pub use theme::*;
pub use ui_common::*;
pub use waifu2x::*;
pub use waterfall::*;
pub use widgets::*;
