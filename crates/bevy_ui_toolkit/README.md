# Bevy UI Toolkit

Bevy 0.17 通用 UI 组件库，提供滚动条、分页、瀑布流等常用组件。

## 功能特性

| 模块 | 功能 | 说明 |
|------|------|------|
| **Theme** | 主题/颜色系统 | 深色/浅色主题，可自定义颜色 |
| **Scrollbar** | 自定义滚动条 | VSCode 风格，支持轨道点击、滑块拖拽 |
| **Pagination** | 泛型分页组件 | 支持多页面复用，上/下一页按钮 |
| **Waterfall** | 瀑布流显示 | 预创建 + 分批显示，避免布局卡顿 |

## 安装

### 从 Git 仓库安装

```toml
[dependencies]
bevy_ui_toolkit = { git = "https://git.yang.cafe:30080/loosqk/bevy_ui_toolkit.git" }
```

### 从本地路径安装

```toml
[dependencies]
bevy_ui_toolkit = { path = "../bevy_ui_toolkit" }
```

## 快速开始

```rust
use bevy::prelude::*;
use bevy_ui_toolkit::BevyUiToolkitPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyUiToolkitPlugin::default())
        .run();
}
```

---

## 模块详解

### 1. Theme 主题系统

提供统一的颜色配置，支持深色和浅色主题。

#### 使用默认深色主题

```rust
use bevy_ui_toolkit::BevyUiToolkitPlugin;

app.add_plugins(BevyUiToolkitPlugin::default());
```

#### 自定义主题

```rust
use bevy_ui_toolkit::{BevyUiToolkitPlugin, Theme};

app.add_plugins(BevyUiToolkitPlugin {
    theme: Some(Theme::light()),  // 使用浅色主题
});
```

#### 在系统中访问主题

```rust
use bevy_ui_toolkit::CurrentTheme;

fn my_ui_system(theme: Res<CurrentTheme>) {
    let background = theme.background;
    let primary = theme.primary;
    let text_color = theme.text;
    // ...
}
```

#### Theme 字段说明

| 字段 | 说明 |
|------|------|
| `background` | 背景色 |
| `surface` | 表面/卡片背景色 |
| `card_bg` | 卡片背景色（更浅） |
| `primary` | 主色调（按钮、链接） |
| `primary_hover` | 主色调悬停状态 |
| `primary_pressed` | 主色调按下状态 |
| `secondary` | 次要色调 |
| `secondary_hover` | 次要色调悬停状态 |
| `text` | 主文本颜色 |
| `text_secondary` | 次要文本颜色 |
| `text_muted` | 弱化文本颜色 |
| `error` | 错误颜色 |
| `success` | 成功颜色 |
| `warning` | 警告颜色 |
| `border` | 边框颜色 |

---

### 2. Scrollbar 滚动条系统

实现 VSCode 风格的自定义滚动条，支持：
- 轨道点击快速跳转
- 滑块拖拽滚动
- 自动计算滑块大小
- DPI 缩放适配

#### 步骤 1：注册滚动条系统

```rust
use bevy_ui_toolkit::{
    BevyUiToolkitPlugin,
    update_all_scrollbar_thumbs,
    scrollbar_thumb_interaction,
    scrollbar_track_click,
    scrollbar_thumb_drag,
    reset_drag_state_on_release,
};

app.add_plugins(BevyUiToolkitPlugin::default())
    .add_systems(Update, (
        update_all_scrollbar_thumbs,
        scrollbar_thumb_interaction,
        scrollbar_track_click,
        scrollbar_thumb_drag,
        reset_drag_state_on_release,
    ));
```

#### 步骤 2：创建滚动容器

```rust
use bevy::prelude::*;
use bevy::ui::ScrollPosition;
use bevy_ui_toolkit::{ContentSizeInfo, scrollbar_config::*};

// 定义自己的滚动容器标记
#[derive(Component)]
struct MyScrollContainer;

fn setup_ui(mut commands: Commands) {
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Relative,
        ..default()
    })
    .with_children(|parent| {
        // 创建滚动容器，保存 Entity ID
        let scroll_container = parent
            .spawn((
                MyScrollContainer,
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
                // 添加滚动内容...
                for i in 0..50 {
                    content.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                    ));
                }
            })
            .id();  // 保存 Entity ID

        // 创建滚动条
        spawn_scrollbar(parent, scroll_container);
    });
}

/// 创建滚动条 UI
fn spawn_scrollbar(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
    use bevy_ui_toolkit::{ScrollbarContainer, ScrollbarTrack, ScrollbarThumb};

    parent
        .spawn((
            ScrollbarContainer { scroll_container },
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(SCROLLBAR_WIDTH),
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
                    BackgroundColor(TRACK_COLOR),
                    Transform::default(),  // 必须添加！
                ))
                .with_children(|track| {
                    // 滑块
                    track.spawn((
                        ScrollbarThumb { scroll_container },
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(8.0),
                            height: Val::Px(50.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(THUMB_COLOR),
                        BorderRadius::all(Val::Px(4.0)),
                        Transform::default(),  // 必须添加！
                    ));
                });
        });
}
```

#### 步骤 3：更新内容尺寸

```rust
fn update_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<MyScrollContainer>,
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

#### 重要注意事项

1. **Transform 组件必须添加**：轨道和滑块需要 `Transform::default()`，否则 `GlobalTransform` 不可用
2. **ContentSizeInfo 需手动更新**：在 Update 系统中更新 `content_height` 和 `viewport_height`
3. **DPI 缩放**：`ComputedNode::size()` 返回物理像素，需除以 `scale_factor` 转换

---

### 3. Pagination 分页组件

泛型分页组件，支持多个页面复用同一套组件。

#### 步骤 1：定义页面标记类型

```rust
// 每个需要分页的页面定义一个标记类型
pub struct FavoritesPage;
pub struct ComicsPage;
pub struct SearchPage;
```

#### 步骤 2：创建分页 UI

```rust
use bevy_ui_toolkit::{spawn_pagination_controls_with_theme, CurrentTheme};

