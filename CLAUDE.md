# PicACG Rust 客户端开发笔记

> 最后更新: 2026-08-11

## 其他
 - git commit 带emoji

## 项目结构

采用纯 Cargo Workspace 结构，根目录无 `[package]`，所有 crate 版本统一在根 `Cargo.toml` 管理。

```
picacg-rust/
├── Cargo.toml                    # 纯 Workspace 配置（无 [package]）
├── assets/                       # 静态资源（字体、图片）
│   └── fonts/SarasaTermSCNerd/   # 内置更纱黑体 CJK 字体
├── docs/                         # 文档
├── migrations/                   # SQLite 数据库迁移脚本
├── scripts/                      # 部署脚本
│   ├── deploy.sh                 # Bash 部署脚本
│   └── deploy-windows.ps1        # PowerShell 部署脚本
└── crates/
    ├── picacg_app/               # 主应用 (picacg)
    │   └── src/
    │       ├── main.rs           # 入口（mimalloc 全局分配器）
    │       ├── error.rs          # 错误类型
    │       ├── components/       # Bevy ECS 组件
    │       ├── events/           # Bevy 事件定义
    │       ├── resources/        # Bevy 资源
    │       ├── systems/          # Bevy 系统函数（页面逻辑）
    │       ├── plugins/          # Bevy 插件
    │       └── utils/            # 工具模块
    │           ├── content_filter.rs  # 内容过滤（繁简转换+多维度匹配）
    │           └── tokio_tasks.rs     # Bevy-tokio 集成
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
    │       ├── channel.rs        # 分流通道路由（直连/CDN/反代）
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
    │       └── settings.rs       # AppSettings, ProxySettings, ChannelSettings, FilterSettings
    └── picacg_app/src/systems/   # 共享 UI 层（原 bevy_ui_toolkit 已合并回，2026-08）
        ├── theme.rs              # 设计令牌：Theme（控件运行时面）+ Scale（尺度档位）
        ├── widgets.rs            # ButtonStyle 组件 + 全局交互配色系统
        ├── scrollbar/            # 上游 ScrollArea/Scrollbar 的外观包装 + 滑块配色
        ├── pagination/           # 分页控件（值组件 + 内联观察者）
        ├── waterfall.rs          # 瀑布流状态机（有界列表页使用）
        └── ui_common.rs          # 徽章/时间信息/右键菜单/格式化工具
```

> 颜色常量面 `AppColors` 定义在 `systems/login.rs`（1000+ 处存量引用），
> 与 `theme.rs` 的 `Theme` 数值必须一致；焦点/文本输入共享层在 `utils/text_input.rs`。

### Crate 依赖关系

```
picacg_core          ← 无依赖（错误类型）
    ↑
picacg_config        ← 依赖 picacg_core
    ↑
picacg_api           ← 依赖 picacg_core, picacg_config
    ↑
picacg_db            ← 依赖 picacg_core, picacg_api

picacg_app (主应用)  ← 依赖以上所有 crate
```

### Workspace 依赖管理

所有共享依赖在根 `Cargo.toml` 的 `[workspace.dependencies]` 中定义版本：

```toml
# 根 Cargo.toml
[workspace.dependencies]
bevy = { version = "0.19", default-features = true, features = ["bevy_debug_stepping"] }
reqwest = { version = "0.13", features = ["json", "cookies", "stream", "rustls", "socks", "gzip", "brotli"] }
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

**当前框架**: **Bevy 0.19.0** (ECS 架构 + BSN 场景系统)

### Bevy 0.19 API 速查

| API | 说明 |
|-----|------|
| `#[derive(Event)]` | 定义事件/消息 |
| `MessageWriter::write()` | 发送消息 |
| `MessageReader<T>` | 接收消息 |
| `add_message::<T>()` | 注册消息类型 |
| `BorderColor::all(color)` | 设置边框颜色 |
| `despawn()` | 删除实体（自动递归删除子实体） |
| `KeyboardInput` + `logical_key` | 键盘输入处理 |
| `TextFont.font: FontSource` | 0.19 起 `Handle<Font>` 需 `.into()`；BSN 中省略该字段 = 默认 CJK 字体 |
| `TextFont.font_size: FontSize` | 0.19 起用 `FontSize::Px(14.0)`（有 `From<f32>`） |
| `Font::from_bytes(Vec<u8>)` | 0.19 起替代 `try_from_bytes`，不再返回 Result（Parley 后端延迟解析） |

