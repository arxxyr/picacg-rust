# PicACG Rust 客户端开发笔记

> 最后更新: 2026-02-17

## 其他
 - git commit 带emoji

## 项目结构

采用纯 Cargo Workspace 结构，根目录无 `[package]`，所有 crate 版本统一在根 `Cargo.toml` 管理。

```
picacg-rust/
├── Cargo.toml                    # 纯 Workspace 配置（无 [package]）
├── assets/                       # 静态资源（字体、图片）
├── docs/                         # 文档
├── scripts/                      # 部署脚本
│   ├── deploy.sh                 # Bash 部署脚本
│   └── deploy-windows.ps1        # PowerShell 部署脚本
└── crates/
    ├── picacg_app/               # 主应用 (picacg)
    │   └── src/
    │       ├── main.rs           # 入口
    │       ├── error.rs          # 错误类型
    │       ├── components/       # Bevy ECS 组件
    │       ├── events/           # Bevy 事件定义
    │       ├── resources/        # Bevy 资源
    │       ├── systems/          # Bevy 系统函数（页面逻辑）
    │       └── plugins/          # Bevy 插件
    ├── picacg_core/              # 核心类型库
    │   └── src/
    │       ├── lib.rs
    │       └── error.rs          # PicacgError, Result
    ├── picacg_api/               # API 客户端
    │   └── src/
    │       ├── lib.rs
    │       ├── client.rs         # ApiClient
    │       ├── signer.rs         # 请求签名
    │       ├── models.rs         # API 数据模型
    │       └── endpoints/        # API 端点实现
    ├── picacg_db/                # 数据库层
    │   └── src/
    │       ├── lib.rs
    │       ├── database.rs       # SQLite 数据库
    │       ├── cache.rs          # Moka 缓存
    │       └── models.rs         # 数据库模型
    ├── picacg_config/            # 配置管理
    │   └── src/
    │       ├── lib.rs
    │       └── settings.rs       # AppSettings, ProxySettings
    └── bevy_ui_toolkit/          # 通用 UI 组件库
        └── src/
            ├── lib.rs
            ├── theme.rs          # 主题系统
            ├── scrollbar/        # 滚动条组件
            ├── pagination/       # 分页组件
            └── waterfall/        # 瀑布流布局
```

### Crate 依赖关系

```
picacg_core          ← 无依赖（错误类型）
    ↑
picacg_api           ← 依赖 picacg_core
    ↑
picacg_db            ← 依赖 picacg_core, picacg_api

picacg_config        ← 依赖 picacg_core

bevy_ui_toolkit      ← 依赖 bevy（独立 UI 库）

picacg (主应用)      ← 依赖以上所有 crate
```

### Workspace 依赖管理

所有共享依赖在根 `Cargo.toml` 的 `[workspace.dependencies]` 中定义版本：

```toml
# 根 Cargo.toml
[workspace.dependencies]
bevy = { version = "0.18", default-features = true }
reqwest = { version = "0.12", features = ["json", "cookies", "stream", "rustls-tls", "socks"] }
serde = { version = "1.0", features = ["derive"] }
# ... 更多依赖
```

各 crate 使用 `.workspace = true` 引用：

```toml
# crates/picacg_api/Cargo.toml
[dependencies]
reqwest.workspace = true
serde.workspace = true
```

---

## 框架概述

**当前框架**: **Bevy 0.18.0** (ECS 架构)

### Bevy 0.18 API 速查

| API | 说明 |
|-----|------|
| `#[derive(Event)]` | 定义事件/消息 |
| `MessageWriter::write()` | 发送消息 |
| `MessageReader<T>` | 接收消息 |
| `add_message::<T>()` | 注册消息类型 |
| `BorderColor::all(color)` | 设置边框颜色 |
| `despawn()` | 删除实体（自动递归删除子实体） |
| `KeyboardInput` + `logical_key` | 键盘输入处理 |

### 键盘输入新 API (Bevy 0.18)

```rust
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;

fn keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
) {
    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Backspace => { /* 删除字符 */ }
            Key::Character(input) => {
                // input 是 &str，包含输入的字符串
                for c in input.chars() {
                    // 处理每个字符
                }
            }
            _ => {}
        }
    }
}
```

### IME 输入法支持 (Bevy 0.18)

**重要：** `KeyboardInput` 只能处理英文直接输入，中文输入法需要使用 `Ime` 事件。

```rust
use bevy::window::Ime;

fn handle_ime_input(
    mut ime_events: MessageReader<Ime>,
) {
    for event in ime_events.read() {
        match event {
            Ime::Commit { value, .. } => {
                // IME 提交的文本（用户确认输入后）
                // value 是完整的输入字符串，如 "你好"
                keyword.push_str(value);
            }
            Ime::Preedit { value, cursor, .. } => {
                // IME 预览文本（输入过程中的候选）
                // 可用于显示输入法预览
            }
            Ime::Enabled { .. } => { /* IME 启用 */ }
            Ime::Disabled { .. } => { /* IME 禁用 */ }
        }
    }
}
```

**注意事项：**
- 英文输入：使用 `KeyboardInput` + `Key::Character`
- 中文输入：使用 `Ime::Commit` 事件
- 两个系统需要同时注册才能支持双语输入
 经验总结



Bevy 0.18 IME（输入法）支持要点

1. 启用 IME 的必要条件：
- 在输入框获取焦点时设置 window.ime_enabled = true
- 设置 window.ime_position 指定 IME 候选框位置
- 失去焦点时设置 window.ime_enabled = false
2. Query 需要 GlobalTransform 时的注意事项（参考 CLAUDE.md 10.3）：
- Bevy UI 节点默认不包含 Transform/GlobalTransform
- 需要显式添加 Transform::default() 组件，Bevy 才会自动添加 GlobalTransform
- 否则 Query 会返回空结果
3. rfd 文件对话框：
- rfd::FileDialog::new().pick_folder() 是同步阻塞调用
- 适合简单场景，复杂场景可考虑异步版本


