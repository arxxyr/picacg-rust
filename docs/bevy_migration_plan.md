# PicACG Rust → Bevy 0.17.3 重构计划

## 概述

将 PicACG Rust 客户端从 **iced 0.13**（Elm 架构）重构为 **Bevy 0.17.3**（ECS 架构）。

### 前置步骤

```bash
# 1. 创建备份分支保留 iced 版本
cd picacg-rust
git checkout -b iced-backup
git push -u origin iced-backup

# 2. 切回主分支进行重构
git checkout main
```

### 架构对比

| 方面 | iced 0.13 | Bevy 0.17 |
|------|----------|-----------|
| 架构模式 | Elm（Message → Update → View） | ECS（Entity-Component-System） |
| 状态管理 | 集中式 AppState | 分布式 Component + Resource |
| 事件处理 | Message 枚举 | Event + Observer + System |
| UI 布局 | 函数式声明 | 实体层级 + Node 组件 |
| 异步任务 | Task::perform | bevy-tokio-tasks |

### 代码量评估

| 模块 | 现有代码行数 | 重构方式 | 预计工作量 |
|------|-------------|---------|-----------|
| api/ | ~1,100 | **保留**（无需修改） | 0 |
| db/ | ~720 | **保留**（无需修改） | 0 |
| download/ | ~930 | **保留**（无需修改） | 0 |
| config/ | ~150 | **保留**（无需修改） | 0 |
| error.rs | ~100 | **保留**（无需修改） | 0 |
| ui/ | ~2,300 | **完全重写** | ~2,500 行 |
| main.rs | ~40 | **重写** | ~100 行 |
| **总计** | ~7,340 | | ~2,600 行新代码 |

---

## 第一阶段：项目基础设施（预计 1-2 天）

### 1.1 更新 Cargo.toml

```toml
[package]
name = "picacg"
version = "0.2.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
# === Bevy 核心 ===
bevy = { version = "0.17.3", features = [
    "default",
    "wayland",           # Linux Wayland 支持
] }

# === 异步运行时集成 ===
bevy-tokio-tasks = "0.17"   # Tokio 集成（版本号与 Bevy 同步）
tokio = { version = "1.36", features = ["full"] }

# === UI 扩展 ===
bevy_ui_text_input = "0.9"  # 文本输入框（cosmic text）

# === 保留的依赖（无需改动） ===
reqwest = { version = "0.12", features = ["json", "cookies", "stream", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.9"
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
uuid = { version = "1.7", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
urlencoding = "2.1"
parking_lot = "0.12"
once_cell = "1.19"
directories = "6.0"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
moka = { version = "0.12", features = ["future"] }
image = "0.25"

[profile.release]
opt-level = 3
lto = "fat"
strip = true
panic = "abort"
codegen-units = 1
```

### 1.2 新目录结构

```
picacg-rust/
├── src/
│   ├── main.rs              # Bevy App 入口（重写）
│   ├── api/                 # API 层（保留）
│   ├── db/                  # 数据库层（保留）
│   ├── download/            # 下载管理（保留）
│   ├── config/              # 配置管理（保留）
│   ├── error.rs             # 错误处理（保留）
│   ├── plugins/             # 新增：Bevy 插件
│   │   ├── mod.rs
│   │   ├── ui_plugin.rs         # UI 主插件
│   │   ├── api_plugin.rs        # API 异步任务插件
│   │   └── download_plugin.rs   # 下载管理插件
│   ├── components/          # 新增：ECS 组件
│   │   ├── mod.rs
│   │   ├── ui_components.rs     # UI 相关组件
│   │   └── state_components.rs  # 状态组件
│   ├── resources/           # 新增：全局资源
│   │   ├── mod.rs
│   │   ├── app_state.rs         # 应用状态资源
│   │   ├── api_client.rs        # API 客户端资源
│   │   └── image_cache.rs       # 图片缓存资源
│   ├── events/              # 新增：事件定义
│   │   ├── mod.rs
│   │   ├── navigation.rs        # 导航事件
│   │   ├── api_events.rs        # API 响应事件
│   │   └── ui_events.rs         # UI 交互事件
│   ├── systems/             # 新增：ECS 系统
│   │   ├── mod.rs
│   │   ├── setup.rs             # 初始化系统
│   │   ├── login.rs             # 登录相关系统
│   │   ├── navigation.rs        # 导航系统
│   │   ├── categories.rs        # 分类浏览系统
│   │   ├── comics.rs            # 漫画列表系统
│   │   ├── detail.rs            # 详情页系统
│   │   └── reader.rs            # 阅读器系统
│   └── ui/                  # 新增：UI 布局定义
│       ├── mod.rs
│       ├── styles.rs            # 样式定义
│       ├── login_ui.rs          # 登录界面
│       ├── main_layout.rs       # 主布局
│       ├── sidebar.rs           # 侧边栏
│       ├── categories_ui.rs     # 分类界面
│       ├── comics_list_ui.rs    # 漫画列表界面
│       ├── detail_ui.rs         # 详情界面
│       └── reader_ui.rs         # 阅读器界面
└── resources/
    └── fonts/               # 字体资源（保留）
```

