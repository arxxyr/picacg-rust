# PicACG Rust 客户端开发笔记

> 最后更新: 2026-08-29

## 其他
 - git commit 带emoji

## 项目结构

采用纯 Cargo Workspace 结构，根目录无 `[package]`，所有 crate 版本统一在根 `Cargo.toml` 管理。

```
picacg-rust/
├── Cargo.toml                    # 纯 Workspace 配置（无 [package]）
├── assets/                       # 静态资源（字体、图标）
│   ├── fonts/SarasaTermSCNerd/   # 内置更纱黑体 CJK 字体
│   └── icons/                    # 应用图标（官方素材 192 权威源 + 512/256 投放尺寸）
├── docs/                         # 文档
├── migrations/                   # SQLite 数据库迁移脚本
├── scripts/                      # 部署脚本
│   ├── deploy.sh                 # Bash 部署脚本
│   ├── deploy-windows.ps1        # PowerShell 部署脚本
│   └── make-icon.sh              # 由官方素材生成图标尺寸（--fetch 可重抓）
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
    │           ├── profiling.rs       # 系统耗时统计（tracing Layer，F4 打榜）
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

### 高级搜索的排序只认请求体（2026-08 修复）

`POST /comics/advanced-search` 的排序参数必须放在**请求体** `sort` 字段里；
查询串上的 `s=` 会被服务端忽略（`GET /comics` 才认 `s`）。
历史实现只发查询串，表现为「点排序按钮无任何效果」——四种排序返回完全相同的列表。

```rust
// SearchComicsRequest::body()
serde_json::json!({ "keyword": self.keyword, "sort": self.sort })
```

同一文件里的排序按钮系统 `sort_button_interaction` **不能加 `Changed<Interaction>` 过滤器**：
选中态刷新要覆盖所有按钮（点 B 时 A 的 Interaction 并未变化，加过滤器会让 A 一直亮着）。
重复触发由 `search_state.sort != btn.sort` 挡掉。

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

### 漫画列表虚拟滚动 + 节点复用（comics.rs）

无限滚动页面的实体数不再随翻页累积：`comics_virtual_scroll` 只为可见窗口 ±2 行
维持卡片实体，容器首尾各一个 spacer（`ComicsTopSpacer`/`ComicsBottomSpacer`，
width:100% 独占整行）撑起总高度——引擎 `content_size()` 与上游滚动条因此天然正确，
`auto_load_more_comics` 的触底判定不受影响。

**滚动路径零 spawn/despawn**（2026-08 下旬）：移出窗口的卡片不销毁，直接作为空闲池
改绑到移入位置的数据上（RecyclerView 那套）。

- 状态：`ComicsVirtualState`（过滤索引缓存 / 实测行高 / 列数 / 窗口区间 / 在场实体表 /
  待改绑清单）
- `plan_recycle(old_range, new_range)` 是**纯函数**，算「哪些槽位可沿用、哪些卡片空闲」；
  复用只在滚动时触发，肉眼很难覆盖「窗口缩小」「完全不重叠」等边界，故有单测
- 两步走：`comics_virtual_scroll` 排出 `pending_rebind` → `comics_rebind_cards` 改内容
  （`.chain()` 保证同帧顺序）。拆开是为了让滚动系统不必持有一大堆改 UI 的可变查询
- **卡片必须是固定形态**：徽章槽（分类 3 + 标签 3）、时间槽（更新/创建）常驻，
  多余的 `Display::None`。形态随数据变的话就没法「只改内容」了
- 封面是**单实体**：`update_comics_images` 就地补 `ImageNode`，不再「销毁占位实体 +
  新建图片实体 + insert_children(0)」——那套会打乱子节点顺序，与复用冲突
- 改绑后 `DownloadStatusBadge` / `ComicSelectionMark` 的组件被写过，
  `refresh_download_status_badges` / `refresh_comics_selection_ui` 靠 `Ref<T>::is_changed()`
  只刷这几个，不必全场重刷
- `comics_rebind_cards` 的查询把 `Text`/`TextColor`/`BackgroundColor` 全塞进**同一个**
  `node_query` 做 `Option<&mut>`：拆成多个 `&mut Node`/`&mut Text` 查询会直接撞 B0001
- 数据/列数变化：全量重建窗口；屏蔽词过滤只在数据变化的低频路径执行
- 其余分页页面数据量有界，仍用瀑布流分帧显示（`WaterfallState`）

### 漫画列表批量下载

`ComicsSelectionState { active, selected }`：开启选择模式后卡片点击变勾选而非跳详情。
标题栏右侧「选择」开关 + 选择模式下的「已选 N / 全选 / 清空 / 下载选中」。

- 「全选」以 `ComicsVirtualState.filtered` 为准——屏蔽掉的漫画根本没建卡，不该被捎带
- 选中标记在封面**左上角**，与右下角下载角标错开
- 下载请求带上 `remote_eps_count`，下完即有更新基准
- 换分类（`setup_comics_list_ui`）会 `exit()`：换了数据集，旧选中项不该带过来

### 阅读器缩放（reader.rs）

三条等价入口共用 `apply_scale()`（钳位 + 刷工具栏百分比，避免各写一份后走样）：
工具栏 `− / xx% / + / ⟲` 按钮、键盘 `+` `-` `0`、`Ctrl/⌘ + 滚轮`。

- ⚠️ **macOS 会把 `Ctrl+滚轮` 当系统缩放手势吃掉**，触控板上基本到不了应用；
  所以按钮与键盘入口是必需的，不是锦上添花。⌘ 也一并认（不被系统抢）
- ⚠️ 条漫模式此前**完全无视 `scale`**：宽度硬编码 `WEBTOON_IMAGE_WIDTH_PERCENT`(80%)，
  表现为"缩放没反应"。现在宽度 = `80% × scale`，且**未加载的占位槽也要跟着改**，
  否则放大后已载入的图和占位槽宽度不一致，滚动条与锚定都会跳
- 放大超过 125% 时图片宽于视口，容器改 `Overflow::scroll()`（双向），
  `Shift+滚轮` 横向平移

### 条漫滚动：锚点是唯一真相（reader.rs，2026-08 下旬重设计）

条漫槽位以占位高度（1000px）起步，真实图高陆续就位——同一滚动像素映射到的页码
会**级联漂移**。

**现行模型**：`ReaderState.webtoon_anchor: (页, 页内偏移)` 是滚动位置的**唯一真相**，
每帧由它算出 `ScrollPosition` 写下去。锚定页上方的高度一变，换算结果同步变，
视觉位置自然不动，**且与用户是否正在滚动无关**。

| 职责 | 系统 |
|------|------|
| 实测页高 → 由锚点算 `ScrollPosition` | `sync_webtoon_scroll`（每帧无条件） |
| 按当前页决定加载哪些图 | `update_webtoon_window` |
| 滚轮 → 改**锚点**（不碰 ScrollPosition） | `reader_mouse_wheel_control` |

- `page_top` / `anchor_to_scroll` / `scroll_to_anchor` / `content_height` 是**纯函数**，
  配有单测（含核心不变量：锚定页上方高度变化不改变其视觉位置）——滚动错位靠肉眼
  复现代价太高
- 页高表 `webtoon_page_heights` 与 `pictures` 平行，测量值覆盖占位值。**不做**
  「按已测均值重估未测页」——那会让锚点上方的估算高度变动，反而制造跳动
- 本容器**不挂 `ScrollArea`**（滚轮由阅读器模态处理），所以 `sync_webtoon_scroll`
  是 `ScrollPosition.y` 的唯一写者。这正是能把它当「锚点的投影」而非状态的前提
- ⚠️ 上界优先用引擎 `content_size()`，但**首帧它是 0**，直接用会把 max_scroll
  算成 0、把「恢复上次阅读页」的锚点当场钳回第 1 页——引擎值不可信时退回页高表自算
- 横向（缩放放大后 Shift+滚轮平移）直接写 `ScrollPosition.x`：横向没有「内容宽度
  会变」的问题，不需要锚点

**旧实现错在哪**（别再走回去）：`ScrollPosition` 是真相、锚点当补丁，且补偿只在
「非用户滚动帧」执行，用 `Local<bool>` 区分补偿写入与用户滚动。用户一路拖到底时
**每帧都是用户滚动帧**，补偿永远轮不到 → 新图一加载就错位。

### 图片占位骨架屏微光

图片没到位时占位块在两档底色间脉动（`ui_common::LoadingShimmer` +
全局 `animate_loading_shimmer`）。接入：漫画列表 / 搜索 / 收藏 / 首页封面占位、
条漫槽位。

- **挂在占位节点自身，不建子实体**：占位块要么被 `ImageNode` 就地覆盖
  （漫画列表的节点复用要求封面是单实体），要么整个换掉；多一个子节点就得跟着
  处理生命周期，还会盖在图片上面
- 动画只改 `BackgroundColor`，不碰布局
- 查询带 `Without<ImageNode>`：图片一就位节点自动退出查询，**零清理代码**
- 每个占位块按实体号错开相位，否则整屏同频闪烁，像坏了而不像在加载
- ⚠️ **终局失败必须摘掉 `LoadingShimmer`**（comics 与 reader 的图片系统各有一处），
  否则失败的框会一直脉动，等于骗用户"还在加载"

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

## 下载更新：普通更新 vs 强制更新

「已下载」项与标题栏各有两个按钮，走同一套下载流程，差别只在**要不要做前置检查**：

| 动作 | 消息 | 行为 | 代价 |
|------|------|------|------|
| 更新 / 全部更新 | `RedownloadRequest { force: false }` | `handle_redownload_precheck` 三段判定，见下 | 快路径 1 个请求 |
| 强制更新 / 全部强制更新 | `RedownloadRequest { force: true }` | 直接进 `handle_redownload`，逐章拉图片列表、逐图比对文件名补缺 | 每本 N 章 × 若干请求 |

**前置检查三段**（`handle_redownload_precheck`）：

1. 取漫画详情拿当前 `epsCount`（1 个请求）
2. 有基准且 `epsCount` 没变大 → **已是最新，收工**
3. 否则拉一次真实章节列表（分页，每页 ~40 条）核对：
   - 真实条数 ≤ 本地章节数 → 只是 `epsCount` 漂移，**把新基准写回去**再跳过
   - 否则 → 真有新章节 → `RedownloadConfirmed` → `handle_redownload`（与强制更新同一条路径）

- 判定已是最新 / 检查失败 → 写 `RedownloadSkipped` → `handle_redownload_skipped` 按帧聚合成一条
  Toast（「全部更新」一次几十条，逐条弹没法看）
- 第 3 段兼作**老记录的基准回填**：升级前的下载记录没有基准，第一次「更新」会走
  第 3 段并把基准补上，之后就都是 1 个请求的快路径
- **已下载列表项在「确认要下载」时才摘除**（`remove_completed_item_on_redownload`），
  不能在点击时摘——普通更新可能被判定已是最新，摘了还得补回来

## 封面下载角标

漫画卡片封面右下角显示下载状态：绿色 ✓ = 已下载，橙色 ⟳ = 有新章节。

### ⚠️ 更新判定不能用 `epsCount` 直接比本地章节数

服务端 `Comic::eps_count` 与 `/comics/{id}/eps` 的真实条数**长期对不上，且两个方向都会偏**
（实测：48↔49、46↔48、12↔15、55↔53）。它是个漂移的冗余计数。拿它跟本地章节数直接比，
早就下完的漫画会常年亮「有更新」。

正解：**存下载当时的 `epsCount` 快照，跟今天的 `epsCount` 自比**——同一字段自比，
系统偏差相消；漫画真加了章节，这个计数一定变大。

- 快照存 `download_task.remote_eps_count`（`ALTER TABLE` 迁移，None = 升级前的老记录）
- 老记录判不了 → 只报「已下载」，不猜更新；第一次点「更新」会把基准补上（见上一节第 3 段）
- `upsert` 用 `COALESCE(excluded.remote_eps_count, download_task.remote_eps_count)`：
  调用方没拿到 `epsCount` 时保留旧基准，别把它冲成 NULL

### 其余实现

- 数据源 `DownloadedComicsIndex`（`comic_id → epsCount 快照`）：建卡是高频路径，不能每张卡查库；
  只在**启动**（`setup_download_manager`）、**下载完成**（`handle_download_completed`）、
  **删除记录**（`confirm_delete_button_interaction`）、**前置检查回填基准**四处同步
- 未下载时 `Visibility::Hidden` 而非不创建——否则下载完回到列表页要等整页重建才看得到
- `refresh_download_status_badges` 全局注册一次，索引变化时统一改可见性/图标/底色，卡片不必重建
- 定位：`PositionType::Absolute` + `BadgeAnchor` 两种锚法。taffy 的 inset 参照系是
  **父节点 padding box**（`container_size − border`，见 `compute/flexbox.rs`
  "Insets are resolved against the container size minus border"），故：
  - `CardCover`（角标与封面同为卡片直接子节点，卡片 180 宽 / padding 8 / border 1，封面 164×220）
    → `right: 10, top: 204`（= 178−172+4 / 8+220−4−20）
  - `CoverContainer`（角标在封面容器内）→ `right: 4, bottom: 4`
- **角标是「定尺寸圆底容器 + 文本子节点」两个实体**，不是一个 Text 顶着背景色：
  Bevy 把文本画在 content box 的**左上角**（`bevy_ui_render/text.rs` 用 `content_box().min`），
  既不水平也不垂直居中。曾用 padding 手调，结果图标左偏 3.25px——Nerd Font 图标在
  更纱黑体里是**半角**（advance 0.5em），按整角估的 padding 必然偏。交给 flex 的
  `justify_content` / `align_items` 居中文本节点才与字体度量无关
- 字号上限由行高定：Sarasa 的 hhea 行高 1.25em，字号 15 → 行盒 18.75px，20px 徽章里余 1.25px
- 已接入：comics / search / favorites / rankings / home 五个页面

## 应用图标（窗口 + macOS Dock）

`systems/app_icon.rs`，用的是**哔咔漫画官方 app 图标**，`include_bytes!` 编译进二进制
（部署漏拷 assets 也不会退化成系统默认方块）。

| 文件 | 角色 |
|------|------|
| `assets/icons/picacg-official-192.png` | **权威源**：官网 PWA 的 `logo_round`（192×192，粉底圆角 + 哔咔娘，自带 alpha），已入库，构建不依赖网络 |
| `assets/icons/icon.png` | 512×512，macOS dock / ⌘Tab（由权威源 Catrom 放大，勿手改） |
| `assets/icons/icon-256.png` | 256×256，Windows 任务栏 / Linux 标题栏（同上） |

`scripts/make-icon.sh` 从权威源生成两个投放尺寸；`--fetch` 从官网重抓素材
（走代理时设 `ALL_PROXY`）。官方只提供到 192，放大用 Catrom——Lanczos 会在
网点边缘振铃。

| 平台 | 生效位置 | 手段 |
|------|----------|------|
| Windows / Linux | 标题栏、任务栏 | winit `Window::set_window_icon`（`icon-256.png`） |
| macOS | Dock、⌘Tab | AppKit `NSApplication::setApplicationIconImage:`（`icon.png`） |

- macOS 上 winit 的窗口图标是**空操作**（系统只认 .app 包里的 icns），dock 图标必须运行时经
  AppKit 设置——这样 `cargo run` 跑裸二进制也有图标
- bevy 0.19 的 winit 窗口存在主线程 thread_local `WINIT_WINDOWS`（**不再是 `NonSend` 资源**，
  写 `NonSend<WinitWindows>` 会每帧报 "Non-send data not found"）；用 `NonSendMarker`
  把系统钉在主线程，与上游 `changed_windows` 同款
- 系统常驻 `Update`：合盖等场景窗口会被销毁重建，新窗口要重新贴图标（已处理的记在 `Local` 集合里）

## 下载全部完成后退出

设置页「下载全部完成后退出」（`AppSettings::exit_after_downloads`），挂机下载用。

- `exit_after_downloads_complete`：还有活儿 = 任务处于 `Downloading`/`Queued`，或 CBZ 还在打包；
  **已暂停/已失败的不算**——它们不会自己往前走，等下去等于永不退出
- `Local<bool> was_busy` 保证只在本次运行确实跑过下载后才触发（刚启动队列本来就空，不能直接退）
- `CbzPackagingState.in_flight` 计数（请求 +1，完成/失败 −1）：打包在后台线程写文件，
  进程此时退出会留下半截 .cbz
- 退出前调 `save_window_geometry_to_config`，与用户主动关闭走同一套收尾
- 两个下载行为开关合在 `DownloadBehaviorState`（原 `AutoResumeDownloadsState`）

## 性能追踪

**入口都在设置页「性能追踪」分组**（打包成 .app 后没有终端，诊断必须能在界面里完成）：

| 入口 | 回答什么 | 前提 |
|------|----------|------|
| 「性能叠加层」开关 / `F3` | **卡不卡、多卡**：FPS / 帧时间 / 实体数 / UI 节点数 | 无，立即生效 |
| 「系统耗时追踪」开关 | 打开统计 | **重启后生效** |
| 「刷新耗时榜」按钮 / `F4` | **谁在卡**：Top N 直接渲染在页面里 | 开关已开 |
| 自动 | 掉帧时自动打榜（默认 >100ms，`PICACG_PROFILE_SLOW_MS` 可调，5 秒冷却） | 同上 |

榜单**追加落盘**到 `AppSettings::profiling_log_path()`（配置目录 `logs/profiling.log`），
设置页有「日志目录」按钮直达。标题里带**当前页面**——卡顿多半是某页一次性建大量节点
导致的，不带页面信息只能看出「谁慢」、看不出「在哪慢」。

**原理**：bevy 给每个系统的每次执行套了 `info_span!("system", name = ...)`，
`utils/profiling.rs` 挂一个 `tracing` Layer 把 enter/exit 差值按系统名累加——
不必改动那 120+ 个系统的注册代码。

**为什么开关要重启**：两层门。① 编译期，那对 span 在 bevy_ecs 里是
`#[cfg(feature = "trace")]` 门控的，不编进来就不存在——所以 `profiling` 是**默认 feature**；
② 启动期，bevy 在系统初始化时**一次性**建好 Span 对象，那一刻没有订阅者感兴趣就
永远是禁用态，运行中再装 Layer 也救不回来。反过来这也是关掉时几乎零成本的原因：
tracing 把 callsite 缓存成 `Interest::never()`，每系统每帧只多一次分支。