---

### 字体加载配置

必须显式配置 `AssetPlugin` 的 `file_path` 以确保字体正确加载：

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
    )
```

### Nerd Font 图标使用

项目使用 **SarasaTermSCNerd** 字体，内置 Nerd Font 图标。在 UI 按钮中使用图标代替 emoji（emoji 可能显示为乱码）。

**常用图标码：**

| 功能 | Unicode | 图标 | Nerd Font 名称 |
|-----|---------|------|---------------|
| 暂停 | `\u{F03E4}` | 󰏤 | nf-md-pause |
| 播放 | `\u{F040A}` | 󰐊 | nf-md-play |
| 刷新 | `\u{F0453}` | 󰑓 | nf-md-refresh |
| 删除 | `\u{F01B4}` | 󰆴 | nf-md-delete |
| 同步 | `\u{F04E6}` | 󰓦 | nf-md-sync |
| 文件夹 | `\u{F0770}` | 󰝰 | nf-md-folder_open |
| 下载 | `\u{F01DA}` | 󰇚 | nf-md-download |
| 搜索 | `\u{F0349}` | 󰍉 | nf-md-magnify |
| 设置 | `\u{F0493}` | 󰒓 | nf-md-cog |
| 首页 | `\u{F02DC}` | 󰋜 | nf-md-home |

**使用示例：**
```rust
btn.spawn((
    Text::new("\u{F03E4}"),  // 󰏤 nf-md-pause
    TextFont {
        font: font.clone(),
        font_size: 14.0,
        ..default()
    },
    TextColor(Color::WHITE),
));
```

**查找更多图标：** https://www.nerdfonts.com/cheat-sheet

---

### Bevy 0.18 DPI 缩放处理

**核心原则：** Bevy UI 使用逻辑像素，但 `ComputedNode::size()` 返回物理像素。

| API | 返回值类型 | 说明 |
|-----|-----------|------|
| `ComputedNode::size()` | **物理像素** | 需要除以 `scale_factor` 转换 |
| `Window::cursor_position()` | **逻辑像素** | 屏幕坐标系（原点左上，Y 向下） |
| `GlobalTransform::translation()` | **逻辑像素** | Bevy UI 坐标系（原点中心，Y 向上） |
| `Node` 的 `Val::Px(x)` | 逻辑像素 | Bevy 自动处理 DPI 缩放 |
| `ScrollPosition` | 逻辑像素 | 自定义组件，存储逻辑像素 |
| `ContentSizeInfo` | 逻辑像素 | 自定义组件，存储逻辑像素 |

**获取 scale_factor：**
```rust
fn get_scale_factor(window_query: &Query<&Window, With<PrimaryWindow>>) -> f32 {
    window_query
        .single()
        .ok()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0)
}
```

**典型场景：**
```rust
// ❌ 错误：直接使用 ComputedNode::size()
let viewport_height = scroll_computed.size().y;

// ✅ 正确：转换为逻辑像素
let scale_factor = get_scale_factor(&window_query);
let viewport_height = scroll_computed.size().y / scale_factor;
```

---

### 坐标系统转换（重要）

Bevy 中存在两套坐标系统，进行鼠标位置计算时必须正确转换：

**1. 屏幕坐标系（Window::cursor_position()）**
- 原点：窗口**左上角**
- Y 轴：**向下**为正
- 单位：逻辑像素

**2. Bevy UI 坐标系（GlobalTransform::translation()）**
- 原点：窗口**中心**
- Y 轴：**向上**为正
- 单位：逻辑像素

**坐标转换公式：**
```rust
// 屏幕坐标 → Bevy UI 坐标
let window_height = window.height();
let cursor_y_bevy = window_height - cursor_pos.y;  // 翻转 Y 轴
```

**滚动条轨道点击计算示例：**
```rust
// 获取轨道中心位置（Bevy UI 坐标系）
let track_center = track_transform.translation();
let track_height = track_computed.size().y / scale_factor;

// 轨道顶部 Y 坐标（Bevy 坐标系中，Y 增大 = 向上）
let track_top_y = track_center.y + track_height / 2.0;

// 点击位置相对于轨道顶部的距离
let click_offset_from_top = track_top_y - cursor_y_bevy;

// 点击比例（0.0 = 顶部，1.0 = 底部）
let click_ratio = (click_offset_from_top / track_height).clamp(0.0, 1.0);
```

**常见错误：**
```rust
// ❌ 错误：使用 RelativeCursorPosition.normalized
//    该组件在高 DPI 环境下可能出现 1/scale_factor 的偏差
let click_ratio = relative_cursor.normalized.y;

