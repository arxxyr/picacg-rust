# PicACG Rust 客户端架构文档

> 最后更新: 2025-12-08

## 项目概述

PicACG Rust 是原 Python 版 PicACG 漫画客户端的 Rust 重写版本，使用 **Bevy 0.17.3** 游戏引擎作为 UI 框架，采用 ECS (Entity-Component-System) 架构。

### 技术栈

| 类别 | 技术选型 |
|------|----------|
| UI 框架 | Bevy 0.17.3 (ECS 架构) |
| 异步运行时 | Tokio |
| HTTP 客户端 | reqwest (HTTP/2 + SOCKS5) |
| 数据库 | SQLite (sqlx) |
| 缓存 | Moka (LRU + TTL) |
| 序列化 | serde + serde_json |
| 内存分配 | mimalloc |

### 项目状态

- **完成度**: ~80%
- **代码量**: ~8000 行 Rust
- **二进制大小**: ~15 MB (Release)
- **启动时间**: < 500ms

---

## 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Bevy App                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  UiPlugin   │  │  ApiPlugin  │  │DownloadPlugin│            │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                     │
│         ▼                ▼                ▼                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    ECS Systems                           │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ │  │
│  │  │ login  │ │category│ │ comics │ │scrollbar│ │  ...   │ │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐            │
│         ▼                    ▼                    ▼            │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐       │
│  │  Resources  │     │  Components │     │   Events    │       │
│  │ ─────────── │     │ ─────────── │     │ ─────────── │       │
│  │ AuthState   │     │ ScrollPos   │     │ LoginReq    │       │
│  │ ImageCache  │     │ ComicCard   │     │ LoadComics  │       │
│  │ AppFont     │     │ CategoryCard│     │ Navigate    │       │
│  └─────────────┘     └─────────────┘     └─────────────┘       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Core Crates (crates/)                      │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────┐   │
│  │picacg_api │ │ picacg_db │ │picacg_    │ │bevy_ui_toolkit│   │
│  │ ───────── │ │ ───────── │ │  config   │ │ ───────────── │   │
│  │ client    │ │ database  │ │ ───────── │ │ scrollbar     │   │
│  │ signer    │ │ cache     │ │ settings  │ │ pagination    │   │
│  │ models    │ │ models    │ │           │ │ waterfall     │   │
│  │ endpoints │ │           │ │           │ │ theme         │   │
│  └───────────┘ └───────────┘ └───────────┘ └───────────────┘   │
│                      ↑ 依赖 picacg_core (错误类型)              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 目录结构

采用纯 Cargo Workspace 结构：

```
picacg-rust/
├── Cargo.toml              # 纯 Workspace 配置（无 [package]）
├── assets/                 # 资源文件
│   └── fonts/
│       └── SarasaTermSCNerd-Regular.ttf
├── docs/                   # 文档
└── crates/
    ├── picacg_app/         # 主应用 (picacg)
    │   └── src/
    │       ├── main.rs     # 入口
    │       ├── error.rs    # 错误类型
    │       ├── plugins/    # Bevy 插件
    │       │   ├── ui_plugin.rs
    │       │   └── api_plugin.rs
    │       ├── components/ # ECS 组件
    │       ├── resources/  # 全局资源
    │       ├── events/     # 事件定义
    │       └── systems/    # ECS 系统（页面逻辑）
    │           ├── login.rs, register.rs
    │           ├── categories.rs, comics.rs
    │           ├── detail.rs, reader.rs
    │           ├── favorites.rs, rankings.rs
    │           ├── search.rs, downloads.rs
    │           ├── settings.rs, proxy_settings.rs
    │           ├── scrollbar.rs, pagination.rs
    │           └── waterfall.rs
    │
    ├── picacg_core/        # 核心类型库
    │   └── src/
    │       └── error.rs    # PicacgError, Result
    │
    ├── picacg_api/         # API 客户端
    │   └── src/
    │       ├── client.rs   # ApiClient
    │       ├── signer.rs   # 请求签名
    │       ├── models.rs   # 数据模型
    │       └── endpoints/  # API 端点 (28+)
    │
    ├── picacg_db/          # 数据库层
    │   └── src/
    │       ├── database.rs # SQLite 操作
    │       ├── cache.rs    # Moka 缓存
    │       └── models.rs   # 数据库模型
    │
    ├── picacg_config/      # 配置管理
    │   └── src/
    │       └── settings.rs # AppSettings, ProxySettings
    │
    └── bevy_ui_toolkit/    # 通用 UI 组件库
        └── src/
            ├── theme.rs    # 主题系统
            ├── scrollbar/  # 滚动条组件
            ├── pagination/ # 分页组件
            └── waterfall/  # 瀑布流布局
```

---

## 核心模块说明

### 1. 插件系统 (`plugins/`)

#### UiPlugin (`ui_plugin.rs`)
UI 主插件，负责：
- 注册应用状态 (`AppRoute`)
- 注册全局资源 (AuthState, ImageCache 等)
- 注册 UI 事件 (Bevy 0.17 使用 `add_message`)
- 配置页面生命周期 (`OnEnter`/`OnExit`)
- 注册 Update 系统