---

## 第二阶段：核心基础设施（预计 2-3 天）

### 2.1 主入口重写 (main.rs)

```rust
use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksPlugin;

mod api;
mod config;
mod db;
mod download;
mod error;
mod plugins;
mod components;
mod resources;
mod events;
mod systems;
mod ui;

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("PicACG Rust 客户端启动 (Bevy 版)");

    App::new()
        // Bevy 默认插件
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PicACG - Rust 版本".to_string(),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Tokio 集成
        .add_plugins(TokioTasksPlugin::default())
        // 自定义插件
        .add_plugins((
            plugins::UiPlugin,
            plugins::ApiPlugin,
            plugins::DownloadPlugin,
        ))
        .run();
}
```

### 2.2 全局资源定义 (resources/)

```rust
// resources/app_state.rs
use bevy::prelude::*;

/// 应用路由状态
#[derive(Debug, Clone, PartialEq, Eq, Default, States, Hash)]
pub enum AppRoute {
    #[default]
    Login,
    ProxySettings,
    Home,
    Categories,
    ComicsList { category: String },
    ComicDetail { comic_id: String },
    ReadView { comic_id: String, episode_order: i32 },
    Search,
    Favorites,
    Downloads,
    Settings,
}

/// 认证状态
#[derive(Resource, Default)]
pub struct AuthState {
    pub token: Option<String>,
    pub is_logged_in: bool,
}

/// 登录表单状态
#[derive(Resource, Default)]
pub struct LoginFormState {
    pub email: String,
    pub password: String,
    pub is_loading: bool,
    pub error: Option<String>,
}

/// 分类列表状态
#[derive(Resource, Default)]
pub struct CategoriesState {
    pub categories: Vec<crate::api::models::Category>,
    pub is_loading: bool,
    pub error: Option<String>,
}

/// 漫画列表状态
#[derive(Resource, Default)]
pub struct ComicsListState {
    pub category: String,
    pub comics: Vec<crate::api::models::Comic>,
    pub page: i32,
    pub total_pages: i32,
    pub is_loading: bool,
    pub error: Option<String>,
}

// ... 其他状态资源
```

### 2.3 事件定义 (events/)

```rust
// events/api_events.rs
use bevy::prelude::*;

/// 登录请求事件
#[derive(Event)]
pub struct LoginRequestEvent {
    pub email: String,
    pub password: String,
}

/// 登录响应事件
#[derive(Event)]
pub struct LoginResponseEvent {
    pub result: Result<String, String>, // Ok(token) 或 Err(error)
}

/// 加载分类请求
#[derive(Event)]
pub struct LoadCategoriesRequest;

/// 分类加载完成
#[derive(Event)]
pub struct CategoriesLoadedEvent {
    pub categories: Vec<crate::api::models::Category>,
}

/// 加载漫画列表请求
#[derive(Event)]
pub struct LoadComicsRequest {
    pub category: String,
    pub page: i32,
}

/// 漫画列表加载完成
#[derive(Event)]
pub struct ComicsLoadedEvent {
    pub comics: Vec<crate::api::models::Comic>,
    pub total_pages: i32,
}

// events/navigation.rs
use bevy::prelude::*;

/// 导航事件
#[derive(Event)]
pub struct NavigateEvent(pub crate::resources::AppRoute);

/// 返回上一页事件
#[derive(Event)]
pub struct NavigateBackEvent;
```

### 2.4 API 插件 (plugins/api_plugin.rs)

