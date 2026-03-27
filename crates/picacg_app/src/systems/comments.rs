//! 评论系统
//!
//! 实现漫画评论页面的 UI 和交互

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::scrollbar_config::*,
        ui_common::{Scrollable, spawn_scrollbar},
    },
    utils::icons::*,
};

/// 滚动条宽度
const SCROLLBAR_WIDTH_PX: f32 = 12.0;

// ==================== 组件定义 ====================

/// 评论页面根节点
#[derive(Component)]
pub struct CommentsRoot;

/// 评论滚动容器
#[derive(Component)]
pub struct CommentsScrollContainer;

/// 评论项
#[derive(Component)]
pub struct CommentItem {
    #[allow(dead_code)]
    pub comment_id: String,
}

/// 评论点赞按钮
#[derive(Component)]
pub struct CommentLikeButton {
    pub comment_id: String,
}

/// 评论回复按钮
#[derive(Component)]
pub struct CommentReplyButton {
    pub comment_id: String,
    pub user_name: String,
}

/// 子评论容器
#[derive(Component)]
pub struct CommentChildrenContainer {
    #[allow(dead_code)]
    pub comment_id: String,
}

/// 展开子评论按钮
#[derive(Component)]
pub struct ExpandChildrenButton {
    pub comment_id: String,
    #[allow(dead_code)]
    pub total_children: i64,
}

/// 加载更多子评论按钮
#[derive(Component)]
pub struct LoadMoreChildrenButton {
    pub comment_id: String,
}

/// 评论输入框容器
#[derive(Component)]
pub struct CommentInputContainer;

/// 评论输入框
#[derive(Component)]
pub struct CommentInputField;

/// 评论输入框文本
#[derive(Component)]
pub struct CommentInputText;

/// 评论发送按钮
#[derive(Component)]
pub struct CommentSendButton;

/// 回复提示文本
#[derive(Component)]
pub struct CommentReplyHint;

/// 取消回复按钮
#[derive(Component)]
pub struct CancelReplyButton;

/// 评论返回按钮
#[derive(Component)]
pub struct CommentsBackButton;

/// 评论页面标题文本
#[derive(Component)]
pub struct CommentsTitleText;

/// 评论分页标记类型（预留，后续接入分页组件时使用）
#[allow(dead_code)]
pub struct CommentsPage;

/// 评论点赞数文本
#[derive(Component)]
pub struct CommentLikesText {
    #[allow(dead_code)]
    pub comment_id: String,
}

// ==================== 页面生命周期 ====================

