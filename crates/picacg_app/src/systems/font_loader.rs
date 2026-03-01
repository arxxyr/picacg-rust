//! 系统字体检测与加载
//!
//! 跨平台检测系统中文字体，避免捆绑字体文件。
//! 优先使用中文字体，回退到英文字体。
//!
//! 字体字节在模块首次访问时同步加载（`OnceLock`），
//! 保证任何 Bevy 系统调用 `get_font()` 时句柄已就绪。

use std::sync::OnceLock;

use bevy::{prelude::*, text::Font};

/// 全局字体句柄（`setup_fonts` 初始化后可用）
static FONT_HANDLE: OnceLock<Handle<Font>> = OnceLock::new();

/// 预加载的字体字节（同步初始化，不依赖 Bevy）
static FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();

/// 获取全局字体句柄，供所有 UI 系统使用
///
/// 如果 `setup_fonts` 尚未运行（理论上不应该），返回默认句柄而非 panic
pub fn get_font() -> Handle<Font> {
    FONT_HANDLE.get().cloned().unwrap_or_default()
}

/// 预加载系统字体字节（同步，可在 App 构建阶段调用）
///
/// 仅加载字节到内存，不依赖 Bevy 资产系统
pub fn preload_font_bytes() {
    FONT_BYTES.get_or_init(load_system_font_bytes);
}

/// Bevy 启动系统：将预加载的字体注册到 Bevy 资产系统
pub fn setup_fonts(mut fonts: ResMut<Assets<Font>>, mut app_font: ResMut<AppFont>) {
    // 确保字节已加载
    let bytes = FONT_BYTES.get_or_init(load_system_font_bytes);
    if bytes.is_empty() {
        tracing::error!("字体字节为空，UI 文字将无法显示");
        return;
    }

    match Font::try_from_bytes(bytes.clone()) {
        Ok(font) => {
            let handle = fonts.add(font);
            app_font.0 = handle.clone();
            FONT_HANDLE.set(handle).ok();
            tracing::info!("系统字体加载成功");
        }
        Err(e) => {
            tracing::error!("系统字体解析失败: {:?}", e);
        }
    }
}

/// 应用字体资源
#[derive(Resource, Clone, Default)]
pub struct AppFont(pub Handle<Font>);

// ============ 字体检测 ============

fn load_system_font_bytes() -> Vec<u8> {
    if let Some(path) = detect_system_font_path() {
        tracing::info!("使用系统字体: {}", path);
        match std::fs::read(&path) {
            Ok(bytes) => return bytes,
            Err(e) => tracing::error!("读取字体文件失败: {} - {}", path, e),
        }
    }
    tracing::error!("未找到任何可用的系统字体");
    Vec::new()
}

fn detect_system_font_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        detect_windows_font()
    }
    #[cfg(target_os = "linux")]
    {
        detect_linux_font()
    }
    #[cfg(target_os = "macos")]
    {
        detect_macos_font()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

// ============ Windows ============

#[cfg(target_os = "windows")]
fn detect_windows_font() -> Option<String> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let fonts_dir = format!("{}\\Fonts", windir);

    // 中文字体优先
    let candidates: &[(&str, &[&str])] = &[
        ("Microsoft YaHei", &["msyh.ttc", "msyh.ttf"]),
        ("SimHei", &["simhei.ttf"]),
        ("SimSun", &["simsun.ttc", "simsun.ttf"]),
        ("KaiTi", &["simkai.ttf"]),
        // 英文回退
        ("Segoe UI", &["segoeui.ttf"]),
        ("Arial", &["arial.ttf"]),
    ];

    for (name, files) in candidates {
        for file in *files {
            let path = format!("{}\\{}", fonts_dir, file);
            if std::path::Path::new(&path).exists() {
                tracing::info!("检测到字体: {} ({})", name, file);
                return Some(path);
            }
        }
    }
    None
}

// ============ Linux ============

#[cfg(target_os = "linux")]
fn detect_linux_font() -> Option<String> {
    let preferred = [
        "Noto Sans CJK SC",
        "Noto Sans CJK",
        "WenQuanYi Micro Hei",
        "WenQuanYi Zen Hei",
        "Source Han Sans SC",
        "Source Han Sans CN",
        "Droid Sans Fallback",
        // 英文回退
        "DejaVu Sans",
        "Liberation Sans",
        "Noto Sans",
    ];

    for font_name in &preferred {
        if let Some(path) = fc_match_font_file(font_name) {
            tracing::info!("检测到字体: {} ({})", font_name, path);
            return Some(path);
        }
    }

    // 最后尝试匹配任意默认字体
    if let Some(path) = fc_match_font_file("sans-serif") {
        tracing::info!("使用默认字体: {}", path);
        return Some(path);
    }

    tracing::warn!(
        "未找到 CJK 字体，中文可能无法显示。\n\
         安装建议:\n\
         - Ubuntu/Debian: sudo apt install fonts-noto-cjk\n\
         - Fedora: sudo dnf install google-noto-sans-cjk-fonts\n\
         - Arch: sudo pacman -S noto-fonts-cjk"
    );
    None
}

#[cfg(target_os = "linux")]
fn fc_match_font_file(font_name: &str) -> Option<String> {
    let output = std::process::Command::new("fc-match")
        .args([font_name, "--format=%{file}"])
        .output()
        .ok()?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !path.is_empty() && std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

// ============ macOS ============

#[cfg(target_os = "macos")]
fn detect_macos_font() -> Option<String> {
    let candidates = [
        // 中文字体
        "/System/Library/Fonts/PingFang.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        "/System/Library/Fonts/STHeiti Light.ttc",
        // 英文回退
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/SFNSText.ttf",
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            tracing::info!("检测到字体: {}", path);
            return Some(path.to_string());
        }
    }
    None
}