**踩过的坑**：

- 只做运行时开关不开 feature → 榜单恒空（曾照着"不受 feature 门控"的错误判断写过一版）
- span 是 `info` 级：日志等级低于 info，tracing 会把 span 整个滤掉，榜单同样是空的
- `Extensions::insert` 撞到同类型已存在会 **panic**；同一 span 每帧重新进入，
  记录进入时刻必须用 `replace`（第一版在登录页直接崩）
- **次数会有偏差**：实测某 `Startup` 系统函数体只跑 1 次，span 却被进出 280 次且
  enter 套 enter——bevy 内部共享/克隆同一个 `Span` 对象，多个系统的进出落到同一个
  id 上。上游行为，本层改不了；稳态数字是准的（所有系统次数 = 帧数）

**读榜提示**：

- `bevy_render::view::window::prepare_windows` 常年十几毫秒/帧，**那是在等 vsync**
  （`get_current_texture()` 阻塞到下一张交换链图像），不是瓶颈；`run_render_schedule`
  包着它，所以也一样大。看这两个数会把人带偏
- `bevy_app::main_schedule::Main::run_main` 是所有子调度的父区间，看它下面的具体系统
- **看「单次峰值」而不是「总耗时」找卡顿**：稳态开销看均值，一次性卡顿看峰值。
  实测一份 48844 帧的榜单里，`text_system` 均值 0.15ms 但峰值 **1079ms**——
  那一帧 `run_main` 峰值 1127ms，几乎全是文本整形，这才是卡顿源