```rust
use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;

use crate::{
    api::ApiClient,
    events::*,
    resources::*,
};

pub struct ApiPlugin;

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册资源
            .insert_resource(ApiClientResource::new())
            // 注册事件
            .add_event::<LoginRequestEvent>()
            .add_event::<LoginResponseEvent>()
            .add_event::<LoadCategoriesRequest>()
            .add_event::<CategoriesLoadedEvent>()
            .add_event::<LoadComicsRequest>()
            .add_event::<ComicsLoadedEvent>()
            // 注册系统
            .add_systems(Update, (
                handle_login_request,
                handle_login_response,
                handle_load_categories,
                handle_categories_loaded,
                handle_load_comics,
                handle_comics_loaded,
            ));
    }
}

/// API 客户端资源包装
#[derive(Resource)]
pub struct ApiClientResource(pub ApiClient);

impl ApiClientResource {
    pub fn new() -> Self {
        Self(ApiClient::new().expect("Failed to create API client"))
    }
}

/// 处理登录请求 - 使用 Tokio 异步执行
fn handle_login_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut events: EventReader<LoginRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for event in events.read() {
        let email = event.email.clone();
        let password = event.password.clone();
        let client = api_client.0.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use crate::api::endpoints::LoginRequest;

            let request = LoginRequest { email, password };
            let result = match client.request(request).await {
                Ok(response) => Ok(response.token),
                Err(e) => Err(e.to_string()),
            };

            // 发送结果回主线程
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.send_event(LoginResponseEvent { result });
            }).await;
        });
    }
}

/// 处理登录响应
fn handle_login_response(
    mut events: EventReader<LoginResponseEvent>,
    mut auth_state: ResMut<AuthState>,
    mut login_form: ResMut<LoginFormState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    api_client: Res<ApiClientResource>,
) {
    for event in events.read() {
        login_form.is_loading = false;

        match &event.result {
            Ok(token) => {
                api_client.0.set_token(token.clone());
                auth_state.token = Some(token.clone());
                auth_state.is_logged_in = true;
                login_form.error = None;
                next_route.set(AppRoute::Home);
            }
            Err(error) => {
                login_form.error = Some(format!("登录失败: {}", error));
            }
        }
    }
}

// ... 其他 API 处理系统
```

---

## 第三阶段：UI 系统实现（预计 3-4 天）

### 3.1 UI 插件 (plugins/ui_plugin.rs)

```rust
use bevy::prelude::*;
use bevy_ui_text_input::TextInputPlugin;

use crate::{
    resources::AppRoute,
    systems::*,
    ui::*,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 添加文本输入支持
            .add_plugins(TextInputPlugin)
            // 注册状态
            .init_state::<AppRoute>()
            // 注册资源
            .init_resource::<LoginFormState>()
            .init_resource::<CategoriesState>()
            .init_resource::<ComicsListState>()
            // 设置系统
            .add_systems(Startup, setup_camera)
            // 路由相关系统
            .add_systems(OnEnter(AppRoute::Login), setup_login_ui)
            .add_systems(OnExit(AppRoute::Login), cleanup_login_ui)
            .add_systems(OnEnter(AppRoute::Home), setup_main_layout)
            .add_systems(OnEnter(AppRoute::Categories), setup_categories_ui)
            .add_systems(OnExit(AppRoute::Categories), cleanup_categories_ui)
            // ... 其他路由
            // 更新系统
            .add_systems(Update, (
                login_button_interaction,
                category_card_interaction,
                comic_card_interaction,
                pagination_interaction,
            ).run_if(in_state(AppRoute::Login).or(in_state(AppRoute::Categories))));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

### 3.2 登录界面 (ui/login_ui.rs)

```rust
use bevy::prelude::*;
use bevy_ui_text_input::{TextInputNode, TextInputSettings, InputFocus};

use crate::resources::LoginFormState;

/// 登录界面根节点标记
#[derive(Component)]
pub struct LoginRoot;

/// 用户名输入框标记
#[derive(Component)]
pub struct UsernameInput;

/// 密码输入框标记
#[derive(Component)]
pub struct PasswordInput;

/// 登录按钮标记
#[derive(Component)]
pub struct LoginButton;

/// 代理设置按钮标记
#[derive(Component)]
pub struct ProxySettingsButton;

