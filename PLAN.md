# GUI 库抽象实现计划

## 目标

将 PicACG Rust 项目中的通用 GUI 组件抽象为独立的 crate `bevy_ui_toolkit`，实现：
- 滚动条系统（Scrollbar）
- 分页组件（Pagination）
- 瀑布流系统（Waterfall）
- 主题/颜色系统（Theme）

## 架构设计

### 目录结构

```
picacg-rust/
├── Cargo.toml                 # 改为 workspace 根配置
├── crates/
│   └── bevy_ui_toolkit/       # 新建的通用 GUI 库
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs         # 库入口，导出所有模块
│           ├── theme.rs       # 主题/颜色系统
│           ├── scrollbar/     # 滚动条系统
│           │   ├── mod.rs
│           │   ├── components.rs
│           │   └── systems.rs
│           ├── pagination/    # 分页组件
│           │   ├── mod.rs
│           │   ├── components.rs
│           │   └── systems.rs
│           └── waterfall/     # 瀑布流系统
│               ├── mod.rs
│               └── state.rs
└── src/                       # 主应用（picacg）
    ├── main.rs
    ├── systems/
    │   ├── scrollbar.rs       # 删除，移到 crate
    │   ├── pagination.rs      # 删除，移到 crate
    │   ├── waterfall.rs       # 保留业务特定部分
    │   └── login.rs           # 移除 AppColors，使用 crate 的 theme
    └── components/
        └── ui_components.rs   # 移除滚动条组件，使用 crate 的
```

### 模块设计

#### 1. Theme 模块 (`theme.rs`)

将 `AppColors` 重构为可配置的主题系统：

```rust
/// 主题配置
#[derive(Clone)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub card_bg: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_pressed: Color,
    pub secondary: Color,
    pub secondary_hover: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub error: Color,
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()  // 默认深色主题
    }
}

impl Theme {
    /// 深色主题（当前 AppColors 配色）
    pub fn dark() -> Self { ... }

    /// 浅色主题（预留）
    pub fn light() -> Self { ... }
}

/// 全局主题资源
#[derive(Resource, Deref, DerefMut)]
pub struct CurrentTheme(pub Theme);
```

#### 2. Scrollbar 模块

**components.rs:**
```rust
/// 滚动条容器
#[derive(Component)]
pub struct ScrollbarContainer {
    pub scroll_container: Entity,
}

/// 滚动条轨道
#[derive(Component)]
pub struct ScrollbarTrack {
    pub scroll_container: Entity,
}

/// 滚动条滑块
#[derive(Component)]
pub struct ScrollbarThumb {
    pub scroll_container: Entity,
}

/// 拖拽状态（Resource）
#[derive(Resource, Default)]
pub struct ScrollbarDragState { ... }

/// 内容尺寸信息
#[derive(Component, Default)]
pub struct ContentSizeInfo { ... }

/// 滚动条配置
pub struct ScrollbarConfig {
    pub width: f32,
    pub thumb_min_height: f32,
    pub track_color: Color,
    pub thumb_color: Color,
    pub thumb_hover_color: Color,
    pub thumb_pressed_color: Color,
}
```

**systems.rs:**
- `update_all_scrollbar_thumbs`
- `scrollbar_thumb_interaction`
- `scrollbar_track_click`
- `scrollbar_thumb_drag`
- `reset_drag_state_on_release`

#### 3. Pagination 模块

**components.rs:**
```rust
#[derive(Component)]
pub struct PaginationControl<T> { ... }

#[derive(Component)]
pub struct PaginationPrevButton<T> { ... }

#[derive(Component)]
pub struct PaginationNextButton<T> { ... }

#[derive(Component)]
pub struct PaginationPageText<T> { ... }

pub struct PaginationConfig { ... }
```

**systems.rs:**
```rust
pub fn spawn_pagination_controls<T>(...) { ... }
pub fn spawn_pagination_controls_with_config<T>(...) { ... }
pub fn update_pagination_display<T>(...) { ... }
pub fn check_pagination_interaction<T>(...) -> Option<bool> { ... }
```

#### 4. Waterfall 模块

仅包含通用的 `WaterfallState<T, C>` 结构体：

```rust
/// 瀑布式显示状态（泛型）
#[derive(Resource)]
pub struct WaterfallState<T, C = ()>
where
    T: Send + Sync + 'static,
    C: Send + Sync + Default + 'static,
{
    pub pending_entities: Vec<Entity>,
    pub font_handle: Option<Handle<Font>>,
    pub is_creating: bool,
    pub timer: Timer,
    pub first_batch: bool,
    pub batch_size: usize,
    pub precreate_count: usize,
    pub context: C,
    _marker: PhantomData<T>,
}

// 所有方法保持不变
```

**业务特定类型留在主应用：**
- `RankingsWaterfall`, `CategoriesWaterfall` 等标记类型
- `RankingsContext` 等上下文类型
- `RankingsCardCreationState` 等类型别名

## 实现步骤

### 第一阶段：创建 crate 结构

1. **修改根 `Cargo.toml` 为 workspace 配置**
   - 添加 `[workspace]` 段
   - 定义 members

2. **创建 `crates/bevy_ui_toolkit/` 目录和 `Cargo.toml`**
   - 依赖：`bevy = "0.17"`
   - 暂不发布到 crates.io

### 第二阶段：迁移 Theme 模块