### BSN（Bevy Scene Notation）—— UI 构建标准写法

UI 构建代码已全部迁移到 `bsn!` 场景函数（`bevy::prelude` 直接可用；默认 feature 经 `ui → scene` 链启用 bevy_scene）。
样板参考：`systems/login.rs`（页面场景化）、`systems/scrollbar/scenes.rs`（`#Name` 实体引用）、
`systems/pagination/scenes.rs`（泛型标记 + 内联观察者）。

```rust
fn my_page(state: &SomeState) -> impl Scene + use<> {   // 引用参数 → 必须精确捕获 use<>（泛型则 use<T>）
    let title = state.title.clone();                    // 动态值先取成 owned 局部变量
    bsn! {
        PageRoot                                        // marker 组件裸写
        Node { width: Val::Percent(100.0) }             // 补丁语义：只写非默认字段，无 ..default()
        TextFont { font_size: FontSize::Px(14.0) }      // font 字段省略 = 默认 CJK 字体
        Interaction                                     // 按钮必备的 Interaction 直接裸写
        Children [
            ( Text({title}) TextColor(AppColors::TEXT) ),   // {} 包局部变量/复杂表达式
            my_fragment("参数", value),                     // 场景函数组合；实参不加 {}
            {dynamic_list},                                  // Vec<impl Scene>/Box<dyn SceneList> 裸插入
        ]
    }
}
// spawn：commands.spawn_scene(my_page(&state)) → EntityCommands（可 .id()/.insert()）
// 增量追加子实体：commands.spawn_scene(card(..)).insert(ChildOf(container))
```

**关键规则（全部编译验证过）：**

| 规则 | 说明 |
|------|------|
| derive 门槛 | 进 bsn 的组件 `#[derive(Component, Default, Clone)]`；字段枚举需 `Default` + 首变体 `#[default]` |
| Entity 字段组件 | `#[derive(Component, FromTemplate)]`（与 Default 互斥），字段可接 `#Name` 引用或具体 Entity |
| 泛型标记组件 | `PhantomData<fn() -> T>` + 手写 Default/Clone（derive 会给 T 加多余 bound；`PhantomData<T>` 触发 Unpin 错误） |
| template_value 三场景 | 外部枚举变体（`FocusPolicy::Block`）、关联函数构造器（`BorderColor::all(..)`）、builder 值（`TextInput::new(..)`） |
| 实体引用 | 同一 bsn! 内 `#ScrollArea` 命名 + 任意位置引用；`scrollbar(target)` 接 `impl Into<EntityTemplate>` |
| ⚠️ 列表括号陷阱 | `Children [ {list} ]` 展开多实体；`({list})` 只出一个实体，**不报错**——动态列表一律裸 `{list}` |
| ⚠️ 构造器实参陷阱 | 实参含运算/局部变量时整个值包 `{}`：`width: {Val::Percent(ratio * 100.0)}` |
| 保留命令式 | 瀑布流分帧建卡状态机（单卡已场景化）、refresh 系统的原地修改、非 UI 单组件 spawn（Camera2d） |

### 键盘输入 API (Bevy 0.19)

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

### 文本输入与焦点（2026-08 重构后）

全部输入框走共享层 `utils/text_input.rs`，页面零焦点代码：

- **焦点单一真相源**：上游 `bevy::input_focus::InputFocus`（此前 8 套页面级
  Focus 资源/sync 系统/影子布尔全部废除）。页面读焦点一律
  `input_focus.get() == Some(entity)`。
- 点击聚焦+光标定位（CJK 全角步进精确计算）、点击空白失焦、聚焦边框、
  IME 开关与候选框位置：通用系统统一处理，页面无需注册任何输入交互系统
