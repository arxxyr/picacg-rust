//! 分页控件 BSN 场景函数
//!
//! 控件自含翻页行为：上一页/下一页按钮通过内联 `on(Pointer<Click>)` 观察者
//! 直接修改根实体上的 `Pagination` 组件（带边界检查）。
//! 页面无需注册任何按钮交互系统，只需消费 `Changed<Pagination>`。

use bevy::{
    picking::events::{Click, Pointer},
    prelude::*,
};

use super::components::{
    Pagination, PaginationConfig, PaginationControl, PaginationNext, PaginationPageText,
    PaginationPrev,
};
use crate::systems::theme::Theme;

/// 分页控件场景（默认主题 + 默认配置）
pub fn pagination_controls<T: Send + Sync + 'static>(
    current_page: u32,
    total_pages: u32,
) -> impl Scene + use<T> {
    pagination_controls_with_theme::<T>(
        &Theme::dark(),
        current_page,
        total_pages,
        &PaginationConfig::default(),
    )
}

/// 分页控件场景（自定义主题与配置）
///
/// 布局结构：
/// PaginationControl<T> + Pagination
///   ├── PaginationPrev [上一页]（内联观察者：current_page -= 1）
///   ├── PaginationPageText  "当前页 / 总页数"
///   └── PaginationNext [下一页]（内联观察者：current_page += 1）
pub fn pagination_controls_with_theme<T: Send + Sync + 'static>(
    theme: &Theme,
    current_page: u32,
    total_pages: u32,
    config: &PaginationConfig,
) -> impl Scene + use<T> {
    let pagination = Pagination {
        current_page,
        total_pages,
    };
    let prev_color = if pagination.has_prev() {
        theme.primary
    } else {
        theme.secondary
    };
    let next_color = if pagination.has_next() {
        theme.primary
    } else {
        theme.secondary
    };
    let border = theme.border;
    let surface = theme.surface;
    let text = theme.text;
    let page_label = format!("{} / {}", current_page, total_pages);
    let font_size = FontSize::Px(config.font_size);
    let container_height = config.container_height;
    let gap = config.gap;
    let button_width = config.button_width;
    let button_height = config.button_height;

    bsn! {
        template_value(PaginationControl::<T>::default())
        template_value(pagination)
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(container_height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(gap),
            border: UiRect::top(Val::Px(1.0)),
        }
        template_value(BorderColor::all(border))
        BackgroundColor(surface)
        Children [
            (
                // 上一页按钮
                PaginationPrev
                Button
                Interaction
                Node {
                    width: Val::Px(button_width),
                    height: Val::Px(button_height),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor(prev_color)
                on(|click: On<Pointer<Click>>,
                    child_of: Query<&ChildOf>,
                    mut paginations: Query<&mut Pagination>| {
                    let Ok(parent) = child_of.get(click.entity) else {
                        return;
                    };
                    let Ok(mut pagination) = paginations.get_mut(parent.parent()) else {
                        return;
                    };
                    if pagination.has_prev() {
                        pagination.current_page -= 1;
                    }
                })
                Children [
                    (
                        Text("上一页")
                        TextFont { font_size: {font_size} }
                        TextColor(text)
                    )
                ]
            ),
            (
                // 页码显示
                PaginationPageText
                Text({page_label})
                TextFont { font_size: {font_size} }
                TextColor(text)
            ),
            (
                // 下一页按钮
                PaginationNext
                Button
                Interaction
                Node {
                    width: Val::Px(button_width),
                    height: Val::Px(button_height),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor(next_color)
                on(|click: On<Pointer<Click>>,
                    child_of: Query<&ChildOf>,
                    mut paginations: Query<&mut Pagination>| {
                    let Ok(parent) = child_of.get(click.entity) else {
                        return;
                    };
                    let Ok(mut pagination) = paginations.get_mut(parent.parent()) else {
                        return;
                    };
                    if pagination.has_next() {
                        pagination.current_page += 1;
                    }
                })
                Children [
                    (
                        Text("下一页")
                        TextFont { font_size: {font_size} }
                        TextColor(text)
                    )
                ]
            ),
        ]
    }
}