3. **创建 `theme.rs`**
   - 实现 `Theme` 结构体
   - 实现 `CurrentTheme` 资源
   - 提供 `dark()` 和 `light()` 预设

4. **更新主应用**
   - `login.rs` 中删除 `AppColors`
   - 所有使用 `AppColors` 的地方改为使用 `CurrentTheme`

### 第三阶段：迁移 Scrollbar 模块

5. **创建 `scrollbar/` 模块**
   - 移动组件定义（从 `ui_components.rs`）
   - 移动系统函数（从 `scrollbar.rs`）
   - 添加 `ScrollbarConfig` 配置支持

6. **更新主应用**
   - 删除 `src/systems/scrollbar.rs`
   - 更新 `ui_components.rs` 移除滚动条组件
   - 更新所有使用滚动条的页面的 import

### 第四阶段：迁移 Pagination 模块

7. **创建 `pagination/` 模块**
   - 移动所有分页组件和函数
   - 更新依赖（使用 `Theme` 而非 `AppColors`）

8. **更新主应用**
   - 删除 `src/systems/pagination.rs`
   - 更新 `favorites.rs`, `comics.rs` 的 import

### 第五阶段：迁移 Waterfall 模块

9. **创建 `waterfall/` 模块**
   - 只移动 `WaterfallState<T, C>` 结构体和方法

10. **更新主应用 `waterfall.rs`**
    - 保留业务特定的标记类型和上下文
    - 重新导出 `WaterfallState` 从 crate

### 第六阶段：完善和测试

11. **添加 Plugin 封装**
    ```rust
    pub struct BevyUiToolkitPlugin {
        pub theme: Theme,
    }

    impl Plugin for BevyUiToolkitPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(CurrentTheme(self.theme.clone()))
               .init_resource::<ScrollbarDragState>()
               // 注册滚动条系统...
        }
    }
    ```

12. **运行测试和构建验证**
    ```powershell
    cargo fmt --all
    cargo clippy --all
    cargo build
    cargo run
    ```

## 依赖关系

```
bevy_ui_toolkit (新 crate)
├── theme.rs          # 无内部依赖
├── scrollbar/        # 依赖 theme
├── pagination/       # 依赖 theme
└── waterfall/        # 无依赖

picacg (主应用)
├── 依赖 bevy_ui_toolkit
├── waterfall.rs      # 使用 bevy_ui_toolkit::WaterfallState，定义业务类型
└── 其他页面          # 使用 bevy_ui_toolkit 的组件和系统
```

## 公开 API

```rust
// crates/bevy_ui_toolkit/src/lib.rs

pub mod theme;
pub mod scrollbar;
pub mod pagination;
pub mod waterfall;

pub use theme::{Theme, CurrentTheme};
pub use scrollbar::{
    ScrollbarContainer, ScrollbarTrack, ScrollbarThumb,
    ScrollbarDragState, ContentSizeInfo, ScrollbarConfig,
    update_all_scrollbar_thumbs, scrollbar_thumb_interaction,
    scrollbar_track_click, scrollbar_thumb_drag, reset_drag_state_on_release,
};
pub use pagination::{
    PaginationControl, PaginationPrevButton, PaginationNextButton, PaginationPageText,
    PaginationConfig, spawn_pagination_controls, spawn_pagination_controls_with_config,
    update_pagination_display, check_pagination_interaction, PaginationState,
};
pub use waterfall::WaterfallState;

pub struct BevyUiToolkitPlugin { ... }
```

## 风险和注意事项

1. **API 兼容性**：确保主应用的现有代码能平滑迁移
2. **主题注入**：分页组件需要从 `CurrentTheme` 资源读取颜色
3. **系统注册顺序**：滚动条系统需要在特定 Update 阶段运行
4. **测试覆盖**：迁移后需要手动测试所有使用这些组件的页面

## 预计影响文件

### 新建文件
- `crates/bevy_ui_toolkit/Cargo.toml`
- `crates/bevy_ui_toolkit/src/lib.rs`
- `crates/bevy_ui_toolkit/src/theme.rs`
- `crates/bevy_ui_toolkit/src/scrollbar/mod.rs`
- `crates/bevy_ui_toolkit/src/scrollbar/components.rs`
- `crates/bevy_ui_toolkit/src/scrollbar/systems.rs`
- `crates/bevy_ui_toolkit/src/pagination/mod.rs`
- `crates/bevy_ui_toolkit/src/pagination/components.rs`
- `crates/bevy_ui_toolkit/src/pagination/systems.rs`
- `crates/bevy_ui_toolkit/src/waterfall/mod.rs`
- `crates/bevy_ui_toolkit/src/waterfall/state.rs`

### 修改文件
- `Cargo.toml` - 改为 workspace
- `src/systems/login.rs` - 移除 AppColors
- `src/systems/waterfall.rs` - 保留业务类型，重新导出
- `src/components/ui_components.rs` - 移除滚动条组件
- `src/plugins/ui_plugin.rs` - 更新系统注册

### 删除文件
- `src/systems/scrollbar.rs`
- `src/systems/pagination.rs`

### 需要更新 import 的文件
- `src/systems/categories.rs`
- `src/systems/comics.rs`
- `src/systems/detail.rs`
- `src/systems/downloads.rs`
- `src/systems/favorites.rs`
- `src/systems/home.rs`
- `src/systems/rankings.rs`
- `src/systems/search.rs`
- `src/systems/settings.rs`
- `src/systems/register.rs`
- `src/systems/proxy_settings.rs`