/// 创建评论页面 UI
pub fn setup_comments_ui(
    mut commands: Commands,
    comments_state: Res<CommentsState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut load_messages: MessageWriter<LoadCommentsRequest>,
    existing_query: Query<Entity, With<CommentsRoot>>,
) {
    // 每次进入评论页面都销毁旧的重建（不同漫画的评论数据不同，不适合缓存）
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let comments_root = commands
        .spawn((
            CommentsRoot,
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
                // 返回按钮
                header
                    .spawn((
                        CommentsBackButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_CHEVRON_LEFT),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                header.spawn((
                    CommentsTitleText,
                    Text::new("评论"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // 内容区域（可滚动）
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|wrapper| {
                let scroll_container_id = wrapper
                    .spawn((
                        CommentsScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect {
                                left: Val::Px(15.0),
                                right: Val::Px(15.0 + SCROLLBAR_WIDTH_PX),
                                top: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                            },
                            overflow: Overflow::scroll_y(),
                            row_gap: Val::Px(0.0),
                            ..default()
                        },
                        Scrollable,
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|content| {
                        if comments_state.is_loading {
                            content.spawn((
                                LoadingIndicator,
                                Text::new("加载评论中..."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        } else if comments_state.comments.is_empty() {
                            content.spawn((
                                Text::new("暂无评论"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        }
                    })
                    .id();

                spawn_scrollbar(wrapper, scroll_container_id);
            });

            // 底部输入栏（固定不滚动）
            spawn_comment_input_bar(root, &font, &comments_state);
        })
        .id();

    // 挂载到 ContentArea
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(comments_root);
    }

    // 触发加载评论
    if !comments_state.comic_id.is_empty() && comments_state.comments.is_empty() {
        load_messages.write(LoadCommentsRequest {
            comic_id: comments_state.comic_id.clone(),
            page: 1,
        });
    }
}

/// 创建底部输入栏
fn spawn_comment_input_bar(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    comments_state: &CommentsState,
) {
    parent
        .spawn((
            CommentInputContainer,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(Val::Px(15.0), Val::Px(15.0), Val::Px(8.0), Val::Px(8.0)),
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(AppColors::BORDER),
            BackgroundColor(AppColors::SURFACE),
        ))
        .with_children(|bar| {
            // 回复提示行（有回复目标时显示）
            let reply_display = if comments_state.reply_to.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            let reply_text = if let Some(ref name) = comments_state.reply_to_name {
                format!("回复 @{}", name)
            } else {
                String::new()
            };

            bar.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::bottom(Val::Px(6.0)),
                display: reply_display,
                ..default()
            })
            .with_children(|hint_row| {
                hint_row.spawn((
                    CommentReplyHint,
                    Text::new(reply_text),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::PRIMARY),
                ));

                hint_row
                    .spawn((
                        CancelReplyButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("取消"),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });
            });

            // 输入行
            bar.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|input_row| {
                // 输入框
                let display_text = if comments_state.input_text.is_empty() {
                    "写下你的评论..."
                } else {
                    &comments_state.input_text
                };
                let text_color = if comments_state.input_text.is_empty() {
                    AppColors::TEXT_SECONDARY
                } else {
                    AppColors::TEXT
                };
                let border_color = if comments_state.input_focused {
                    AppColors::PRIMARY
                } else {
                    AppColors::BORDER
                };

                input_row
                    .spawn((
                        CommentInputField,
                        Button,
                        Interaction::default(),
                        Node {
                            flex_grow: 1.0,
                            height: Val::Px(36.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(AppColors::BACKGROUND),
                        BorderColor::all(border_color),
                    ))
                    .with_children(|field| {
                        field.spawn((
                            CommentInputText,
                            Text::new(display_text),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(text_color),
                        ));
                    });

                // 发送按钮
                let send_enabled = !comments_state.input_text.trim().is_empty();
                let send_color = if send_enabled {
                    AppColors::PRIMARY
                } else {
                    AppColors::SECONDARY
                };

                input_row
                    .spawn((
                        CommentSendButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(send_color),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("发送"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });
            });
        });
}

/// 清理评论页面（隐藏而非销毁）
pub fn cleanup_comments_ui(
    mut commands: Commands,
    query: Query<Entity, With<CommentsRoot>>,
    mut comments_state: ResMut<CommentsState>,
) {
    // 清理输入状态
    comments_state.input_focused = false;
    // 销毁评论页面 UI（参数化页面，每次进入数据不同，不适合缓存）
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ==================== 刷新 UI ====================

/// 刷新评论列表 UI（当状态变化时）
pub fn refresh_comments_ui(
    mut commands: Commands,
    mut comments_state: ResMut<CommentsState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<CommentsScrollContainer>>,
    _comment_item_query: Query<&CommentItem>,
    // 底部栏需要更新
    reply_hint_query: Query<Entity, With<CommentReplyHint>>,
    input_text_query: Query<Entity, With<CommentInputText>>,
    input_field_query: Query<Entity, With<CommentInputField>>,
) {
    if !comments_state.is_changed() || !comments_state.needs_rebuild {
        return;
    }
    comments_state.needs_rebuild = false;

    let font: Handle<Font> = get_font();

    // 检查滚动容器是否存在
    let Ok((container_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 清除滚动容器的所有子实体
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // 重新构建评论列表
    let state_snapshot = CommentsStateSnapshot {
        is_loading: comments_state.is_loading,
        error: comments_state.error.clone(),
        comments: comments_state.comments.clone(),
        page: comments_state.page,
        total_pages: comments_state.total_pages,
        children_map: comments_state.children_map.clone(),
    };

    commands.entity(container_entity).with_children(|content| {
        build_comments_content(content, &font, &state_snapshot);
    });

    // 更新底部栏的回复提示
    update_reply_hint_ui(
        &mut commands,
        &comments_state,
        &font,
        &reply_hint_query,
        &input_text_query,
        &input_field_query,
    );
}

/// 评论状态快照（避免借用冲突）
struct CommentsStateSnapshot {
    is_loading: bool,
    error: Option<String>,
    comments: Vec<picacg_api::models::Comment>,
    page: i32,
    total_pages: i32,
    children_map: std::collections::HashMap<String, ChildCommentsState>,
}

/// 构建评论列表内容
fn build_comments_content(
    content: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    state: &CommentsStateSnapshot,
) {
    if state.is_loading && state.comments.is_empty() {
        content.spawn((
            LoadingIndicator,
            Text::new("加载评论中..."),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
        ));
        return;
    }

    if let Some(ref error) = state.error {
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
        return;
    }

    if state.comments.is_empty() {
        content.spawn((
            Text::new("暂无评论，来发表第一条评论吧"),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
            Node {
                margin: UiRect::vertical(Val::Px(20.0)),
                ..default()
            },
        ));
        return;
    }

    // 页码信息
    if state.total_pages > 1 {
        content.spawn((
            Text::new(format!("第 {} / {} 页", state.page, state.total_pages)),
            TextFont {
                font: font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));
    }

    // 渲染评论列表
    for comment in &state.comments {
        spawn_comment_item(content, font, comment, &state.children_map);
    }

    // 分页按钮
    if state.total_pages > 1 {
        content
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                margin: UiRect::vertical(Val::Px(15.0)),
                ..default()
            })
            .with_children(|row| {
                // 上一页
                let prev_enabled = state.page > 1;
                let prev_color = if prev_enabled {
                    AppColors::SECONDARY
                } else {
                    Color::srgb(0.2, 0.2, 0.25)
                };
                row.spawn((
                    CommentsPrevPageButton,
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(32.0),
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
                        TextColor(if prev_enabled {
                            AppColors::TEXT
                        } else {
                            AppColors::TEXT_SECONDARY
                        }),
                    ));
                });

                // 页码
                row.spawn((
                    Text::new(format!("{} / {}", state.page, state.total_pages)),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 下一页
                let next_enabled = state.page < state.total_pages;
                let next_color = if next_enabled {
                    AppColors::SECONDARY
                } else {
                    Color::srgb(0.2, 0.2, 0.25)
                };
                row.spawn((
                    CommentsNextPageButton,
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(32.0),
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
                        TextColor(if next_enabled {
                            AppColors::TEXT
                        } else {
                            AppColors::TEXT_SECONDARY
                        }),
                    ));
                });
            });
    }

    // 底部间距
    content.spawn(Node {
        height: Val::Px(20.0),
        min_height: Val::Px(20.0),
        ..default()
    });
}

/// 上一页按钮
#[derive(Component)]
pub struct CommentsPrevPageButton;

/// 下一页按钮
#[derive(Component)]
pub struct CommentsNextPageButton;

/// 渲染单条评论
fn spawn_comment_item(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    comment: &picacg_api::models::Comment,
    children_map: &std::collections::HashMap<String, ChildCommentsState>,
) {
    // 置顶标记
    let is_top = comment.is_top.unwrap_or(false);

    parent
        .spawn((
            CommentItem {
                comment_id: comment.id.clone(),
            },
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect::bottom(Val::Px(2.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.5)),
            BackgroundColor(Color::NONE),
        ))
        .with_children(|item| {
            // 第一行：用户名 + 等级 + 时间
            item.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            })
            .with_children(|header| {
                // 置顶标记
                if is_top {
                    header
                        .spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.8, 0.2, 0.2, 0.6)),
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                Text::new("置顶"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.8, 0.8)),
                            ));
                        });
                }

                // 用户名
                header.spawn((
                    Text::new(&comment.user.name),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::PRIMARY),
                ));

                // 等级
                header
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.5, 0.8, 0.4)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(format!("Lv.{}", comment.user.level)),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.85, 1.0)),
                        ));
                    });

                // 称号
                if !comment.user.title.is_empty() {
                    header
                        .spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.6, 0.4, 0.8, 0.3)),
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                Text::new(&comment.user.title),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.7, 1.0)),
                            ));
                        });
                }

                // 弹性间距
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // 时间
                let date = comment
                    .created_at
                    .split('T')
                    .next()
                    .unwrap_or(&comment.created_at);
                header.spawn((
                    Text::new(date),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });

            // 评论内容
            if comment.hide {
                item.spawn((
                    Text::new("[该评论已被隐藏]"),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));
            } else {
                item.spawn((
                    Text::new(&comment.content),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        max_width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }

            // 操作栏：点赞 + 回复 + 查看子评论
            item.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            })
            .with_children(|actions| {
                // 点赞按钮
                let is_liked = comment.is_liked.unwrap_or(false);
                let like_color = if is_liked {
                    Color::srgb(1.0, 0.4, 0.4)
                } else {
                    AppColors::TEXT_SECONDARY
                };

                actions
                    .spawn((
                        CommentLikeButton {
                            comment_id: comment.id.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_HEART),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(like_color),
                        ));
                        btn.spawn((
                            CommentLikesText {
                                comment_id: comment.id.clone(),
                            },
                            Text::new(format!("{}", comment.likes_count)),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(like_color),
                        ));
                    });

                // 回复按钮
                actions
                    .spawn((
                        CommentReplyButton {
                            comment_id: comment.id.clone(),
                            user_name: comment.user.name.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("回复"),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });

                // 查看子评论按钮（如果有子评论）
                if comment.comments_count > 0 {
                    let child_state = children_map.get(&comment.id);
                    let is_expanded = child_state.map(|s| !s.comments.is_empty()).unwrap_or(false);

                    let btn_text = if is_expanded {
                        format!("收起回复 ({})", comment.comments_count)
                    } else {
                        format!("查看 {} 条回复", comment.comments_count)
                    };

                    actions
                        .spawn((
                            ExpandChildrenButton {
                                comment_id: comment.id.clone(),
                                total_children: comment.comments_count,
                            },
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|btn| {
                            let icon = if is_expanded {
                                ICON_CHEVRON_UP
                            } else {
                                ICON_CHEVRON_DOWN
                            };
                            btn.spawn((
                                Text::new(format!("{} {}", icon, btn_text)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(AppColors::PRIMARY),
                            ));
                        });
                }
            });

            // 子评论区域
            if let Some(child_state) = children_map.get(&comment.id)
                && !child_state.comments.is_empty()
            {
                item.spawn((
                    CommentChildrenContainer {
                        comment_id: comment.id.clone(),
                    },
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::top(Val::Px(8.0)),
                        padding: UiRect::left(Val::Px(16.0)),
                        border: UiRect::left(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.4)),
                ))
                .with_children(|children_container| {
                    for child_comment in &child_state.comments {
                        spawn_child_comment_item(children_container, font, child_comment);
                    }

                    // 子评论加载指示器
                    if child_state.is_loading {
                        children_container.spawn((
                            Text::new("加载中..."),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                            Node {
                                margin: UiRect::vertical(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                    }

                    // 加载更多子评论按钮
                    if child_state.page < child_state.total_pages && !child_state.is_loading {
                        children_container
                            .spawn((
                                LoadMoreChildrenButton {
                                    comment_id: comment.id.clone(),
                                },
                                Button,
                                Interaction::default(),
                                Node {
                                    padding: UiRect::vertical(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("加载更多回复..."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::PRIMARY),
                                ));
                            });
                    }
                });
            }
        });
}

/// 渲染子评论
fn spawn_child_comment_item(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    comment: &picacg_api::models::Comment,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(6.0), Val::Px(6.0)),
            ..default()
        })
        .with_children(|item| {
            // 用户名 + 时间
            item.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new(&comment.user.name),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::PRIMARY),
                ));

                header
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(3.0), Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.5, 0.8, 0.3)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(format!("Lv.{}", comment.user.level)),
                            TextFont {
                                font: font.clone(),
                                font_size: 9.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.85, 1.0)),
                        ));
                    });

                // 弹性间距
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                let date = comment
                    .created_at
                    .split('T')
                    .next()
                    .unwrap_or(&comment.created_at);
                header.spawn((
                    Text::new(date),
                    TextFont {
                        font: font.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });

            // 内容
            if comment.hide {
                item.spawn((
                    Text::new("[该回复已被隐藏]"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            } else {
                item.spawn((
                    Text::new(&comment.content),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                    Node {
                        max_width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }

            // 子评论的点赞
            item.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            })
            .with_children(|actions| {
                let is_liked = comment.is_liked.unwrap_or(false);
                let like_color = if is_liked {
                    Color::srgb(1.0, 0.4, 0.4)
                } else {
                    AppColors::TEXT_SECONDARY
                };

                actions
                    .spawn((
                        CommentLikeButton {
                            comment_id: comment.id.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_HEART),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(like_color),
                        ));
                        btn.spawn((
                            CommentLikesText {
                                comment_id: comment.id.clone(),
                            },
                            Text::new(format!("{}", comment.likes_count)),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(like_color),
                        ));
                    });

                // 回复按钮
                actions
                    .spawn((
                        CommentReplyButton {
                            comment_id: comment.id.clone(),
                            user_name: comment.user.name.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node {
                            margin: UiRect::left(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("回复"),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });
            });
        });
}

/// 更新底部栏的回复提示
fn update_reply_hint_ui(
    commands: &mut Commands,
    comments_state: &CommentsState,
    _font: &Handle<Font>,
    reply_hint_query: &Query<Entity, With<CommentReplyHint>>,
    input_text_query: &Query<Entity, With<CommentInputText>>,
    _input_field_query: &Query<Entity, With<CommentInputField>>,
) {
    // 更新回复提示文本
    for entity in reply_hint_query.iter() {
        if let Some(ref name) = comments_state.reply_to_name {
            commands
                .entity(entity)
                .insert(Text::new(format!("回复 @{}", name)));
        } else {
            commands.entity(entity).insert(Text::new(" "));
        }
    }

    // 更新回复提示行的显示/隐藏
    // 回复提示行是 reply_hint 的父节点，通过 Parent 组件找到
    // 这里简单处理：需要通过 needs_rebuild 触发全量刷新

    // 更新输入框文本
    for entity in input_text_query.iter() {
        let display_text = if comments_state.input_text.is_empty() {
            "写下你的评论...".to_string()
        } else {
            comments_state.input_text.clone()
        };
        let text_color = if comments_state.input_text.is_empty() {
            AppColors::TEXT_SECONDARY
        } else {
            AppColors::TEXT
        };
        commands.entity(entity).insert(Text::new(display_text));
        commands.entity(entity).insert(TextColor(text_color));
    }
}

// ==================== 交互系统 ====================

/// 返回按钮交互
pub fn comments_back_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CommentsBackButton>),
    >,
    mut navigate_back_messages: MessageWriter<NavigateBackEvent>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                navigate_back_messages.write(NavigateBackEvent);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.30));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 评论点赞交互
