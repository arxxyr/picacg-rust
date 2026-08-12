//! 字体加载
//!
//! 启动时同步加载 CJK 字体，替换 Bevy 默认字体（FiraMono，仅英文）。
//! 替换后 `Handle::default()` 即指向 CJK 字体。
//!
//! 全局统一接口：[`get_font()`] 返回 `Handle::default()`。
//! 所有 UI 系统通过此函数获取字体，无需额外的 Resource 或 static。

use bevy::{asset::AssetId, prelude::*, text::Font};

/// 获取全局字体句柄
///
/// `setup_fonts` 已将 CJK 字体注入 `AssetId::default()`，
/// 因此 `Handle::default()` 就是 CJK 字体。
#[inline]
pub fn get_font() -> Handle<Font> {
    Handle::default()
}

/// 内置字体的完整相对路径（从项目根或可执行文件目录，含 assets/ 前缀）
const BUNDLED_FONT_RELATIVE: &str = "assets/fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf";

/// Bevy 启动系统：同步加载 CJK 字体并替换默认字体
///
/// 通过 `std::fs::read()` + `Font::try_from_bytes()` 同步读取字体文件，
/// 然后用 `Assets::insert(AssetId::default(), font)` 替换 Bevy 内置的
/// FiraMono（纯英文字体），使所有文本节点默认就能渲染中文。
pub fn setup_fonts(mut fonts: ResMut<Assets<Font>>) {
    // 加载 CJK 字体（内置优先，系统回退）
    let font_asset = load_font_asset(&detect_bundled_font_path(), "内置字体").or_else(|| {
        tracing::warn!("内置字体未找到，尝试加载系统字体");
        load_font_asset(&detect_system_font_path(), "系统字体")
    });

    let Some(font) = font_asset else {
        tracing::error!("未找到任何可用字体，UI 中文将无法显示");
        return;
    };

    // 替换 Bevy 默认字体（FiraMono → CJK 字体）
    // Bevy TextPlugin 在 build() 中用 assets.insert(AssetId::default(), FiraMono)
    // 设置了默认字体。我们用同样的 API 替换它，之后 Handle::default() = CJK 字体。
    if let Err(e) = fonts.insert(AssetId::default(), font) {
        tracing::error!("设置默认字体失败: {:?}", e);
        return;
    }
    tracing::info!("已替换 Bevy 默认字体为 CJK 字体");
}

/// 从文件路径同步读取字体数据，成功返回 Font 资产
fn load_font_asset(path: &Option<String>, label: &str) -> Option<Font> {
    let path = path.as_ref()?;
    tracing::info!("使用{}: {}", label, path);
    match std::fs::read(path) {
        Ok(bytes) => {
            tracing::info!("{}读取成功: {} bytes", label, bytes.len());
            // Bevy 0.19: Font::from_bytes 不再返回 Result，解析延迟到文本布局阶段
            Some(Font::from_bytes(bytes))
        }
        Err(e) => {
            tracing::error!("{}读取失败: {} - {}", label, path, e);
            None
        }
    }
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

        // 2.1 macOS .app Bundle: 可执行文件在 Contents/MacOS/，
        //     资源在 Contents/Resources/
        #[cfg(target_os = "macos")]
        if exe_dir.ends_with("Contents/MacOS")
            && let Some(contents_dir) = exe_dir.parent()
        {
            let path = contents_dir.join("Resources").join(BUNDLED_FONT_RELATIVE);
            if path.exists() {
                tracing::debug!("检测到内置字体（macOS Bundle）: {}", path.display());
                return Some(path.to_string_lossy().to_string());
            }
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