fn setup_pagination(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<CurrentTheme>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.spawn(Node::default()).with_children(|parent| {
        spawn_pagination_controls_with_theme::<FavoritesPage>(
            parent,
            &font,
            &theme,
            1,   // 当前页码
            10,  // 总页数
        );
    });
}
```

#### 步骤 3：处理分页交互

```rust
use bevy_ui_toolkit::{
    check_pagination_interaction,
    PaginationPrevButton, PaginationNextButton,
};

fn handle_pagination(
    prev_query: Query<
        &Interaction,
        (Changed<Interaction>, With<PaginationPrevButton<FavoritesPage>>),
    >,
    next_query: Query<
        &Interaction,
        (Changed<Interaction>, With<PaginationNextButton<FavoritesPage>>),
    >,
    mut favorites_state: ResMut<FavoritesState>,
) {
    let current = favorites_state.current_page;
    let total = favorites_state.total_pages;

    // 返回 Some(true)=下一页, Some(false)=上一页, None=无操作
    if let Some(is_next) = check_pagination_interaction::<FavoritesPage>(
        &prev_query,
        &next_query,
        current,
        total,
    ) {
        if is_next {
            favorites_state.current_page += 1;
        } else {
            favorites_state.current_page -= 1;
        }
        // 触发数据加载...
    }
}
```

#### 步骤 4：更新分页显示

```rust
use bevy_ui_toolkit::{
    update_pagination_display_with_theme,
    PaginationPageText, PaginationPrevButton, PaginationNextButton,
    CurrentTheme,
};

fn refresh_pagination(
    mut page_text_query: Query<&mut Text, With<PaginationPageText<FavoritesPage>>>,
    mut prev_btn_query: Query<
        &mut BackgroundColor,
        (With<PaginationPrevButton<FavoritesPage>>, Without<PaginationNextButton<FavoritesPage>>),
    >,
    mut next_btn_query: Query<
        &mut BackgroundColor,
        (With<PaginationNextButton<FavoritesPage>>, Without<PaginationPrevButton<FavoritesPage>>),
    >,
    favorites_state: Res<FavoritesState>,
    theme: Res<CurrentTheme>,
) {
    update_pagination_display_with_theme::<FavoritesPage>(
        &mut page_text_query,
        &mut prev_btn_query,
        &mut next_btn_query,
        &theme,
        favorites_state.current_page,
        favorites_state.total_pages,
    );
}
```

---

### 4. Waterfall 瀑布流系统

预创建隐藏的 UI 元素，然后分批显示，避免一次性创建大量元素导致的布局卡顿。

#### 步骤 1：定义状态类型

```rust
use bevy_ui_toolkit::WaterfallState;

// 简单用法（无额外上下文）
pub struct CategoriesWaterfall;
pub type CategoriesCreationState = WaterfallState<CategoriesWaterfall>;

// 带上下文（如排行榜需要记录当前标签类型）
pub struct RankingsWaterfall;

#[derive(Default)]
pub struct RankingsContext {
    pub current_type: Option<RankingType>,
}

pub type RankingsCreationState = WaterfallState<RankingsWaterfall, RankingsContext>;
```

#### 步骤 2：注册资源

```rust
app.init_resource::<CategoriesCreationState>()
    .init_resource::<RankingsCreationState>();
```

#### 步骤 3：实现瀑布流系统

```rust
fn waterfall_create_cards(
    mut commands: Commands,
    mut creation_state: ResMut<CategoriesCreationState>,
    time: Res<Time>,
    data_state: Res<CategoriesState>,
    asset_server: Res<AssetServer>,
    scroll_container_query: Query<Entity, With<CategoriesScrollContainer>>,
    // ...
) {
    // 1. 检查是否需要启动预创建
    if !creation_state.is_creating && !data_state.categories.is_empty() {
        let font = asset_server.load("fonts/font.ttf");
        creation_state.start_precreate(data_state.categories.len(), font);
        return;
    }

    // 2. 预创建阶段：一次性创建所有隐藏卡片
    if creation_state.needs_precreate() {
        let scroll_container = scroll_container_query.single().unwrap();
        let mut entities = Vec::new();

        commands.entity(scroll_container).with_children(|parent| {
            for category in &data_state.categories {
                let entity = parent
                    .spawn((
                        CategoryCard { id: category.id.clone() },
                        Node { /* ... */ },
                        Visibility::Hidden,  // 初始隐藏
                    ))
                    .id();
                entities.push(entity);
            }
        });

        creation_state.set_precreated_entities(entities);
        return;
    }

    // 3. 显示阶段：分批显示卡片
    if creation_state.should_show_batch(time.delta()) {
        let batch = creation_state.take_batch();

        for entity in batch {
            if let Ok(mut visibility) = commands.get_entity(entity) {
                visibility.insert(Visibility::Visible);
            }
        }

        // 检查是否全部显示完成
        if !creation_state.has_pending() {
            creation_state.finish();
        }
    }
}
```

#### 配置参数

```rust
// 可以在启动前修改配置
creation_state.batch_size = 8;  // 每批显示数量，默认 4

// 默认值
pub const DEFAULT_BATCH_SIZE: usize = 4;
pub const DEFAULT_INTERVAL_MS: u64 = 64;  // 约 60fps
```

---

## 完整示例

完整示例请参考 [examples/](./examples/) 目录。

## 兼容性

- **Bevy**: 0.17.x
- **Rust**: nightly 2024+

## License

Apache-2.0