// ✅ 正确：手动计算相对位置
let click_ratio = (track_top_y - cursor_y_bevy) / track_height;
```

**滚动条系统 DPI 修复要点：**
1. `update_all_scrollbar_thumbs`: track_height 需要除以 scale_factor
2. `scrollbar_thumb_drag`: track_height、thumb_height 需要除以 scale_factor


**影响文件：**
- `src/systems/categories.rs` - viewport 计算
- `src/systems/comics.rs` - viewport 计算
- `src/systems/scrollbar.rs` - 滚动条所有系统

---

## 自定义滚动条系统

实现 VSCode 风格的滚动条，支持：
- 轨道点击快速跳转
- 滑块拖拽滚动
- 自动计算滑块大小和位置
- DPI 缩放适配

### 关键组件

| 组件 | 用途 |
|------|------|
| `ScrollContainer` | 可滚动容器标记 |
| `ScrollPosition` | Bevy 内置滚动位置，使用 `.y` 字段 |
| `ContentSizeInfo` | 内容/视口尺寸信息（需手动更新） |
| `ScrollbarTrack` | 滚动条轨道，存储关联的滚动容器 Entity |
| `ScrollbarThumb` | 滚动条滑块，存储关联的滚动容器 Entity |
| `ScrollbarDragState` | 拖拽状态资源（全局） |
| `ScrollbarContainer` | 滚动条容器（可选） |

### 系统函数（需在 ui_plugin.rs 注册）

- `update_all_scrollbar_thumbs` - 更新滑块位置和大小
- `scrollbar_thumb_interaction` - 滑块悬停/按下状态
- `scrollbar_track_click` - 轨道点击跳转
- `scrollbar_thumb_drag` - 滑块拖拽
- `reset_drag_state_on_release` - 鼠标释放时重置状态

### 使用步骤

**1. 在 ui_plugin.rs 注册系统（在对应页面的 Update 中）：**
```rust
.add_systems(
    Update,
    (
        // ... 其他系统
        update_all_scrollbar_thumbs,
        scrollbar_thumb_interaction,
        scrollbar_track_click,
        scrollbar_thumb_drag,
        reset_drag_state_on_release,
    )
        .run_if(in_state(AppRoute::YourPage)),
)
```

**2. 创建可滚动容器（保存 Entity ID）：**
```rust
let scroll_container = parent
    .spawn((
        YourScrollContainerMarker,  // 自定义标记组件
        ScrollContainer,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            overflow: Overflow::scroll_y(),  // 启用垂直滚动
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ScrollPosition::default(),
        ContentSizeInfo::default(),
    ))
    .with_children(|content| {
        // 滚动内容...
    })
    .id();  // 保存 Entity ID
```

**3. 创建滚动条（使用保存的 scroll_container Entity）：**
```rust
fn spawn_scrollbar_inline(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
    parent
        .spawn((
            ScrollbarContainer { scroll_container },
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(12.0),  // 滚动条宽度
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|scrollbar| {
            // 轨道
            scrollbar
                .spawn((
                    ScrollbarTrack { scroll_container },
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.3)),
                    Transform::default(),  // 必须！否则 GlobalTransform 不可用
                ))
                .with_children(|track| {
                    // 滑块
                    track.spawn((
                        ScrollbarThumb { scroll_container },
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(8.0),
                            height: Val::Px(50.0),  // 初始高度，会被系统自动更新
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
                        Transform::default(),  // 必须！
                    ));
                });
        });
}
```

**4. 实现滚动处理系统：**
```rust
pub fn handle_your_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<YourScrollContainerMarker>,
    >,
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        let scroll_delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        };

        for (mut scroll_pos, content_info) in scroll_query.iter_mut() {
            let max_scroll = content_info
                .map(|info| (info.content_height - info.viewport_height).max(0.0))
                .unwrap_or(0.0);
            scroll_pos.y = (scroll_pos.y - scroll_delta).clamp(0.0, max_scroll);
        }
    }
}
```

**5. 实现内容尺寸更新系统：**
```rust
pub fn update_your_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<YourScrollContainerMarker>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0);

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        // 计算内容高度
        let mut content_height = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(*child) {
                content_height += child_computed.size().y / scale_factor;
            }
        }

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}
```

### 注意事项

1. **Transform 组件必须添加**：轨道和滑块需要 `Transform::default()`，否则 `GlobalTransform` 不可用，导致点击/拖拽不响应
2. **ScrollPosition 使用 `.y` 字段**：Bevy 0.18 的 `ScrollPosition` 有 `x` 和 `y` 字段
3. **ContentSizeInfo 需手动更新**：在每帧的 Update 系统中更新 `content_height` 和 `viewport_height`
4. **滚动容器需要 `overflow: Overflow::scroll_y()`**：启用 Bevy 内置的滚动裁剪

---

## 常见陷阱

### Query 组件缺失导致查询返回空

**问题场景：** Query 需要某个组件，但实体没有该组件，导致查询返回 0 个实体。

**典型案例：GlobalTransform 缺失**

```rust
// ❌ 错误：查询需要 GlobalTransform，但 UI 实体可能没有
pub fn scrollbar_track_click(
    track_query: Query<(&ScrollbarTrack, &GlobalTransform, &ComputedNode)>,
) {
    for (track, transform, computed) in &track_query {
        // 如果实体没有 GlobalTransform，这里永远不会执行！
    }
}
```

**症状：**
- 系统函数不报错，但完全没有响应
- 调试时发现 `track_query.iter().count() == 0`
- 其他不需要该组件的 Query 可以正常匹配到实体

**根本原因：**
- Bevy UI 节点默认**不包含** `Transform`/`GlobalTransform`
- 只有显式添加 `Transform` 组件后，Bevy 才会自动添加 `GlobalTransform`

**修复方法：** 在创建实体时添加 `Transform::default()`

```rust
// ✅ 正确：显式添加 Transform，Bevy 会自动添加 GlobalTransform
scrollbar.spawn((
    ScrollbarTrack { scroll_container },
    Button,
    Interaction::default(),
    Node { /* ... */ },
    BackgroundColor(TRACK_COLOR),
    // 添加 Transform 以获得 GlobalTransform
    Transform::default(),
));
```

**诊断技巧：**
1. 添加调试日志检查 Query 匹配数量：`info!("实体数量={}", query.iter().count())`
2. 对比工作正常的 Query 和有问题的 Query，找出组件差异
3. 检查实体创建代码，确认是否缺少必要组件

**相关组件依赖：**
| 组件 | 自动添加条件 |
|------|-------------|
| `GlobalTransform` | 需要显式添加 `Transform` |
| `ComputedNode` | UI 节点自动获得 |
| `Interaction` | 需要显式添加，配合 `Button` 或 `FocusPolicy` |

---

### 瀑布式系统与 refresh 函数冲突

**问题场景：** 页面第一次进入时不显示卡片，切换页面后返回才显示。

**典型案例：refresh_xxx_ui 重建整个 UI**

```rust
// ❌ 错误：refresh 函数在数据变化时重建整个 UI
pub fn refresh_comics_list_ui(
    comics_state: Res<ComicsListState>,
    ...
) {
    if !comics_state.is_changed() { return; }

    // 删除旧 UI（包括瀑布式系统刚创建的卡片！）
    for entity in root_query.iter() {
        commands.entity(entity).despawn();
    }

    // 重建 UI，但没有卡片（卡片由瀑布式系统创建）
    // ...
}
```

**时序问题：**
1. `setup_xxx_ui` 创建基本 UI 结构（含"加载中..."指示器）
2. API 请求发出
3. 瀑布式系统检测到没有数据，不启动预创建
4. API 数据返回，`xxx_state` 变化
5. `refresh_xxx_ui` 检测到 `is_changed()`，**删除整个 UI 并重建**
6. 重建的 UI 只有基本结构，没有卡片
7. 瀑布式系统检测到数据存在，启动预创建
8. 但 `refresh_xxx_ui` 可能再次检测到变化，**又删除刚创建的卡片**

**正确架构：**

```rust
// ✅ 正确：refresh 函数只处理错误状态，不重建整个 UI
pub fn refresh_comics_list_ui(
    comics_state: Res<ComicsListState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ComicsScrollContainer>>,
    card_query: Query<&ComicCard>,
    ...
) {
    if !comics_state.is_changed() { return; }

    // 如果有错误，显示错误信息
    if let Some(ref error) = comics_state.error {
        // 添加错误信息 UI...
        return;
    }

    // 如果数据存在或已有卡片，让瀑布式系统处理，不干涉！
    if !comics_state.comics.is_empty() {
        return;
    }

    // 检查是否已有卡片
    if let Ok((_, children)) = scroll_container_query.single() {
        let has_cards = children
            .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
            .unwrap_or(false);
        if has_cards {
            return;
        }
    }
}
```

**瀑布式系统自动启动预创建：**

```rust
// ✅ 正确：瀑布式系统自动检测并启动预创建
pub fn waterfall_create_comic_cards(
    mut creation_state: ResMut<ComicsCardCreationState>,
    comics_state: Res<ComicsListState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<ComicsScrollContainer>>,
    card_query: Query<&ComicCard>,
    ...
) {
    // 自动检测：数据存在但没有卡片，启动预创建
    if !creation_state.is_creating
        && !comics_state.comics.is_empty()
        && comics_state.error.is_none()
    {
        if let Ok((_, children)) = scroll_container_query.single() {
            let has_cards = children
                .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
                .unwrap_or(false);

            if !has_cards {
                // 删除加载指示器，启动预创建
                creation_state.start_precreate(comics_state.comics.len(), font);
            }
        }
    }
    // ...
}
```

**职责分离原则：**
| 函数 | 职责 |
|------|------|
| `setup_xxx_ui` | 创建基本 UI 结构（标题栏、滚动容器、加载指示器） |
| `refresh_xxx_ui` | 只处理错误状态，不重建整个 UI |
| `waterfall_create_xxx_cards` | 自动检测数据并创建卡片，瀑布式显示 |

**排行榜标签切换特殊处理：**

```rust
// 检查类型是否匹配（处理标签切换）
let type_matches = creation_state.context.current_type
    .map(|t| t == rankings_state.current_type)
    .unwrap_or(false);

