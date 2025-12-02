# PicACG Rust 客户端开发笔记

> 最后更新: 2025-12-02

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

**关键组件：**
| 组件 | 用途 |
|------|------|
| `ScrollContainer` | 可滚动容器标记 |
| `ScrollPosition` | 滚动位置（逻辑像素） |
| `ContentSizeInfo` | 内容/视口尺寸信息 |
| `ScrollbarTrack` | 滚动条轨道 |
| `ScrollbarThumb` | 滚动条滑块 |
| `ScrollbarDragState` | 拖拽状态资源 |

**系统函数：**
- `update_all_scrollbar_thumbs` - 更新滑块位置和大小
- `scrollbar_thumb_interaction` - 滑块悬停/按下状态
- `scrollbar_track_click` - 轨道点击跳转
- `scrollbar_thumb_drag` - 滑块拖拽
- `reset_drag_state_on_release` - 鼠标释放时重置状态

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
