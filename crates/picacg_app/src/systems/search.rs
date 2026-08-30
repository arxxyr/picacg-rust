//! 搜索界面系统

use bevy::{
    input::keyboard::Key,
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    ui::RelativeCursorPosition,
};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        pagination::{Pagination, PaginationControl, pagination_controls},
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        ui_common::{
            BadgeAnchor, LoadingShimmer, TagColor, comic_time_info, download_status_badge,
            tag_badge,
        },
        waterfall::SearchCardCreationState,
        widgets::ButtonStyle,
    },
    utils::{
        content_filter::CompiledFilter,
        icons::*,
        text_input::{TextInput, TextInputDisplay},
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
#[derive(Component, Default, Clone)]
pub struct SearchRoot;

/// 搜索输入框标记（配合 TextInput 使用）
#[derive(Component, Default, Clone)]
pub struct SearchInputField;

/// 搜索按钮标记
#[derive(Component, Default, Clone)]
pub struct SearchButton;

/// 重置搜索按钮标记
#[derive(Component, Default, Clone)]
pub struct SearchResetButton;

/// 搜索结果网格标记
#[derive(Component, Default, Clone)]
pub struct SearchResultsGrid;

/// 搜索结果卡片标记
#[derive(Component, Default, Clone)]
pub struct SearchResultCard {
    pub comic_id: String,
}

/// 搜索结果图片标记
#[derive(Component, Default, Clone)]
pub struct SearchResultImage {
    #[allow(dead_code)]
    pub comic_id: String,
    pub url: String,
}

/// 搜索页面标记类型（用于分页组件的泛型参数）
pub struct SearchPage;

/// 搜索加载提示标记
#[derive(Component, Default, Clone)]
pub struct SearchLoadingText;

/// 搜索错误提示标记
#[derive(Component, Default, Clone)]
pub struct SearchErrorText;

/// 搜索空结果提示标记
#[derive(Component, Default, Clone)]
pub struct SearchEmptyText;

/// 排序按钮标记
#[derive(Component, Default, Clone)]
pub struct SortButton {
    pub sort: String,
}

/// 分类过滤展开/折叠按钮
#[derive(Component, Default, Clone)]
pub struct CategoryFilterToggle;

/// 分类过滤面板
#[derive(Component, Default, Clone)]
pub struct CategoryFilterPanel;

/// 分类复选框标记
#[derive(Component, Default, Clone)]
pub struct CategoryCheckbox {
    pub category: String,
}

/// 全选分类按钮
#[derive(Component, Default, Clone)]
pub struct SelectAllCategoriesButton;

/// 清空分类按钮
#[derive(Component, Default, Clone)]
pub struct ClearAllCategoriesButton;

/// 热词标签按钮标记
#[derive(Component, Default, Clone)]
pub struct HotKeywordTag {
    pub keyword: String,
}

/// 热词容器标记
#[derive(Component, Default, Clone)]
pub struct HotKeywordsContainer;

// ==================== 系统函数 ====================

/// 搜索页面场景（供 setup 和 refresh 共用）
///
/// `input_value` 是输入框应显示的文本：重建时取旧输入框里正在编辑的内容，
/// 建页时取状态里的已提交关键词。
fn search_page(
    search_state: &SearchState,
    input_value: &str,
    available_categories: &[String],
) -> impl Scene + use<> {
    let current_page = search_state.page.max(1) as u32;
    let total_pages = search_state.total_pages.max(1) as u32;

    bsn! {
        SearchRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            // 搜索头部（输入框 + 按钮）
            search_header(input_value),
            // 过滤工具栏（排序 + 分类过滤）
            filter_toolbar(search_state, available_categories),
            // 滚动区域包装器
            scroll_area(search_state),
            // 分页控件（使用通用分页组件）
            pagination_controls::<SearchPage>(current_page, total_pages),
        ]
    }
}