- 启动那一帧的 `text_system`（首次字形整形）与 `apply_app_icon`（PNG 解码 + AppKit
  刷 dock）是一次性成本，不是稳态开销

## 检查更新（只查不装）

设置页「关于」分组：`⟳ 检查更新` 手动触发，`⬇ 前往下载` 在检出新版本时才显示
（`open::that` 打开 GitHub Release 页），另有「启动时自动检查更新」开关。

- **只跳转、不自替换**：原地升级在三平台各有坑（Windows 不能覆盖运行中的 exe、
  macOS 未签名替换触发 Gatekeeper），投入产出比差，另案评估
- ⚠️ `compare_versions` 必须先切掉 `+build` / `-prerelease` 后缀再比数字段。
  本项目发版格式就是 `v{version}+{commit}`，旧实现用
  `filter_map(parse::<u64>)` 把 `0+abc1234` 这种段**静默丢掉**，
  `0.5.1+abc` 被解析成 `[0,5]`，patch 位一带后缀就误判为「无更新」。已加单测覆盖
- ⚠️ 历史坑：`check_update_button_interaction` / `refresh_update_status`
  **从未注册到调度**，按钮点了没有任何反应，而 CLAUDE.md 里已标记为完成。
  新增页面系统后务必确认 `ui_plugin.rs` 里接线了

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

