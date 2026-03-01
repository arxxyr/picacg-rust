//! 搜索界面系统

use bevy::{
    input::keyboard::Key,
    prelude::*,
    window::{Ime, PrimaryWindow},
};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        downloads::ScrollContainer,
        login::AppColors,
        scrollbar::scrollbar_config::SCROLLBAR_WIDTH,
        ui_common::{calculate_scroll_delta, spawn_comic_time_info, spawn_scrollbar},
        waterfall::SearchCardCreationState,
    },
};

/// 排序方式定义
const SORT_OPTIONS: &[(&str, &str)] = &[
    ("dd", "新到旧"),
    ("da", "旧到新"),
    ("ld", "点赞最多"),
    ("vd", "浏览最多"),
];

/// 搜索布局常量
mod search_layout {
    /// 卡片宽度（预留）
    #[allow(dead_code)]
    pub const CARD_WIDTH: f32 = 180.0;
    /// 卡片高度（预留）
    #[allow(dead_code)]
    pub const CARD_HEIGHT: f32 = 300.0;
    /// 列间距
    pub const COLUMN_GAP: f32 = 15.0;
    /// 行间距
    pub const ROW_GAP: f32 = 15.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距（预留）
    #[allow(dead_code)]
    pub const PADDING_TOP: f32 = 20.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 40.0;
}

// ==================== 组件标记 ====================

/// 搜索页面根标记
#[derive(Component)]
pub struct SearchRoot;

/// 搜索输入框标记
#[derive(Component)]
pub struct SearchInputField {
    pub focused: bool,
}

/// 搜索输入框文本标记
#[derive(Component)]
pub struct SearchInputText;

/// 搜索按钮标记
#[derive(Component)]
pub struct SearchButton;

/// 搜索结果滚动容器标记
#[derive(Component)]
pub struct SearchScrollContainer;

/// 搜索结果网格标记
#[derive(Component)]
pub struct SearchResultsGrid;

/// 搜索结果卡片标记
#[derive(Component)]
pub struct SearchResultCard {
    pub comic_id: String,
}

/// 搜索结果图片标记
#[derive(Component)]
pub struct SearchResultImage {
    #[allow(dead_code)]
    pub comic_id: String,
    pub url: String,
}

/// 搜索页码文本标记
#[derive(Component)]
pub struct SearchPageNumberText;

/// 搜索上一页按钮
#[derive(Component)]
pub struct SearchPrevPageButton;

/// 搜索下一页按钮
#[derive(Component)]
pub struct SearchNextPageButton;

/// 搜索分页容器
#[derive(Component)]
pub struct SearchPaginationContainer;

/// 搜索加载提示标记
#[derive(Component)]
pub struct SearchLoadingText;

/// 搜索错误提示标记
#[derive(Component)]
pub struct SearchErrorText;

/// 搜索空结果提示标记
#[derive(Component)]
pub struct SearchEmptyText;

/// 排序按钮标记
#[derive(Component)]
pub struct SortButton {
    pub sort: String,
}

/// 分类过滤展开/折叠按钮
#[derive(Component)]
pub struct CategoryFilterToggle;

/// 分类过滤面板
#[derive(Component)]
pub struct CategoryFilterPanel;

/// 分类复选框标记
#[derive(Component)]
pub struct CategoryCheckbox {
    pub category: String,
}

/// 全选分类按钮
#[derive(Component)]
pub struct SelectAllCategoriesButton;

/// 清空分类按钮
#[derive(Component)]
pub struct ClearAllCategoriesButton;

// ==================== 系统函数 ====================

/// 搜索页面 UI 构建参数
struct SearchUiBuildParams<'a> {
    font: &'a Handle<Font>,
    search_state: &'a SearchState,
    input_focused: bool,
    /// 可用分类列表（从 CategoriesState 获取）
    available_categories: Vec<String>,
}

/// 构建搜索页面 UI（内部函数，供 setup 和 refresh 共用）
fn build_search_ui(commands: &mut Commands, params: &SearchUiBuildParams) -> Entity {
    let SearchUiBuildParams {
        font,
        search_state,
        input_focused,
        available_categories,
    } = params;

    let input_border_color = if *input_focused {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };

    commands
        .spawn((
            SearchRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
            Transform::default(),
        ))
        .with_children(|root| {
            // 搜索头部（输入框 + 按钮）
            spawn_search_header(root, font, search_state, *input_focused, input_border_color);

            // 过滤工具栏（排序 + 分类过滤）
            spawn_filter_toolbar(root, font, search_state, available_categories);

            // 滚动区域包装器
            spawn_scroll_area(root, font, search_state);

            // 分页控件
            spawn_pagination_controls(root, font, search_state);
        })
        .id()
}