/// 创建登录界面
pub fn setup_login_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    login_state: Res<LoginFormState>,
) {
    let font = asset_server.load("fonts/SarasaTermSCNerd-Regular.ttf");

    commands
        .spawn((
            LoginRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("PicACG 漫画客户端"),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.4, 0.8)),
            ));

            // 副标题
            parent.spawn((
                Text::new("Rust 重写版 (Bevy)"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 表单容器
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(400.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|form| {
                    // 用户名行
                    form.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("用户名:"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            Node {
                                width: Val::Px(80.0),
                                ..default()
                            },
                        ));

                        // 用户名输入框
                        row.spawn((
                            UsernameInput,
                            TextInput,
                            TextInputSettings {
                                mask_character: None,
                                retain_on_submit: true,
                            },
                            TextInputValue(login_state.email.clone()),
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor(Color::srgb(0.3, 0.3, 0.4)),
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                        ));
                    });

                    // 密码行
                    form.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("密码:"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            Node {
                                width: Val::Px(80.0),
                                ..default()
                            },
                        ));

                        // 密码输入框（使用掩码）
                        row.spawn((
                            PasswordInput,
                            TextInput,
                            TextInputSettings {
                                mask_character: Some('*'),
                                retain_on_submit: true,
                            },
                            TextInputValue(login_state.password.clone()),
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor(Color::srgb(0.3, 0.3, 0.4)),
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                        ));
                    });

                    // 登录按钮
                    form.spawn((
                        LoginButton,
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.4, 0.8)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(if login_state.is_loading { "登录中..." } else { "登录" }),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // 代理设置按钮
                    form.spawn((
                        ProxySettingsButton,
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("代理设置"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

            // 错误信息
            if let Some(ref error) = login_state.error {
                parent.spawn((
                    Text::new(error.clone()),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    Node {
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }
        });
}

/// 清理登录界面
pub fn cleanup_login_ui(
    mut commands: Commands,
    query: Query<Entity, With<LoginRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
```

### 3.3 按钮交互系统 (systems/login.rs)

```rust
use bevy::prelude::*;
use bevy_ui_text_input::TextInputValue;

use crate::{
    events::{LoginRequestEvent, NavigateEvent},
    resources::{AppRoute, LoginFormState},
    ui::login_ui::{LoginButton, PasswordInput, ProxySettingsButton, UsernameInput},
};

/// 登录按钮交互
pub fn login_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<LoginButton>),
    >,
    username_query: Query<&TextInputValue, With<UsernameInput>>,
    password_query: Query<&TextInputValue, With<PasswordInput>>,
    mut login_state: ResMut<LoginFormState>,
    mut login_events: EventWriter<LoginRequestEvent>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.3, 0.6));

                // 获取输入值
                let email = username_query
                    .get_single()
                    .map(|v| v.0.clone())
                    .unwrap_or_default();
                let password = password_query
                    .get_single()
                    .map(|v| v.0.clone())
                    .unwrap_or_default();

                // 验证
                if email.is_empty() || password.is_empty() {
                    login_state.error = Some("请输入用户名和密码".to_string());
                    return;
                }

                // 发送登录请求
                login_state.is_loading = true;
                login_state.error = None;
                login_events.send(LoginRequestEvent { email, password });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.45, 0.85));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.4, 0.8));
            }
        }
    }
}

/// 代理设置按钮交互
pub fn proxy_settings_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ProxySettingsButton>),
    >,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
                next_route.set(AppRoute::ProxySettings);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.45));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            }
        }
    }
}
```

---

## 第四阶段：高级 UI 组件（预计 2-3 天）

### 4.1 图片加载与显示

```rust
// resources/image_cache.rs
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ImageCache {
    pub handles: HashMap<String, Handle<Image>>,
    pub loading: HashMap<String, bool>,
}