- 键盘/IME 编辑按焦点实体**定向分发**（Backspace/Delete/方向键/Home/End/
  Ctrl+C/V/X/IME Commit）
- Tab 导航：`TabIndex(n)` 组件 + 上游 `TabNavigationPlugin`（ui_plugin 注册），
  取代手写 Tab 链
- 页面只保留「动作键」系统（Enter 提交等），用 InputFocus 判定目标

### 字体加载方案

采用**同步替换 Bevy 默认字体**的方案，启动时用 CJK 字体替换 FiraMono。

**核心实现**（`systems/font_loader.rs`）：

```rust
/// 获取全局字体句柄 —— 所有 UI 统一调用此函数
pub fn get_font() -> Handle<Font> {
    Handle::default()  // 已被替换为 CJK 字体
}

/// 启动系统：同步加载字体 → 替换 Bevy 默认字体
pub fn setup_fonts(mut fonts: ResMut<Assets<Font>>) {
    // 1. 优先加载内置更纱黑体（assets/fonts/SarasaTermSCNerd/）
    // 2. 回退：检测系统中文字体（微软雅黑、思源黑体、PingFang SC 等）
    // 3. 用 fonts.insert(AssetId::default(), font) 替换默认字体
}
```

**优势：**
- `Handle::default()` 全局统一，无需传递字体 Resource
- 同步加载，解决启动时中文不显示的时序问题
- 内置字体优先，系统字体回退，跨平台兼容

### UI 图标使用

项目使用系统字体，UI 图标使用 **Unicode 通用符号**（不依赖 Nerd Font）。

**常用图标：**

| 功能 | 符号 | 说明 |
|-----|------|------|
| 暂停 | `⏸` | U+23F8 |
| 播放 | `▶` | U+25B6 |
| 刷新 | `↻` | U+21BB |
| 删除/关闭 | `✕` | U+2715 |
| 同步 | `⟳` | U+27F3 |
| 文件夹 | `📂` | U+1F4C2 |
| 下载 | `⬇` | U+2B07 |
| 等待 | `⏳` | U+23F3 |
| 停止 | `⏹` | U+23F9 |
| 勾选 | `✓` | U+2713 |
| 设置 | `⚙` | U+2699 |
| 左箭头 | `◀` | U+25C0 |
| 右箭头/展开 | `▶` | U+25B6 |
| 下箭头/折叠 | `▼` | U+25BC |
| 上箭头 | `▲` | U+25B2 |

**使用示例：**
```rust
btn.spawn((
    Text::new("⏸"),  // 暂停按钮
    TextFont {
        font: font.clone(),
        font_size: 14.0,
        ..default()
    },
    TextColor(Color::WHITE),
));
```

---

### Bevy 0.19 DPI 缩放处理

**核心原则：** Bevy UI 使用逻辑像素，但 `ComputedNode::size()`/`content_size()` 返回物理像素。

| API | 返回值类型 | 说明 |
|-----|-----------|------|
| `ComputedNode::size()` / `content_size()` | **物理像素** | 乘 `computed.inverse_scale_factor` 转逻辑像素（优先；免查 Window） |
| `Window::cursor_position()` | **逻辑像素** | 屏幕坐标系（原点左上，Y 向下） |
| `Node` 的 `Val::Px(x)` | 逻辑像素 | Bevy 自动处理 DPI 缩放 |
| `ScrollPosition` | 逻辑像素 | Bevy 内置组件 |

**惯用写法（新代码统一用逐节点因子，不再查询 Window）：**
```rust
let viewport_h = computed.size().y * computed.inverse_scale_factor;
let content_h = computed.content_size().y * computed.inverse_scale_factor;
```

---

### 坐标系统与滚动条（0.19 起由上游接管）

> 历史上本项目自研滚动条，坐标系翻转/DPI 换算/内容尺寸手工喂养曾是主要事故源
> （轨道点击跳错、高分屏滑块错位、GlobalTransform 缺失查询空转等）。
> **自研实现已整体退役**，相关陷阱文档随之删除——这正是换上游的目的。

现行方案（`bevy_ui_widgets`，随 DefaultPlugins 加载，零插件成本）：