/// 创建搜索头部（图标 + 输入框 + 按钮）
fn spawn_search_header(
    root: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    search_state: &SearchState,
    input_focused: bool,
    input_border_color: Color,
) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(15.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(AppColors::BORDER),
        Transform::default(),
    ))
    .with_children(|header| {
        // 搜索图标
        header.spawn((
            Text::new("\u{1F50D}"),
            TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
        ));

        // 搜索输入框容器
        let (display_text, text_color) = if search_state.keyword.is_empty() {
            (
                "输入关键词搜索漫画、作者、标签...".to_string(),
                AppColors::TEXT_SECONDARY,
            )
        } else {
            (search_state.keyword.clone(), AppColors::TEXT)
        };

        header
            .spawn((
                SearchInputField {
                    focused: input_focused,
                },
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(40.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(input_border_color),
                BackgroundColor(AppColors::CARD_BG),
                Transform::default(),
            ))
            .with_children(|input| {
                input.spawn((
                    SearchInputText,
                    Text::new(display_text),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(text_color),
                ));
            });

        // 搜索按钮
        header
            .spawn((
                SearchButton,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(AppColors::PRIMARY),
                Transform::default(),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("搜索"),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });
    });
}

/// 创建过滤工具栏（排序按钮组 + 分类过滤按钮）
fn spawn_filter_toolbar(
    root: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    search_state: &SearchState,
    available_categories: &[String],
) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(AppColors::BORDER),
        Transform::default(),
    ))
    .with_children(|toolbar| {
        // 排序按钮行
        toolbar
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::new(Val::Px(15.0), Val::Px(15.0), Val::Px(8.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                Transform::default(),
            ))
            .with_children(|row| {
                // 排序标签
                row.spawn((
                    Text::new("排序:"),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                // 排序按钮组
                for &(sort_key, sort_label) in SORT_OPTIONS {
                    let is_active = search_state.sort == sort_key;
                    let bg = if is_active {
                        AppColors::PRIMARY
                    } else {
                        Color::srgb(0.15, 0.15, 0.2)
                    };
                    let border = if is_active {
                        AppColors::PRIMARY
                    } else {
                        AppColors::BORDER
                    };

                    row.spawn((
                        SortButton {
                            sort: sort_key.to_string(),
                        },
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::new(
                                Val::Px(10.0),
                                Val::Px(10.0),
                                Val::Px(4.0),
                                Val::Px(4.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderColor::all(border),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(sort_label),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(if is_active {
                                AppColors::TEXT
                            } else {
                                AppColors::TEXT_SECONDARY
                            }),
                        ));
                    });
                }

                // 分隔符
                row.spawn((
                    Node {
                        width: Val::Px(1.0),
                        height: Val::Px(20.0),
                        margin: UiRect::horizontal(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(AppColors::BORDER),
                ));

                // 分类过滤按钮
                let filter_text = if search_state.selected_categories.is_empty() {
                    "分类过滤".to_string()
                } else {
                    format!("分类过滤 ({})", search_state.selected_categories.len())
                };
                row.spawn((
                    CategoryFilterToggle,
                    Button,
                    Interaction::default(),
                    Node {
                        padding: UiRect::new(
                            Val::Px(10.0),
                            Val::Px(10.0),
                            Val::Px(4.0),
                            Val::Px(4.0),
                        ),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(if search_state.show_category_filter {
                        AppColors::PRIMARY
                    } else {
                        Color::srgb(0.15, 0.15, 0.2)
                    }),
                    BorderColor::all(if search_state.show_category_filter {
                        AppColors::PRIMARY
                    } else {
                        AppColors::BORDER
                    }),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(filter_text),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(if search_state.show_category_filter {
                            AppColors::TEXT
                        } else {
                            AppColors::TEXT_SECONDARY
                        }),
                    ));
                    btn.spawn((
                        Text::new(if search_state.show_category_filter {
                            "▲" // chevron_up
                        } else {
                            "▼" // chevron_down
                        }),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });
            });

        // 分类过滤面板（可折叠）
        if search_state.show_category_filter {
            spawn_category_filter_panel(toolbar, font, search_state, available_categories);
        }
    });
}

/// 创建分类过滤面板
fn spawn_category_filter_panel(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    search_state: &SearchState,
    available_categories: &[String],
) {
    parent
        .spawn((
            CategoryFilterPanel,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(15.0), Val::Px(15.0), Val::Px(4.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.14, 0.5)),
            Transform::default(),
        ))
        .with_children(|panel| {
            // 分类复选框网格
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(6.0),
                        row_gap: Val::Px(6.0),
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|grid| {
                    for category_name in available_categories {
                        let is_checked = search_state.selected_categories.contains(category_name);
                        spawn_category_checkbox(grid, font, category_name, is_checked);
                    }
                });

            // 全选/清空按钮行
            panel
                .spawn((
                    Node {
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|btn_row| {
                    // 全选按钮
                    btn_row
                        .spawn((
                            SelectAllCategoriesButton,
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::new(
                                    Val::Px(8.0),
                                    Val::Px(8.0),
                                    Val::Px(3.0),
                                    Val::Px(3.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                            BorderColor::all(AppColors::BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("全选"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        });

                    // 清空按钮
                    btn_row
                        .spawn((
                            ClearAllCategoriesButton,
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::new(
                                    Val::Px(8.0),
                                    Val::Px(8.0),
                                    Val::Px(3.0),
                                    Val::Px(3.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                            BorderColor::all(AppColors::BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("清空"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        });

                    // 已选计数
                    let count_text = if search_state.selected_categories.is_empty() {
                        "未选择分类（搜索所有分类）".to_string()
                    } else {
                        format!("已选 {} 个分类", search_state.selected_categories.len())
                    };
                    btn_row.spawn((
                        Text::new(count_text),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });
        });
}

/// 创建单个分类复选框
fn spawn_category_checkbox(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    category: &str,
    checked: bool,
) {
    let bg = if checked {
        AppColors::PRIMARY
    } else {
        Color::srgb(0.15, 0.15, 0.2)
    };
    let border = if checked {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };

    parent
        .spawn((
            CategoryCheckbox {
                category: category.to_string(),
            },
            Button,
            Interaction::default(),
            Node {
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(3.0), Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(border),
        ))
        .with_children(|cb| {
            // 勾选图标
            cb.spawn((
                Text::new(if checked { "✓" } else { "" }),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    width: Val::Px(12.0),
                    ..default()
                },
            ));
            // 分类名称
            cb.spawn((
                Text::new(category),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(if checked {
                    AppColors::TEXT
                } else {
                    AppColors::TEXT_SECONDARY
                }),
            ));
        });
}

/// 创建滚动区域
fn spawn_scroll_area(
    root: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    search_state: &SearchState,
) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: Val::Px(0.0),
            min_height: Val::Px(0.0),
            position_type: PositionType::Relative,
            ..default()
        },
        Transform::default(),
    ))
    .with_children(|wrapper| {
        let scroll_container = wrapper
            .spawn((
                SearchScrollContainer,
                ScrollContainer,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                ContentSizeInfo::default(),
            ))
            .with_children(|scroll| {
                spawn_scroll_content(scroll, font, search_state);
            })
            .id();

        spawn_scrollbar(wrapper, scroll_container);
    });
}

/// 创建滚动内容（根据状态显示不同内容）
fn spawn_scroll_content(
    scroll: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    search_state: &SearchState,
) {
    if search_state.is_loading {
        scroll.spawn((
            SearchLoadingText,
            Text::new("正在搜索..."),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
    } else if let Some(error) = &search_state.error {
        scroll.spawn((
            SearchErrorText,
            Text::new(format!("搜索失败: {}", error)),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.3, 0.3)),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
    } else if search_state.has_searched && search_state.results.is_empty() {
        scroll.spawn((
            SearchEmptyText,
            Text::new(format!("未找到与 \"{}\" 相关的漫画", search_state.keyword)),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
    } else if !search_state.has_searched {
        scroll.spawn((
            Text::new("输入关键词开始搜索"),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
    } else {
        // 搜索结果
        scroll.spawn((
            Text::new(format!(
                "共找到 {} 页结果（第 {} 页）",
                search_state.total_pages, search_state.page
            )),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
            Node {
                margin: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(15.0), Val::Px(10.0)),
                ..default()
            },
        ));

        scroll.spawn((
            SearchResultsGrid,
            Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect {
                    left: Val::Px(search_layout::PADDING_LEFT),
                    right: Val::Px(search_layout::PADDING_RIGHT),
                    top: Val::Px(0.0),
                    bottom: Val::Px(search_layout::PADDING_BOTTOM),
                },
                column_gap: Val::Px(search_layout::COLUMN_GAP),
                row_gap: Val::Px(search_layout::ROW_GAP),
                ..default()
            },
        ));
    }
}

/// 创建分页控件
fn spawn_pagination_controls(
    root: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    search_state: &SearchState,
) {
    root.spawn((
        SearchPaginationContainer,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(20.0),
            border: UiRect::top(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(AppColors::BORDER),
        BackgroundColor(AppColors::SURFACE),
        Transform::default(),
    ))
    .with_children(|pagination| {
        // 上一页按钮
        let prev_color = if search_state.page > 1 {
            AppColors::PRIMARY
        } else {
            AppColors::SECONDARY
        };
        pagination
            .spawn((
                SearchPrevPageButton,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(prev_color),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("上一页"),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

        // 页码
        pagination.spawn((
            SearchPageNumberText,
            Text::new(format!(
                "{} / {}",
                search_state.page,
                search_state.total_pages.max(1)
            )),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(AppColors::TEXT),
        ));

        // 下一页按钮
        let next_color = if search_state.page < search_state.total_pages {
            AppColors::PRIMARY
        } else {
            AppColors::SECONDARY
        };
        pagination
            .spawn((
                SearchNextPageButton,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(next_color),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("下一页"),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });
    });
}

/// 创建搜索界面
pub fn setup_search_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    search_state: Res<SearchState>,
    categories_state: Res<CategoriesState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<SearchCardCreationState>,
) {
    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    creation_state.clear();

    let available_categories: Vec<String> = categories_state
        .categories
        .iter()
        .map(|c| c.title.clone())
        .collect();

    let params = SearchUiBuildParams {
        font: &font,
        search_state: &search_state,
        input_focused: false,
        available_categories,
    };
    let search_root = build_search_ui(&mut commands, &params);

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(search_root);
    }

    if search_state.has_searched && !search_state.results.is_empty() && search_state.error.is_none()
    {
        creation_state.start_precreate(search_state.results.len(), font);
    }
}

/// 创建搜索结果卡片
fn spawn_search_result_card(
    parent: &mut ChildSpawnerCommands,
    comic: &picacg_api::models::Comic,
    font: &Handle<Font>,
    image_cache: &ImageCache,
    hidden: bool,
) -> Entity {
    parent
        .spawn((
            SearchResultCard {
                comic_id: comic.id.clone(),
            },
            Button,
            Node {
                width: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(AppColors::CARD_BG),
            if hidden {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
        ))
        .with_children(|card| {
            // 封面图片容器
            card.spawn((
                Node {
                    width: Val::Px(164.0),
                    height: Val::Px(220.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(AppColors::SECONDARY),
                Transform::default(),
            ))
            .with_children(|img_container| {
                let cover_url = comic.thumb.url();
                if let Some(handle) = image_cache.get(&cover_url) {
                    img_container.spawn((
                        SearchResultImage {
                            comic_id: comic.id.clone(),
                            url: cover_url,
                        },
                        ImageNode::new(handle.clone()),
                        Node {
                            width: Val::Px(164.0),
                            height: Val::Px(220.0),
                            ..default()
                        },
                    ));
                } else {
                    img_container.spawn((
                        SearchResultImage {
                            comic_id: comic.id.clone(),
                            url: cover_url,
                        },
                        Node {
                            width: Val::Px(164.0),
                            height: Val::Px(220.0),
                            ..default()
                        },
                        BackgroundColor(AppColors::SECONDARY),
                    ));
                }
            });

            // 标题
            card.spawn((
                Text::new(&comic.title),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
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
                    font_size: 11.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(4.0), Val::Px(4.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ));

            // 分类标签容器
            if !comic.categories.is_empty() {
                card.spawn((
                    Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(2.0),
                        max_width: Val::Px(164.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|tags_container| {
                    // 最多显示 3 个分类
                    for category in comic.categories.iter().take(3) {
                        tags_container
                            .spawn((
                                Node {
                                    padding: UiRect::new(
                                        Val::Px(4.0),
                                        Val::Px(4.0),
                                        Val::Px(1.0),
                                        Val::Px(1.0),
                                    ),
                                    border_radius: BorderRadius::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.4, 0.8, 0.3)),
                            ))
                            .with_children(|badge| {
                                badge.spawn((
                                    Text::new(category),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 10.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.6, 0.8, 1.0)),
                                ));
                            });
                    }
                });
            }

            // 标签容器
            if !comic.tags.is_empty() {
                card.spawn((
                    Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(2.0),
                        max_width: Val::Px(164.0),
                        margin: UiRect::top(Val::Px(2.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|tags_container| {
                    // 最多显示 3 个标签
                    for tag in comic.tags.iter().take(3) {
                        tags_container
                            .spawn((
                                Node {
                                    padding: UiRect::new(
                                        Val::Px(4.0),
                                        Val::Px(4.0),
                                        Val::Px(1.0),
                                        Val::Px(1.0),
                                    ),
                                    border_radius: BorderRadius::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.6, 0.3, 0.6, 0.3)),
                            ))
                            .with_children(|badge| {
                                badge.spawn((
                                    Text::new(tag),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 10.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.7, 0.9)),
                                ));
                            });
                    }
                });
            }

            // 创建/更新时间
            spawn_comic_time_info(
                card,
                font,
                comic.created_at.as_deref(),
                comic.updated_at.as_deref(),
            );
        })
        .id()
}

/// 清理搜索界面
pub fn cleanup_search_ui(
    mut commands: Commands,
    query: Query<Entity, With<SearchRoot>>,
    mut creation_state: ResMut<SearchCardCreationState>,
) {
    // 清空瀑布式创建状态（防止对已销毁的 Entity 操作）
    creation_state.clear();

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 搜索输入框交互
pub fn search_input_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut SearchInputField,
            &GlobalTransform,
            &ComputedNode,
        ),
        Changed<Interaction>,
    >,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for (interaction, mut bg_color, mut border_color, mut input, _transform, computed) in
        interaction_query.iter_mut()
    {
        match *interaction {
            Interaction::Pressed => {
                input.focused = true;
                tracing::info!("搜索输入框获得焦点");
                *border_color = BorderColor::all(AppColors::PRIMARY);
                *bg_color = BackgroundColor(AppColors::CARD_BG);

                // 启用 IME 并设置位置
                if let Ok(mut window) = window_query.single_mut() {
                    window.ime_enabled = true;

                    // 使用当前鼠标位置设置 IME 候选框位置
                    if let Some(cursor_pos) = window.cursor_position() {
                        let scale_factor = window.scale_factor();
                        let input_height = computed.size().y / scale_factor;
                        // IME 候选框显示在点击位置下方
                        let ime_x = cursor_pos.x;
                        let ime_y = cursor_pos.y + input_height / 2.0 + 5.0;
                        window.ime_position = bevy::math::Vec2::new(ime_x, ime_y);
                        tracing::info!("启用 IME，位置: ({:.0}, {:.0})", ime_x, ime_y);
                    } else {
                        tracing::info!("启用 IME");
                    }
                }
            }
            Interaction::Hovered => {
                if !input.focused {
                    *border_color = BorderColor::all(AppColors::PRIMARY.with_alpha(0.5));
                }
            }
            Interaction::None => {
                if !input.focused {
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 处理键盘输入
pub fn handle_search_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut input_query: Query<&mut SearchInputField>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<SearchInputText>>,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    // 检查是否有聚焦的输入框
    let has_focus = input_query.iter().any(|input| input.focused);

    // 检查修饰键状态
    let ctrl_pressed =
        key_input.pressed(KeyCode::ControlLeft) || key_input.pressed(KeyCode::ControlRight);

    for event in keyboard_events.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }

        if !has_focus {
            continue;
        }

        match &event.logical_key {
            Key::Backspace => {
                search_state.keyword.pop();
                update_input_text(&search_state.keyword, &mut text_query);
            }
            Key::Enter if !search_state.keyword.is_empty() => {
                search_state.is_loading = true;
                search_state.needs_rebuild = true;
                search_state.page = 1;
                search_messages.write(SearchComicsRequestEvent {
                    keyword: search_state.keyword.clone(),
                    page: 1,
                    sort: search_state.sort.clone(),
                    categories: search_state.selected_categories.clone(),
                });
                // 取消输入框焦点
                for mut input in input_query.iter_mut() {
                    input.focused = false;
                }
            }
            Key::Escape => {
                for mut input in input_query.iter_mut() {
                    input.focused = false;
                }
            }
            Key::Character(input) => {
                // 处理 Ctrl 组合键
                if ctrl_pressed {
                    match input.as_str() {
                        "v" | "V" => {
                            // Ctrl+V 粘贴
                            if let Ok(mut clipboard) = arboard::Clipboard::new()
                                && let Ok(text) = clipboard.get_text()
                            {
                                // 过滤控制字符，只保留可打印字符
                                let filtered: String =
                                    text.chars().filter(|c| !c.is_control()).collect();
                                search_state.keyword.push_str(&filtered);
                                update_input_text(&search_state.keyword, &mut text_query);
                                tracing::info!("粘贴内容: {:?}", filtered);
                            }
                        }
                        "a" | "A" => {
                            // Ctrl+A 全选（这里实现为清空，因为没有选择状态）
                            // 实际上什么都不做，防止 'a' 被输入
                        }
                        "c" | "C" => {
                            // Ctrl+C 复制当前内容到剪贴板
                            if !search_state.keyword.is_empty()
                                && let Ok(mut clipboard) = arboard::Clipboard::new()
                            {
                                let _ = clipboard.set_text(&search_state.keyword);
                                tracing::info!("复制内容: {:?}", search_state.keyword);
                            }
                        }
                        "x" | "X" if !search_state.keyword.is_empty() => {
                            // Ctrl+X 剪切（复制并清空）
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                let _ = clipboard.set_text(&search_state.keyword);
                                tracing::info!("剪切内容: {:?}", search_state.keyword);
                            }
                            search_state.keyword.clear();
                            update_input_text(&search_state.keyword, &mut text_query);
                        }
                        _ => {}
                    }
                } else {
                    // 普通字符输入
                    for c in input.chars() {
                        if !c.is_control() {
                            search_state.keyword.push(c);
                        }
                    }
                    update_input_text(&search_state.keyword, &mut text_query);
                }
            }
            _ => {}
        }
    }
}

/// 更新输入框文本
fn update_input_text(
    keyword: &str,
    text_query: &mut Query<(&mut Text, &mut TextColor), With<SearchInputText>>,
) {
    for (mut text, mut color) in text_query.iter_mut() {
        if keyword.is_empty() {
            **text = "输入关键词搜索漫画、作者、标签...".to_string();
            *color = TextColor(AppColors::TEXT_SECONDARY);
        } else {
            **text = keyword.to_string();
            *color = TextColor(AppColors::TEXT);
        }
    }
}

/// 处理 IME 输入（中文输入法）
pub fn handle_search_ime_input(
    mut ime_events: MessageReader<Ime>,
    input_query: Query<&SearchInputField>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<SearchInputText>>,
    mut search_state: ResMut<SearchState>,
) {
    // 检查是否有聚焦的输入框
    let has_focus = input_query.iter().any(|input| input.focused);

    if !has_focus {
        return;
    }

    for event in ime_events.read() {
        match event {
            Ime::Commit { value, .. } => {
                // IME 提交完成的文本（用户按下空格或回车确认输入）
                tracing::info!("IME 提交: {:?}", value);
                search_state.keyword.push_str(value);
                update_input_text(&search_state.keyword, &mut text_query);
            }
            Ime::Preedit { value, cursor, .. } => {
                // IME 预览文本（输入过程中）
                // 这里可以显示输入法的候选文字预览
                if !value.is_empty() {
                    tracing::debug!("IME 预览: {:?}, cursor: {:?}", value, cursor);
                }
            }
            Ime::Enabled { .. } => {
                tracing::debug!("IME 已启用");
            }
            Ime::Disabled { .. } => {
                tracing::debug!("IME 已禁用");
            }
        }
    }
}

/// 搜索按钮交互
pub fn search_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SearchButton>),
    >,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
    mut search_state: ResMut<SearchState>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.8));
                if !search_state.keyword.is_empty() && !search_state.is_loading {
                    search_state.is_loading = true;
                    search_state.needs_rebuild = true;
                    search_state.page = 1;
                    search_messages.write(SearchComicsRequestEvent {
                        keyword: search_state.keyword.clone(),
                        page: 1,
                        sort: search_state.sort.clone(),
                        categories: search_state.selected_categories.clone(),
                    });
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.9));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
        }
    }
}

/// 搜索结果卡片交互
pub fn search_result_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &SearchResultCard),
        Changed<Interaction>,
    >,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, mut bg_color, card) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::CARD_BG.with_alpha(0.6));
                detail_messages.write(NavigateToComicDetailEvent {
                    comic_id: card.comic_id.clone(),
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::CARD_BG.with_alpha(0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::CARD_BG);
            }
        }
    }
}

/// 搜索分页按钮交互
pub fn search_pagination_interaction(
    mut prev_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<SearchPrevPageButton>,
            Without<SearchNextPageButton>,
        ),
    >,
    mut next_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<SearchNextPageButton>,
            Without<SearchPrevPageButton>,
        ),
    >,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    // 上一页
    for (interaction, mut bg_color) in prev_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if search_state.page > 1 && !search_state.is_loading {
                    search_state.page -= 1;
                    search_state.is_loading = true;
                    search_state.needs_rebuild = true;
                    search_messages.write(SearchComicsRequestEvent {
                        keyword: search_state.keyword.clone(),
                        page: search_state.page,
                        sort: search_state.sort.clone(),
                        categories: search_state.selected_categories.clone(),
                    });
                }
                *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.8));
            }
            Interaction::Hovered => {
                if search_state.page > 1 {
                    *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.9));
                }
            }
            Interaction::None => {
                *bg_color = BackgroundColor(if search_state.page > 1 {
                    AppColors::PRIMARY
                } else {
                    AppColors::SECONDARY
                });
            }
        }
    }

    // 下一页
    for (interaction, mut bg_color) in next_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if search_state.page < search_state.total_pages && !search_state.is_loading {
                    search_state.page += 1;
                    search_state.is_loading = true;
                    search_state.needs_rebuild = true;
                    search_messages.write(SearchComicsRequestEvent {
                        keyword: search_state.keyword.clone(),
                        page: search_state.page,
                        sort: search_state.sort.clone(),
                        categories: search_state.selected_categories.clone(),
                    });
                }
                *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.8));
            }
            Interaction::Hovered => {
                if search_state.page < search_state.total_pages {
                    *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.9));
                }
            }
            Interaction::None => {
                *bg_color = BackgroundColor(if search_state.page < search_state.total_pages {
                    AppColors::PRIMARY
                } else {
                    AppColors::SECONDARY
                });
            }
        }
    }
}