// systems/image_loader.rs
pub fn load_image_system(
    runtime: ResMut<TokioTasksRuntime>,
    mut cache: ResMut<ImageCache>,
    mut events: EventReader<LoadImageRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in events.read() {
        if cache.handles.contains_key(&event.url) || cache.loading.contains_key(&event.url) {
            continue;
        }

        cache.loading.insert(event.url.clone(), true);
        let url = event.url.clone();
        let client = api_client.0.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            match download_image_bytes(&client, &url).await {
                Ok(bytes) => {
                    ctx.run_on_main_thread(move |ctx| {
                        // 创建 Bevy Image
                        let image = Image::from_buffer(
                            &bytes,
                            ImageType::Extension("png"),
                            CompressedImageFormats::all(),
                            true,
                            ImageSampler::default(),
                        ).unwrap_or_default();

                        let handle = ctx.world.add_asset(image);

                        ctx.world.send_event(ImageLoadedEvent {
                            url: url.clone(),
                            handle,
                        });
                    }).await;
                }
                Err(e) => {
                    tracing::error!("加载图片失败 {}: {}", url, e);
                }
            }
        });
    }
}
```

### 4.2 漫画卡片网格布局

```rust
// ui/comics_list_ui.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct ComicsGrid;

#[derive(Component)]
pub struct ComicCard {
    pub comic_id: String,
}

pub fn setup_comics_grid(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    comics_state: Res<ComicsListState>,
    image_cache: Res<ImageCache>,
) {
    let font = asset_server.load("fonts/SarasaTermSCNerd-Regular.ttf");

    commands
        .spawn((
            ComicsGrid,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::FlexStart,
                align_content: AlignContent::FlexStart,
                padding: UiRect::all(Val::Px(20.0)),
                column_gap: Val::Px(15.0),
                row_gap: Val::Px(15.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .with_children(|grid| {
            for comic in &comics_state.comics {
                spawn_comic_card(grid, comic, &font, &image_cache);
            }
        });
}

fn spawn_comic_card(
    parent: &mut ChildBuilder,
    comic: &crate::api::models::Comic,
    font: &Handle<Font>,
    image_cache: &ImageCache,
) {
    parent
        .spawn((
            ComicCard {
                comic_id: comic._id.clone(),
            },
            Button,
            Node {
                width: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgb(0.3, 0.3, 0.4)),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        ))
        .with_children(|card| {
            // 封面图片
            let thumb_url = comic.thumb.url();
            if let Some(handle) = image_cache.handles.get(&thumb_url) {
                card.spawn((
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(164.0),
                        height: Val::Px(220.0),
                        ..default()
                    },
                ));
            } else {
                // 占位符
                card.spawn((
                    Node {
                        width: Val::Px(164.0),
                        height: Val::Px(220.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                ));
            }

            // 标题
            card.spawn((
                Text::new(&comic.title),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ));

            // 作者
            card.spawn((
                Text::new(&comic.author),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
            ));
        });
}
```

### 4.3 滚动容器与阅读器

```rust
// ui/reader_ui.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct ReaderRoot;

#[derive(Component)]
pub struct ReaderImage;

#[derive(Resource)]
pub struct ReaderState {
    pub scale: f32,
    pub current_page: i32,
    pub total_pages: i32,
}

pub fn setup_reader_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    reader_state: Res<ReaderState>,
    image_cache: Res<ImageCache>,
) {
    commands
        .spawn((
            ReaderRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            // 顶部工具栏
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                ..default()
            })
            .with_children(|toolbar| {
                // 返回按钮、页码显示、缩放控制等
            });

            // 图片显示区域
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    overflow: Overflow::scroll(),
                    ..default()
                },
            ))
            .with_children(|container| {
                // 显示当前页图片
            });

            // 底部导航栏
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            })
            .with_children(|nav| {
                // 上一页、下一页按钮
            });
        });
}

/// 键盘导航
pub fn reader_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    mut prev_page_events: EventWriter<PrevPageEvent>,
    mut next_page_events: EventWriter<NextPageEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        prev_page_events.send(PrevPageEvent);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        next_page_events.send(NextPageEvent);
    }
    if keyboard_input.just_pressed(KeyCode::Equal) {
        reader_state.scale = (reader_state.scale + 0.1).min(3.0);
    }
    if keyboard_input.just_pressed(KeyCode::Minus) {
        reader_state.scale = (reader_state.scale - 0.1).max(0.5);
    }
}
```

---

## 第五阶段：完善与优化（预计 1-2 天）

### 5.1 样式系统

```rust
// ui/styles.rs
use bevy::prelude::*;

pub struct AppColors;

