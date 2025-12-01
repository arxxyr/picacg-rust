# Bevy 0.17.3 重构进度报告

**日期**: 2025-12-01

## 概述

将 PicACG Rust 客户端从 **iced 0.13** 重构为 **Bevy 0.17.3**，今日完成了核心基础设施和登录/代理设置页面的实现。

## 完成内容

### 1. 项目基础设施 ✅

- 更新 `Cargo.toml` 使用 Bevy 0.17.3
- 创建 ECS 架构目录结构：
  - `plugins/` - Bevy 插件
  - `components/` - ECS 组件
  - `resources/` - 全局资源
  - `events/` - 事件定义
  - `systems/` - ECS 系统

### 2. 核心模块 ✅

- **ApiPlugin** (`plugins/api_plugin.rs`)
  - 使用 `bevy-tokio-tasks` 集成异步 API 调用
  - 登录请求/响应处理
  - 分类加载
  - 漫画列表加载

- **UiPlugin** (`plugins/ui_plugin.rs`)
  - 状态路由管理 (`AppRoute`)
  - 登录页面系统
  - 代理设置页面系统
  - 分类页面系统
  - 漫画列表页面系统

### 3. 登录页面 ✅

- 完整的登录 UI 实现
- 用户名/密码输入（点击聚焦 + 键盘输入）
- 登录按钮交互（悬停/按下状态）
- 代理设置按钮跳转
- 错误消息显示

### 4. 代理设置页面 ✅

- 启用代理开关
- 代理类型选择（HTTP/HTTPS/SOCKS5）
- 主机地址输入
- 端口输入（仅数字）
- 保存/返回按钮
- 配置持久化到 `AppSettings`

### 5. API 变更适配 ✅

#### Bevy 0.17 API 变更

| iced 0.13 | Bevy 0.17.3 |
|-----------|-------------|
| `Message` 枚举 | `Event` / `Message` trait |
| `EventWriter::send()` | `MessageWriter::write()` |
| `EventReader` | `MessageReader` |
| `add_event::<T>()` | `add_message::<T>()` |
| `BorderColor(color)` | `BorderColor::all(color)` |
| `despawn_recursive()` | `despawn()` (自动递归) |
| `ReceivedCharacter` | `KeyboardInput` + `logical_key` |

#### 键盘输入 API 变更

Bevy 0.17 弃用了 `ReceivedCharacter` 事件，改用 `KeyboardInput` 事件的 `logical_key` 字段：

```rust
// 旧 API (已弃用)
fn keyboard_input(
    mut char_events: EventReader<ReceivedCharacter>,
) {
    for event in char_events.read() {
        let c = event.char;
        // ...
    }
}

// 新 API (Bevy 0.17)
fn keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
) {
    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Backspace => { /* 删除 */ }
            Key::Character(input) => {
                // input 是 &str，包含输入的字符
            }
            _ => {}
        }
    }
}
```

### 6. 字体渲染修复 ✅

**问题**: 中文字体显示为乱码

**原因**: `AssetPlugin` 默认 asset 路径不正确

**解决方案**: 显式配置 `AssetPlugin` 的 `file_path`:

```rust
use bevy::asset::AssetPlugin;

let manifest_dir = env!("CARGO_MANIFEST_DIR");
let assets_path = std::path::Path::new(manifest_dir).join("assets");

App::new()
    .add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: assets_path.to_string_lossy().to_string(),
                ..default()
            })
            // ...
    )
```

## 当前状态

### 编译状态
- ✅ 编译通过
- ⚠️ 警告：未使用的导入和类型（可忽略，后续清理）

### 功能状态
- ✅ 登录页面显示正常
- ✅ 中文字体渲染正常
- ✅ 代理设置页面完整实现
- ✅ 键盘输入功能（用户名/密码/代理设置）
- ✅ 分类页面显示正常
- ⏳ 漫画列表（已有代码，待测试）

### 7. 登录后路由修复 ✅

**问题**: 登录成功后界面空白

**原因**: 登录成功后路由设置为 `AppRoute::Home`，但 Home 页面没有实现 UI 系统

**解决方案**: 将登录成功后的路由从 `AppRoute::Home` 改为 `AppRoute::Categories`

```rust
// src/plugins/api_plugin.rs
match &event.result {
    Ok(token) => {
        // ...
        // 登录成功后直接进入分类页面
        next_route.set(AppRoute::Categories);
    }
}
```