// 如果有卡片但类型不匹配，清除旧卡片
if has_cards && !type_matches {
    // 清除所有子元素
    for child in children.iter() {
        commands.get_entity(child).map(|e| e.despawn());
    }
    creation_state.clear();
    return;  // 下一帧会检测到没有卡片，启动预创建
}
```

**影响文件：**
- `src/systems/categories.rs` - `refresh_categories_ui`, `waterfall_create_category_cards`
- `src/systems/comics.rs` - `refresh_comics_list_ui`, `waterfall_create_comic_cards`
- `src/systems/rankings.rs` - `refresh_rankings_ui`, `waterfall_create_cards`

---

### UI 重建导致输入框焦点丢失

**问题场景：** 输入框输入一个字符后焦点丢失，需要重新点击才能继续输入。

**典型案例：SearchState 变化触发 UI 重建**

```rust
// ❌ 错误：每次 keyword 变化都重建整个 UI，焦点状态丢失
pub fn refresh_search_ui(search_state: Res<SearchState>, ...) {
    if !search_state.is_changed() { return; }

    // 重建 UI 时，focused 被重置为 false
    header.spawn((
        SearchInputField { focused: false },  // ← 焦点丢失！
        ...
    ));
}
```

**症状：**
- 输入第一个字符后焦点立即丢失
- 日志显示 `has_focus=true`（第一个字符），然后 `has_focus=false`（后续字符）
- 必须用鼠标重新点击输入框

**根本原因：**
- `ResMut<SearchState>` 被修改时，`is_changed()` 返回 `true`
- `refresh_search_ui` 系统重建整个 UI
- 新创建的 `SearchInputField` 组件 `focused` 字段被重置

**修复方法：** 在重建 UI 前保存焦点状态，重建后恢复

```rust
// ✅ 正确：保存并恢复焦点状态
pub fn refresh_search_ui(
    input_query: Query<&SearchInputField>,
    ...
) {
    // 1. 保存焦点状态
    let was_focused = input_query.iter().any(|input| input.focused);

    // 2. 销毁旧 UI
    for entity in search_root_query.iter() {
        commands.entity(entity).despawn();
    }

    // 3. 重建时恢复焦点
    let border_color = if was_focused { PRIMARY } else { BORDER };
    header.spawn((
        SearchInputField { focused: was_focused },  // ← 恢复焦点
        BorderColor::all(border_color),             // ← 恢复边框颜色
        ...
    ));
}
```

**相关 Commit：**
- `fix: 修复搜索输入框焦点丢失和中文输入法问题`

---

### 按钮缺少 Interaction 组件导致无法点击

**问题场景：** 按钮添加了 `Button` 组件，但点击没有任何响应。

**典型案例：日志等级按钮无响应**

```rust
// ❌ 错误：缺少 Interaction 组件
row.spawn((
    MyButton,
    Button,
    Node { ... },
    BackgroundColor(AppColors::PRIMARY),
));
```

**症状：**
- 按钮显示正常，但点击无响应
- 交互系统的 `Changed<Interaction>` 查询永远不会匹配到该实体

**根本原因：**
- Bevy 的 `Button` 组件只是一个标记，不会自动添加 `Interaction` 组件
- 必须显式添加 `Interaction::default()` 才能启用交互检测

**修复方法：** 在按钮创建时添加 `Interaction::default()`

```rust
// ✅ 正确：显式添加 Interaction 组件
row.spawn((
    MyButton,
    Button,
    Interaction::default(),  // 必须添加！
    Node { ... },
    BackgroundColor(AppColors::PRIMARY),
));
```

---

### Query 遍历顺序不确定导致位置计算错误

**问题场景：** 需要按 UI 布局顺序计算多个区域的累加位置，但 Query 返回顺序是不确定的。

**典型案例：浮动标题点击跳转**

```rust
// ❌ 错误：Query 遍历顺序不确定，current_y 累加顺序错误
pub fn floating_header_click_interaction(
    section_query: Query<(
        &ComputedNode,
        Option<&DownloadingSection>,
        Option<&WaitingSection>,
        Option<&StoppedSection>,
        Option<&CompletedSection>,
    )>,
) {
    let mut current_y: f32 = 0.0;
    for (computed, is_downloading, is_waiting, is_stopped, is_completed) in section_query.iter() {
        // Query 遍历顺序是 Bevy 内部顺序，不是 UI 布局顺序！
        if section_type == Some(target_section) {
            target_y = Some(current_y);  // current_y 可能是错的！
            break;
        }
        current_y += height + 10.0;  // 累加顺序错误
    }
}
```

**症状：**
- 点击跳转到错误位置（如跳到最下面而不是目标区域）
- 每次跳转位置不一致（取决于实体创建顺序）

**根本原因：**
- Bevy ECS Query 的 `iter()` 返回顺序是**实体创建顺序或内部存储顺序**
- 这个顺序**不等于** UI 布局的视觉顺序

**修复方法：** 分别查询每个区域，按固定顺序计算位置

```rust
// ✅ 正确：分别查询每个区域，按布局顺序计算
pub fn floating_header_click_interaction(
    downloading_query: Query<&ComputedNode, With<DownloadingSection>>,
    waiting_query: Query<&ComputedNode, With<WaitingSection>>,
    stopped_query: Query<&ComputedNode, With<StoppedSection>>,
) {
    // 按固定顺序获取每个区域的高度
    let downloading_height = downloading_query.single().ok()
        .map(|n| n.size().y / scale_factor).unwrap_or(0.0);
    let waiting_height = waiting_query.single().ok()
        .map(|n| n.size().y / scale_factor).unwrap_or(0.0);
    let stopped_height = stopped_query.single().ok()
        .map(|n| n.size().y / scale_factor).unwrap_or(0.0);

    // 按布局顺序计算目标位置
    let target_y = match target_section {
        SectionType::Downloading => 0.0,
        SectionType::Waiting => downloading_height + GAP,
        SectionType::Stopped => downloading_height + GAP + waiting_height + GAP,
        SectionType::Completed => downloading_height + GAP + waiting_height + GAP + stopped_height + GAP,
    };
}
```

**关键原则：**
- 当需要**按顺序**处理多个实体时，不要依赖 Query 的遍历顺序
- 使用**独立 Query** 分别查询每种类型的实体
- 按**业务逻辑顺序**（如布局顺序）显式计算

**影响文件：**
- `src/systems/downloads.rs` - `floating_header_click_interaction`

---

### MessageReader 消费事件导致 Bevy 原生滚动失效

**问题场景：** 使用 `MessageReader<MouseWheel>` 处理鼠标滚轮事件后，Bevy 的原生 `ScrollPosition` 滚动不再工作。

**典型案例：阅读器条漫模式滚动失效**

```rust
// ❌ 错误：MessageReader 消费了所有事件，Bevy 原生滚动收不到
pub fn reader_mouse_wheel_control(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    // ...
) {
    for event in mouse_wheel_events.read() {
        // 事件已被消费！
        match reader_state.read_mode {
            ReadMode::SinglePage => { /* 处理翻页 */ }
            ReadMode::Webtoon => {
                // 期望 Bevy 原生 ScrollPosition 处理，但事件已被消费
                // 滚动容器不会响应！
            }
        }
    }
}
```

**症状：**
- 滚动容器设置了 `overflow: Overflow::scroll_y()` 和 `ScrollPosition::default()`
- 其他页面的滚动正常工作
- 但该页面的滚动完全不响应

**根本原因：**
- `MessageReader<T>::read()` 会**消费**消息队列中的事件
- 一旦被读取，其他系统（包括 Bevy 内置的滚动系统）就收不到这些事件
- Bevy 的原生滚动依赖于未被消费的 `MouseWheel` 事件

**修复方法：** 在需要滚动的分支中手动更新 `ScrollPosition`

```rust
// ✅ 正确：手动更新 ScrollPosition
pub fn reader_mouse_wheel_control(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut webtoon_scroll_query: Query<&mut ScrollPosition, With<WebtoonScrollContainer>>,
    // ...
) {
    for event in mouse_wheel_events.read() {
        let scroll_delta = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        };

        match reader_state.read_mode {
            ReadMode::SinglePage => { /* 处理翻页 */ }
            ReadMode::Webtoon => {
                // 手动更新 ScrollPosition
                for mut scroll_pos in webtoon_scroll_query.iter_mut() {
                    let scroll_amount = -scroll_delta * SCROLL_SPEED;
                    scroll_pos.y = (scroll_pos.y + scroll_amount).max(0.0);
                }
            }
        }
    }
}
```

**关键原则：**
- `MessageReader` 读取事件会消费它们，其他系统无法再收到
- 如果需要同时处理事件和使用 Bevy 原生功能，必须手动实现原生功能的逻辑
- 滚动方向：`scroll_delta > 0` 向上滚，`scroll_pos.y` 减小；反之增大

**影响文件：**
- `src/systems/reader.rs` - `reader_mouse_wheel_control`

---

### 固定底部栏与滚动容器布局

**问题场景：** 页面需要固定底部栏（如状态提示栏），同时中间内容可滚动。

**典型案例：设置页面底部状态栏**

**正确布局结构：**
```
PageRoot (Column, 100% height)
├── Header (固定高度，如 50px)
├── ContentWrapper (flex_grow: 1.0, overflow: clip)
│   ├── ScrollContainer (100% height, overflow: scroll_y)
│   │   ├── 内容1
│   │   ├── 内容2
│   │   └── 底部间距 (height: 30px)  ← 确保最后内容可滚动到可见区域
│   └── Scrollbar (Absolute 定位)
└── BottomBar (固定高度，初始 display: None，按需显示)
```

**关键点：**

1. **ContentWrapper 必须设置 `overflow: Overflow::clip()`**
   - 防止滚动内容溢出到 BottomBar 区域

2. **底部间距设置（推荐 30px）**
   - 确保最后的内容可以完全滚动到可见区域
   - 过大的 padding 可能导致布局计算问题

3. **使用 Flexbox 自动分配空间**
   ```rust
   // ContentWrapper
   Node {
       flex_grow: 1.0,      // 占据剩余空间
       flex_shrink: 1.0,    // 允许收缩
       flex_basis: Val::Px(0.0),
       min_height: Val::Px(0.0),
       overflow: Overflow::clip(),  // 关键！
       ..default()
   }
   ```

**示例代码：**
```rust
root.spawn(Node {
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    flex_direction: FlexDirection::Column,
    ..default()
})
.with_children(|root| {
    // 标题栏
    spawn_header(root);

    // 内容区域（可滚动）
    root.spawn(Node {
        flex_grow: 1.0,
        overflow: Overflow::clip(),  // 关键！
        position_type: PositionType::Relative,
        ..default()
    })
    .with_children(|wrapper| {
        // 滚动容器
        wrapper.spawn((
            ScrollContainer,
            Node {
                height: Val::Percent(100.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .with_children(|scroll| {
            // 内容...

            // 底部间距
            scroll.spawn(Node {
                height: Val::Px(30.0),
                min_height: Val::Px(30.0),
                ..default()
            });
        });
    });

    // 固定底部栏
    spawn_bottom_bar(root);
});
```

---

### needs_rebuild 模式：避免输入时全量 UI 重建

**问题场景：** 资源状态（如 `SearchState`）在输入文字时被修改，`is_changed()` 触发 `refresh_xxx_ui` 重建整个页面，导致输入卡顿和焦点丢失。

**解决方案：** 添加 `needs_rebuild` 标志，只在结构性变化时触发重建。

```rust
pub struct SearchState {
    pub keyword: String,
    // ... 其他字段
    /// 是否需要重建 UI（仅在搜索结果/排序/分类/翻页/错误变化时设置）
    pub needs_rebuild: bool,
}

pub fn refresh_search_ui(
    mut search_state: ResMut<SearchState>,  // 需要 ResMut 来重置标志
    // ...
) {
    if !search_state.is_changed() || !search_state.needs_rebuild {
        return;
    }
    search_state.needs_rebuild = false;
    // ... 重建 UI
}
```

**设置 `needs_rebuild = true` 的场景：**
- 搜索结果返回（成功/失败）
- 切换排序方式
- 切换分类过滤
- 翻页
- 按下 Enter 搜索

**不设置的场景：**
- 键盘输入修改 keyword（通过 `update_input_text` 原地更新文本节点）
- IME 输入修改 keyword

**影响文件：**
- `src/resources/app_state.rs` - `SearchState` 添加 `needs_rebuild` 字段
- `src/systems/search.rs` - `refresh_search_ui` + 各触发点
- `src/plugins/api_plugin.rs` - 搜索响应处理

---

### 设置页面屏蔽词输入系统

**组件架构：**

| 组件 | 用途 |
|------|------|
| `NewKeywordInput` | 输入框标记（焦点状态由 `FilterSettingsState.input_focused` 管理） |
| `NewKeywordInputText` | 输入框内文本节点标记（用于原地更新文本） |
| `BlockedKeywordsListContainer` | 屏蔽词列表容器标记（用于局部刷新） |
| `KeywordSuggestionPanel` | 分类建议下拉面板容器 |
| `KeywordSuggestionItem` | 建议项按钮（存储分类名） |
| `KeywordSuggestionToggle` | 展开/折叠建议面板按钮 |

**系统函数：**

| 系统 | 职责 |
|------|------|
| `new_keyword_input_interaction` | 输入框点击交互，启用 IME |
| `new_keyword_keyboard_input` | 键盘输入（含 Ctrl+V 粘贴） |
| `new_keyword_ime_input` | IME 中文输入提交 |
| `unfocus_keyword_input` | 点击外部失焦，关闭 IME |
| `refresh_blocked_keywords_ui` | 监听 `FilterSettingsState` 变化，局部刷新屏蔽词列表 |
| `keyword_suggestion_toggle_interaction` | 展开/折叠分类建议面板 |
| `keyword_suggestion_item_interaction` | 点击建议项添加屏蔽词 |

**关键设计：**
- 焦点状态统一由 `FilterSettingsState.input_focused` 管理，不在组件上存储
- 文本更新使用 `update_keyword_input_text()` 辅助函数，处理占位符/实际文本颜色切换
- 建议面板数据来源：`CategoriesState.categories` 中的分类标题
- 已存在于屏蔽词列表中的分类显示为灰色禁用状态

---

## 通用分页组件

实现了泛型分页组件模块 `src/systems/pagination.rs`，支持多个页面复用。

### 使用方法

**1. 定义页面标记类型：**
```rust
pub struct FavoritesPage;
pub struct ComicsPage;
```

**2. 创建分页 UI：**
```rust
use crate::systems::pagination::spawn_pagination_controls;

spawn_pagination_controls::<FavoritesPage>(
    parent,
    &font,
    current_page,  // u32
    total_pages,   // u32
);
```

**3. 处理分页交互：**
```rust
use crate::systems::pagination::{
    check_pagination_interaction, PaginationPrevButton, PaginationNextButton,
};

pub fn pagination_interaction(
    prev_query: Query<&Interaction, (Changed<Interaction>, With<PaginationPrevButton<FavoritesPage>>)>,
    next_query: Query<&Interaction, (Changed<Interaction>, With<PaginationNextButton<FavoritesPage>>)>,
    // ...
) {
    // 返回 Some(true) = 下一页, Some(false) = 上一页, None = 无操作
    if let Some(is_next) = check_pagination_interaction::<FavoritesPage>(
        &prev_query, &next_query, current_page, total_pages
    ) {
        // 处理翻页...
    }
}
```

**4. 更新分页显示：**
```rust
use crate::systems::pagination::{update_pagination_display, PaginationPageText};

pub fn refresh_pagination_ui(
    mut page_text_query: Query<&mut Text, With<PaginationPageText<FavoritesPage>>>,
    mut prev_btn_query: Query<&mut BackgroundColor, (With<PaginationPrevButton<FavoritesPage>>, Without<PaginationNextButton<FavoritesPage>>)>,
    mut next_btn_query: Query<&mut BackgroundColor, (With<PaginationNextButton<FavoritesPage>>, Without<PaginationPrevButton<FavoritesPage>>)>,
) {
    update_pagination_display::<FavoritesPage>(
        &mut page_text_query,
        &mut prev_btn_query,
        &mut next_btn_query,
        current_page,
        total_pages,
    );
}
```

### 组件列表

| 组件 | 用途 |
|------|------|
| `PaginationControl<T>` | 分页容器标记 |
| `PaginationPrevButton<T>` | 上一页按钮标记 |
| `PaginationNextButton<T>` | 下一页按钮标记 |
| `PaginationPageText<T>` | 页码文本标记 |

### 已使用的页面
- `FavoritesPage` - 收藏页面 (`favorites.rs`)
- `ComicsPage` - 漫画列表页面 (`comics.rs`)

---

## 设置页面自动保存模式

设置页面采用**自动保存**机制，修改即生效，无需手动点击保存按钮。

### 架构设计

| 组件 / 系统 | 职责 |
|-------------|------|
| `SettingsSaveStatus` | 资源：保存状态（visible、timer、message、is_error） |
| `SettingsStatusBar` / `SettingsStatusText` | 组件：底部状态栏 UI 标记 |
| `auto_save_settings` | 系统：监听所有设置状态 `is_changed()`，有变化时自动保存 |
| `update_settings_save_status` | 系统：控制状态栏显示/隐藏，2 秒后自动消失 |
| `save_all_settings()` | 辅助函数：从各状态资源读取值写入 `AppSettings` 并保存到磁盘 |

### 关键实现细节

**1. 跳过初始化帧：**

`setup_settings_ui` 插入资源时会触发 `is_changed() = true`，需要用 `Local<bool>` 跳过第一帧：

```rust
pub fn auto_save_settings(
    // ...各种 Res<XxxState>
    mut initialized: Local<bool>,
) {
    if !any_changed { return; }
    if !*initialized {
        *initialized = true;
        return; // 跳过初始化帧
    }
    // 执行保存...
}
```

**2. 状态栏显示/隐藏：**

使用 `Display::None` / `Display::Flex` 控制底部状态栏的显示。`Timer` 倒计时 2 秒后自动隐藏：

```rust
// 显示
node.display = Display::Flex;

// Timer 到期后隐藏
if save_status.timer.just_finished() {
    save_status.visible = false;
    node.display = Display::None;
}
```

**3. 错误/成功区分：**

`SettingsSaveStatus.is_error` 控制文本颜色（绿色成功 / 红色失败）。

### 影响文件
- `src/systems/settings.rs` — 自动保存逻辑、状态栏 UI
- `src/plugins/ui_plugin.rs` — 系统注册

---

## 登录状态与异步操作

### 问题：启动时自动下载报错"未登录"

**场景：** 启用"启动后自动恢复下载"设置后，启动时会立即尝试下载，但此时用户还没有登录。

**解决方案：** 使用事件系统等待登录成功

```rust
// 1. 定义登录成功事件 (events/api_events.rs)
#[derive(Message)]
pub struct UserLoggedInEvent;

// 2. 登录成功时发送事件 (api_plugin.rs)
fn handle_login_response(
    mut user_logged_in_messages: MessageWriter<UserLoggedInEvent>,
    // ...
) {
    if login_success {
        user_logged_in_messages.write(UserLoggedInEvent);
    }
}

// 3. 监听事件后再执行操作 (api_plugin.rs)
fn auto_resume_downloads_on_startup(
    mut has_run: Local<bool>,
    mut user_logged_in_events: MessageReader<UserLoggedInEvent>,
    // ...
) {
    // 只执行一次
    if *has_run {
        for _ in user_logged_in_events.read() {} // 消费事件
        return;
    }

    // 等待登录成功事件
    let mut logged_in = false;
    for _ in user_logged_in_events.read() {
        logged_in = true;
    }
    if !logged_in {
        return;
    }

    *has_run = true;
    // 执行需要登录后才能进行的操作...
}
```

**关键点：**
- 使用 `Local<bool>` 确保只执行一次
- 在 `has_run = true` 后仍需消费事件，避免累积
- 注册事件：`.add_message::<UserLoggedInEvent>()`

---

## 调试技巧

1. **使用完整堆栈追踪**
   ```powershell
   $env:RUST_BACKTRACE = "1"
   cargo run
   ```

2. **查看编译警告**
   ```powershell
   cargo clippy --all
   ```

3. **格式化代码**
   ```powershell
   cargo fmt --all
   ```

---

## 部署与 CI/CD

### 本地部署脚本

```bash
# Linux / macOS
./scripts/deploy.sh [release|debug]

# Windows (PowerShell)
.\scripts\deploy-windows.ps1 [-Profile release|debug]
```

脚本流程：清理旧 bin → 创建目录 → 复制可执行文件 → 复制字体 → 创建版本压缩包。
产物位于 `bin/` 目录，压缩包命名格式 `picacg-v{版本号}.zip`。

### GitHub Actions CI/CD

**工作流文件：** `.github/workflows/ci.yml`

| Job | 触发条件 | 说明 |
|-----|---------|------|
| `fmt` | push/PR | `cargo fmt --all -- --check` |
| `clippy` | push/PR | `cargo clippy --all --all-targets` |
| `test` | clippy 通过后 | `cargo test --all --release` |
| `build` | test 通过后 | Linux x64 + Windows x64 + macOS ARM64 矩阵构建 |
| `release` | 推送 `v*` 标签 | 下载产物、创建 GitHub Release |
| `dev-build-summary` | master/main/develop 推送 | 生成构建摘要 |

**版本号格式：**
- Release（标签触发）：`v{版本号}+{commit短哈希}`
- Dev（分支推送）：`v{版本号}+{日期}.{commit短哈希}`

**构建平台：**
- Linux x64（Ubuntu 22.04 LTS，glibc 2.35）
- Windows x64（MSVC）
- macOS ARM64（Apple Silicon）

**构建优化：**
- `Swatinem/rust-cache@v2` 缓存 Cargo 依赖
- UPX `--best --lzma` 压缩二进制（macOS 跳过）
- Linux 构建验证 ELF 完整性
- 产物保留 30 天

### GitLab CI/CD

**工作流文件：** `.gitlab-ci.yml`

| Job | 阶段 | 说明 |
|-----|------|------|
| `fmt` | check | 代码格式检查 |
| `clippy` | check | 静态分析 |
| `test` | test | 单元测试（依赖 fmt + clippy） |
| `build-release` | build | Release 构建（main/master/tags/MR） |
| `build-debug` | build | Debug 构建（feature 分支快速验证） |

---

## 待办事项

### 当前功能开发
- [ ] 清理编译警告（未使用的导入和变量）
- [x] 实现基础阅读器（单页模式、键盘翻页、顶部/底部工具栏）
- [x] 阅读器增强功能
  - [x] 条漫模式（Webtoon 垂直无限滚动）
  - [x] 模式切换按钮（单页/条漫）
  - [x] 鼠标滚轮翻页（单页）/滚动（条漫）
  - [x] Ctrl+滚轮 缩放
  - [x] 键盘 +/-/0 缩放控制
  - [x] 缩放比例显示
- [x] 实现搜索功能
- [x] 实现收藏页面
- [x] 下载管理 UI
- [x] 优化图片加载性能（MAX_CONCURRENT_LOADS 从 5 提升到 15）
- [x] 修复瀑布式系统与 refresh 函数冲突问题（分类、漫画列表、排行榜）
- [x] 修复排行榜标签切换不刷新问题
- [x] 通用分页组件（favorites.rs, comics.rs 已使用）
- [x] 登录状态管理（自动下载等待登录成功后再执行）
- [x] 完善漫画详情页面（返回按钮、汉化组、更新时间、评论数、分类/标签点击跳转）
- [x] 下载列表标题/分类/标签点击跳转
- [x] 删除下载任务后 UI 立即更新
- [x] 设置页面自动保存（移除保存按钮，修改即生效，底部状态栏提示）
- [x] 搜索分类过滤（排序选择器 + 分类复选框面板）
- [x] 关键词屏蔽（按分类/标签/标题屏蔽，设置页面管理，配置持久化）
- [x] 修复 sanitize_filename 未清理全角特殊字符导致 CBZ 打包兼容性问题
- [x] 屏蔽词输入 IME 中文支持 + 分类建议面板 + 列表动态刷新
- [x] 搜索页面 needs_rebuild 优化（输入不触发全量 UI 重建）
- [x] 部署脚本（deploy.sh + deploy-windows.ps1）
- [x] CI/CD 流水线（GitHub Actions：fmt/clippy/build/release）

### 已完成：Workspace 重构与模块拆分

- [x] 抽取通用 GUI 组件为独立 crate (`bevy_ui_toolkit`)
  - 主题系统（Theme, CurrentTheme）
  - 自定义滚动条系统（ScrollbarPlugin）
  - 通用分页组件（PaginationPlugin）
  - 瀑布流布局（WaterfallState）
- [x] 拆分核心模块为独立 crate
  - `picacg_core` - 错误类型
  - `picacg_api` - API 客户端
  - `picacg_db` - 数据库层
  - `picacg_config` - 配置管理
- [x] 统一 Workspace 依赖版本管理

---

## 参考资料

- [Bevy 0.18 发布说明](https://bevy.org/news/bevy-0-17/)
- [Bevy 官方文档](https://docs.rs/bevy/latest/bevy/)
- [Tokio 官方文档](https://tokio.rs/)
- [PicACG API 文档](../docs/)
- python版本在"C:\Users\ffqi\dev\py\picacg-windows"