//! 字体检测与加载
//!
//! 双路加载策略：
//! 1. 主路径：Bevy AssetServer 异步加载内置 Sarasa 字体（标准管线，最可靠）
//! 2. 回退：手动读取系统字体字节（当内置字体不可用时）
//!
//! AssetServer 加载的字体经过 Bevy 完整的资产处理管线，
//! 与 cosmic_text 文本渲染引擎完全兼容。

use std::sync::OnceLock;

use bevy::{prelude::*, text::Font};

/// 全局字体句柄（`setup_fonts` 初始化后可用）
static FONT_HANDLE: OnceLock<Handle<Font>> = OnceLock::new();

/// 获取全局字体句柄，供所有 UI 系统使用
///
/// 如果 `setup_fonts` 尚未运行（理论上不应该），返回默认句柄而非 panic
pub fn get_font() -> Handle<Font> {
    FONT_HANDLE.get().cloned().unwrap_or_default()
}

/// 应用字体资源
#[derive(Resource, Clone, Default)]
pub struct AppFont(pub Handle<Font>);

/// 内置字体的 Bevy 资产路径（相对于 AssetPlugin::file_path 配置的 assets 目录）
const BUNDLED_FONT_ASSET_PATH: &str = "fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf";

/// 内置字体的完整相对路径（从项目根或可执行文件目录，含 assets/ 前缀）
const BUNDLED_FONT_RELATIVE: &str = "assets/fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf";

/// Bevy 启动系统：加载字体
///
/// 优先通过 AssetServer 加载内置 Sarasa 字体（走 Bevy 标准管线），
/// 如果内置字体文件不存在则回退到手动加载系统字体。
pub fn setup_fonts(
    asset_server: Res<AssetServer>,
    mut fonts: ResMut<Assets<Font>>,
    mut app_font: ResMut<AppFont>,
) {
    // 检查内置字体是否存在于文件系统中
    let bundled_exists = detect_bundled_font_path().is_some();

    if bundled_exists {
        // 主路径：通过 AssetServer 加载（Bevy 标准管线）
        // AssetServer 使用 AssetPlugin::file_path 配置的 assets 目录
        let handle: Handle<Font> = asset_server.load(BUNDLED_FONT_ASSET_PATH);
        app_font.0 = handle.clone();
        FONT_HANDLE.set(handle).ok();
        tracing::info!(
            "使用内置字体（AssetServer 加载）: {}",
            BUNDLED_FONT_ASSET_PATH
        );
        return;
    }

    // 回退：手动加载系统字体字节
    tracing::warn!("内置字体未找到，尝试加载系统字体");
    if let Some(path) = detect_system_font_path() {
        tracing::info!("使用系统字体: {}", path);
        match std::fs::read(&path) {
            Ok(bytes) => {
                tracing::info!("系统字体读取成功: {} bytes", bytes.len());
                match Font::try_from_bytes(bytes) {
                    Ok(font) => {
                        let handle = fonts.add(font);
                        app_font.0 = handle.clone();
                        FONT_HANDLE.set(handle).ok();
                        tracing::info!("系统字体加载成功");
                        return;
                    }
                    Err(e) => {
                        tracing::error!("系统字体解析失败: {:?}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("系统字体读取失败: {} - {}", path, e);
            }
        }
    }

    tracing::error!("未找到任何可用字体，UI 文字将无法显示");
}

// ============ 路径检测 ============

/// 检测内置字体是否存在于文件系统中
fn detect_bundled_font_path() -> Option<String> {
    // 1. 通过 CARGO_MANIFEST_DIR 定位（开发环境） CARGO_MANIFEST_DIR =
    //    crates/picacg_app，需要向上两级到项目根目录
    if let Some(dir) = option_env!("CARGO_MANIFEST_DIR")
        && let Some(root) = std::path::Path::new(dir).parent().and_then(|p| p.parent())
    {
        let path = root.join(BUNDLED_FONT_RELATIVE);
        if path.exists() {
            tracing::debug!("检测到内置字体（开发环境）: {}", path.display());
            return Some(path.to_string_lossy().to_string());
        }
    }

    // 2. 相对于可执行文件目录（生产环境，deploy 脚本将 assets 放在 bin 同级）
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let path = exe_dir.join(BUNDLED_FONT_RELATIVE);
        if path.exists() {
            tracing::debug!("检测到内置字体（生产环境）: {}", path.display());
            return Some(path.to_string_lossy().to_string());
        }
    }

    // 3. 相对于当前工作目录
    let path = std::path::Path::new(BUNDLED_FONT_RELATIVE);
    if path.exists() {
        tracing::debug!("检测到内置字体（工作目录）: {}", path.display());
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