impl AppColors {
    pub const BACKGROUND: Color = Color::srgb(0.1, 0.1, 0.15);
    pub const SURFACE: Color = Color::srgb(0.15, 0.15, 0.2);
    pub const PRIMARY: Color = Color::srgb(0.2, 0.4, 0.8);
    pub const PRIMARY_HOVER: Color = Color::srgb(0.25, 0.45, 0.85);
    pub const TEXT: Color = Color::WHITE;
    pub const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.7);
    pub const ERROR: Color = Color::srgb(1.0, 0.3, 0.3);
    pub const SUCCESS: Color = Color::srgb(0.3, 0.8, 0.3);
    pub const BORDER: Color = Color::srgb(0.3, 0.3, 0.4);
}

pub struct AppSizes;

impl AppSizes {
    pub const FONT_TITLE: f32 = 32.0;
    pub const FONT_SUBTITLE: f32 = 24.0;
    pub const FONT_BODY: f32 = 16.0;
    pub const FONT_SMALL: f32 = 14.0;

    pub const SIDEBAR_WIDTH: f32 = 200.0;
    pub const CARD_WIDTH: f32 = 180.0;
    pub const CARD_IMAGE_HEIGHT: f32 = 220.0;
}
```

### 5.2 清理编译警告

```bash
# 运行 clippy 检查
cargo clippy --all -- -W clippy::all

# 格式化代码
cargo fmt --all
```

---

## 迁移检查清单

### 已保留模块 ✅

- [x] `api/client.rs` - API 客户端
- [x] `api/signer.rs` - 签名算法
- [x] `api/models.rs` - 数据模型
- [x] `api/endpoints/` - API 端点
- [x] `db/database.rs` - 数据库管理
- [x] `db/cache.rs` - 缓存层
- [x] `download/manager.rs` - 下载管理
- [x] `config/settings.rs` - 配置管理
- [x] `error.rs` - 错误处理

### 需要新建模块 📝

- [ ] `plugins/ui_plugin.rs` - UI 插件
- [ ] `plugins/api_plugin.rs` - API 插件
- [ ] `resources/app_state.rs` - 状态资源
- [ ] `events/*.rs` - 事件定义
- [ ] `systems/*.rs` - ECS 系统
- [ ] `ui/*.rs` - UI 布局

### 功能对照表

| iced 功能 | Bevy 对应 | 状态 |
|----------|----------|------|
| `Message` 枚举 | `Event` 结构体 | 📝 |
| `update()` | ECS System | 📝 |
| `view()` | Spawn UI 实体 | 📝 |
| `Task::perform` | `bevy-tokio-tasks` | 📝 |
| `text_input` | `bevy_simple_text_input` | 📝 |
| `scrollable` | `Overflow::scroll_y()` | 📝 |
| `image` | `ImageNode` | 📝 |
| `button` | `Button` + `Interaction` | 📝 |

---

## 风险与注意事项

### 1. TextInput 支持

Bevy 0.17 原生不支持 TextInput，使用 `bevy_ui_text_input`（基于 cosmic text）：
- 功能丰富，支持多行文本
- 使用 `InputFocus` 资源管理焦点
- 通过 `TextInputNode` 组件创建输入框

Bevy 0.18 将原生支持 TextInput。

### 2. 滚动容器

Bevy UI 的滚动通过 `Overflow::scroll_y()` 实现，与 iced 的 `scrollable` 不同：
- 不支持平滑滚动动画（需自行实现）
- 需要固定容器高度

### 3. 图片加载

- Bevy 使用 `Handle<Image>` 而非 `image::Handle`
- 需要通过 `AssetServer` 或手动创建 `Image` 资源
- 远程图片加载需要异步处理后转换

### 4. 字体加载

```rust
// Bevy 字体加载
let font: Handle<Font> = asset_server.load("fonts/SarasaTermSCNerd-Regular.ttf");

// 需要将字体文件放在 assets/ 目录下
// 或使用嵌入式字体
```

---

## 参考资源

- [Bevy 0.17 发布说明](https://bevy.org/news/bevy-0-17/)
- [Bevy UI 教程](https://taintedcoders.com/bevy/ui)
- [bevy-tokio-tasks](https://crates.io/crates/bevy-tokio-tasks)
- [bevy_ui_text_input](https://crates.io/crates/bevy_ui_text_input) - 文本输入框
- [bevy::ui_widgets](https://docs.rs/bevy/latest/bevy/ui_widgets/index.html)