### 8. 分类页面 UI 刷新修复 ✅

**问题**: 进入分类页面后显示空白

**原因**:
- `setup_categories_ui` 在 `OnEnter` 时运行
- 此时 `categories` 为空，`is_loading` 为 false
- 空的 for 循环不会创建任何 UI 元素
- 异步加载完成后，UI 不会自动更新

**解决方案**:
1. 修改 `setup_categories_ui`：如果 categories 为空，显示"加载中..."
2. 添加 `refresh_categories_ui` 系统监听分类加载完成事件，重建 UI

```rust
// src/systems/categories.rs
pub fn refresh_categories_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    categories_state: Res<CategoriesState>,
    image_cache: Res<ImageCache>,
    mut loaded_messages: MessageReader<CategoriesLoadedEvent>,
    mut failed_messages: MessageReader<CategoriesLoadFailedEvent>,
    root_query: Query<Entity, With<CategoriesRoot>>,
) {
    // 检查是否有加载完成或失败的事件
    let has_loaded = loaded_messages.read().count() > 0;
    let has_failed = failed_messages.read().count() > 0;

    if !has_loaded && !has_failed {
        return;
    }

    // 删除旧的 UI，重新创建
    // ...
}
```

**关键点**: Bevy UI 不会像 React 那样自动响应数据变化重新渲染，需要手动监听事件并重建 UI。

## 技术亮点

1. **ECS 架构**：使用 Bevy 的 Entity-Component-System 模式，状态管理更清晰

2. **异步集成**：通过 `bevy-tokio-tasks` 无缝集成 Tokio 异步运行时

3. **状态路由**：使用 Bevy States 实现页面路由，`OnEnter`/`OnExit` 自动管理 UI 生命周期

4. **点击聚焦输入**：使用 Button 组件模拟输入框，点击后捕获键盘输入

### 9. 分类页面首次加载问题修复 ✅

**问题**: 分类页面首次打开时一直显示"加载中..."，切换到其他标签再切回来才能显示内容。

**原因**:
- `refresh_categories_ui` 通过 `MessageReader<CategoriesLoadedEvent>` 读取事件
- `handle_categories_response`（在 `api_plugin.rs` 中）也通过 `MessageReader` 读取同一个事件
- 当其中一个系统读取了事件，另一个系统就读不到了（事件被消费）
- 首次加载时，`handle_categories_response` 先消费了事件，`refresh_categories_ui` 读不到，导致 UI 不刷新

**解决方案**:
将 `refresh_categories_ui` 改为监听 `CategoriesState` 资源的变化，而不是直接读取事件：

```rust
// src/systems/categories.rs
pub fn refresh_categories_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    categories_state: Res<CategoriesState>,
    image_cache: Res<ImageCache>,
    root_query: Query<Entity, With<CategoriesRoot>>,
    content_area_query: Query<Entity, With<ContentArea>>,
) {
    // 只在状态变化时刷新
    if !categories_state.is_changed() {
        return;
    }

    // 如果还在加载中，不刷新（等待加载完成）
    if categories_state.is_loading && categories_state.categories.is_empty() {
        return;
    }

    // 删除旧的 UI，重新创建
    // ...
}
```

**关键点**: 使用 `Resource::is_changed()` 方法监听资源变化，避免事件消费竞争问题。

### 10. 侧边栏宽度不固定问题修复 ✅

**问题**: 侧边栏宽度在不同页面间切换时会发生变化，特别是在分类页面（有内容）和其他空页面（无内容）之间切换时。

**原因**:
- 侧边栏只设置了 `width: Val::Px(SIDEBAR_WIDTH)`
- 没有设置 `min_width`、`max_width` 和 `flex_shrink`
- Flexbox 布局在内容区域大小变化时会重新计算侧边栏宽度

**解决方案**:
为侧边栏添加完整的宽度约束：

```rust
// src/systems/main_layout.rs
.spawn((
    Sidebar,
    Node {
        width: Val::Px(SIDEBAR_WIDTH),
        min_width: Val::Px(SIDEBAR_WIDTH),  // 新增：最小宽度
        max_width: Val::Px(SIDEBAR_WIDTH),  // 新增：最大宽度
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,  // 新增：不收缩
        border: UiRect::right(Val::Px(1.0)),
        ..default()
    },
    BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
    BorderColor::all(AppColors::BORDER),
))
```