| 能力 | 提供方 | 说明 |
|------|--------|------|
| 滚轮/触控板滚动 | `ScrollArea` 标记 | `Pointer<Scroll>` 悬停派发 + 事件冒泡，嵌套滚动语义自动正确 |
| 滑块尺寸/位置 | `Scrollbar { target, orientation, min_thumb_length }` | 可视/内容比例自动算，布局后阶段定位，DPI 内部处理 |
| 轨道点击/拖拽 | 上游内置 | 每滑块独立 `ScrollbarDragState`（require 注入） |
| 内容尺寸 | `ComputedNode::content_size()` | 引擎布局原生输出；`× inverse_scale_factor` 得逻辑像素 |

**页面接线（BSN）**：

```rust
bsn! {
    Node { .. }
    Children [
        (
            #ContentScroll
            ScrollArea                              // 滚轮滚动
            Node { overflow: Overflow::scroll_y(), .. }
        ),
        scrollbar(#ContentScroll),                  // systems/scrollbar 包装：VSCode 外观 + 上游机制
    ]
}
```

**保留的自定义**：滑块悬停/拖拽配色（`update_scrollbar_thumb_colors`，全局注册一次）；
阅读器滚轮模态逻辑（单页=翻页/条漫=滚动，条漫容器**不加** ScrollArea）。

**仍然有效的 DPI 常识**：`ComputedNode::size()`/`content_size()` 返回物理像素，
换逻辑像素乘 `computed.inverse_scale_factor`；`Window::cursor_position()` 是逻辑像素。

## 常见陷阱

### B0001：同系统 Query 读写冲突（运行时 panic，编译期查不出）

同一系统内两个 Query 对同一组件「一写一读/两写」且不可证不相交 → 系统**首次初始化时 panic**
（页面级系统 = 首次进入该页才炸；check/clippy 全绿拦不住）。判定语义：

- 读 = `&X`/`Ref<X>`/`Option<&X>`/过滤器 `Changed<X>`/`Added<X>`；`With`/`Without`/`Has`/`Entity` 不算访问
- 可证不相交的**唯一**方式：一方 `With<M>` 出现在另一方 `Without<M>`。
  ⚠️ `With<A>` vs `With<B>` **不算**不相交
- 修复优先级：交叉 `Without` 隔离 → 合并查询 + match 分流 → 探针改 `Mut::is_changed()`
  （`Changed<X>` 探针 + `&mut X` 场景，见 reader.rs `update_webtoon_window`）→ `ParamSet`

**静态防线**：`python3 scripts/check_query_conflicts.py`（CI clippy 之后自动跑；
括号配对解析全部系统签名，命中即失败）。写含多个 Query 的系统前先过一遍脑内判定，提交前跑脚本。

### Query 组件缺失导致查询返回空（通识）

Query 里出现实体没有的组件 → 匹配 0 个实体，**不报错、只是静默不执行**。
两类易中招的组件：

- `GlobalTransform`：UI 节点默认没有 Transform/GlobalTransform，需显式加
  `Transform::default()` 才会自动补全（历史上自研滚动条因此坑过；现行代码
  已无手动 Transform 需求——上游滚动条不依赖它）
- 恒空过滤器：`Without<Interaction>` 配 `Button` 实体恒为空
  （0.19 起 `Button` require(Interaction)）——写这种旁路 Query 前先想清楚

**诊断技巧**：`info!("匹配数={}", query.iter().count())`；对比能匹配的相邻 Query 找组件差异。

---

### 漫画列表虚拟滚动（comics.rs）

无限滚动页面的实体数不再随翻页累积：`comics_virtual_scroll` 只为可见窗口 ±2 行
维持卡片实体，容器首尾各一个 spacer（`ComicsTopSpacer`/`ComicsBottomSpacer`，
width:100% 独占整行）撑起总高度——引擎 `content_size()` 与上游滚动条因此天然正确，
`auto_load_more_comics` 的触底判定不受影响。