pub fn comment_like_interaction(
    mut interaction_query: Query<
        (&Interaction, &CommentLikeButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut like_messages: MessageWriter<LikeCommentRequestEvent>,
) {
    for (interaction, like_btn, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                like_messages.write(LikeCommentRequestEvent {
                    comment_id: like_btn.comment_id.clone(),
                });
                tracing::info!("点赞评论: {}", like_btn.comment_id);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 评论回复按钮交互
pub fn comment_reply_interaction(
    mut interaction_query: Query<
        (&Interaction, &CommentReplyButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut comments_state: ResMut<CommentsState>,
) {
    for (interaction, reply_btn, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                comments_state.reply_to = Some(reply_btn.comment_id.clone());
                comments_state.reply_to_name = Some(reply_btn.user_name.clone());
                comments_state.needs_rebuild = true;
                comments_state.input_focused = true;
                tracing::info!(
                    "回复评论: {} (@{})",
                    reply_btn.comment_id,
                    reply_btn.user_name
                );
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 取消回复交互
pub fn cancel_reply_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CancelReplyButton>),
    >,
    mut comments_state: ResMut<CommentsState>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                comments_state.reply_to = None;
                comments_state.reply_to_name = None;
                comments_state.needs_rebuild = true;
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 展开/折叠子评论交互
pub fn expand_children_interaction(
    mut interaction_query: Query<
        (&Interaction, &ExpandChildrenButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut comments_state: ResMut<CommentsState>,
    mut load_children_messages: MessageWriter<LoadChildCommentsRequest>,
) {
    for (interaction, expand_btn, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                let comment_id = &expand_btn.comment_id;

                // 检查是否已展开
                let is_expanded = comments_state
                    .children_map
                    .get(comment_id)
                    .map(|s| !s.comments.is_empty())
                    .unwrap_or(false);

                if is_expanded {
                    // 折叠：清空子评论
                    comments_state.children_map.remove(comment_id);
                    comments_state.needs_rebuild = true;
                } else {
                    // 展开：加载子评论
                    comments_state.children_map.insert(
                        comment_id.clone(),
                        ChildCommentsState {
                            is_loading: true,
                            page: 1,
                            ..Default::default()
                        },
                    );
                    comments_state.needs_rebuild = true;

                    load_children_messages.write(LoadChildCommentsRequest {
                        comment_id: comment_id.clone(),
                        page: 1,
                    });
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 加载更多子评论交互
pub fn load_more_children_interaction(
    mut interaction_query: Query<
        (&Interaction, &LoadMoreChildrenButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut comments_state: ResMut<CommentsState>,
    mut load_children_messages: MessageWriter<LoadChildCommentsRequest>,
) {
    for (interaction, btn, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Some(child_state) = comments_state.children_map.get_mut(&btn.comment_id) {
                    let next_page = child_state.page + 1;
                    child_state.is_loading = true;

                    load_children_messages.write(LoadChildCommentsRequest {
                        comment_id: btn.comment_id.clone(),
                        page: next_page,
                    });
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 发送评论交互
pub fn comment_send_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CommentSendButton>),
    >,
    mut comments_state: ResMut<CommentsState>,
    mut post_comment_messages: MessageWriter<PostCommentRequest>,
    mut post_reply_messages: MessageWriter<PostCommentReplyRequest>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                let content = comments_state.input_text.trim().to_string();
                if content.is_empty() {
                    return;
                }

                if let Some(ref comment_id) = comments_state.reply_to {
                    // 回复评论
                    post_reply_messages.write(PostCommentReplyRequest {
                        comment_id: comment_id.clone(),
                        content: content.clone(),
                    });
                    tracing::info!("发送回复: comment_id={}", comment_id);
                } else {
                    // 发表顶层评论
                    post_comment_messages.write(PostCommentRequest {
                        comic_id: comments_state.comic_id.clone(),
                        content: content.clone(),
                    });
                    tracing::info!("发表评论: comic_id={}", comments_state.comic_id);
                }

                // 清空输入
                comments_state.input_text.clear();
                comments_state.reply_to = None;
                comments_state.reply_to_name = None;
                comments_state.needs_rebuild = true;
            }
            Interaction::Hovered => {
                if !comments_state.input_text.trim().is_empty() {
                    *bg_color = BackgroundColor(Color::srgb(0.35, 0.55, 0.85));
                }
            }
            Interaction::None => {
                let send_enabled = !comments_state.input_text.trim().is_empty();
                *bg_color = if send_enabled {
                    BackgroundColor(AppColors::PRIMARY)
                } else {
                    BackgroundColor(AppColors::SECONDARY)
                };
            }
        }
    }
}

/// 输入框点击交互
pub fn comment_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<CommentInputField>),
    >,
    mut comments_state: ResMut<CommentsState>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for (interaction, mut border_color) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            comments_state.input_focused = true;
            *border_color = BorderColor::all(AppColors::PRIMARY);

            // 启用 IME
            if let Ok(mut window) = window_query.single_mut() {
                window.ime_enabled = true;
            }
        }
    }
}

/// 输入框失焦处理（点击输入框和发送按钮以外的区域）
pub fn unfocus_comment_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut comments_state: ResMut<CommentsState>,
    input_query: Query<&Interaction, With<CommentInputField>>,
    send_query: Query<&Interaction, With<CommentSendButton>>,
    cancel_query: Query<&Interaction, With<CancelReplyButton>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut border_query: Query<&mut BorderColor, With<CommentInputField>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // 检查是否点击了输入框、发送按钮或取消按钮
    let input_pressed = input_query
        .iter()
        .any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);
    let send_pressed = send_query
        .iter()
        .any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);
    let cancel_pressed = cancel_query
        .iter()
        .any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);

    if !input_pressed && !send_pressed && !cancel_pressed && comments_state.input_focused {
        comments_state.input_focused = false;

        // 禁用 IME
        if let Ok(mut window) = window_query.single_mut() {
            window.ime_enabled = false;
        }

        // 恢复边框颜色
        for mut border_color in border_query.iter_mut() {
            *border_color = BorderColor::all(AppColors::BORDER);
        }
    }
}

