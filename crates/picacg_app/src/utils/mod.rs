//! 工具模块

pub mod content_filter;
pub mod i18n;
pub mod icons;
pub mod text_input;
pub mod tokio_tasks;
pub mod websocket;

pub use tokio_tasks::{TaskContext, TokioTasksPlugin, TokioTasksRuntime};

/// 清理文件名中的非法字符
///
/// 替换 Windows 文件系统禁止的字符以及可能导致兼容性问题的全角标点
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // ASCII 非法文件名字符
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            // 全角标点（可能导致 ZIP 兼容性问题）
            '\u{FF1A}' // ： 全角冒号
            | '\u{FF0F}' // ／ 全角斜杠
            | '\u{FF3C}' // ＼ 全角反斜杠
            | '\u{FF1C}' // ＜ 全角小于号
            | '\u{FF1E}' // ＞ 全角大于号
            | '\u{FF5C}' // ｜ 全角竖线
            | '\u{FF02}' // ＂ 全角双引号
            | '\u{FF0A}' // ＊ 全角星号
            | '\u{FF1F}' // ？ 全角问号
            => '_',
            _ => c,
        })
        .collect()
}