- 状态：`ComicsVirtualState`（过滤索引缓存 / 实测行高 / 列数 / 窗口区间 / 在场实体表）
- 滚动跨行：行级增量 spawn/despawn（`insert_children` 定位到 spacer 之间）
- 数据/列数变化：全量重建窗口；屏蔽词过滤只在数据变化的低频路径执行
- 卡片场景 `comic_card` 复用；徽章为单实体（Text 自带 padding/圆角/底色）
- 其余分页页面数据量有界，仍用瀑布流分帧显示（`WaterfallState`）

### 条漫滚动锚定（reader.rs）

条漫槽位以占位高度（1000px）起步，真实图高陆续就位——同一滚动偏移映射到的页码
会**级联漂移**（曾表现为：打开显示中间页码、图片加载后跌回第 1 页，恢复上次
阅读页功能一并失效）。治法：

- `ReaderState.webtoon_anchor: Option<(页, 页内偏移)>`——创建视图时锚到目标页；
  非用户滚动帧若上方内容高度变化，按锚点补偿 `ScrollPosition` 保持视觉位置不动
  （补偿写入用 `Local<bool>` 标志与用户滚动区分）
- 用户滚动 → 重锚到当前页；当前页判定用**视口顶边**规则（开屏恒为第 1 页；
  原「视口中心」规则在占位高度下会指到中间值）
- 阅读中途上方图片补载的跳动同样被锚定消除

### 图片加载管线（重试机制）

`resources/image_cache.rs` 状态机：Pending → Loading → Loaded / Failed。

- **有界重试 + 指数退避**：失败后 2s、4s 自动重排队，累计 3 次尝试耗尽才**终局失败**
  （`api_plugin::process_image_queue` 每帧调 `requeue_ready_retries()`，无待重试项 O(1) 直返）
- `is_failed()` 只对终局失败返回真——重试期间消费系统保持占位符存活，成功后图片正常落位；
  终局失败才摘除占位标记（实体退出每帧扫描集）
- `enqueue()` 对任何已有状态的 URL 不重复入队（防重复请求）；手动强制重来用 `retry()`（计数归零）
- 下载并发上限 15；解码走 `spawn_blocking`（不占 tokio worker）；显存单份（RENDER_WORLD）

### 瀑布式系统与 refresh 函数的职责分离

**适用范围**：数据量有界的列表页（categories/rankings/favorites/search/home 等）。
漫画列表（无限滚动）已改虚拟滚动，见上一节。

**问题本质**：refresh 系统若在数据变化时重建整个 UI，会删掉瀑布式系统刚建的卡片，
两者互相打架（首进不显示、切页才出现）。

**职责分离原则**：

| 函数 | 职责 |
|------|------|
| `setup_xxx_ui` | 创建基本 UI 结构（标题栏、滚动容器、加载指示器） |
| `refresh_xxx_ui` | 只处理错误状态与文本原地更新，**不重建整个 UI** |
| `waterfall_create_xxx_cards` | 检测「数据有而卡片无」时启动预创建，分帧显示 |

**标签切换特殊处理**（rankings）：卡片存在但类型不匹配 → 清除全部子元素 +
`creation_state.clear()`，下一帧自动重启预创建。

**性能纪律**：屏蔽词过滤（`CompiledFilter`）只允许出现在启动检测/预创建分支**内部**
（惰性计算），严禁放在每帧必经的函数顶部——这是 2026-08 评审抓出的最大每帧开销源。

---

### UI 重建与输入框焦点（重构后）

历史陷阱：焦点存在组件字段里，重建 UI 即丢失，需手工保存/恢复。
现行架构下焦点在 `InputFocus` 资源中、以实体为键——重建输入框会产生新实体，
**重建方仍需在 spawn 后把新实体 set 回 `InputFocus`**（参考 search.rs 的
needs_rebuild 流程）；除此之外无需任何手工状态搬运。

### 按钮 Interaction（0.19 已过时的陷阱）