/// 评论输入框键盘输入
pub fn comment_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut comments_state: ResMut<CommentsState>,
    mut input_text_query: Query<(&mut Text, &mut TextColor), With<CommentInputText>>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if !comments_state.input_focused {
        // 消费事件避免累积
        for _ in keyboard_events.read() {}
        return;
    }

    use bevy::input::{ButtonState, keyboard::Key};

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Backspace => {
                comments_state.input_text.pop();
                update_comment_input_display(&comments_state, &mut input_text_query);
            }
            Key::Character(input) => {
                // 检查是否是粘贴操作（Ctrl+V / Cmd+V）
                let ctrl_or_cmd = key_input.pressed(KeyCode::ControlLeft)
                    || key_input.pressed(KeyCode::ControlRight)
                    || key_input.pressed(KeyCode::SuperLeft)
                    || key_input.pressed(KeyCode::SuperRight);

                if ctrl_or_cmd && (input.as_str() == "v" || input.as_str() == "V") {
                    // 粘贴由系统处理
                    continue;
                }

                // 忽略其他 Ctrl 组合键
                if ctrl_or_cmd {
                    continue;
                }

                for c in input.chars() {
                    comments_state.input_text.push(c);
                }
                update_comment_input_display(&comments_state, &mut input_text_query);
            }
            _ => {}
        }
    }
}