/// 处理搜索滚动
pub fn handle_search_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<SearchScrollContainer>,
    >,
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        let scroll_delta = calculate_scroll_delta(event);

        for (mut scroll_pos, content_info) in scroll_query.iter_mut() {
            let max_scroll = content_info
                .map(|info| (info.content_height - info.viewport_height).max(0.0))
                .unwrap_or(0.0);
            scroll_pos.y = (scroll_pos.y - scroll_delta).clamp(0.0, max_scroll);
        }
    }
}

/// 更新搜索内容尺寸
pub fn update_search_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<SearchScrollContainer>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        let mut content_height = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(child) {
                content_height += child_computed.size().y / scale_factor;
            }
        }

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 更新搜索结果图片
pub fn update_search_images(
    mut commands: Commands,
    image_query: Query<(Entity, &SearchResultImage), Without<ImageNode>>,
    image_cache: Res<ImageCache>,
) {
    // 注意：不使用 is_changed() 检查，因为系统执行顺序可能导致检测失败
    // Query 使用 Without<ImageNode> 过滤已设置图片的实体，性能影响不大

    for (entity, img) in image_query.iter() {
        if let Some(handle) = image_cache.get(&img.url) {
            commands
                .entity(entity)
                .insert(ImageNode::new(handle.clone()));
        }
    }
}