> ⚠️ **提交前先 `rustup update nightly`**
>
> CI 用 `dtolnay/rust-toolchain@nightly`，每次装的是**当前最新** nightly。本地
> nightly 一旦落后，就会出现「本地 `cargo fmt --all --check` 通过、CI 的
> Format Check 却挂」——rustfmt 的行为在 nightly 之间会变（实测 2026-08-29
> 那版起把 CJK 字符按双宽计算，中文注释折行位置整体改变，十几个文件被重排）。
>
> 仓库根的 `rust-toolchain.toml` 只声明 `channel = "nightly"`（**不钉日期**，
> 跟随最新），它保证「用 nightly」，**不保证「用同一个 nightly」**——
> 版本对齐只能靠上面这条命令。更新后按全局 CLAUDE.md §5 跑一次
> `cargo sweep -r --installed ~/repo/rust` 清旧工具链产物。
>
> 触发分支必须包含 **`dev`**：本项目日常开发在 dev，配置里原先只写了
> `develop`，导致 dev 的推送长期不跑 CI、格式漂移无人发现。

| Job | 触发条件 | 说明 |
|-----|---------|------|
| `fmt` | push/PR | `cargo fmt --all -- --check` |
| `clippy` | push/PR | `cargo clippy --all --all-targets` |
| `test` | clippy 通过后 | `cargo test --all`（**不用 release**，见下） |
| `build` | test 通过后 | Linux x64 + Windows x64 + macOS ARM64 矩阵构建 |
| `release` | 推送 `v*` 标签 | 下载产物、创建 GitHub Release |
| `dev-build-summary` | master/main/develop 推送 | 生成构建摘要 |

