//! PicACG 配置管理
//!
//! 应用配置管理

#![allow(dead_code)]
#![allow(unused_imports)]

mod settings;

// 重新导出
pub use picacg_core::{PicacgError, Result};
pub use settings::{
    AppSettings, ChannelSettings, ChannelType, CloseBehavior, FilterSettings, Language, LogLevel,
    LoginSettings, NasSettings, ProxySettings, ProxyType, ThemeMode, Waifu2xSettings,
    get_log_level_handle, set_log_level_handle, update_log_level,
};