/// IME 输入处理
pub fn comment_ime_input(
    mut ime_events: MessageReader<bevy::window::Ime>,
    mut comments_state: ResMut<CommentsState>,
    mut input_text_query: Query<(&mut Text, &mut TextColor), With<CommentInputText>>,
) {
    if !comments_state.input_focused {
        for _ in ime_events.read() {}
        return;
    }

    for event in ime_events.read() {
        if let bevy::window::Ime::Commit { value, .. } = event {
            comments_state.input_text.push_str(value);
            update_comment_input_display(&comments_state, &mut input_text_query);
        }
    }
}

/// 更新输入框文本显示
fn update_comment_input_display(
    comments_state: &CommentsState,
    input_text_query: &mut Query<(&mut Text, &mut TextColor), With<CommentInputText>>,
) {
    for (mut text, mut text_color) in input_text_query.iter_mut() {
        if comments_state.input_text.is_empty() {
            *text = Text::new("写下你的评论...");
            *text_color = TextColor(AppColors::TEXT_SECONDARY);
        } else {
            *text = Text::new(&comments_state.input_text);
            *text_color = TextColor(AppColors::TEXT);
        }
    }
}

/// 评论分页交互
pub fn comments_pagination_interaction(
    mut prev_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CommentsPrevPageButton>),
    >,
    mut next_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<CommentsNextPageButton>,
            Without<CommentsPrevPageButton>,
        ),
    >,
    mut comments_state: ResMut<CommentsState>,
    mut load_messages: MessageWriter<LoadCommentsRequest>,
    mut scroll_query: Query<&mut ScrollPosition, With<CommentsScrollContainer>>,
) {
    // 上一页
    for (interaction, mut bg_color) in prev_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if comments_state.page > 1 {
                    let new_page = comments_state.page - 1;
                    comments_state.page = new_page;
                    comments_state.is_loading = true;
                    comments_state.comments.clear();
                    comments_state.children_map.clear();
                    comments_state.needs_rebuild = true;

                    // 重置滚动位置
                    for mut scroll_pos in scroll_query.iter_mut() {
                        scroll_pos.y = 0.0;
                    }

                    load_messages.write(LoadCommentsRequest {
                        comic_id: comments_state.comic_id.clone(),
                        page: new_page,
                    });
                }
            }
            Interaction::Hovered => {
                if comments_state.page > 1 {
                    *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.40));
                }
            }
            Interaction::None => {
                *bg_color = if comments_state.page > 1 {
                    BackgroundColor(AppColors::SECONDARY)
                } else {
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25))
                };
            }
        }
    }

    // 下一页
    for (interaction, mut bg_color) in next_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if comments_state.page < comments_state.total_pages {
                    let new_page = comments_state.page + 1;
                    comments_state.page = new_page;
                    comments_state.is_loading = true;
                    comments_state.comments.clear();
                    comments_state.children_map.clear();
                    comments_state.needs_rebuild = true;

                    // 重置滚动位置
                    for mut scroll_pos in scroll_query.iter_mut() {
                        scroll_pos.y = 0.0;
                    }

                    load_messages.write(LoadCommentsRequest {
                        comic_id: comments_state.comic_id.clone(),
                        page: new_page,
                    });
                }
            }
            Interaction::Hovered => {
                if comments_state.page < comments_state.total_pages {
                    *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.40));
                }
            }
            Interaction::None => {
                *bg_color = if comments_state.page < comments_state.total_pages {
                    BackgroundColor(AppColors::SECONDARY)
                } else {
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25))
                };
            }
        }
    }
}

