//! 应用图标
//!
//! 图标用的是**哔咔漫画官方 app 图标**：权威源
//! `assets/icons/picacg-official-192.png`（取自官网 PWA 的 logo_round），
//! 两个投放尺寸由 `scripts/make-icon.sh` 放大生成。
//! 全部 `include_bytes!` 编译进二进制——图标属于程序身份，
//! 不能因为部署时漏拷 assets 就退化成系统默认方块。
//!
//! 两条设置路径，缺一不可：
//!
//! | 平台 | 生效位置 | 手段 |
//! |------|----------|------|
//! | Windows / Linux | 标题栏、任务栏 | winit `Window::set_window_icon` |
//! | macOS | Dock、⌘Tab 切换器 | AppKit `NSApplication::setApplicationIconImage:` |
//!
//! macOS 上 winit 的窗口图标是**空操作**（系统只认 .app 包里的 icns），
//! 所以 dock 图标必须在运行时经 AppKit 设置——这样 `cargo run` 直接跑
//! 裸二进制也有图标，不必先打包成 .app。

use bevy::{ecs::system::NonSendMarker, prelude::*, winit::WINIT_WINDOWS};

/// 窗口/任务栏图标（官方图标，256×256 PNG）
const WINDOW_ICON_PNG: &[u8] = include_bytes!("../../../../assets/icons/icon-256.png");

/// 应用图标（官方图标，512×512 PNG，macOS dock 用）
#[cfg(target_os = "macos")]
const APP_ICON_PNG: &[u8] = include_bytes!("../../../../assets/icons/icon.png");

/// 设置窗口图标 + macOS dock 图标
///
/// 常驻 `Update` 而非只跑一次：窗口在合盖等场景会被系统销毁并由
/// `ensure_primary_window` 重建，重建出的新窗口需要重新贴图标。
/// 已处理过的窗口记在 `done` 里，稳态下每帧只是遍历一个 map 项。
///
/// winit 窗口在 bevy 0.19 存于主线程的 thread_local `WINIT_WINDOWS`
/// （不再是 `NonSend` 资源），故用 `NonSendMarker` 把本系统钉在主线程上
/// ——AppKit 与 winit 的窗口 API 都只能主线程调，与上游
/// `changed_windows` 的做法一致。
pub fn apply_app_icon(
    _non_send_marker: NonSendMarker,
    mut done: Local<std::collections::HashSet<Entity>>,
    mut cached_icon: Local<Option<winit::window::Icon>>,
    #[cfg(target_os = "macos")] mut dock_icon_set: Local<bool>,
) {
    // macOS dock 图标是应用级的，与窗口无关，设置一次即可
    #[cfg(target_os = "macos")]
    if !*dock_icon_set {
        *dock_icon_set = true;
        set_macos_dock_icon();
    }

    WINIT_WINDOWS.with_borrow(|windows| {
        // 窗口消失后从已处理集合里摘掉，重建时才会重新贴图标
        done.retain(|entity| windows.entity_to_winit.contains_key(entity));

        let pending: Vec<Entity> = windows
            .entity_to_winit
            .keys()
            .copied()
            .filter(|entity| !done.contains(entity))
            .collect();
        if pending.is_empty() {
            return;
        }

        // 解码结果缓存在 Local 里：窗口被系统销毁重建时不必重新解一遍 PNG
        //（release 实测单次约 8ms，白扔在启动路径上不值）
        if cached_icon.is_none() {
            *cached_icon = decode_window_icon();
        }
        let Some(icon) = cached_icon.as_ref() else {
            // 解码失败是编译期资源出了问题，重试也无意义——记为已处理，避免每帧重试
            done.extend(pending);
            return;
        };

        for entity in pending {
            if let Some(window) = windows.get_window(entity) {
                window.set_window_icon(Some(icon.clone()));
                tracing::debug!("已设置窗口图标: {:?}", entity);
            }
            done.insert(entity);
        }
    });
}

/// 解码内置 PNG 为 winit 图标
fn decode_window_icon() -> Option<winit::window::Icon> {
    let image = match image::load_from_memory(WINDOW_ICON_PNG) {
        Ok(image) => image.into_rgba8(),
        Err(e) => {
            tracing::error!("解码窗口图标失败: {}", e);
            return None;
        }
    };
    let (width, height) = image.dimensions();

    match winit::window::Icon::from_rgba(image.into_raw(), width, height) {
        Ok(icon) => Some(icon),
        Err(e) => {
            tracing::error!("构造窗口图标失败: {}", e);
            None
        }
    }
}

/// 设置 macOS dock 图标
///
/// 必须在主线程调用（由 `apply_app_icon` 的 `NonSendMarker` 参数保证）。
#[cfg(target_os = "macos")]
fn set_macos_dock_icon() {
    use objc2::{AnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(mtm) = MainThreadMarker::new() else {
        tracing::error!("设置 dock 图标失败: 不在主线程");
        return;
    };

    let data = NSData::with_bytes(APP_ICON_PNG);
    let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) else {
        tracing::error!("设置 dock 图标失败: PNG 解码失败");
        return;
    };

    // SAFETY: 主线程调用（MainThreadMarker 已证），image 为刚构造的有效 NSImage
    unsafe {
        NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&image));
    }
    tracing::info!("已设置 macOS dock 图标");
}
