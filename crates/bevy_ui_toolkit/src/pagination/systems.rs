//! 分页系统函数

use bevy::prelude::*;

use super::components::*;
use crate::theme::Theme;

/// 创建分页控件 UI（使用默认主题颜色）
///
/// # 类型参数
/// - `T`: 页面标记类型，用于区分不同页面的分页组件
///
/// # 参数
/// - `parent`: 父容器
/// - `font`: 字体句柄
/// - `current_page`: 当前页码
/// - `total_pages`: 总页数
pub fn spawn_pagination_controls<T: Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_page: u32,
    total_pages: u32,
) {
    spawn_pagination_controls_with_theme::<T>(
        parent,
        font,
        &Theme::dark(),
        current_page,
        total_pages,
        &PaginationConfig::default(),
    );
}

/// 使用自定义配置创建分页控件 UI（使用默认主题颜色）
pub fn spawn_pagination_controls_with_config<T: Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_page: u32,
    total_pages: u32,
    config: &PaginationConfig,
) {
    spawn_pagination_controls_with_theme::<T>(
        parent,
        font,
        &Theme::dark(),
        current_page,
        total_pages,
        config,
    );
}

/// 使用自定义主题创建分页控件 UI
pub fn spawn_pagination_controls_with_theme<T: Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    theme: &Theme,
    current_page: u32,
    total_pages: u32,
    config: &PaginationConfig,
) {
    parent
        .spawn((
            PaginationControl::<T>::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(config.container_height),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(config.gap),
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(theme.border),
            BackgroundColor(theme.surface),
        ))
        .with_children(|pagination| {
            // 上一页按钮
            pagination
                .spawn((
                    PaginationPrevButton::<T>::default(),
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Px(config.button_width),
                        height: Val::Px(config.button_height),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if current_page > 1 {
                        theme.primary
                    } else {
                        theme.secondary
                    }),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("上一页"),
                        TextFont {
                            font: font.clone(),
                            font_size: config.font_size,
                            ..default()
                        },
                        TextColor(theme.text),
                    ));
                });

            // 页码显示
            pagination.spawn((
                PaginationPageText::<T>::default(),
                Text::new(format!("{} / {}", current_page, total_pages)),
                TextFont {
                    font: font.clone(),
                    font_size: config.font_size,
                    ..default()
                },
                TextColor(theme.text),
            ));

            // 下一页按钮
            pagination
                .spawn((
                    PaginationNextButton::<T>::default(),
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Px(config.button_width),
                        height: Val::Px(config.button_height),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if current_page < total_pages {
                        theme.primary
                    } else {
                        theme.secondary
                    }),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("下一页"),
                        TextFont {
                            font: font.clone(),
                            font_size: config.font_size,
                            ..default()
                        },
                        TextColor(theme.text),
                    ));
                });
        });
}

/// 更新分页显示（页码文本和按钮状态）- 使用默认主题颜色
///
/// # 类型参数
/// - `T`: 页面标记类型
///
/// # 参数
/// - `page_text_query`: 页码文本查询
/// - `prev_btn_query`: 上一页按钮查询
/// - `next_btn_query`: 下一页按钮查询
/// - `current_page`: 当前页码
/// - `total_pages`: 总页数
pub fn update_pagination_display<T: Send + Sync + 'static>(
    page_text_query: &mut Query<&mut Text, With<PaginationPageText<T>>>,
    prev_btn_query: &mut Query<&mut BackgroundColor, PrevBtnFilter<T>>,
    next_btn_query: &mut Query<&mut BackgroundColor, NextBtnFilter<T>>,
    current_page: u32,
    total_pages: u32,
) {
    update_pagination_display_with_theme::<T>(
        page_text_query,
        prev_btn_query,
        next_btn_query,
        &Theme::dark(),
        current_page,
        total_pages,
    );
}

/// 更新分页显示（页码文本和按钮状态）- 使用自定义主题
pub fn update_pagination_display_with_theme<T: Send + Sync + 'static>(
    page_text_query: &mut Query<&mut Text, With<PaginationPageText<T>>>,
    prev_btn_query: &mut Query<&mut BackgroundColor, PrevBtnFilter<T>>,
    next_btn_query: &mut Query<&mut BackgroundColor, NextBtnFilter<T>>,
    theme: &Theme,
    current_page: u32,
    total_pages: u32,
) {
    // 更新页码文本
    for mut text in page_text_query.iter_mut() {
        **text = format!("{} / {}", current_page, total_pages);
    }

    // 更新上一页按钮状态
    for mut bg_color in prev_btn_query.iter_mut() {
        *bg_color = BackgroundColor(if current_page > 1 {
            theme.primary
        } else {
            theme.secondary
        });
    }

    // 更新下一页按钮状态
    for mut bg_color in next_btn_query.iter_mut() {
        *bg_color = BackgroundColor(if current_page < total_pages {
            theme.primary
        } else {
            theme.secondary
        });
    }
}

/// 检查分页按钮交互，返回是否需要翻页以及翻页方向
///
/// # 返回值
/// - `Some(true)`: 点击了下一页
/// - `Some(false)`: 点击了上一页
/// - `None`: 没有点击或不满足翻页条件
pub fn check_pagination_interaction<T: Send + Sync + 'static>(
    prev_query: &Query<&Interaction, (Changed<Interaction>, With<PaginationPrevButton<T>>)>,
    next_query: &Query<&Interaction, (Changed<Interaction>, With<PaginationNextButton<T>>)>,
    current_page: u32,
    total_pages: u32,
) -> Option<bool> {
    // 检查上一页
    for interaction in prev_query.iter() {
        if *interaction == Interaction::Pressed && current_page > 1 {
            return Some(false); // 上一页
        }
    }

    // 检查下一页
    for interaction in next_query.iter() {
        if *interaction == Interaction::Pressed && current_page < total_pages {
            return Some(true); // 下一页
        }
    }

    None
}
