# PicACG Rust 客户端开发笔记

> 最后更新: 2025-12-03

## 其他
 - git commit 带emoji

## 框架概述

**当前框架**: **Bevy 0.17.3** (ECS 架构)

### Bevy 0.17 API 速查

| API | 说明 |
|-----|------|
| `#[derive(Event)]` | 定义事件/消息 |
| `MessageWriter::write()` | 发送消息 |
| `MessageReader<T>` | 接收消息 |
| `add_message::<T>()` | 注册消息类型 |
| `BorderColor::all(color)` | 设置边框颜色 |
| `despawn()` | 删除实体（自动递归删除子实体） |
| `KeyboardInput` + `logical_key` | 键盘输入处理 |

### 键盘输入新 API (Bevy 0.17)

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

### IME 输入法支持 (Bevy 0.17)

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

---

### Bevy 0.17 DPI 缩放处理

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
2. **ScrollPosition 使用 `.y` 字段**：Bevy 0.17 的 `ScrollPosition` 有 `x` 和 `y` 字段
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

## 待办事项

- [ ] 清理编译警告（未使用的导入和变量）
- [ ] 完善漫画详情页面
- [ ] 实现基础阅读器
- [ ] 实现搜索功能
- [ ] 实现收藏/历史管理
- [ ] 下载管理 UI
- [ ] 优化图片加载性能

---

## 参考资料

- [Bevy 0.17 发布说明](https://bevy.org/news/bevy-0-17/)
- [Bevy 官方文档](https://docs.rs/bevy/latest/bevy/)
- [Tokio 官方文档](https://tokio.rs/)
- [PicACG API 文档](../docs/)
- python版本在"C:\Users\ffqi\dev\py\picacg-windows"