/// 刷新搜索 UI（响应状态变化）
pub fn refresh_search_ui(
    mut commands: Commands,
    mut search_state: ResMut<SearchState>,
    categories_state: Res<CategoriesState>,
    search_root_query: Query<Entity, With<SearchRoot>>,
    _asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
    input_query: Query<&SearchInputField>,
    mut creation_state: ResMut<SearchCardCreationState>,
) {
    if !search_state.is_changed() || !search_state.needs_rebuild {
        return;
    }

    // 重建前重置标志
    search_state.needs_rebuild = false;

    // 保存输入框焦点状态
    let was_focused = input_query.iter().any(|input| input.focused);

    // 移除旧的 UI
    for entity in search_root_query.iter() {
        commands.entity(entity).despawn();
    }

    creation_state.clear();

    let font: Handle<Font> = get_font();
    let Some(content_entity) = content_area_query.single().ok() else {
        return;
    };

    let available_categories: Vec<String> = categories_state
        .categories
        .iter()
        .map(|c| c.title.clone())
        .collect();

    let params = SearchUiBuildParams {
        font: &font,
        search_state: &search_state,
        input_focused: was_focused,
        available_categories,
    };
    let search_root = build_search_ui(&mut commands, &params);

    commands.entity(content_entity).add_child(search_root);

    if search_state.has_searched && !search_state.results.is_empty() && search_state.error.is_none()
    {
        creation_state.start_precreate(search_state.results.len(), font);
    }
}

