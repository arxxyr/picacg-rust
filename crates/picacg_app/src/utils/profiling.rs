//! 性能追踪
//!
//! 两层工具，回答两个不同的问题：
//!
//! | 入口 | 回答 | 前提 |
//! |------|------|------|
//! | 设置页「性能叠加层」/ `F3` | **卡不卡、多卡**：FPS / 帧时间 / 实体数 / UI 节点数 | 无，立即生效 |
//! | 设置页「系统耗时追踪」/ `F4` | **谁在卡**：按累计耗时排序的系统 Top N | 开关打开 + **重启** |
//!
//! 榜单直接渲染在设置页里（打包后的 .app 看不到终端日志），同时**追加落盘**到
//! [`report_log_path`]（配置目录下的 `logs/profiling.log`）——排查卡顿要的是
//! 现场，靠肉眼记榜或截屏都留不住，落盘才能事后翻、能整份发出去。
//!
//! ## 系统耗时榜的原理
//!
//! bevy 给每个系统的每次执行套了 `info_span!("system", name = ...)`
//! （`bevy_ecs/system/function_system.rs`）。这里挂一个 `tracing` Layer 把这些
//! span 的 enter/exit 差值按系统名累加，于是不必改动那 120+ 个系统的注册代码
//! 就能拿到逐系统耗时。
//!
//! ## 为什么开关要重启才生效
//!
//! 两层门，缺一不可：
//!
//! 1. **编译期**：那对 span 在 bevy_ecs 里是 `#[cfg(feature = "trace")]`
//!    门控的， 不编进来就压根不存在（曾照着"不受 feature
//!    门控"的错误判断实现过一版， 跑出来榜单恒空）。故 `profiling` 是**默认
//!    feature**。
//! 2. **进程启动期**：bevy 在系统初始化时**一次性**建好那些 `Span` 对象并存进
//!    `SystemMeta`。建的那一刻若没有订阅者对该 callsite 感兴趣，span
//!    就是禁用态， 此后 `.enter()` 永远是空操作——**运行中再装 Layer
//!    也救不回来**。
//!
//! 反过来说，这也正是关掉时几乎零成本的原因：没人感兴趣 → tracing 把 callsite
//! 缓存成 `Interest::never()` → 每个系统每帧只多一次分支。
//!
//! ⚠️ span 是 `info` 级：设置里的日志等级低于 info 时 tracing 会把 span 整个
//! 过滤掉，榜单会是空的——profiling 期间把日志等级保持在 info 或以上。
//!
//! ⚠️ **已知偏差**：个别启动期系统的「次数」会明显偏高（实测某 `Startup` 系统
//! 函数体只跑了 1 次，span 却被进出 280 次，且出现 enter 套 enter）。原因是
//! bevy 内部会共享/克隆同一个 `Span` 对象，多个系统的进出落到同一个 id 上，
//! 名字只能记成其中一个——这是上游行为，本层改不了。稳态运行的数字是准的
//! （所有系统的次数都等于帧数），受影响的条目耗时也都在零点几毫秒量级，
//! 不影响"谁最慢"的判断。

use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use parking_lot::Mutex;
use tracing::span::{Attributes, Id};
use tracing_subscriber::{Layer, field::Visit, layer::Context, registry::LookupSpan};

/// Layer 是否已安装（决定 F4 是打榜还是提示怎么开）
static ENABLED: AtomicBool = AtomicBool::new(false);

/// 单个系统的累计数据
#[derive(Default, Clone, Copy)]
struct SystemStat {
    /// 执行次数
    calls: u64,
    /// 累计耗时（纳秒）
    total_ns: u128,
    /// 单次最长耗时（纳秒）
    max_ns: u128,
}

/// 系统名 → 累计数据
static STATS: Mutex<Option<HashMap<String, SystemStat>>> = Mutex::new(None);

/// 一行榜单
pub struct SystemTiming {
    pub name: String,
    pub calls: u64,
    pub total_ms: f64,
    pub max_ms: f64,
}

/// 是否已启用系统耗时统计
#[must_use]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// 本次构建是否编进了系统耗时 span
#[must_use]
pub fn is_compiled_in() -> bool {
    cfg!(feature = "profiling")
}

/// 本次启动是否要装聚合 Layer
///
/// = 编译期编进了 span **且** 配置里打开了开关。只在启动时读一次——
/// 见模块文档「为什么开关要重启才生效」。
#[must_use]
pub fn wants_profiling() -> bool {
    is_compiled_in() && picacg_config::AppSettings::global().read().enable_profiling
}

