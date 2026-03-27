//! 锅贴社区系统
//!
//! 展示锅贴帖子列表，支持分页浏览

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::scrollbar_config::SCROLLBAR_WIDTH,
        ui_common::{Scrollable, spawn_scrollbar},
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 锅贴社区根节点
#[derive(Component)]
pub struct FriedRoot;

/// 锅贴社区滚动容器
#[derive(Component)]
pub struct FriedScrollContainer;

/// 锅贴帖子卡片
#[derive(Component)]
pub struct FriedPostCard {
    #[allow(dead_code)]
    pub post_id: String,
}

/// 锅贴分页：上一页按钮
#[derive(Component)]
pub struct FriedPrevPageButton;

/// 锅贴分页：下一页按钮
#[derive(Component)]
pub struct FriedNextPageButton;

/// 锅贴分页：页码文本
#[derive(Component)]
pub struct FriedPageText;

/// 刷新按钮
#[derive(Component)]
pub struct FriedRefreshButton;

// ==================== 布局常量 ====================

mod fried_layout {
    /// 卡片间距
    pub const CARD_GAP: f32 = 12.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 15.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
}

// ==================== 系统函数 ====================

/// 创建锅贴社区界面（如果已存在则只显示）
pub fn setup_fried_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    fried_state: Res<FriedState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_posts_messages: MessageWriter<LoadFriedPostsRequest>,
    mut load_apps_messages: MessageWriter<LoadAppsRequest>,
    mut existing_query: Query<&mut Node, With<FriedRoot>>,
) {
    // 如果 FriedRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if fried_state.posts.is_empty() && !fried_state.is_loading && fried_state.error.is_none() {
            if fried_state.fried_token.is_none() {
                load_apps_messages.write(LoadAppsRequest);
            } else {
                load_posts_messages.write(LoadFriedPostsRequest {
                    page: fried_state.page,
                });
            }
        }
        return;
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let fried_root = commands
        .spawn((
            FriedRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // 标题栏
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
            ))
            .with_children(|header| {
                // 图标
                header.spawn((
                    Text::new(ICON_FORUM),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(AppColors::PRIMARY),
                ));

                header.spawn((
                    Text::new("锅贴社区"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 弹性占位
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // 刷新按钮
                header
                    .spawn((
                        FriedRefreshButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::new(
                                Val::Px(10.0),
                                Val::Px(10.0),
                                Val::Px(5.0),
                                Val::Px(5.0),
                            ),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(AppColors::PRIMARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("{} 刷新", ICON_REFRESH)),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });

            // 滚动区域包装器
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                ..default()
            })
            .with_children(|wrapper| {
                // 可滚动内容区域
                let scroll_container_id = wrapper
                    .spawn((
                        FriedScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect {
                                left: Val::Px(fried_layout::PADDING_LEFT),
                                right: Val::Px(fried_layout::PADDING_RIGHT),
                                top: Val::Px(fried_layout::PADDING_TOP),
                                bottom: Val::Px(fried_layout::PADDING_BOTTOM),
                            },
                            row_gap: Val::Px(fried_layout::CARD_GAP),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        Scrollable,
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|content| {
                        if fried_state.is_loading {
                            content.spawn((
                                LoadingIndicator,
                                Text::new("加载中..."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        } else if let Some(ref error) = fried_state.error {
                            content.spawn((
                                ErrorMessage,
                                Text::new(format!("加载失败: {}", error)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.4, 0.4)),
                            ));
                        } else if fried_state.posts.is_empty() {
                            content.spawn((
                                Text::new("暂无帖子"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        } else {
                            // 显示帖子列表
                            for post in &fried_state.posts {
                                spawn_fried_post_card(content, &font, post);
                            }

                            // 分页控件
                            let total_pages = calculate_total_pages(&fried_state);
                            if total_pages > 1 {
                                spawn_fried_pagination(
                                    content,
                                    &font,
                                    fried_state.page + 1, // 显示从 1 开始的页码
                                    total_pages,
                                );
                            }
                        }

                        // 底部间距
                        content.spawn(Node {
                            height: Val::Px(30.0),
                            min_height: Val::Px(30.0),
                            ..default()
                        });
                    })
                    .id();

                // 滚动条
                spawn_scrollbar(wrapper, scroll_container_id);
            });
        })
        .id();

    // 挂载到内容区域
    if let Some(content_area) = content_area {
        commands.entity(content_area).add_child(fried_root);
    }

    // 如果没有帖子数据且不在加载中，触发加载
    if fried_state.posts.is_empty() && !fried_state.is_loading && fried_state.error.is_none() {
        // 如果还没有锅贴 token，先加载小程序列表获取入口
        if fried_state.fried_token.is_none() {
            load_apps_messages.write(LoadAppsRequest);
        } else {
            load_posts_messages.write(LoadFriedPostsRequest {
                page: fried_state.page,
            });
        }
    }
}

/// 计算总页数
fn calculate_total_pages(state: &FriedState) -> i32 {
    if state.limit > 0 {
        (state.total + state.limit - 1) / state.limit
    } else {
        1
    }
}

/// 创建单个帖子卡片
fn spawn_fried_post_card(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    post: &picacg_api::endpoints::fried::FriedPost,
) {
    parent
        .spawn((
            FriedPostCard {
                post_id: post.id.clone(),
            },
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|card| {
            // 用户信息行
            card.spawn(Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                // 用户头像占位
                row.spawn((
                    Node {
                        width: Val::Px(36.0),
                        height: Val::Px(36.0),
                        min_width: Val::Px(36.0),
                        min_height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
                    BorderColor::all(AppColors::BORDER),
                ))
                .with_children(|avatar| {
                    avatar.spawn((
                        Text::new(ICON_USER),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

                // 用户名 + 等级
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|info| {
                    if let Some(ref user) = post.user {
                        // 用户名
                        info.spawn(Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|name_row| {
                            name_row.spawn((
                                Text::new(&user.name),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));

                            // 等级标签
                            name_row
                                .spawn((
                                    Node {
                                        padding: UiRect::new(
                                            Val::Px(4.0),
                                            Val::Px(4.0),
                                            Val::Px(1.0),
                                            Val::Px(1.0),
                                        ),
                                        border_radius: BorderRadius::all(Val::Px(3.0)),
                                        ..default()
                                    },
                                    BackgroundColor(AppColors::PRIMARY),
                                ))
                                .with_children(|badge| {
                                    badge.spawn((
                                        Text::new(format!("Lv{}", user.level)),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // 称号
                            if !user.title.is_empty() {
                                name_row
                                    .spawn((
                                        Node {
                                            padding: UiRect::new(
                                                Val::Px(4.0),
                                                Val::Px(4.0),
                                                Val::Px(1.0),
                                                Val::Px(1.0),
                                            ),
                                            border_radius: BorderRadius::all(Val::Px(3.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.6, 0.3, 0.8)),
                                    ))
                                    .with_children(|badge| {
                                        badge.spawn((
                                            Text::new(&user.title),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 10.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            }
                        });
                    }

                    // 时间
                    if !post.created_at.is_empty() {
                        let time_display = format_time(&post.created_at);
                        info.spawn((
                            Text::new(time_display),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    }
                });
            });

            // 帖子内容
            if !post.content.is_empty() {
                // 截取前 200 个字符
                let content_text = if post.content.chars().count() > 200 {
                    format!("{}...", post.content.chars().take(200).collect::<String>())
                } else {
                    post.content.clone()
                };
                card.spawn((
                    Text::new(content_text),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                    Node {
                        max_width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }

            // 媒体附件提示
            if !post.medias.is_empty() {
                card.spawn((
                    Node {
                        padding: UiRect::new(
                            Val::Px(6.0),
                            Val::Px(6.0),
                            Val::Px(3.0),
                            Val::Px(3.0),
                        ),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.5, 0.8, 0.2)),
                ))
                .with_children(|media_hint| {
                    media_hint.spawn((
                        Text::new(format!("📷 {} 张图片", post.medias.len())),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.7, 1.0)),
                    ));
                });
            }

            // 底部操作栏（点赞数、评论数）
            card.spawn(Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            })
            .with_children(|footer| {
                // 点赞
                footer
                    .spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|like_row| {
                        let like_color = if post.liked {
                            Color::srgb(1.0, 0.4, 0.4)
                        } else {
                            AppColors::TEXT_SECONDARY
                        };
                        like_row.spawn((
                            Text::new(ICON_HEART),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(like_color),
                        ));
                        like_row.spawn((
                            Text::new(format!("{}", post.total_likes)),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });

                // 评论
                footer
                    .spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|comment_row| {
                        comment_row.spawn((
                            Text::new(ICON_FORUM),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                        comment_row.spawn((
                            Text::new(format!("{}", post.total_comments)),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });
            });
        });
}

/// 格式化时间字符串（简单截取日期部分）
fn format_time(time_str: &str) -> String {
    // 尝试解析 ISO 8601 格式的时间，如 "2024-03-20T12:34:56.789Z"
    if time_str.len() >= 19 {
        // 截取 "2024-03-20 12:34:56" 格式
        let date_part = &time_str[..10];
        let time_part = &time_str[11..19.min(time_str.len())];
        format!("{} {}", date_part, time_part)
    } else {
        time_str.to_string()
    }
}

/// 创建分页控件
fn spawn_fried_pagination(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_page: i32,
    total_pages: i32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(15.0),
            padding: UiRect::vertical(Val::Px(15.0)),
            ..default()
        })
        .with_children(|row| {
            // 上一页
            let prev_enabled = current_page > 1;
            let prev_color = if prev_enabled {
                AppColors::PRIMARY
            } else {
                Color::srgb(0.3, 0.3, 0.3)
            };
            row.spawn((
                FriedPrevPageButton,
                Button,
                Interaction::default(),
                Node {
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
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
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 页码
            row.spawn((
                FriedPageText,
                Text::new(format!("{} / {}", current_page, total_pages)),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));

            // 下一页
            let next_enabled = current_page < total_pages;
            let next_color = if next_enabled {
                AppColors::PRIMARY
            } else {
                Color::srgb(0.3, 0.3, 0.3)
            };
            row.spawn((
                FriedNextPageButton,
                Button,
                Interaction::default(),
                Node {
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
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
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 清理锅贴社区界面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_fried_ui(mut query: Query<&mut Node, With<FriedRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新锅贴社区 UI（数据变化时重建滚动容器内容）
pub fn refresh_fried_ui(
    mut commands: Commands,
    fried_state: Res<FriedState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<FriedScrollContainer>>,
) {
    if !fried_state.is_changed() {
        return;
    }

    // 跳过仅 is_loading 变化的场景
    let has_data = !fried_state.posts.is_empty();
    let has_error = fried_state.error.is_some();
    let is_loading = fried_state.is_loading;

    if is_loading && !has_data && !has_error {
        return;
    }

    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 清除滚动容器内的所有子元素
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // 重建滚动容器内容
    let font: Handle<Font> = get_font();
    commands.entity(scroll_entity).with_children(|content| {
        if is_loading {
            content.spawn((
                LoadingIndicator,
                Text::new("加载中..."),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        } else if let Some(ref error) = fried_state.error {
            content.spawn((
                ErrorMessage,
                Text::new(format!("加载失败: {}", error)),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.4)),
            ));
        } else if fried_state.posts.is_empty() {
            content.spawn((
                Text::new("暂无帖子"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        } else {
            for post in &fried_state.posts {
                spawn_fried_post_card(content, &font, post);
            }
            let total_pages = calculate_total_pages(&fried_state);
            if total_pages > 1 {
                spawn_fried_pagination(content, &font, fried_state.page + 1, total_pages);
            }
        }

        // 底部间距
        content.spawn(Node {
            height: Val::Px(30.0),
            min_height: Val::Px(30.0),
            ..default()
        });
    });
}

/// 刷新按钮交互
pub fn fried_refresh_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<FriedRefreshButton>)>,
    mut fried_state: ResMut<FriedState>,
    mut load_posts_messages: MessageWriter<LoadFriedPostsRequest>,
    mut load_apps_messages: MessageWriter<LoadAppsRequest>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed && !fried_state.is_loading {
            fried_state.page = 0;
            fried_state.posts.clear();
            fried_state.error = None;
            fried_state.is_loading = true;

            if fried_state.fried_token.is_none() {
                load_apps_messages.write(LoadAppsRequest);
            } else {
                load_posts_messages.write(LoadFriedPostsRequest { page: 0 });
            }
        }
    }
}

/// 分页按钮交互
pub fn fried_pagination_interaction(
    prev_query: Query<&Interaction, (Changed<Interaction>, With<FriedPrevPageButton>)>,
    next_query: Query<&Interaction, (Changed<Interaction>, With<FriedNextPageButton>)>,
    mut fried_state: ResMut<FriedState>,
    mut load_messages: MessageWriter<LoadFriedPostsRequest>,
) {
    let total_pages = calculate_total_pages(&fried_state);
    let current_display_page = fried_state.page + 1; // 显示从 1 开始

    // 上一页
    for interaction in &prev_query {
        if *interaction == Interaction::Pressed && current_display_page > 1 {
            fried_state.page -= 1;
            fried_state.posts.clear();
            fried_state.is_loading = true;
            fried_state.error = None;
            load_messages.write(LoadFriedPostsRequest {
                page: fried_state.page,
            });
        }
    }

    // 下一页
    for interaction in &next_query {
        if *interaction == Interaction::Pressed && current_display_page < total_pages {
            fried_state.page += 1;
            fried_state.posts.clear();
            fried_state.is_loading = true;
            fried_state.error = None;
            load_messages.write(LoadFriedPostsRequest {
                page: fried_state.page,
            });
        }
    }
}

/// 处理锅贴滚动
pub fn handle_fried_scroll(
    _scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<FriedScrollContainer>,
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新锅贴内容尺寸
pub fn update_fried_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<FriedScrollContainer>,
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

        // 加上间距
        let child_count = children.len();
        if child_count > 1 {
            content_height += (child_count - 1) as f32 * fried_layout::CARD_GAP;
        }

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

/// 处理小程序列表加载完成
pub fn handle_apps_loaded(
    mut loaded_messages: MessageReader<AppsLoadedEvent>,
    mut fried_state: ResMut<FriedState>,
    mut load_posts_messages: MessageWriter<LoadFriedPostsRequest>,
) {
    for event in loaded_messages.read() {
        fried_state.apps = event.apps.clone();
        tracing::info!("小程序列表加载完成: {} 个应用", fried_state.apps.len());

        // 加载完小程序列表后，直接尝试获取锅贴帖子
        // （锅贴 token 将在 api_plugin 中通过 PicACG token 换取）
        load_posts_messages.write(LoadFriedPostsRequest {
            page: fried_state.page,
        });
    }
}

/// 处理小程序列表加载失败
pub fn handle_apps_load_failed(
    mut failed_messages: MessageReader<AppsLoadFailedEvent>,
    mut fried_state: ResMut<FriedState>,
) {
    for event in failed_messages.read() {
        fried_state.is_loading = false;
        fried_state.error = Some(event.error.clone());
        tracing::warn!("小程序列表加载失败: {}", event.error);
    }
}

/// 处理锅贴帖子列表加载完成
pub fn handle_fried_posts_loaded(
    mut loaded_messages: MessageReader<FriedPostsLoadedEvent>,
    mut fried_state: ResMut<FriedState>,
) {
    for event in loaded_messages.read() {
        fried_state.posts = event.posts.clone();
        fried_state.total = event.total;
        fried_state.limit = event.limit.max(1);
        fried_state.is_loading = false;
        fried_state.error = None;
        tracing::info!(
            "锅贴帖子加载完成: {} 个, 总计 {}",
            fried_state.posts.len(),
            fried_state.total
        );
    }
}

/// 处理锅贴帖子列表加载失败
pub fn handle_fried_posts_load_failed(
    mut failed_messages: MessageReader<FriedPostsLoadFailedEvent>,
    mut fried_state: ResMut<FriedState>,
) {
    for event in failed_messages.read() {
        fried_state.is_loading = false;
        fried_state.error = Some(event.error.clone());
        tracing::warn!("锅贴帖子加载失败: {}", event.error);
    }
}