```rust
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<AppRoute>()
            .init_resource::<AuthState>()
            .add_message::<NavigateToCategoriesEvent>()
            .add_systems(OnEnter(AppRoute::Login), setup_login_ui)
            .add_systems(Update, login_button_interaction.run_if(in_state(AppRoute::Login)));
    }
}
```

#### ApiPlugin (`api_plugin.rs`)
API 异步任务插件，使用 `bevy-tokio-tasks` 集成 Tokio：
- 处理登录请求
- 加载分类数据
- 加载漫画列表
- 图片异步下载

### 2. 状态路由 (`resources/app_state.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default, States, Hash)]
pub enum AppRoute {
    #[default]
    Login,
    ProxySettings,
    Categories,
    ComicsList,
    ComicDetail,
    Reader,
}
```

### 3. 事件系统 (`events/`)

Bevy 0.17 使用 `Message` trait (而非 `Event`):

```rust
// 定义消息
#[derive(Event, Clone)]
pub struct LoadCategoriesRequest;

// 发送消息
fn trigger_load(mut writer: MessageWriter<LoadCategoriesRequest>) {
    writer.write(LoadCategoriesRequest);
}

// 接收消息
fn handle_load(mut reader: MessageReader<LoadCategoriesRequest>) {
    for _ in reader.read() {
        // 处理加载
    }
}
```

### 4. UI 组件 (`components/`)

关键 UI 组件标记：

| 组件 | 用途 |
|------|------|
| `LoginRoot` | 登录页面根节点 |
| `MainLayoutRoot` | 主布局根节点 |
| `ScrollContainer` | 可滚动容器 |
| `ScrollPosition` | 滚动位置 |
| `ScrollbarTrack` | 滚动条轨道 |
| `ScrollbarThumb` | 滚动条滑块 |
| `CategoryCard` | 分类卡片 |
| `ComicCard` | 漫画卡片 |

### 5. 自定义滚动条 (`systems/scrollbar.rs`)

实现 VSCode 风格滚动条：
- 轨道点击快速跳转
- 滑块拖拽滚动
- 自动计算滑块大小
- DPI 缩放适配

---

## 关键技术要点

### Bevy 0.17 API 变更

| iced 0.13 / Bevy 0.16 | Bevy 0.17.3 |
|----------------------|-------------|
| `Event` trait | `Message` trait |
| `EventWriter::send()` | `MessageWriter::write()` |
| `EventReader<T>` | `MessageReader<T>` |
| `add_event::<T>()` | `add_message::<T>()` |
| `BorderColor(color)` | `BorderColor::all(color)` |
| `despawn_recursive()` | `despawn()` (自动递归) |
| `ReceivedCharacter` | `KeyboardInput` + `logical_key` |

### DPI 缩放处理

**核心原则**: Bevy UI 使用逻辑像素，但 `ComputedNode::size()` 返回物理像素。

```rust
fn get_scale_factor(window_query: &Query<&Window, With<PrimaryWindow>>) -> f32 {
    window_query
        .single()
        .ok()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0)
}

// 使用时需要转换
let viewport_height = scroll_computed.size().y / scale_factor;
```

### 字体配置

必须显式配置 `AssetPlugin`:

```rust
let manifest_dir = env!("CARGO_MANIFEST_DIR");
let assets_path = std::path::Path::new(manifest_dir).join("assets");

App::new()
    .add_plugins(
        DefaultPlugins.set(AssetPlugin {
            file_path: assets_path.to_string_lossy().to_string(),
            ..default()
        })
    )
```

---

## 页面实现状态

| 页面 | 状态 | 说明 |
|------|------|------|
| 登录页面 | ✅ 完成 | 用户名/密码输入，键盘导航 |
| 代理设置 | ✅ 完成 | HTTP/HTTPS/SOCKS5 配置 |
| 分类页面 | ✅ 完成 | 分类卡片网格，图片加载 |
| 漫画列表 | ✅ 完成 | 漫画卡片，分页控制 |
| 自定义滚动条 | ✅ 完成 | 轨道点击，滑块拖拽 |
| 漫画详情 | ⏳ 进行中 | 基础框架已有 |
| 阅读器 | ⏳ 待实现 | - |
| 下载管理 | ⏳ 待实现 | 核心逻辑已有 |
| 搜索 | ⏳ 待实现 | API 已就绪 |
| 收藏 | ⏳ 待实现 | API 已就绪 |

---

## 性能指标

| 指标 | Rust 版本 | Python 版本 | 提升 |
|------|----------|------------|------|
| 启动时间 | < 500ms | 2-3s | 5-6x |
| 内存占用 | 30-50 MB | 100-150 MB | 3x |
| CPU 占用 (空闲) | < 1% | 2-5% | 2-5x |
| 二进制大小 | 15 MB | ~50 MB | 70% 减少 |

---

## 后续计划

1. **短期** (1-2 周)
   - 完善漫画详情页面
   - 实现基础阅读器
   - 图片异步加载优化

2. **中期** (1 个月)
   - 搜索功能
   - 收藏/历史管理
   - 下载管理 UI

3. **长期** (2 个月)
   - Waifu2x 图片增强
   - 自动更新
   - v1.0.0 发布

---

## 参考资料

- [Bevy 0.17 发布说明](https://bevy.org/news/bevy-0-17/)
- [原 Python 版本](https://github.com/tonquer/picacg-qt)
- [Tokio 官方文档](https://tokio.rs/)