历史版本的陷阱：`Button` 不自动带 `Interaction`，漏加导致按钮点不动。
**Bevy 0.19 起 `Button` 已 `#[require(Node, FocusPolicy::Block, Interaction)]`**，
bsn 里写 `Button` 即可，无需（也不要）再裸写 `Interaction`。
按钮三态配色统一走 `systems/widgets.rs` 的 `ButtonStyle` 组件 +
全局 `apply_button_interaction` 系统，页面不再手写 hover/pressed 分支。

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
| `NewKeywordInput` | 输入框标记（焦点由全局 `InputFocus` 仲裁，组件不存状态） |
| `BlockedKeywordsListContainer` | 屏蔽词列表容器标记（用于局部刷新） |
| `KeywordSuggestionPanel` / `KeywordSuggestionItem` / `KeywordSuggestionToggle` | 分类建议面板 |

**系统函数**（点击聚焦/IME/编辑/失焦/边框全部由通用 TextInput 系统接管，页面只剩）：

| 系统 | 职责 |
|------|------|
| `new_keyword_keyboard_input` | 动作键：Enter 添加屏蔽词、Escape 失焦 |
| `refresh_blocked_keywords_ui` | 监听 `FilterSettingsState` 变化，局部刷新屏蔽词列表 |
| `keyword_suggestion_toggle_interaction` | 展开/折叠分类建议面板（写 `ButtonStyle.selected`） |
| `keyword_suggestion_item_interaction` | 点击建议项添加屏蔽词 |

**关键设计：**
- 建议面板数据来源：`CategoriesState.categories` 中的分类标题
- 已存在于屏蔽词列表中的分类显示为灰色禁用状态

---

## 分流通道系统

支持 6 种分流通道，API 和图片可独立配置，解决网络访问问题。

### 通道类型

| 类型 | 说明 | 实现方式 |
|------|------|---------|
| Direct | 直连（默认） | 直接请求原始域名 |
| CdnIp1 | CDN IP 1 (104.21.91.145) | DNS 覆盖，`ClientBuilder::resolve()` |
| CdnIp2 | CDN IP 2 (188.114.98.153) | DNS 覆盖 |
| CustomCdnIp | 自定义 CDN IP | 用户指定 IP，DNS 覆盖 |
| JpProxy | 日本反代 (bika-api.jpacg.cc) | URL 重写，签名用原始域名 |
| UsProxy | 美国反代 (bika2-api.jpacg.cc) | URL 重写，签名用原始域名 |

### 关键实现

**文件：** `crates/picacg_api/src/channel.rs`

```rust
/// 通道路由结果
pub struct ChannelRoute {
    pub request_url: String,  // 实际请求的 URL
    pub sign_url: String,     // 签名用的 URL（始终用原始域名）
}
```

**配置（`picacg_config`）：**
```rust
pub struct ChannelSettings {
    pub api_channel: ChannelType,      // API 分流
    pub image_channel: ChannelType,    // 图片分流
    pub custom_cdn_api_ip: String,     // 自定义 API CDN IP
    pub custom_cdn_img_ip: String,     // 自定义图片 CDN IP
}
```

**注意事项：**
- 反代模式下签名**始终使用原始域名** `picaapi.picacomic.com`
- CDN 模式通过 `ClientBuilder::resolve()` 指定 IP，不修改 URL
- 修改通道后自动重建 `ApiClient`，Token 保持不变

---

## 内容过滤系统

基于屏蔽词列表过滤漫画，支持繁简转换和多维度匹配。

### 过滤维度

| 维度 | 匹配方式 | 说明 |
|------|---------|------|
| 分类 | 精确匹配 | 漫画分类列表中包含屏蔽词 |
| 标签 | 精确匹配 | 漫画标签列表中包含屏蔽词 |
| 标题 | 子串匹配 | 标题包含屏蔽词子串 |

### 关键实现

**文件：** `crates/picacg_app/src/utils/content_filter.rs`

- 入口：`CompiledFilter::from_settings()` —— 构造时一次读锁 + 一次性标准化全部关键词
  （**繁体转简体** + **小写**，`zhconv` crate）
- 匹配时每个字段只标准化一次、与全部关键词比较（循环反转；分配量从
  `漫画数×词数×字段数` 降到 `漫画数×字段数 + 词数`）
- 方法：`should_block_comic()` / `filter_comic_indices()` / `filter_comics_cloned()`
  （后者只克隆保留项，api_plugin 响应过滤用）
