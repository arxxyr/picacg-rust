//! Tokio 任务运行时集成
//!
//! 基于 bevy-tokio-tasks 的本地实现，适配 Bevy 0.18
//! 原项目: https://github.com/EkardNT/bevy-tokio-tasks
//!
//! MIT License

#![allow(clippy::type_complexity)]

use std::{
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        prelude::World,
        resource::Resource,
        schedule::{InternedScheduleLabel, ScheduleLabel},
    },
};
use tokio::{
    runtime::{Builder, Handle, Runtime},
    sync::{mpsc, oneshot, watch},
};

/// 插件配置
pub struct TokioTasksPlugin {
    // 使用 Mutex 包裹以满足 Sync trait 要求
    make_runtime: Mutex<Option<Box<dyn FnOnce() -> Runtime + Send>>>,
    schedule: InternedScheduleLabel,
}

impl Default for TokioTasksPlugin {
    fn default() -> Self {
        Self {
            make_runtime: Mutex::new(Some(Box::new(|| {
                Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build Tokio runtime")
            }))),
            schedule: Update.intern(),
        }
    }
}

impl TokioTasksPlugin {
    /// 创建使用当前线程运行时的插件
    #[allow(dead_code)]
    pub fn current_thread() -> Self {
        Self {
            make_runtime: Mutex::new(Some(Box::new(|| {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build Tokio runtime")
            }))),
            schedule: Update.intern(),
        }
    }

    /// 设置自定义运行时
    #[allow(dead_code)]
    pub fn with_runtime(self, make_runtime: impl FnOnce() -> Runtime + Send + 'static) -> Self {
        *self.make_runtime.lock().unwrap() = Some(Box::new(make_runtime));
        self
    }

    /// 设置运行调度
    #[allow(dead_code)]
    pub fn with_schedule(mut self, schedule: impl ScheduleLabel) -> Self {
        self.schedule = schedule.intern();
        self
    }
}

impl Plugin for TokioTasksPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let make_runtime = self
            .make_runtime
            .lock()
            .unwrap()
            .take()
            .expect("TokioTasksPlugin runtime already taken");
        let runtime = make_runtime();
        let handle = runtime.handle().clone();

        let (update_run_tx, update_run_rx) = mpsc::unbounded_channel();
        let (update_watch_tx, update_watch_rx) = watch::channel(());
        let ticks = Arc::new(AtomicU64::new(0));

        app.insert_resource(TokioTasksRuntime(Box::new(TokioTasksRuntimeInner {
            runtime,
            handle,
            update_run_tx,
            update_watch_rx,
            ticks: Arc::clone(&ticks),
        })));
        app.insert_resource(UpdateTicks {
            update_watch_tx,
            update_run_rx,
            ticks,
        });
        app.add_systems(self.schedule, tick_runtime_update);
    }
}

/// 更新计数资源
#[derive(Resource)]
struct UpdateTicks {
    update_watch_tx: watch::Sender<()>,
    update_run_rx: mpsc::UnboundedReceiver<Box<dyn FnOnce(&mut World) + Send>>,
    ticks: Arc<AtomicU64>,
}

fn tick_runtime_update(world: &mut World) {
    world.resource_scope(
        |world, mut ticks: bevy::ecs::change_detection::Mut<UpdateTicks>| {
            ticks.ticks.fetch_add(1, Ordering::SeqCst);
            let _ = ticks.update_watch_tx.send(());

            while let Ok(runnable) = ticks.update_run_rx.try_recv() {
                runnable(world);
            }
        },
    );
}

/// Tokio 任务运行时资源
#[derive(Resource)]
pub struct TokioTasksRuntime(Box<TokioTasksRuntimeInner>);

struct TokioTasksRuntimeInner {
    #[allow(dead_code)]
    runtime: Runtime,
    handle: Handle,
    update_run_tx: mpsc::UnboundedSender<Box<dyn FnOnce(&mut World) + Send>>,
    update_watch_rx: watch::Receiver<()>,
    ticks: Arc<AtomicU64>,
}

impl TokioTasksRuntime {
    /// 获取 Tokio 运行时句柄
    #[allow(dead_code)]
    pub fn handle(&self) -> &Handle {
        &self.0.handle
    }

    /// 在后台启动异步任务
    pub fn spawn_background_task<Task, Output>(
        &self,
        task: impl FnOnce(TaskContext) -> Task + Send + 'static,
    ) where
        Task: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
    {
        let ctx = TaskContext {
            update_watch_rx: self.0.update_watch_rx.clone(),
            ticks: Arc::clone(&self.0.ticks),
            update_run_tx: self.0.update_run_tx.clone(),
        };
        self.0.handle.spawn(task(ctx));
    }
}

/// 任务上下文，用于与主线程交互
pub struct TaskContext {
    update_watch_rx: watch::Receiver<()>,
    ticks: Arc<AtomicU64>,
    update_run_tx: mpsc::UnboundedSender<Box<dyn FnOnce(&mut World) + Send>>,
}

impl TaskContext {
    /// 在主线程上运行代码
    ///
    /// # Panics
    ///
    /// 如果主线程 channel 已关闭（通常意味着应用正在终止），会 panic。
    /// 这是预期行为，因为主线程关闭后无法执行任何回调。
    pub async fn run_on_main_thread<Runnable, Output>(&mut self, runnable: Runnable) -> Output
    where
        Runnable: FnOnce(&mut MainThreadContext) -> Output + Send + 'static,
        Output: Send + 'static,
    {
        let (result_tx, result_rx) = oneshot::channel();
        // 如果发送失败，说明主线程已终止，应用正在关闭
        if self
            .update_run_tx
            .send(Box::new(move |world| {
                let mut ctx = MainThreadContext { world };
                let result = runnable(&mut ctx);
                let _ = result_tx.send(result);
            }))
            .is_err()
        {
            tracing::warn!("主线程 channel 已关闭，无法执行回调");
            panic!("主线程 channel 已关闭");
        }
        // 等待结果，如果接收失败说明回调未执行
        result_rx.await.expect("回调结果接收失败，主线程可能已终止")
    }

    /// 等待下一个更新周期
    #[allow(dead_code)]
    pub async fn sleep_updates(&mut self, updates: u64) {
        let target_ticks = self.ticks.load(Ordering::SeqCst) + updates;
        while self.ticks.load(Ordering::SeqCst) < target_ticks {
            if self.update_watch_rx.changed().await.is_err() {
                break;
            }
        }
    }
}

/// 主线程上下文
pub struct MainThreadContext<'a> {
    /// 对 World 的可变引用
    pub world: &'a mut World,
}

impl<'a> MainThreadContext<'a> {
    /// 向 World 写入消息
    #[allow(dead_code)]
    pub fn write_message<E: bevy::prelude::Message>(&mut self, event: E) {
        // Bevy 0.18: 使用 Messages 资源发送消息
        if let Some(mut messages) = self.world.get_resource_mut::<bevy::prelude::Messages<E>>() {
            messages.write(event);
        }
    }
}