**关键点**:
- `min_width` + `max_width` 确保宽度固定不变
- `flex_shrink: 0.0` 防止 Flexbox 在空间不足时压缩侧边栏

### 11. 自定义滚动条实现 ✅

**问题**: Bevy UI 框架不支持原生滚动条组件，滚动区域没有可视化滚动条。

**解决方案**: 实现自定义滚动条组件，类似 VSCode 风格。

#### 组件结构

```
滚动区域包装器 (position: relative)
├── 滚动容器 (overflow: scroll_y)
└── 滚动条容器 (position: absolute, right: 0)
    └── 滚动条轨道 (Button, 支持点击跳转)
        └── 滚动条滑块 (Button, 支持拖拽)
```

#### 新增组件 (`src/components/ui_components.rs`)

```rust
/// 滚动条容器
#[derive(Component)]
pub struct ScrollbarContainer {
    pub scroll_container: Entity,
}

/// 滚动条轨道（可点击跳转）
#[derive(Component)]
pub struct ScrollbarTrack {
    pub scroll_container: Entity,
}

/// 滚动条滑块（可拖拽）
#[derive(Component)]
pub struct ScrollbarThumb {
    pub scroll_container: Entity,
}

/// 滚动条拖拽状态
#[derive(Resource, Default)]
pub struct ScrollbarDragState {
    pub is_dragging: bool,
    pub dragging_thumb: Option<Entity>,
    pub drag_start_y: f32,
    pub drag_start_scroll: f32,
}
```

#### 滚动条系统 (`src/systems/scrollbar.rs`)

1. **`update_all_scrollbar_thumbs`**: 根据滚动位置更新滑块位置和大小
2. **`scrollbar_thumb_interaction`**: 处理滑块悬停/按下状态变化
3. **`scrollbar_track_click`**: 点击轨道快速跳转到对应位置
4. **`scrollbar_thumb_drag`**: 拖拽滑块滚动
5. **`reset_drag_state_on_release`**: 鼠标释放时重置拖拽状态

#### 配置常量 (`scrollbar_config`)

```rust
pub const SCROLLBAR_WIDTH: f32 = 12.0;
pub const THUMB_MIN_HEIGHT: f32 = 30.0;
pub const TRACK_COLOR: Color = Color::srgba(0.2, 0.2, 0.25, 0.3);
pub const THUMB_COLOR: Color = Color::srgba(0.5, 0.5, 0.55, 0.6);
pub const THUMB_HOVER_COLOR: Color = Color::srgba(0.6, 0.6, 0.65, 0.8);
pub const THUMB_PRESSED_COLOR: Color = Color::srgba(0.7, 0.7, 0.75, 0.9);
```

#### 集成页面

- ✅ 分类页面 (`src/systems/categories.rs`)
- ✅ 漫画列表页面 (`src/systems/comics.rs`)

#### 功能特性

1. **滑块大小自适应**: 根据内容高度和视口高度自动计算滑块大小
2. **滑块位置同步**: 滚动时滑块位置实时更新
3. **点击轨道跳转**: 点击轨道任意位置快速跳转
4. **拖拽滑块滚动**: 拖拽滑块平滑滚动内容
5. **悬停状态反馈**: 滑块悬停时颜色变化
6. **半透明设计**: VSCode 风格的半透明滚动条

#### 技术难点

1. **内容高度估算**: Bevy 不直接暴露内容高度，使用滚动位置反推估算
2. **实体关联**: 滚动条组件通过 `scroll_container: Entity` 关联到对应的滚动容器
3. **拖拽状态管理**: 使用 `ScrollbarDragState` Resource 跟踪拖拽起点和滚动起点

## 下一步计划

1. **测试登录流程**：验证 API 调用和状态切换
2. **完善分类页面**：图片加载和卡片交互
3. **实现漫画详情**：详情页 UI 和章节列表
4. **实现阅读器**：图片显示和翻页导航

## 参考资源

- [Bevy 0.17 发布说明](https://bevy.org/news/bevy-0-17/)
- [Bevy Cheat Book - Keyboard Input](https://bevy-cheatbook.github.io/input/keyboard.html)
- [bevy-tokio-tasks](https://crates.io/crates/bevy-tokio-tasks)