> ⚠️ **test job 不能加 `--release`**：根 `Cargo.toml` 的 release profile 是
> `lto = "fat"` + `codegen-units = 1`，后者强制把整个 crate 的代码生成塞进单个
> 单元，**峰值内存 = 整个 crate 一次性展开**——`bevy_core_pipeline` / `bevy_render`
> 这类巨型 crate 单个 rustc 就要约 **7GB**。实测即便 `CARGO_BUILD_JOBS=2`，
> 两个进程也能把 16GB 的 runner 吃穿（可用内存 80 秒内从 14GB 掉到 28MB）。
> 单测全是纯函数，release 零额外覆盖；release 的编译覆盖由 `build` job 承担。
> **这个 profile 是为产物刻意配的，不要为了 CI 去改它。**
>
> ⚠️ **仓库必须保持 public**：私有仓库的免费 runner 只有 **2 核 / 7.8GB**，
> 编译 `bevy_ui` / `bevy_render` 会把内存吃穿，runner 直接被杀
> （`The runner has received a shutdown signal` + exit 143，日志停在编译中途、
> 无任何现场）。public 仓库是 4 核 / 16GB，同一套配置 13 分钟就能跑完。
> 曾在这上面连挂 7 次 CI，先后错怪过 release 编译、并行度、`bevy/trace`、
> `Cargo.lock` 未入库——全都不是。**先看 `nproc` 和 `free -h`，别猜**。

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