/// 搜索头部场景（图标 + 输入框 + 按钮）
///
/// 边框颜色不在这里区分焦点：聚焦/失焦一律由 `text_input_focus_visuals` 按
/// `InputFocus` 统一刷新，场景只给失焦态初值。
fn search_header(input_value: &str) -> impl Scene + use<> {
    // 搜索输入框容器
    let (display_text, text_color) = if input_value.is_empty() {
        (
            "输入关键词搜索漫画、作者、标签...".to_string(),
            AppColors::TEXT_SECONDARY,
        )
    } else {
        (input_value.to_string(), AppColors::TEXT)
    };

    let text_input = TextInput::new("输入关键词搜索漫画、作者、标签...").with_value(input_value);

    bsn! {
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(15.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            border: UiRect::bottom(Val::Px(1.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 搜索图标
                Text("\u{1F50D}")
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 搜索输入框
                SearchInputField
                template_value(text_input)
                Button
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(40.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                BackgroundColor(AppColors::CARD_BG)
                RelativeCursorPosition
                Children [
                    (
                        TextInputDisplay
                        Text({display_text})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(text_color)
                    )
                ]
            ),
            (
                // 搜索按钮
                SearchButton
                Button
                template_value(ButtonStyle::primary())
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(AppColors::PRIMARY)
                Children [
                    (
                        Text("搜索")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
            (
                // 重置按钮
                SearchResetButton
                Button
                template_value(ButtonStyle::ghost())
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(Color::NONE)
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        Text("重置")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    )
                ]
            ),
        ]
    }
}

/// 过滤工具栏场景（排序按钮组 + 分类过滤按钮）
fn filter_toolbar(
    search_state: &SearchState,
    available_categories: &[String],
) -> impl Scene + use<> {
    // 排序按钮组
    let sort_buttons: Vec<_> = SORT_OPTIONS
        .iter()
        .map(|&(sort_key, sort_label)| {
            sort_button(sort_key, sort_label, search_state.sort == sort_key)
        })
        .collect();

    // 分类过滤按钮
    let filter_text = if search_state.selected_categories.is_empty() {
        "分类过滤".to_string()
    } else {
        format!("分类过滤 ({})", search_state.selected_categories.len())
    };
    let show_filter = search_state.show_category_filter;
    let toggle_bg = if show_filter {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let toggle_border = if show_filter {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };
    let toggle_text_color = if show_filter {
        AppColors::TEXT
    } else {
        AppColors::TEXT_SECONDARY
    };
    let toggle_icon = if show_filter {
        ICON_CHEVRON_UP
    } else {
        ICON_CHEVRON_DOWN
    };

    // 分类过滤面板（可折叠）
    let filter_panel: Box<dyn SceneList> = if show_filter {
        Box::new(bsn_list![category_filter_panel(
            search_state,
            available_categories
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            border: UiRect::bottom(Val::Px(1.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 排序按钮行
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::new(Val::Px(15.0), Val::Px(15.0), Val::Px(8.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(6.0),
                }
                Children [
                    (
                        // 排序标签
                        Text("排序:")
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    {sort_buttons},
                    (
                        // 分隔符
                        Node {
                            width: Val::Px(1.0),
                            height: Val::Px(20.0),
                            margin: UiRect::horizontal(Val::Px(6.0)),
                        }
                        BackgroundColor(AppColors::BORDER)
                    ),
                    (
                        // 分类过滤按钮
                        CategoryFilterToggle
                        Button
                        template_value(ButtonStyle::segment(show_filter))
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
                        }
                        BackgroundColor(toggle_bg)
                        template_value(BorderColor::all(toggle_border))
                        Children [
                            (
                                Text({filter_text})
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(toggle_text_color)
                            ),
                            (
                                Text({toggle_icon})
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                        ]
                    ),
                ]
            ),
            {filter_panel},
        ]
    }
}

/// 单个排序按钮场景
fn sort_button(sort_key: &str, sort_label: &str, is_active: bool) -> impl Scene + use<> {
    let sort = sort_key.to_string();
    let label = sort_label.to_string();
    let bg = if is_active {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let border = if is_active {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };
    let text_color = if is_active {
        AppColors::TEXT
    } else {
        AppColors::TEXT_SECONDARY
    };

    bsn! {
        SortButton { sort: {sort} }
        Button
        template_value(ButtonStyle::segment(is_active))
        Node {
            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(4.0), Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(bg)
        template_value(BorderColor::all(border))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(text_color)
            )
        ]
    }
}

/// 分类过滤面板场景
fn category_filter_panel(
    search_state: &SearchState,
    available_categories: &[String],
) -> impl Scene + use<> {
    // 分类复选框
    let checkboxes: Vec<_> = available_categories
        .iter()
        .map(|category_name| {
            category_checkbox(
                category_name,
                search_state.selected_categories.contains(category_name),
            )
        })
        .collect();

    // 已选计数
    let count_text = if search_state.selected_categories.is_empty() {
        "未选择分类（搜索所有分类）".to_string()
    } else {
        format!("已选 {} 个分类", search_state.selected_categories.len())
    };

    bsn! {
        CategoryFilterPanel
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::new(Val::Px(15.0), Val::Px(15.0), Val::Px(4.0), Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        BackgroundColor(Color::srgba(0.1, 0.1, 0.14, 0.5))
        Children [
            (
                // 分类复选框网格
                Node {
                    width: Val::Percent(100.0),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                }
                Children [ {checkboxes} ]
            ),
            (
                // 全选/清空按钮行
                Node {
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        // 全选按钮
                        SelectAllCategoriesButton
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            padding: UiRect::new(
                                Val::Px(8.0),
                                Val::Px(8.0),
                                Val::Px(3.0),
                                Val::Px(3.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text("全选")
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    (
                        // 清空按钮
                        ClearAllCategoriesButton
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            padding: UiRect::new(
                                Val::Px(8.0),
                                Val::Px(8.0),
                                Val::Px(3.0),
                                Val::Px(3.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text("清空")
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    (
                        // 已选计数
                        Text({count_text})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
        ]
    }
}

/// 单个分类复选框场景
fn category_checkbox(category: &str, checked: bool) -> impl Scene + use<> {
    let category_name = category.to_string();
    let label = category.to_string();
    let bg = if checked {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let border = if checked {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };
    let icon = if checked { ICON_CHECK } else { "" };
    let label_color = if checked {
        AppColors::TEXT
    } else {
        AppColors::TEXT_SECONDARY
    };

    bsn! {
        CategoryCheckbox { category: {category_name} }
        Button
        template_value(ButtonStyle::segment(checked))
        Node {
            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(3.0), Val::Px(3.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
        }
        BackgroundColor(bg)
        template_value(BorderColor::all(border))
        Children [
            (
                // 勾选图标
                Text({icon})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(12.0) }
            ),
            (
                // 分类名称
                Text({label})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(label_color)
            ),
        ]
    }
}

/// 滚动区域场景（滚动容器 + 滚动条）
fn scroll_area(search_state: &SearchState) -> impl Scene + use<> {
    let content = scroll_content(search_state);

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: Val::Px(0.0),
            min_height: Val::Px(0.0),
            position_type: PositionType::Relative,
        }
        Children [
            (
                #SearchScroll
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                }
                ScrollArea
                Children [ {content} ]
            ),
            scrollbar(#SearchScroll),
        ]
    }
}

/// 滚动内容场景（根据状态显示不同内容）
fn scroll_content(search_state: &SearchState) -> Box<dyn SceneList> {
    if search_state.is_loading {
        return Box::new(bsn_list![(
            SearchLoadingText
            Text("正在搜索...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node { margin: UiRect::all(Val::Px(20.0)) }
        )]);
    }

    if let Some(error) = &search_state.error {
        let error_text = format!("搜索失败: {}", error);
        return Box::new(bsn_list![(
            SearchErrorText
            Text({error_text})
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::ERROR)
            Node { margin: UiRect::all(Val::Px(20.0)) }
        )]);
    }

    if search_state.has_searched && search_state.results.is_empty() {
        let empty_text = format!("未找到与 \"{}\" 相关的漫画", search_state.keyword);
        return Box::new(bsn_list![(
            SearchEmptyText
            Text({empty_text})
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node { margin: UiRect::all(Val::Px(20.0)) }
        )]);
    }

    if !search_state.has_searched {
        let mut items: Vec<Box<dyn Scene>> = vec![Box::new(bsn! {
            Text("输入关键词开始搜索")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node { margin: UiRect::all(Val::Px(20.0)) }
        })];

        // 热门搜索标签云
        if !search_state.hot_keywords.is_empty() {
            items.push(Box::new(hot_keywords_section(&search_state.hot_keywords)));
        }

        return Box::new(items);
    }

    // 搜索结果
    let result_text = format!(
        "共找到 {} 页结果（第 {} 页）",
        search_state.total_pages, search_state.page
    );
    Box::new(bsn_list![
        (
            Text({result_text})
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node {
                margin: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(15.0), Val::Px(10.0)),
            }
        ),
        (
            SearchResultsGrid
            Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect::new(
                    Val::Px(search_layout::PADDING_LEFT),
                    Val::Px(search_layout::PADDING_RIGHT),
                    Val::Px(0.0),
                    Val::Px(search_layout::PADDING_BOTTOM),
                ),
                column_gap: Val::Px(search_layout::COLUMN_GAP),
                row_gap: Val::Px(search_layout::ROW_GAP),
            }
        ),
    ])
}

/// 热门搜索标签云场景
fn hot_keywords_section(hot_keywords: &[String]) -> impl Scene + use<> {
    let tags: Vec<_> = hot_keywords
        .iter()
        .map(|keyword| hot_keyword_tag(keyword))
        .collect();

    bsn! {
        HotKeywordsContainer
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(0.0), Val::Px(20.0)),
            row_gap: Val::Px(12.0),
        }
        Children [
            (
                // 标题
                Text("热门搜索")
                TextFont { font_size: FontSize::Px(15.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 标签云容器
                Node {
                    width: Val::Percent(100.0),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(5.0),
                    row_gap: Val::Px(5.0),
                }
                Children [ {tags} ]
            ),
        ]
    }
}

/// 单个热词标签按钮场景
fn hot_keyword_tag(keyword: &str) -> impl Scene + use<> {
    let tag_keyword = keyword.to_string();
    let label = keyword.to_string();

    bsn! {
        HotKeywordTag { keyword: {tag_keyword} }
        Button
        template_value(ButtonStyle::card())
        Node {
            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(AppColors::SURFACE)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 创建搜索界面
pub fn setup_search_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    search_state: Res<SearchState>,
    categories_state: Res<CategoriesState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<SearchCardCreationState>,
    mut keywords_messages: MessageWriter<LoadKeywordsRequest>,
    mut existing_query: Query<&mut Node, With<SearchRoot>>,
) {
    // 如果 SearchRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        // 仍然触发热词加载
        if !search_state.hot_keywords_loaded {
            keywords_messages.write(LoadKeywordsRequest);
        }
        return;
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    creation_state.clear();

    // 如果热词未加载，触发加载请求
    if !search_state.hot_keywords_loaded {
        keywords_messages.write(LoadKeywordsRequest);
    }

    let available_categories: Vec<String> = categories_state
        .categories
        .iter()
        .map(|c| c.title.clone())
        .collect();

    let search_root = commands
        .spawn_scene(search_page(
            &search_state,
            &search_state.keyword,
            &available_categories,
        ))
        .id();

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(search_root);
    }

    if search_state.has_searched && !search_state.results.is_empty() && search_state.error.is_none()
    {
        // 应用屏蔽过滤后的数量
        let filtered_count = CompiledFilter::from_settings()
            .filter_comic_indices(&search_state.results)
            .len();
        if filtered_count > 0 {
            creation_state.start_precreate(filtered_count, font);
        }
    }
}

/// 搜索结果卡片场景
fn search_result_card(
    comic: &picacg_api::models::Comic,
    image_cache: &ImageCache,
    downloaded: &DownloadedComicsIndex,
    hidden: bool,
) -> impl Scene + use<> {
    let card_comic_id = comic.id.clone();
    let menu_comic_id = comic.id.clone();
    let menu_comic_title = comic.title.clone();
    let menu_eps_count = comic.eps_count;
    let title = comic.title.clone();
    let author = comic.author.clone();
    let visibility = if hidden {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    // 封面图片（未缓存时留空占位，等 update_search_images 补 ImageNode）
    let cover_url = comic.thumb.url();
    let image_comic_id = comic.id.clone();
    let cached_cover = image_cache.get(&cover_url).cloned();
    let cover: Box<dyn SceneList> = match cached_cover {
        Some(handle) => Box::new(bsn_list![(
            SearchResultImage { comic_id: {image_comic_id}, url: {cover_url} }
            ImageNode { image: {handle} }
            Node {
                width: Val::Px(164.0),
                height: Val::Px(220.0),
            }
        )]),
        None => Box::new(bsn_list![(
            SearchResultImage { comic_id: {image_comic_id}, url: {cover_url} }
            template_value(LoadingShimmer::new(AppColors::SECONDARY))
            Node {
                width: Val::Px(164.0),
                height: Val::Px(220.0),
            }
            BackgroundColor(AppColors::SECONDARY)
        )]),
    };

    // 分类标签容器（最多显示 3 个分类）
    let category_row: Box<dyn SceneList> = if comic.categories.is_empty() {
        Box::new(bsn_list![])
    } else {
        let badges: Vec<_> = comic
            .categories
            .iter()
            .take(3)
            .map(|category| tag_badge(category, TagColor::Category))
            .collect();
        Box::new(bsn_list![(
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(2.0),
                max_width: Val::Px(164.0),
                overflow: Overflow::clip(),
            }
            Children [ {badges} ]
        )])
    };

    // 标签容器（最多显示 3 个标签）
    let tag_row: Box<dyn SceneList> = if comic.tags.is_empty() {
        Box::new(bsn_list![])
    } else {
        let badges: Vec<_> = comic
            .tags
            .iter()
            .take(3)
            .map(|tag| search_tag_badge(tag))
            .collect();
        Box::new(bsn_list![(
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(2.0),
                max_width: Val::Px(164.0),
                margin: UiRect::top(Val::Px(2.0)),
                overflow: Overflow::clip(),
            }
            Children [ {badges} ]
        )])
    };

    // 创建/更新时间
    let time_info = comic_time_info(comic.created_at.as_deref(), comic.updated_at.as_deref());

    // 封面右下角下载角标（挂在封面容器内，直接贴容器右下角）
    let badge: Box<dyn SceneList> = Box::new(bsn_list![download_status_badge(
        &comic.id,
        comic.eps_count,
        downloaded,
        BadgeAnchor::CoverContainer
    )]);

    bsn! {
        SearchResultCard { comic_id: {card_comic_id} }
        ContextMenuTarget { comic_id: {menu_comic_id}, comic_title: {menu_comic_title}, eps_count: {menu_eps_count} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Px(180.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        BackgroundColor(AppColors::SURFACE)
        template_value(visibility)
        Children [
            (
                // 封面图片容器
                Node {
                    width: Val::Px(164.0),
                    height: Val::Px(220.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip(),
                }
                BackgroundColor(AppColors::SECONDARY)
                Children [ {cover}, {badge} ]
            ),
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                }
            ),
            (
                // 作者
                Text({author})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(4.0), Val::Px(4.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                }
            ),
            {category_row},
            {tag_row},
            {time_info},
        ]
    }
}

/// 搜索结果卡片的标签徽章场景（紫色，与 ui_common 的分类/标签配色不同）
fn search_tag_badge(text: &str) -> impl Scene + use<> {
    let label = text.to_string();

    bsn! {
        Node {
            padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(1.0), Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
        }
        BackgroundColor(Color::srgba(0.6, 0.3, 0.6, 0.3))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(10.0) }
                TextColor(Color::srgb(0.9, 0.7, 0.9))
            )
        ]
    }
}

/// 清理搜索界面
pub fn cleanup_search_ui(
    mut query: Query<&mut Node, With<SearchRoot>>,
    mut creation_state: ResMut<SearchCardCreationState>,
    mut input_focus: ResMut<InputFocus>,
    input_query: Query<Entity, With<SearchInputField>>,
) {
    // 页面只隐藏不销毁：焦点若留在隐藏的输入框上，别的页面按键会灌进搜索框，
    // 且 IME 一直开着 —— 离开时必须交还焦点
    if let Some(focused) = input_focus.get()
        && input_query.contains(focused)
    {
        input_focus.clear();
    }

    // 清空瀑布式创建状态
    creation_state.clear();

    // 隐藏而非销毁
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 搜索页面动作键处理（Enter 搜索、Escape 失焦）
///
/// 焦点、字符编辑、IME 全部由通用 TextInput 系统负责，这里只认动作键，
/// 且只在焦点确实落在搜索框上时才响应。
pub fn handle_search_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut input_focus: ResMut<InputFocus>,
    input_query: Query<&TextInput, With<SearchInputField>>,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(input) = input_query.get(focused) else {
        return;
    };

    for event in keyboard_events.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Enter => {
                // 关键词以输入框为准（输入过程只写 TextInput.value，不回写状态）
                let keyword = input.value.trim();
                if keyword.is_empty() {
                    continue;
                }
                search_state.keyword = keyword.to_string();
                search_state.is_loading = true;
                search_state.needs_rebuild = true;
                search_state.page = 1;
                search_messages.write(SearchComicsRequestEvent {
                    keyword: search_state.keyword.clone(),
                    page: 1,
                    sort: search_state.sort.clone(),
                    categories: search_state.selected_categories.clone(),
                });
                input_focus.clear();
            }
            Key::Escape => input_focus.clear(),
            _ => {}
        }
    }
}

/// 搜索按钮交互
pub fn search_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SearchButton>)>,
    input_query: Query<&TextInput, With<SearchInputField>>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
    mut search_state: ResMut<SearchState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed || search_state.is_loading {
            continue;
        }

        // 关键词以输入框为准；输入框取不到（页面已销毁）才退回状态里的旧值
        let keyword = match input_query.single() {
            Ok(input) if !input.value.trim().is_empty() => input.value.trim().to_string(),
            _ => search_state.keyword.clone(),
        };
        if keyword.is_empty() {
            continue;
        }

        search_state.keyword = keyword;
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

/// 重置搜索按钮交互：清空关键词、搜索结果、排序、分类过滤
pub fn search_reset_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SearchResetButton>)>,
    mut search_state: ResMut<SearchState>,
    mut input_query: Query<&mut TextInput, With<SearchInputField>>,
    mut creation_state: ResMut<crate::systems::waterfall::SearchCardCreationState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 清空状态（恢复到初始未搜索状态）
        search_state.keyword.clear();
        search_state.results.clear();
        search_state.page = 1;
        search_state.total_pages = 0;
        search_state.error = None;
        search_state.is_loading = false;
        search_state.has_searched = false;
        search_state.sort = "dd".to_string();
        search_state.selected_categories.clear();
        search_state.show_category_filter = false;
        search_state.needs_rebuild = true;

        // 清空输入框（光标一并归零）
        for mut input in input_query.iter_mut() {
            input.set_value("");
        }

        // 清空瀑布流状态
        creation_state.clear();

        tracing::info!("搜索已重置");
    }
}

/// 搜索结果卡片交互
pub fn search_result_card_interaction(
    interaction_query: Query<(&Interaction, &SearchResultCard), Changed<Interaction>>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
        }
    }
}

/// 消费分页控件状态变化（翻页边界与按钮行为已内联在控件观察者里）
pub fn search_pagination_changed(
    mut pagination_query: Query<
        &mut Pagination,
        (With<PaginationControl<SearchPage>>, Changed<Pagination>),
    >,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    let Ok(mut pagination) = pagination_query.single_mut() else {
        return;
    };
    let new_page = pagination.current_page as i32;
    // 只响应真实翻页（控件重建后的同值回填在此被过滤）
    if new_page == search_state.page.max(1) {
        return;
    }
    // 加载中不翻页：把控件页码退回状态值，避免控件与状态脱节
    if search_state.is_loading {
        pagination.current_page = search_state.page.max(1) as u32;
        return;
    }

    search_state.page = new_page;
    search_state.is_loading = true;
    search_state.needs_rebuild = true;
    search_messages.write(SearchComicsRequestEvent {
        keyword: search_state.keyword.clone(),
        page: new_page,
        sort: search_state.sort.clone(),
        categories: search_state.selected_categories.clone(),
    });

    tracing::debug!("切换到搜索第 {} 页", new_page);
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
        match image_cache.get(&img.url) {
            Some(handle) => {
                commands
                    .entity(entity)
                    .insert(ImageNode::new(handle.clone()));
            }
            // 加载失败的封面摘掉标记，退出每帧扫描集（此前永久残留）
            None if image_cache.is_failed(&img.url) => {
                commands.entity(entity).remove::<SearchResultImage>();
            }
            None => {}
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
    input_focus: Res<InputFocus>,
    input_query: Query<(Entity, &TextInput), With<SearchInputField>>,
    mut creation_state: ResMut<SearchCardCreationState>,
) {
    if !search_state.is_changed() || !search_state.needs_rebuild {
        return;
    }

    // 重建前重置标志
    search_state.needs_rebuild = false;

    // 重建会销毁旧输入框：焦点（实体级）和正在编辑的文本都要接力到新实体上，
    // 否则搜索结果一回来就把用户刚敲的半截关键词冲掉
    let (was_focused, input_value) = match input_query.single() {
        Ok((entity, input)) => (input_focus.get() == Some(entity), input.value.clone()),
        Err(_) => (false, search_state.keyword.clone()),
    };

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

    let search_root = commands
        .spawn_scene(search_page(
            &search_state,
            &input_value,
            &available_categories,
        ))
        .id();

    commands.entity(content_entity).add_child(search_root);

    // 焦点还原：新输入框实体也是命令队列产物，用队列尾的命令去查它，
    // 避免在这里凭空猜实体 ID
    if was_focused {
        commands.queue(|world: &mut World| {
            let mut query = world.query_filtered::<Entity, With<SearchInputField>>();
            let Some(entity) = query.iter(world).next() else {
                return;
            };
            world
                .resource_mut::<InputFocus>()
                .set(entity, FocusCause::Navigated);
        });
    }

    if search_state.has_searched && !search_state.results.is_empty() && search_state.error.is_none()
    {
        // 应用屏蔽过滤后的数量
        let filtered_count = CompiledFilter::from_settings()
            .filter_comic_indices(&search_state.results)
            .len();
        if filtered_count > 0 {
            creation_state.start_precreate(filtered_count, font);
        }
    }
}

/// 瀑布式显示搜索结果卡片（预创建所有隐藏卡片，然后分批显示）
pub fn waterfall_create_search_cards(
    mut commands: Commands,
    mut creation_state: ResMut<SearchCardCreationState>,
    search_state: Res<SearchState>,
    image_cache: Res<ImageCache>,
    downloaded: Res<DownloadedComicsIndex>,
    results_grid_query: Query<Entity, With<SearchResultsGrid>>,
    time: Res<Time>,
) {
    // 检查是否需要预创建
    if creation_state.needs_precreate() {
        let Ok(grid_entity) = results_grid_query.single() else {
            return;
        };

        // 字体句柄随预创建请求写入状态，未就绪时不创建
        // （BSN 场景走默认字体句柄，此处只做就绪校验）
        if creation_state.font_handle.is_none() {
            return;
        }

        let results = &search_state.results;
        // 惰性过滤：仅预创建帧计算（每帧必跑路径零过滤开销）
        let filtered_indices = CompiledFilter::from_settings().filter_comic_indices(results);
        let count = creation_state.get_precreate_count();

        if filtered_indices.is_empty() || count == 0 {
            creation_state.clear();
            return;
        }

        // 一次性创建所有隐藏卡片（使用过滤后的索引）
        let mut entities = Vec::with_capacity(count);
        for i in 0..count {
            if let Some(&original_index) = filtered_indices.get(i)
                && let Some(comic) = results.get(original_index)
            {
                let entity = commands
                    .spawn_scene(search_result_card(comic, &image_cache, &downloaded, true))
                    .insert(ChildOf(grid_entity))
                    .id();
                entities.push(entity);
            }
        }

        // 设置预创建完成后的实体列表
        creation_state.set_precreated_entities(entities);
        tracing::debug!("搜索结果卡片预创建完成: {} 个（过滤后）", count);
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

// ==================== 过滤工具栏交互系统 ====================

/// 排序按钮交互
///
/// 查询**不加** `Changed<Interaction>`：选中态刷新需要覆盖所有排序按钮
/// （点 B 时 A 的 Interaction 并未变化，加过滤器会让 A 一直亮着）。
/// 重复触发由 `search_state.sort != btn.sort` 挡掉——按下第一帧改完状态后，
/// 后续帧该条件即为假。
pub fn sort_button_interaction(
    mut interaction_query: Query<(
        &Interaction,
        &SortButton,
        &mut ButtonStyle,
        &mut BorderColor,
    )>,
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

    // 更新选中态：底色由全局 apply_button_interaction 按 selected 解析，
    // 边框不在 ButtonStyle 管辖内，随选中态一并翻转
    for (_, btn, mut style, mut border_color) in interaction_query.iter_mut() {
        let is_active = search_state.sort == btn.sort;
        if style.selected != is_active {
            style.selected = is_active;
            *border_color = BorderColor::all(if is_active {
                AppColors::PRIMARY
            } else {
                AppColors::BORDER
            });
        }
    }
}

/// 分类过滤面板展开/折叠交互
pub fn category_filter_toggle_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CategoryFilterToggle>)>,
    mut search_state: ResMut<SearchState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
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
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SelectAllCategoriesButton>)>,
    categories_state: Res<CategoriesState>,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

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
}

/// 清空分类按钮交互
pub fn clear_all_categories_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClearAllCategoriesButton>)>,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed || search_state.selected_categories.is_empty() {
            continue;
        }

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

// ==================== 热词交互系统 ====================

/// 热词标签点击交互：点击热词填入搜索框并触发搜索
pub fn hot_keyword_tag_interaction(
    interaction_query: Query<(&Interaction, &HotKeywordTag), Changed<Interaction>>,
    mut search_state: ResMut<SearchState>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
    mut input_query: Query<&mut TextInput, With<SearchInputField>>,
) {
    for (interaction, tag) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 设置关键词
        search_state.keyword.clone_from(&tag.keyword);

        // 同步到输入框（set_value 按字符数落光标，热词含 CJK 时字节长度会越界）
        for mut input in input_query.iter_mut() {
            input.set_value(tag.keyword.as_str());
        }

        // 触发搜索
        search_state.is_loading = true;
        search_state.needs_rebuild = true;
        search_state.page = 1;
        search_messages.write(SearchComicsRequestEvent {
            keyword: tag.keyword.clone(),
            page: 1,
            sort: search_state.sort.clone(),
            categories: search_state.selected_categories.clone(),
        });

        tracing::info!("点击热词搜索: {}", tag.keyword);
    }
}