/// 构造 Layer 并标记为已启用（仅在 `wants_profiling()` 为真时调用）
#[must_use]
pub fn layer() -> SystemProfilerLayer {
    ENABLED.store(true, Ordering::Relaxed);
    *STATS.lock() = Some(HashMap::new());
    SystemProfilerLayer
}

/// 取出榜单（按累计耗时降序），并清空累计——每次 F4 报告的是**上次之后**的区间
pub fn take_report(limit: usize) -> Vec<SystemTiming> {
    let mut guard = STATS.lock();
    let Some(stats) = guard.as_mut() else {
        return Vec::new();
    };

    let mut rows: Vec<SystemTiming> = stats
        .iter()
        .map(|(name, stat)| SystemTiming {
            name: name.clone(),
            calls: stat.calls,
            total_ms: stat.total_ns as f64 / 1.0e6,
            max_ms: stat.max_ns as f64 / 1.0e6,
        })
        .collect();
    stats.clear();
    drop(guard);

    rows.sort_by(|a, b| b.total_ms.total_cmp(&a.total_ms));
    rows.truncate(limit);
    rows
}

/// 榜单落盘路径（配置目录下的 `logs/profiling.log`）
#[must_use]
pub fn report_log_path() -> std::path::PathBuf {
    picacg_config::AppSettings::profiling_log_path()
}

/// 把一次榜单追加进日志文件
///
/// **追加**而非覆盖：排查卡顿常要对比几次采样才看得出规律。
/// 写失败只记一条 warn——诊断产物落不下来不该拖累主流程。
pub fn append_report_to_log(headline: &str, rows: &[SystemTiming]) {
    use std::io::Write;

    let path = report_log_path();
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        tracing::warn!("创建日志目录失败: {} - {}", parent.display(), e);
        return;
    }

    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(e) => {
            tracing::warn!("打开耗时榜日志失败: {} - {}", path.display(), e);
            return;
        }
    };

    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let mut block = format!("\n===== {stamp}  {headline} =====\n");
    block.push_str("    总耗时ms    单次峰值ms   次数   系统\n");
    for row in rows {
        block.push_str(&format!(
            "  {:>10.2}   {:>10.3}   {:>5}   {}\n",
            row.total_ms, row.max_ms, row.calls, row.name
        ));
    }

    if let Err(e) = file.write_all(block.as_bytes()) {
        tracing::warn!("写入耗时榜日志失败: {} - {}", path.display(), e);
    }
}

// ==================== tracing Layer ====================

/// 把 bevy 的 `system` span 折算成逐系统耗时
pub struct SystemProfilerLayer;

/// span 上挂的系统名（`on_new_span` 时解析一次，之后每次进出复用）
struct SpanSystemName(String);

/// span 本次进入的时刻
struct SpanEnteredAt(Instant);

/// 只取 `name` 字段
#[derive(Default)]
struct NameVisitor(Option<String>);

impl Visit for NameVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "name" {
            self.0 = Some(format!("{value:?}").trim_matches('"').to_string());
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "name" {
            self.0 = Some(value.to_string());
        }
    }
}

impl<S> Layer<S> for SystemProfilerLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        // 只认 bevy 的系统 span，其余 span（含业务日志）一概不碰
        if attrs.metadata().name() != "system" {
            return;
        }
        let mut visitor = NameVisitor::default();
        attrs.record(&mut visitor);
        let Some(name) = visitor.0 else {
            return;
        };
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanSystemName(name));
        }
    }

    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        // 没有系统名 = 不是系统 span，直接放行
        if span.extensions().get::<SpanSystemName>().is_none() {
            return;
        }
        // 必须用 replace 而非 insert：同一个 span 每帧都会重新进入，而
        // `Extensions::insert` 撞到同类型已存在会直接 panic
        span.extensions_mut().replace(SpanEnteredAt(Instant::now()));
    }

    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let ext = span.extensions();
        let (Some(name), Some(entered)) = (ext.get::<SpanSystemName>(), ext.get::<SpanEnteredAt>())
        else {
            return;
        };
        let elapsed = entered.0.elapsed().as_nanos();
        let name = name.0.clone();
        drop(ext);

        if let Some(stats) = STATS.lock().as_mut() {
            let entry = stats.entry(name).or_default();
            entry.calls += 1;
            entry.total_ns += elapsed;
            entry.max_ns = entry.max_ns.max(elapsed);
        }
    }
}