- **调用纪律**：只允许出现在低频路径（数据变化/预创建分支内），严禁每帧构造

**接入页面：** 搜索、分类、排行、收藏、漫画列表（共 5 个页面）+ api_plugin 入库过滤

---

## mimalloc 内存分配器

全局使用 mimalloc 替换默认分配器，优化多线程内存碎片。

```rust
// main.rs
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

---

## 通用分页组件（自含观察者版）

`systems/pagination/`：`Pagination { current_page, total_pages }` 值组件是**单一事实源**，
翻页行为内联在控件的 `on(Pointer<Click>)` 观察者里（全项目首个观察者范例），
显示刷新由全局唯一的 `refresh_pagination_widgets` 系统统一处理。

**页面接入三件事**（样板：favorites.rs）：

```rust
// 1. 场景里挂控件（T 为页面标记类型，如 pub struct FavoritesPage;）
pagination_controls::<FavoritesPage>(current_page, total_pages),

// 2. 消费翻页：Changed<Pagination> + 与页面状态比较过滤非翻页变化
pub fn favorites_pagination_changed(
    pagination_query: Query<&Pagination, (With<PaginationControl<FavoritesPage>>, Changed<Pagination>)>,
    ...
) { /* 清列表、发加载请求、重置滚动 */ }

// 3. 数据加载后回写（比较后写避免 Changed 循环）
if *pagination != target { *pagination = target; }
```

无需注册任何按钮交互系统；边界判断（首页/末页）在控件观察者内完成。
已接入：favorites / comics / games / fried / search / comments。

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

## 窗口生命周期与合盖保活

**契约：窗口 ≠ 应用生命周期**（`ExitCondition::DontExit` + `close_when_requested: false`）。

- 合盖（无外接显示器且未休眠）→ macOS 移除内置显示器 → winit 销毁该显示器上的窗口。
  旧行为 `OnAllClosed` 直接退出进程、下载全断；现在应用继续运行，
  `ensure_primary_window` 检测到主窗口消失且非用户主动关闭 → 按保存的几何信息**自动重建**
  （暂无显示器时约 2 秒重试）
- 用户主动关闭（CloseBehavior::Close/Ask）→ `ExplicitWindowClose` 置位 → 窗口销毁后
  `ensure_primary_window` 显式发 `AppExit` 退出——常规退出路径不变
- CloseBehavior::Minimize 照旧：拦截关闭改最小化
- 日志时间戳为系统本地时区（`fmt::time::ChronoLocal`，此前 UTC 对不上表）

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

- [ ] 控件库二期（rv-widgets 蓝图）：page_scaffold 页面骨架（22 页三段式收编）、input_row 输入行控件（16 份实现收编）、comic_card 卡片控件（12 份收编，前置：统一图片加载策略为一种）
- [ ] Scale 尺度令牌全库替换（469 处字号 17→6 档、157 处圆角 9→3 档——全局视觉变更需专项+目测）
- [ ] EditableText 迁移：等上游补 placeholder 与密码掩码（bevy_text editing.rs 的 planned 清单）后替换自研 TextInput
- [ ] 错误页补分页/返回控件（games/fried/comments 加载失败后无法翻页返回——存量 UX 缺口）
- [ ] i18n 接线（返回按钮等文案未走 i18n.rs）
- [ ] 虚拟滚动推广评估：rankings/favorites 等分页页面数据量有界暂用瀑布流；若后续也转无限滚动则复用 comics 的虚拟滚动模式

- [ ] 下载动画效果：右键下载漫画后，封面图从卡片位置飞向侧边栏下载按钮，边移动边缩小变为圆形，最终融入下载计数徽章

### 已完成（2026-08 全面重构）

- [x] 依赖全量升级（bevy 0.19 / sqlx 0.9 / hmac 0.13 / sha2 0.11 / toml 1.1 / tungstenite 0.30）+ 删除 6 个零使用依赖 + once_cell→std + bevy feature 按需集（3D/音频不再编译）
- [x] UI 全面迁移 BSN（bsn! 场景函数化，31 文件；净删 4000+ 行）
- [x] bevy_ui_toolkit 解散合并回主项目（theme/scrollbar/pagination/waterfall → systems/）
- [x] 分页控件重设计：Pagination 值组件单一事实源 + 内联 on(Pointer<Click>) 观察者；5 个手搓分页页面收编
- [x] 自研滚动条退役 → 上游 ScrollArea/Scrollbar + 引擎 content_size()（删 ~1500 行几何/DPI/内容尺寸手工代码，含 20 个 update_*_content_size 与 23 个空滚轮系统）
- [x] 调度与热路径修复：ImageCache 状态机化（修无限重试风暴）、CompiledFilter 统一三套过滤实现（循环反转+惰性化）、下载/侧边栏/阅读器每帧深拷贝与无条件写入门控化、图片解码 spawn_blocking、显存单份（RENDER_WORLD）
- [x] ButtonStyle 控件层：305 处手写 hover/pressed 分支收编为一个组件+一个全局系统；设计令牌统一（Theme 运行时面 + AppColors 常量面），481 处硬编码色收编映射
- [x] 焦点统一：8 套并行实现 → 上游 InputFocus；TabIndex 取代手写 Tab 链；修 NAS 输入不显示、聊天室无焦点守卫、CJK 点击定位、字节掩码等 8 个实质 bug
- [x] 漫画列表虚拟滚动（窗口 ±2 行 + spacer；实体数 4200 → ~300）

### 已完成

- [x] 实现基础阅读器（单页模式、键盘翻页、顶部/底部工具栏）
- [x] 阅读器增强功能（条漫模式、缩放控制、滚轮翻页/滚动）
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
- [x] CI/CD 流水线（GitHub Actions + GitLab CI 双轨，多平台构建）
- [x] 分流通道系统（6 种通道：直连/CDN/反代，API 和图片独立配置）
- [x] 内容过滤系统（繁简转换 + 多维度匹配，5 个页面接入）
- [x] 字体方案统一（同步加载内置更纱黑体，系统字体回退，`Handle::default()` 全局统一）
- [x] Nerd Font 图标替换为 Unicode 通用符号（跨字体兼容）
- [x] 下载按钮标签化（图标+中文标签）+ 已下载漫画移动功能
- [x] mimalloc 全局内存分配器集成
- [x] bevy_ui_toolkit 从 git 依赖改为本地 crate
- [x] 修复下载路径问题（停止任务恢复、路径一致性）

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
- [x] 全局滚轮分发系统（侧边栏/内容区独立滚动，基于光标 X 坐标分区）
- [x] 页面缓存架构（21 个主布局页面 Display::None/Flex 显隐，5 个参数化页面保持 spawn/despawn）
- [x] 侧边栏用户头像（登录后自动加载 UserProfile，从 ImageCache 替换占位符）
- [x] 下载计数徽章（下载中蓝色 + 排队中橙色，分开显示）
- [x] 右键上下文菜单（全局生效，漫画卡片右键弹出下载/屏蔽菜单）
- [x] 快速下载（右键直接下载，章节列表未加载时自动异步获取后触发下载）
- [x] 下载页"全部更新"按钮（对所有已下载漫画发送 RedownloadRequest 检查新章节）
- [x] 个人资料修复（handle_profile_loaded 全局运行，刷新不再卡在加载中）
- [x] 网络诊断（设置页测速 + Ping 测试）
- [x] 检查更新（GitHub Releases API，semver 版本比较）
- [x] 侧边栏布局优化（用户信息区/版本号 flex_shrink:0，菜单区精简移除工具分组）

---

## 参考资料

- [Bevy 0.19 发布说明](https://bevy.org/news/bevy-0-19/)（BSN / ui_widgets / input_focus）
- [Bevy 0.18→0.19 迁移指南](https://bevy.org/learn/migration-guides/0-18-to-0-19/)
- [Bevy 官方文档](https://docs.rs/bevy/latest/bevy/)
- [Tokio 官方文档](https://tokio.rs/)
- [PicACG API 文档](../docs/)
- python版本在"C:\Users\ffqi\dev\py\picacg-windows"