/// 瀑布式显示搜索结果卡片（预创建所有隐藏卡片，然后分批显示）
pub fn waterfall_create_search_cards(
    mut commands: Commands,
    mut creation_state: ResMut<SearchCardCreationState>,
    search_state: Res<SearchState>,
    image_cache: Res<ImageCache>,
    results_grid_query: Query<Entity, With<SearchResultsGrid>>,
    time: Res<Time>,
) {
    // 检查是否需要预创建
    if creation_state.needs_precreate() {
        let Ok(grid_entity) = results_grid_query.single() else {
            return;
        };

        let Some(font) = creation_state.font_handle.clone() else {
            return;
        };

        let results = &search_state.results;
        let count = creation_state.get_precreate_count();

        if results.is_empty() || count == 0 {
            creation_state.clear();
            return;
        }

        // 一次性创建所有隐藏卡片
        let mut entities = Vec::with_capacity(count);
        commands.entity(grid_entity).with_children(|parent| {
            for i in 0..count {
                if let Some(comic) = results.get(i) {
                    let entity = spawn_search_result_card(parent, comic, &font, &image_cache, true);
                    entities.push(entity);
                }
            }
        });

        // 设置预创建完成后的实体列表
        creation_state.set_precreated_entities(entities);
        tracing::debug!("搜索结果卡片预创建完成: {} 个", count);
        return;
    }

    // 检查是否应该显示下一批
    if !creation_state.should_show_batch(time.delta()) {
        return;
    }

    // 获取这一批要显示的实体
    let batch = creation_state.take_batch();
    if batch.is_empty() {
        return;
    }

    // 显示这一批卡片（设置 Visibility::Inherited）
    for entity in batch {
        // 安全检查：实体可能在清理时已被销毁
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(Visibility::Inherited);
        }
    }

    // 日志（仅在显示完成时输出）
    if !creation_state.has_pending() {
        creation_state.finish();
        tracing::debug!(
            "搜索结果卡片瀑布式显示完成: {} 个",
            search_state.results.len()
        );
    }
}

