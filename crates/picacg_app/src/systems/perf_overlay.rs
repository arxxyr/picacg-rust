//! 性能叠加层（F3）与系统耗时榜（F4）
//!
//! 用法见 [`crate::utils::profiling`] 的模块文档。设置页「性能追踪」分组是
//! 等价入口（并且能直接在页面里看榜单，打包成 .app 后没有终端可看）。
//! 叠加层默认关闭，关闭时每帧只有一次布尔判断，不查诊断也不改任何节点。

use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{systems::login::AppColors, utils::profiling};

/// 叠加层根节点标记
#[derive(Component, Default, Clone)]
pub struct PerfOverlay;

/// 叠加层文本标记
#[derive(Component, Default, Clone)]
pub struct PerfOverlayText;

/// 叠加层开关
#[derive(Resource, Default)]
pub struct PerfOverlayState {
    pub visible: bool,
}

/// 榜单一次打印几行
const REPORT_ROWS: usize = 15;

/// 自动打榜的默认帧时阈值（毫秒），可用 `PICACG_PROFILE_SLOW_MS` 覆盖
const DEFAULT_SLOW_FRAME_MS: f32 = 100.0;

/// 自动打榜的最小间隔——卡顿常连着来，不加冷却会把日志刷爆
const AUTO_REPORT_COOLDOWN_SECS: f32 = 5.0;

/// F3 切换叠加层显隐
pub fn toggle_perf_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PerfOverlayState>,
    mut overlay_query: Query<&mut Node, With<PerfOverlay>>,
) {
    if !keys.just_pressed(KeyCode::F3) {
        return;
    }
    state.visible = !state.visible;
    for mut node in overlay_query.iter_mut() {
        node.display = if state.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    tracing::info!(
        "性能叠加层: {}",
        if state.visible { "显示" } else { "隐藏" }
    );
}

/// 刷新叠加层文本（仅在显示时工作）
pub fn update_perf_overlay(
    state: Res<PerfOverlayState>,
    diagnostics: Res<DiagnosticsStore>,
    mut text_query: Query<&mut Text, With<PerfOverlayText>>,
    ui_nodes: Query<(), With<Node>>,
) {
    if !state.visible {
        return;
    }

    // DiagnosticPath 是 const 构造，取地址会产生临时量，先绑定成局部变量
    let (fps_path, frame_path, entity_path) = (
        FrameTimeDiagnosticsPlugin::FPS,
        FrameTimeDiagnosticsPlugin::FRAME_TIME,
        EntityCountDiagnosticsPlugin::ENTITY_COUNT,
    );
    let value = |path| {
        diagnostics
            .get(path)
            .and_then(bevy::diagnostic::Diagnostic::smoothed)
    };
    let fps = value(&fps_path).unwrap_or(0.0);
    let frame_ms = value(&frame_path).unwrap_or(0.0);
    let entities = value(&entity_path).unwrap_or(0.0);

    let report = format!(
        "FPS {fps:.0}   帧 {frame_ms:.1}ms\n实体 {entities:.0}   UI 节点 {}\nF4 打印系统耗时榜{}",
        ui_nodes.iter().count(),
        if profiling::is_enabled() {
            ""
        } else {
            "（未启用）"
        },
    );

    for mut text in text_query.iter_mut() {
        if text.as_str() != report {
            **text = report.clone();
        }
    }
}

/// F4 打印系统耗时榜
pub fn print_system_timings(
    keys: Res<ButtonInput<KeyCode>>,
    route: Res<State<crate::resources::AppRoute>>,
) {
    if !keys.just_pressed(KeyCode::F4) {
        return;
    }

    if !profiling::is_enabled() {
        tracing::warn!("系统耗时统计未启用——设置页「性能追踪」打开开关后重启程序");
        return;
    }

    log_report(&format!(
        "系统耗时榜（上次打榜之后的累计，按总耗时降序，页面 {:?}）",
        route.get()
    ));
}

/// 掉帧自动打榜
///
/// 卡顿往往是零星的，等用户反应过来按 F4 时现场早没了。启用性能追踪后，
/// 只要某帧超过阈值就自动把榜单打出来——**捕捉的是刚过去那一段**，因为
/// `take_report` 每次取完即清零。
pub fn auto_report_slow_frames(
    time: Res<Time>,
    route: Res<State<crate::resources::AppRoute>>,
    mut cooldown: Local<f32>,
    mut threshold_ms: Local<Option<f32>>,
) {
    if !profiling::is_enabled() {
        return;
    }

    let threshold = *threshold_ms.get_or_insert_with(|| {
        std::env::var("PICACG_PROFILE_SLOW_MS")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| *v > 0.0)
            .unwrap_or(DEFAULT_SLOW_FRAME_MS)
    });

    let delta_ms = time.delta_secs() * 1000.0;
    *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    if delta_ms < threshold || *cooldown > 0.0 {
        return;
    }
    *cooldown = AUTO_REPORT_COOLDOWN_SECS;

    // 页面写进标题：卡顿多半是某个页面一次性建大量节点导致的，
    // 不带页面信息的榜单只能看出「谁慢」，看不出「在哪慢」
    log_report(&format!(
        "掉帧自动打榜（本帧 {delta_ms:.0}ms，阈值 {threshold:.0}ms，页面 {:?}）",
        route.get()
    ));
}

/// 打印一次榜单（取完即清零），同时追加落盘
fn log_report(headline: &str) {
    let rows = profiling::take_report(REPORT_ROWS);
    if rows.is_empty() {
        tracing::info!("{headline}：本区间无数据");
        return;
    }

    let mut out = format!("{headline}\n    总耗时ms    单次峰值ms   次数   系统\n");
    for row in &rows {
        out.push_str(&format!(
            "  {:>10.2}   {:>10.3}   {:>5}   {}\n",
            row.total_ms, row.max_ms, row.calls, row.name
        ));
    }
    tracing::info!("{}", out.trim_end());
    profiling::append_report_to_log(headline, &rows);
}

/// 创建叠加层节点（Startup，初始隐藏）
pub fn setup_perf_overlay(mut commands: Commands) {
    commands.spawn_scene(perf_overlay_scene());
}

/// 叠加层场景
fn perf_overlay_scene() -> impl Scene {
    bsn! {
        PerfOverlay
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            display: Display::None,
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72))
        // 压在所有页面之上，含右键菜单（ZIndex 100）与浮动标题
        ZIndex(1000)
        Children [
            (
                PerfOverlayText
                Text("")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}