/// 处理评论加载完成
pub fn handle_comments_loaded(
    mut comments_state: ResMut<CommentsState>,
    mut loaded_messages: MessageReader<CommentsLoadedEvent>,
    mut failed_messages: MessageReader<CommentsLoadFailedEvent>,
) {
    for event in loaded_messages.read() {
        comments_state.is_loading = false;
        comments_state.comments = event.comments.clone();
        comments_state.page = event.page;
        comments_state.total_pages = event.total_pages;
        comments_state.error = None;
        comments_state.needs_rebuild = true;
        tracing::info!(
            "评论加载完成: {} 条, 第 {}/{} 页",
            comments_state.comments.len(),
            event.page,
            event.total_pages
        );
    }

    for event in failed_messages.read() {
        comments_state.is_loading = false;
        comments_state.error = Some(event.error.clone());
        comments_state.needs_rebuild = true;
        tracing::warn!("评论加载失败: {}", event.error);
    }
}

/// 处理子评论加载完成
pub fn handle_child_comments_loaded(
    mut comments_state: ResMut<CommentsState>,
    mut loaded_messages: MessageReader<ChildCommentsLoadedEvent>,
) {
    for event in loaded_messages.read() {
        if let Some(child_state) = comments_state.children_map.get_mut(&event.comment_id) {
            child_state.is_loading = false;
            child_state.total_pages = event.total_pages;
            child_state.page = event.page;

            if event.page == 1 {
                // 首次加载
                child_state.comments = event.comments.clone();
            } else {
                // 追加加载
                child_state.comments.extend(event.comments.clone());
            }
        }
        comments_state.needs_rebuild = true;
    }
}

