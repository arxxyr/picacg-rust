//! 分页系统函数
//!
//! 只有一个内部系统：`Pagination` 变化时刷新页码文本与按钮配色。
//! 全局注册一次即可，`Changed<Pagination>` 保证静止零开销。

use bevy::prelude::*;

use super::components::{Pagination, PaginationNext, PaginationPageText, PaginationPrev};
use crate::systems::theme::Theme;

/// 刷新分页控件显示（页码文本 + 按钮可用态配色）
///
/// 翻页来源无论是控件自身的观察者还是页面代码写 `Pagination`，
/// 显示都由本系统统一跟进。
pub fn refresh_pagination_widgets(
    controls: Query<(&Pagination, &Children), Changed<Pagination>>,
    mut text_query: Query<&mut Text, With<PaginationPageText>>,
    mut prev_query: Query<&mut BackgroundColor, (With<PaginationPrev>, Without<PaginationNext>)>,
    mut next_query: Query<&mut BackgroundColor, (With<PaginationNext>, Without<PaginationPrev>)>,
) {
    for (pagination, children) in &controls {
        let theme = Theme::dark();
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                let label = format!("{} / {}", pagination.current_page, pagination.total_pages);
                if **text != label {
                    **text = label;
                }
            } else if let Ok(mut color) = prev_query.get_mut(child) {
                let target = if pagination.has_prev() {
                    theme.primary
                } else {
                    theme.secondary
                };
                if color.0 != target {
                    color.0 = target;
                }
            } else if let Ok(mut color) = next_query.get_mut(child) {
                let target = if pagination.has_next() {
                    theme.primary
                } else {
                    theme.secondary
                };
                if color.0 != target {
                    color.0 = target;
                }
            }
        }
    }
}
