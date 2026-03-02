//! 字体检测与加载
//!
//! 优先使用内置 Sarasa（更纱黑体）字体（支持 CJK + Unicode 符号），
//! 回退到系统中文字体。
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

/// 预加载字体字节（同步，可在 App 构建阶段调用）
///
/// 仅加载字节到内存，不依赖 Bevy 资产系统
pub fn preload_font_bytes() {
    FONT_BYTES.get_or_init(load_font_bytes);
}

/// Bevy 启动系统：将预加载的字体注册到 Bevy 资产系统
pub fn setup_fonts(mut fonts: ResMut<Assets<Font>>, mut app_font: ResMut<AppFont>) {
    // 确保字节已加载
    let bytes = FONT_BYTES.get_or_init(load_font_bytes);
    if bytes.is_empty() {
        tracing::error!("字体字节为空，UI 文字将无法显示");
        return;
    }

    match Font::try_from_bytes(bytes.clone()) {
        Ok(font) => {
            let handle = fonts.add(font);
            app_font.0 = handle.clone();
            FONT_HANDLE.set(handle).ok();
            tracing::info!("字体加载成功");
        }
        Err(e) => {
            tracing::error!("字体解析失败: {:?}", e);
        }
    }
}

/// 应用字体资源
#[derive(Resource, Clone, Default)]
pub struct AppFont(pub Handle<Font>);

// ============ 字体加载 ============

/// 内置字体相对路径（从项目根或可执行文件目录）
const BUNDLED_FONT_RELATIVE: &str = "assets/fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf";

fn load_font_bytes() -> Vec<u8> {
    // 优先加载内置 Sarasa 字体（TTF 格式，CJK + Unicode 符号全覆盖）
    if let Some(path) = detect_bundled_font_path() {
        tracing::info!("使用内置字体: {}", path);
        match std::fs::read(&path) {
            Ok(bytes) => return bytes,
            Err(e) => tracing::warn!("读取内置字体失败: {} - {}", path, e),
        }
    }

    // 回退到系统字体
    if let Some(path) = detect_system_font_path() {
        tracing::info!("使用系统字体: {}", path);
        match std::fs::read(&path) {
            Ok(bytes) => return bytes,
            Err(e) => tracing::error!("读取系统字体失败: {} - {}", path, e),
        }
    }

    tracing::error!("未找到任何可用字体");
    Vec::new()
}

/// 检测内置字体路径
fn detect_bundled_font_path() -> Option<String> {
    // 1. 通过 CARGO_MANIFEST_DIR 定位（开发环境） CARGO_MANIFEST_DIR =
    //    crates/picacg_app，需要向上两级到项目根目录
    let manifest_dir = option_env!("CARGO_MANIFEST_DIR");
    if let Some(dir) = manifest_dir {
        let project_root = std::path::Path::new(dir).parent().and_then(|p| p.parent());
        if let Some(root) = project_root {
            let path = root.join(BUNDLED_FONT_RELATIVE);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    // 2. 相对于可执行文件目录（生产环境，deploy 脚本将 assets 放在 bin 同级）
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let path = exe_dir.join(BUNDLED_FONT_RELATIVE);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    // 3. 相对于当前工作目录
    if std::path::Path::new(BUNDLED_FONT_RELATIVE).exists() {
        return Some(BUNDLED_FONT_RELATIVE.to_string());
    }

    tracing::debug!("未找到内置字体");
    None
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

    // 中文字体优先；TTF 优先于 TTC（避免 TTC 集合格式解析问题）
    let candidates: &[(&str, &[&str])] = &[
        ("Microsoft YaHei", &["msyh.ttf", "msyh.ttc"]),
        ("SimHei", &["simhei.ttf"]),
        ("SimSun", &["simsun.ttf", "simsun.ttc"]),
        ("KaiTi", &["simkai.ttf"]),
        // 英文回退
        ("Segoe UI", &["segoeui.ttf"]),
        ("Arial", &["arial.ttf"]),
    ];

    for (name, files) in candidates {
        for file in *files {
            let path = format!("{}\\{}", fonts_dir, file);
            if std::path::Path::new(&path).exists() {
                tracing::info!("检测到系统字体: {} ({})", name, file);
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
            tracing::info!("检测到系统字体: {} ({})", font_name, path);
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
            tracing::info!("检测到系统字体: {}", path);
            return Some(path.to_string());
        }
    }
    None
}