/// 处理发表评论响应
pub fn handle_post_comment_response(
    mut comments_state: ResMut<CommentsState>,
    mut post_messages: MessageReader<PostCommentResponseEvent>,
    mut post_reply_messages: MessageReader<PostCommentReplyResponseEvent>,
    mut load_messages: MessageWriter<LoadCommentsRequest>,
) {
    let mut should_reload = false;

    for event in post_messages.read() {
        if event.success {
            tracing::info!("评论发表成功");
            should_reload = true;
        } else if let Some(ref error) = event.error {
            tracing::warn!("评论发表失败: {}", error);
            comments_state.error = Some(error.clone());
            comments_state.needs_rebuild = true;
        }
    }

    for event in post_reply_messages.read() {
        if event.success {
            tracing::info!("回复发表成功");
            should_reload = true;
        } else if let Some(ref error) = event.error {
            tracing::warn!("回复发表失败: {}", error);
            comments_state.error = Some(error.clone());
            comments_state.needs_rebuild = true;
        }
    }

    // 发表成功后重新加载当前页
    if should_reload {
        comments_state.children_map.clear();
        load_messages.write(LoadCommentsRequest {
            comic_id: comments_state.comic_id.clone(),
            page: comments_state.page,
        });
    }
}

/// 处理点赞评论响应
pub fn handle_like_comment_response(
    mut comments_state: ResMut<CommentsState>,
    mut like_messages: MessageReader<LikeCommentResponseEvent>,
) {
    for event in like_messages.read() {
        let is_like = event.action == "like";
        tracing::info!("评论点赞: {} -> {}", event.comment_id, event.action);

        // 更新主评论列表中的点赞状态
        for comment in comments_state.comments.iter_mut() {
            if comment.id == event.comment_id {
                comment.is_liked = Some(is_like);
                if is_like {
                    comment.likes_count += 1;
                } else {
                    comment.likes_count = (comment.likes_count - 1).max(0);
                }
                break;
            }
        }

        // 更新子评论中的点赞状态
        for (_parent_id, child_state) in comments_state.children_map.iter_mut() {
            for child_comment in child_state.comments.iter_mut() {
                if child_comment.id == event.comment_id {
                    child_comment.is_liked = Some(is_like);
                    if is_like {
                        child_comment.likes_count += 1;
                    } else {
                        child_comment.likes_count = (child_comment.likes_count - 1).max(0);
                    }
                    break;
                }
            }
        }

        comments_state.needs_rebuild = true;
    }
}

// ==================== 滚动系统 ====================

/// 处理评论页面滚动
pub fn handle_comments_scroll(
    _scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<CommentsScrollContainer>,
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新评论内容尺寸
pub fn update_comments_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<CommentsScrollContainer>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    const SCROLL_PADDING_VERTICAL: f32 = 20.0;

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        let mut content_height = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(child) {
                content_height += child_computed.size().y / scale_factor;
            }
        }

        content_height += SCROLL_PADDING_VERTICAL;

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}