/// 点击其他区域取消输入框焦点
pub fn unfocus_search_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut input_query: Query<(&Interaction, &mut SearchInputField, &mut BorderColor)>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (interaction, mut input, mut border) in input_query.iter_mut() {
            if *interaction == Interaction::None && input.focused {
                input.focused = false;
                *border = BorderColor::all(AppColors::BORDER);

                // 禁用 IME
                if let Ok(mut window) = window_query.single_mut() {
                    window.ime_enabled = false;
                    tracing::info!("输入框失去焦点，禁用 IME");
                }
            }
        }
    }
}

// ==================== 过滤工具栏交互系统 ====================

/// 排序按钮交互
pub fn sort_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &SortButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    let mut new_sort: Option<String> = None;
    for (interaction, btn, _, _) in interaction_query.iter() {
        if *interaction == Interaction::Pressed && search_state.sort != btn.sort {
            new_sort = Some(btn.sort.clone());
            break;
        }
    }

    if let Some(sort) = new_sort {
        tracing::info!("切换排序: {} -> {}", search_state.sort, sort);
        search_state.sort = sort;
        search_state.needs_rebuild = true;

        // 如果已经搜索过，自动重新搜索
        if search_state.has_searched && !search_state.keyword.is_empty() {
            search_state.page = 1;
            search_state.is_loading = true;
            search_messages.write(SearchComicsRequestEvent {
                keyword: search_state.keyword.clone(),
                page: 1,
                sort: search_state.sort.clone(),
                categories: search_state.selected_categories.clone(),
            });
        }
    }

    // 更新按钮外观
    for (interaction, btn, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let is_active = search_state.sort == btn.sort;
        if is_active {
            *bg_color = BackgroundColor(AppColors::PRIMARY);
            *border_color = BorderColor::all(AppColors::PRIMARY);
        } else {
            match *interaction {
                Interaction::Hovered => {
                    *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.24));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
                _ => {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 分类过滤面板展开/折叠交互
pub fn category_filter_toggle_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CategoryFilterToggle>),
    >,
    mut search_state: ResMut<SearchState>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                search_state.show_category_filter = !search_state.show_category_filter;
                search_state.needs_rebuild = true;
                tracing::debug!(
                    "分类过滤面板: {}",
                    if search_state.show_category_filter {
                        "展开"
                    } else {
                        "折叠"
                    }
                );
            }
            Interaction::Hovered => {
                if !search_state.show_category_filter {
                    *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.24));
                }
            }
            Interaction::None => {
                if !search_state.show_category_filter {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
        }
    }
}

