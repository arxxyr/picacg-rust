//! 通用分页组件模块
//!
//! 提供可复用的分页 UI 组件和辅助函数。
//!
//! ## 使用方法
//!
//! 1. 定义页面特定的标记类型：
//! ```ignore
//! pub struct FavoritesPage;
//! pub struct ComicsPage;
//! ```
//!
//! 2. 使用 `spawn_pagination_controls` 创建分页 UI：
//! ```ignore
//! spawn_pagination_controls::<FavoritesPage>(
//!     parent,
//!     &font,
//!     current_page,
//!     total_pages,
//! );
//! ```
//!
//! 3. 使用 `update_pagination_display` 更新分页显示：
//! ```ignore
//! update_pagination_display::<FavoritesPage>(
//!     &mut page_text_query,
//!     &mut prev_btn_query,
//!     &mut next_btn_query,
//!     current_page,
//!     total_pages,
//! );
//! ```

use std::marker::PhantomData;

use bevy::prelude::*;

use super::login::AppColors;

// ==================== 通用分页组件 ====================

/// 分页容器标记（泛型 T 用于区分不同页面）
#[derive(Component)]
pub struct PaginationControl<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationControl<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 上一页按钮标记
#[derive(Component)]
pub struct PaginationPrevButton<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationPrevButton<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 下一页按钮标记
#[derive(Component)]
pub struct PaginationNextButton<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationNextButton<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// 页码文本标记
#[derive(Component)]
pub struct PaginationPageText<T: Send + Sync + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Default for PaginationPageText<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

// ==================== 分页 UI 创建函数 ====================

/// 分页配置
pub struct PaginationConfig {
    /// 按钮宽度
    pub button_width: f32,
    /// 按钮高度
    pub button_height: f32,
    /// 容器高度
    pub container_height: f32,
    /// 按钮间距
    pub gap: f32,
    /// 字体大小
    pub font_size: f32,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            button_width: 80.0,
            button_height: 36.0,
            container_height: 50.0,
            gap: 20.0,
            font_size: 14.0,
        }
    }
}

/// 创建分页控件 UI
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
    spawn_pagination_controls_with_config::<T>(
        parent,
        font,
        current_page,
        total_pages,
        &PaginationConfig::default(),
    );
}

/// 使用自定义配置创建分页控件 UI
pub fn spawn_pagination_controls_with_config<T: Send + Sync + 'static>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
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
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(AppColors::SURFACE),
            Transform::default(),
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
                        AppColors::PRIMARY
                    } else {
                        AppColors::SECONDARY
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
                        TextColor(AppColors::TEXT),
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
                TextColor(AppColors::TEXT),
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
                        AppColors::PRIMARY
                    } else {
                        AppColors::SECONDARY
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
                        TextColor(AppColors::TEXT),
                    ));
                });
        });
}

// ==================== 分页显示更新函数 ====================

/// 更新分页显示（页码文本和按钮状态）
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
    prev_btn_query: &mut Query<
        &mut BackgroundColor,
        (
            With<PaginationPrevButton<T>>,
            Without<PaginationNextButton<T>>,
        ),
    >,
    next_btn_query: &mut Query<
        &mut BackgroundColor,
        (
            With<PaginationNextButton<T>>,
            Without<PaginationPrevButton<T>>,
        ),
    >,
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
            AppColors::PRIMARY
        } else {
            AppColors::SECONDARY
        });
    }

    // 更新下一页按钮状态
    for mut bg_color in next_btn_query.iter_mut() {
        *bg_color = BackgroundColor(if current_page < total_pages {
            AppColors::PRIMARY
        } else {
            AppColors::SECONDARY
        });
    }
}

// ==================== 分页交互辅助函数 ====================

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

/// 分页状态 trait
///
/// 实现此 trait 可以让状态类型与分页系统配合使用
pub trait PaginationState {
    /// 获取当前页码
    fn current_page(&self) -> u32;

    /// 获取总页数
    fn total_pages(&self) -> u32;

    /// 设置当前页码
    fn set_page(&mut self, page: u32);

    /// 设置加载状态
    fn set_loading(&mut self, loading: bool);

    /// 清除数据（翻页时调用）
    fn clear_data(&mut self);
}