### 已完成（2026-08 下旬）

- [x] 修复高级搜索排序不生效（sort 挪进请求体）+ 排序按钮选中态刷新
- [x] 下载更新分普通/强制两档（章节数前置比对 vs 逐图校验），单本与「全部」各两个按钮
- [x] 漫画卡片封面下载角标（已下载 ✓ / 有更新 ⟳，5 个页面接入）
- [x] 应用图标：哔咔官方图标接入窗口/任务栏 + macOS dock（运行时 AppKit 设置，免打 .app 包）
- [x] 「下载全部完成后退出」设置项（等 CBZ 打包收尾后再退）
- [x] 修复封面角标假「有更新」：改用 `epsCount` 快照自比（`epsCount` 与真实章节数长期对不上）
- [x] 修复角标图标未居中（Nerd Font 图标是半角，padding 手调必偏）
- [x] 性能追踪：F3 帧时叠加层 + F4/掉帧自动的系统耗时榜（`--features profiling`）
- [x] 漫画列表节点复用：滚动路径零 spawn/despawn（固定形态卡片 + `plan_recycle` + 单测）
- [x] 漫画列表批量选择下载（选择模式 / 全选 / 下载选中，带更新基准）
- [x] 阅读器手动缩放：工具栏按钮 + 修复条漫模式无视 scale + Shift 横向平移
- [x] 右键/批量下载补上 `epsCount` 基准，下完即可正确显示更新角标
- [x] 条漫滚动重设计：锚点为唯一真相，根除「拉到底加载新图错位」（含 8 个纯函数单测）
- [x] 性能追踪搬进设置页 + 榜单落盘（`logs/profiling.log`，标题带页面）
- [x] 图片占位骨架屏微光动画（5 处占位接入，失败即停）

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
- [x] 检查更新（GitHub Releases API，版本比较 + 前往下载入口 + 启动自动检查）
- [x] 侧边栏布局优化（用户信息区/版本号 flex_shrink:0，菜单区精简移除工具分组）

---

## 参考资料

- [Bevy 0.19 发布说明](https://bevy.org/news/bevy-0-19/)（BSN / ui_widgets / input_focus）
- [Bevy 0.18→0.19 迁移指南](https://bevy.org/learn/migration-guides/0-18-to-0-19/)
- [Bevy 官方文档](https://docs.rs/bevy/latest/bevy/)
- [Tokio 官方文档](https://tokio.rs/)
- [PicACG API 文档](../docs/)
- python版本在"C:\Users\ffqi\dev\py\picacg-windows"