/// 分类复选框交互
pub fn category_checkbox_interaction(
    interaction_query: Query<(&Interaction, &CategoryCheckbox), Changed<Interaction>>,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    let mut toggled_category: Option<String> = None;
    for (interaction, cb) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            toggled_category = Some(cb.category.clone());
            break;
        }
    }

    if let Some(category) = toggled_category {
        if let Some(pos) = search_state
            .selected_categories
            .iter()
            .position(|c| c == &category)
        {
            search_state.selected_categories.remove(pos);
        } else {
            search_state.selected_categories.push(category);
        }
        search_state.needs_rebuild = true;
        tracing::debug!("选中分类: {:?}", search_state.selected_categories);

        // 如果已经搜索过，自动重新搜索
        if search_state.has_searched && !search_state.keyword.is_empty() {
            search_state.page = 1;
            search_state.is_loading = true;
            search_messages.write(SearchComicsRequestEvent {
                keyword: search_state.keyword.clone(),
                page: 1,
                sort: search_state.sort.clone(),
                categories: search_state.selected_categories.clone(),
            });
        }
    }
}

/// 全选分类按钮交互
pub fn select_all_categories_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SelectAllCategoriesButton>),
    >,
    categories_state: Res<CategoriesState>,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));
                let all: Vec<String> = categories_state
                    .categories
                    .iter()
                    .map(|c| c.title.clone())
                    .collect();
                search_state.selected_categories = all;
                search_state.needs_rebuild = true;

                // 如果已搜索过，自动重新搜索
                if search_state.has_searched && !search_state.keyword.is_empty() {
                    search_state.page = 1;
                    search_state.is_loading = true;
                    search_messages.write(SearchComicsRequestEvent {
                        keyword: search_state.keyword.clone(),
                        page: 1,
                        sort: search_state.sort.clone(),
                        categories: search_state.selected_categories.clone(),
                    });
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.24));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}

/// 清空分类按钮交互
pub fn clear_all_categories_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ClearAllCategoriesButton>),
    >,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));
                if !search_state.selected_categories.is_empty() {
                    search_state.selected_categories.clear();
                    search_state.needs_rebuild = true;

                    // 如果已搜索过，自动重新搜索
                    if search_state.has_searched && !search_state.keyword.is_empty() {
                        search_state.page = 1;
                        search_state.is_loading = true;
                        search_messages.write(SearchComicsRequestEvent {
                            keyword: search_state.keyword.clone(),
                            page: 1,
                            sort: search_state.sort.clone(),
                            categories: search_state.selected_categories.clone(),
                        });
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.24));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}
