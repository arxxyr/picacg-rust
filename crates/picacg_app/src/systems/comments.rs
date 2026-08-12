//! 评论系统
//!
//! 实现漫画评论页面的 UI 和交互

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    ui::RelativeCursorPosition,
};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        pagination::{Pagination, PaginationControl, pagination_controls},
        scrollbar::{ScrollArea, scrollbar},
        widgets::{ButtonStyle, ButtonVariant},
    },
    utils::{
        icons::*,
        text_input::{TextInput, TextInputDisplay},
    },
};

/// 滚动条宽度
const SCROLLBAR_WIDTH_PX: f32 = 12.0;

// ==================== 组件定义 ====================

/// 评论页面根节点
#[derive(Component, Default, Clone)]
pub struct CommentsRoot;

/// 评论滚动容器
#[derive(Component, Default, Clone)]
pub struct CommentsScrollContainer;

/// 评论项
#[derive(Component, Default, Clone)]
pub struct CommentItem {
    #[allow(dead_code)]
    pub comment_id: String,
}

/// 评论点赞按钮
#[derive(Component, Default, Clone)]
pub struct CommentLikeButton {
    pub comment_id: String,
}

/// 评论回复按钮
#[derive(Component, Default, Clone)]
pub struct CommentReplyButton {
    pub comment_id: String,
    pub user_name: String,
}

/// 子评论容器
#[derive(Component, Default, Clone)]
pub struct CommentChildrenContainer {
    #[allow(dead_code)]
    pub comment_id: String,
}

/// 展开子评论按钮
#[derive(Component, Default, Clone)]
pub struct ExpandChildrenButton {
    pub comment_id: String,
    #[allow(dead_code)]
    pub total_children: i64,
}

/// 加载更多子评论按钮
#[derive(Component, Default, Clone)]
pub struct LoadMoreChildrenButton {
    pub comment_id: String,
}

/// 评论输入框容器
#[derive(Component, Default, Clone)]
pub struct CommentInputContainer;

/// 评论输入框（配合通用 `TextInput` 使用）
#[derive(Component, Default, Clone)]
pub struct CommentInputField;

/// 评论发送按钮
#[derive(Component, Default, Clone)]
pub struct CommentSendButton;

/// 回复提示行（整行显隐随回复目标切换）
#[derive(Component, Default, Clone)]
pub struct CommentReplyRow;

/// 回复提示文本
#[derive(Component, Default, Clone)]
pub struct CommentReplyHint;

/// 取消回复按钮
#[derive(Component, Default, Clone)]
pub struct CancelReplyButton;

/// 评论返回按钮
#[derive(Component, Default, Clone)]
pub struct CommentsBackButton;

/// 评论页面标题文本
#[derive(Component, Default, Clone)]
pub struct CommentsTitleText;

/// 评论页面标记类型（用于分页组件的泛型参数）
pub struct CommentsPage;

/// 评论点赞数文本
#[derive(Component, Default, Clone)]
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

    let content_area = content_area_query.single().ok();

    let comments_root = commands.spawn_scene(comments_page(&comments_state)).id();

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

/// 评论页面场景
fn comments_page(comments_state: &CommentsState) -> impl Scene + use<> {
    // 滚动区初始占位内容：加载指示器 / 空提示 / 两者皆无
    let content_placeholder: Box<dyn SceneList> = if comments_state.is_loading {
        Box::new(bsn_list![(
            LoadingIndicator
            Text("加载评论中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else if comments_state.comments.is_empty() {
        Box::new(bsn_list![(
            Text("暂无评论，来发表第一条评论吧")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        CommentsRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 标题栏
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
                        // 返回按钮
                        CommentsBackButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_CHEVRON_LEFT)
                                TextFont { font_size: FontSize::Px(20.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        CommentsTitleText
                        Text("评论")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    ),
                ]
            ),
            (
                // 内容区域（可滚动）
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                }
                Children [
                    (
                        #CommentsScroll
                        CommentsScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::new(
                                Val::Px(15.0),
                                Val::Px(15.0 + SCROLLBAR_WIDTH_PX),
                                Val::Px(10.0),
                                Val::Px(10.0),
                            ),
                            overflow: Overflow::scroll_y(),
                            row_gap: Val::Px(0.0),
                        }
                        ScrollArea
                        Children [ {content_placeholder} ]
                    ),
                    scrollbar(#CommentsScroll),
                ]
            ),
            // 底部输入栏（固定不滚动）
            comment_input_bar(comments_state),
        ]
    }
}

/// 底部输入栏场景
///
/// 输入栏只在建页时构建一次，之后不参与重建：
/// - 输入内容归 `TextInput`，显示由通用 `text_input_cursor_blink` 渲染
/// - 边框焦点色归 `text_input_focus_visuals`
/// - 回复提示行由 `refresh_comments_ui` 就地更新
/// - 发送按钮启用态由 `update_comment_send_enabled` 跟随输入内容
fn comment_input_bar(comments_state: &CommentsState) -> impl Scene + use<> {
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

    // 建页时输入必为空 → 发送按钮起于禁用配色（selected 置真才钉在 primary）
    let send_style = ButtonStyle::selectable(ButtonVariant::Secondary, false);

    bsn! {
        CommentInputContainer
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::new(Val::Px(15.0), Val::Px(15.0), Val::Px(8.0), Val::Px(8.0)),
            border: UiRect::top(Val::Px(1.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        BackgroundColor(AppColors::SURFACE)
        Children [
            (
                // 回复提示行
                CommentReplyRow
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(6.0)),
                    display: {reply_display},
                }
                Children [
                    (
                        CommentReplyHint
                        Text({reply_text})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                    (
                        CancelReplyButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)) }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text("取消")
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                ]
            ),
            (
                // 输入行
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                }
                Children [
                    (
                        // 输入框（通用 TextInput 组件，不接入 ButtonStyle）
                        CommentInputField
                        template_value(TextInput::new("写下你的评论..."))
                        Button
                        Node {
                            flex_grow: 1.0,
                            height: Val::Px(36.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                        }
                        BackgroundColor(AppColors::BACKGROUND)
                        template_value(BorderColor::all(AppColors::BORDER))
                        RelativeCursorPosition
                        Children [
                            (
                                TextInputDisplay
                                Text("写下你的评论...")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    (
                        // 发送按钮
                        CommentSendButton
                        Button
                        template_value(send_style)
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        BackgroundColor(AppColors::SECONDARY)
                        Children [
                            (
                                Text("发送")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
        ]
    }
}

/// 清理评论页面（销毁 UI + 交还输入焦点）
pub fn cleanup_comments_ui(
    mut commands: Commands,
    query: Query<Entity, With<CommentsRoot>>,
    mut input_focus: ResMut<InputFocus>,
    input_query: Query<Entity, With<CommentInputField>>,
) {
    // 输入框实体即将销毁：焦点留在死实体上会让 IME 一直开着，且再也失焦不掉
    if let Some(focused) = input_focus.get()
        && input_query.contains(focused)
    {
        input_focus.clear();
    }

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
    // 底部输入栏不参与重建，回复提示行在这里就地更新
    mut reply_hint_query: Query<&mut Text, With<CommentReplyHint>>,
    mut reply_row_query: Query<&mut Node, With<CommentReplyRow>>,
) {
    if !comments_state.is_changed() || !comments_state.needs_rebuild {
        return;
    }
    comments_state.needs_rebuild = false;

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

    // 逐项追加到滚动容器（命令按序执行，Children 顺序与构建顺序一致）
    for scene in comments_content(&state_snapshot) {
        commands
            .spawn_scene(scene)
            .insert(ChildOf(container_entity));
    }

    // 更新底部栏的回复提示（文本 + 整行显隐）
    let reply_text = match comments_state.reply_to_name {
        Some(ref name) => format!("回复 @{}", name),
        None => String::new(),
    };
    for mut text in reply_hint_query.iter_mut() {
        if **text != reply_text {
            **text = reply_text.clone();
        }
    }

    let reply_display = if comments_state.reply_to.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in reply_row_query.iter_mut() {
        if node.display != reply_display {
            node.display = reply_display;
        }
    }
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

/// 构建评论列表内容（滚动容器的直接子实体列表）
fn comments_content(state: &CommentsStateSnapshot) -> Vec<Box<dyn Scene>> {
    if state.is_loading && state.comments.is_empty() {
        return vec![Box::new(bsn! {
            LoadingIndicator
            Text("加载评论中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        })];
    }

    if let Some(ref error) = state.error {
        let error_text = format!("加载失败: {}", error);
        return vec![Box::new(bsn! {
            ErrorMessage
            Text({error_text})
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::ERROR)
        })];
    }

    if state.comments.is_empty() {
        return vec![Box::new(bsn! {
            Text("暂无评论，来发表第一条评论吧")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node { margin: UiRect::vertical(Val::Px(20.0)) }
        })];
    }

    let mut items: Vec<Box<dyn Scene>> = Vec::new();

    // 页码信息
    if state.total_pages > 1 {
        let page_info = format!("第 {} / {} 页", state.page, state.total_pages);
        items.push(Box::new(bsn! {
            Text({page_info})
            TextFont { font_size: FontSize::Px(12.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node { margin: UiRect::bottom(Val::Px(8.0)) }
        }));
    }

    // 渲染评论列表
    for comment in &state.comments {
        items.push(Box::new(comment_item(comment, &state.children_map)));
    }

    // 分页控件（共享控件，翻页行为内联在控件观察者里）
    if state.total_pages > 1 {
        items.push(Box::new(pagination_controls::<CommentsPage>(
            state.page.max(1) as u32,
            state.total_pages.max(0) as u32,
        )));
    }

    // 底部间距
    items.push(Box::new(bsn! {
        Node {
            height: Val::Px(20.0),
            min_height: Val::Px(20.0),
        }
    }));

    items
}

/// 单条评论场景
fn comment_item(
    comment: &picacg_api::models::Comment,
    children_map: &std::collections::HashMap<String, ChildCommentsState>,
) -> impl Scene + use<> {
    // 置顶标记
    let is_top = comment.is_top.unwrap_or(false);
    let top_badge: Box<dyn SceneList> = if is_top {
        Box::new(bsn_list![(
            Node { padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)) }
            BackgroundColor(Color::srgba(0.8, 0.2, 0.2, 0.6))
            Children [
                (
                    Text("置顶")
                    TextFont { font_size: FontSize::Px(10.0) }
                    TextColor(Color::srgb(1.0, 0.8, 0.8))
                )
            ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 称号
    let title_badge: Box<dyn SceneList> = if comment.user.title.is_empty() {
        Box::new(bsn_list![])
    } else {
        let title = comment.user.title.clone();
        Box::new(bsn_list![(
            Node { padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)) }
            BackgroundColor(Color::srgba(0.6, 0.4, 0.8, 0.3))
            Children [
                (
                    Text({title})
                    TextFont { font_size: FontSize::Px(10.0) }
                    TextColor(Color::srgb(0.8, 0.7, 1.0))
                )
            ]
        )])
    };

    // 评论内容
    let content_body: Box<dyn SceneList> = if comment.hide {
        Box::new(bsn_list![(
            Text("[该评论已被隐藏]")
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node { margin: UiRect::bottom(Val::Px(8.0)) }
        )])
    } else {
        let content = comment.content.clone();
        Box::new(bsn_list![(
            Text({content})
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(AppColors::TEXT)
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                max_width: Val::Percent(100.0),
            }
        )])
    };

    // 查看子评论按钮（如果有子评论）
    let expand_button: Box<dyn SceneList> = if comment.comments_count > 0 {
        let child_state = children_map.get(&comment.id);
        let is_expanded = child_state.map(|s| !s.comments.is_empty()).unwrap_or(false);

        let btn_text = if is_expanded {
            format!("收起回复 ({})", comment.comments_count)
        } else {
            format!("查看 {} 条回复", comment.comments_count)
        };
        let icon = if is_expanded {
            ICON_CHEVRON_UP
        } else {
            ICON_CHEVRON_DOWN
        };
        let expand_label = format!("{} {}", icon, btn_text);
        let expand_comment_id = comment.id.clone();
        let total_children = comment.comments_count;

        Box::new(bsn_list![(
            ExpandChildrenButton {
                comment_id: {expand_comment_id},
                total_children: {total_children},
            }
            Button
            template_value(ButtonStyle::ghost())
            Node { padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)) }
            BackgroundColor(Color::NONE)
            Children [
                (
                    Text({expand_label})
                    TextFont { font_size: FontSize::Px(12.0) }
                    TextColor(AppColors::PRIMARY)
                )
            ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 子评论区域（仅在已展开且有子评论时创建）
    let children_section: Box<dyn SceneList> = match children_map.get(&comment.id) {
        Some(child_state) if !child_state.comments.is_empty() => {
            let container_comment_id = comment.id.clone();
            let mut child_rows: Vec<Box<dyn Scene>> = Vec::new();

            for child_comment in &child_state.comments {
                child_rows.push(Box::new(child_comment_item(child_comment)));
            }

            // 子评论加载指示器
            if child_state.is_loading {
                child_rows.push(Box::new(bsn! {
                    Text("加载中...")
                    TextFont { font_size: FontSize::Px(12.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                    Node { margin: UiRect::vertical(Val::Px(4.0)) }
                }));
            }

            // 加载更多子评论按钮
            if child_state.page < child_state.total_pages && !child_state.is_loading {
                let more_comment_id = comment.id.clone();
                child_rows.push(Box::new(bsn! {
                    LoadMoreChildrenButton { comment_id: {more_comment_id} }
                    Button
                    template_value(ButtonStyle::ghost())
                    Node { padding: UiRect::vertical(Val::Px(4.0)) }
                    BackgroundColor(Color::NONE)
                    Children [
                        (
                            Text("加载更多回复...")
                            TextFont { font_size: FontSize::Px(12.0) }
                            TextColor(AppColors::PRIMARY)
                        )
                    ]
                }));
            }

            Box::new(bsn_list![(
                CommentChildrenContainer { comment_id: {container_comment_id} }
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(8.0)),
                    padding: UiRect::left(Val::Px(16.0)),
                    border: UiRect::left(Val::Px(2.0)),
                }
                template_value(BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.4)))
                Children [ {child_rows} ]
            )])
        }
        _ => Box::new(bsn_list![]),
    };

    // 点赞状态
    let is_liked = comment.is_liked.unwrap_or(false);
    let like_color = if is_liked {
        AppColors::ERROR
    } else {
        AppColors::TEXT_SECONDARY
    };

    let item_comment_id = comment.id.clone();
    let like_comment_id = comment.id.clone();
    let likes_comment_id = comment.id.clone();
    let reply_comment_id = comment.id.clone();
    let reply_user_name = comment.user.name.clone();
    let user_name = comment.user.name.clone();
    let level_label = format!("Lv.{}", comment.user.level);
    let likes_label = format!("{}", comment.likes_count);
    let date = comment
        .created_at
        .split('T')
        .next()
        .unwrap_or(&comment.created_at)
        .to_string();

    bsn! {
        CommentItem { comment_id: {item_comment_id} }
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            margin: UiRect::bottom(Val::Px(2.0)),
            border: UiRect::bottom(Val::Px(1.0)),
        }
        template_value(BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.5)))
        BackgroundColor(Color::NONE)
        Children [
            (
                // 第一行：用户名 + 等级 + 时间
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    margin: UiRect::bottom(Val::Px(6.0)),
                }
                Children [
                    // 置顶标记
                    {top_badge},
                    (
                        // 用户名
                        Text({user_name})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                    (
                        // 等级
                        Node { padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)) }
                        BackgroundColor(Color::srgba(0.3, 0.5, 0.8, 0.4))
                        Children [
                            (
                                Text({level_label})
                                TextFont { font_size: FontSize::Px(10.0) }
                                TextColor(Color::srgb(0.7, 0.85, 1.0))
                            )
                        ]
                    ),
                    // 称号
                    {title_badge},
                    (
                        // 弹性间距
                        Node { flex_grow: 1.0 }
                    ),
                    (
                        // 时间
                        Text({date})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            // 评论内容
            {content_body},
            (
                // 操作栏：点赞 + 回复 + 查看子评论
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                }
                Children [
                    (
                        // 点赞按钮
                        CommentLikeButton { comment_id: {like_comment_id} }
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                        }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_HEART)
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(like_color)
                            ),
                            (
                                CommentLikesText { comment_id: {likes_comment_id} }
                                Text({likes_label})
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(like_color)
                            ),
                        ]
                    ),
                    (
                        // 回复按钮
                        CommentReplyButton {
                            comment_id: {reply_comment_id},
                            user_name: {reply_user_name},
                        }
                        Button
                        template_value(ButtonStyle::ghost())
                        Node { padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)) }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text("回复")
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    // 查看子评论按钮
                    {expand_button},
                ]
            ),
            // 子评论区域
            {children_section},
        ]
    }
}

/// 子评论场景
fn child_comment_item(comment: &picacg_api::models::Comment) -> impl Scene + use<> {
    // 内容
    let content_body: Box<dyn SceneList> = if comment.hide {
        Box::new(bsn_list![(
            Text("[该回复已被隐藏]")
            TextFont { font_size: FontSize::Px(13.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        let content = comment.content.clone();
        Box::new(bsn_list![(
            Text({content})
            TextFont { font_size: FontSize::Px(13.0) }
            TextColor(AppColors::TEXT)
            Node { max_width: Val::Percent(100.0) }
        )])
    };

    // 点赞状态
    let is_liked = comment.is_liked.unwrap_or(false);
    let like_color = if is_liked {
        AppColors::ERROR
    } else {
        AppColors::TEXT_SECONDARY
    };

    let like_comment_id = comment.id.clone();
    let likes_comment_id = comment.id.clone();
    let reply_comment_id = comment.id.clone();
    let reply_user_name = comment.user.name.clone();
    let user_name = comment.user.name.clone();
    let level_label = format!("Lv.{}", comment.user.level);
    let likes_label = format!("{}", comment.likes_count);
    let date = comment
        .created_at
        .split('T')
        .next()
        .unwrap_or(&comment.created_at)
        .to_string();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(6.0), Val::Px(6.0)),
        }
        Children [
            (
                // 用户名 + 时间
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                }
                Children [
                    (
                        Text({user_name})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                    (
                        Node { padding: UiRect::axes(Val::Px(3.0), Val::Px(1.0)) }
                        BackgroundColor(Color::srgba(0.3, 0.5, 0.8, 0.3))
                        Children [
                            (
                                Text({level_label})
                                TextFont { font_size: FontSize::Px(9.0) }
                                TextColor(Color::srgb(0.7, 0.85, 1.0))
                            )
                        ]
                    ),
                    (
                        // 弹性间距
                        Node { flex_grow: 1.0 }
                    ),
                    (
                        Text({date})
                        TextFont { font_size: FontSize::Px(10.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            // 内容
            {content_body},
            (
                // 子评论的点赞
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(4.0)),
                }
                Children [
                    (
                        CommentLikeButton { comment_id: {like_comment_id} }
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(3.0),
                        }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_HEART)
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(like_color)
                            ),
                            (
                                CommentLikesText { comment_id: {likes_comment_id} }
                                Text({likes_label})
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(like_color)
                            ),
                        ]
                    ),
                    (
                        // 回复按钮
                        CommentReplyButton {
                            comment_id: {reply_comment_id},
                            user_name: {reply_user_name},
                        }
                        Button
                        template_value(ButtonStyle::ghost())
                        Node { margin: UiRect::left(Val::Px(12.0)) }
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text("回复")
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                ]
            ),
        ]
    }
}

// ==================== 交互系统 ====================

/// 返回按钮交互
pub fn comments_back_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CommentsBackButton>)>,
    mut navigate_back_messages: MessageWriter<NavigateBackEvent>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            navigate_back_messages.write(NavigateBackEvent);
        }
    }
}

/// 评论点赞交互
pub fn comment_like_interaction(
    interaction_query: Query<(&Interaction, &CommentLikeButton), Changed<Interaction>>,
    mut like_messages: MessageWriter<LikeCommentRequestEvent>,
) {
    for (interaction, like_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            like_messages.write(LikeCommentRequestEvent {
                comment_id: like_btn.comment_id.clone(),
            });
            tracing::info!("点赞评论: {}", like_btn.comment_id);
        }
    }
}

/// 评论回复按钮交互
pub fn comment_reply_interaction(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &CommentReplyButton), Changed<Interaction>>,
    input_query: Query<Entity, With<CommentInputField>>,
    mut comments_state: ResMut<CommentsState>,
) {
    for (interaction, reply_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            comments_state.reply_to = Some(reply_btn.comment_id.clone());
            comments_state.reply_to_name = Some(reply_btn.user_name.clone());
            comments_state.needs_rebuild = true;

            // 点回复顺手把焦点送进输入框。同一帧里 text_input_blur 也会因为
            // "点击没落在输入框上" 而清焦点，两者顺序不定 —— 走命令队列压到
            // 本帧同步点之后执行，稳定盖过 blur
            if let Ok(input_entity) = input_query.single() {
                commands.queue(move |world: &mut World| {
                    world
                        .resource_mut::<InputFocus>()
                        .set(input_entity, FocusCause::Pressed);
                });
            }

            tracing::info!(
                "回复评论: {} (@{})",
                reply_btn.comment_id,
                reply_btn.user_name
            );
        }
    }
}

/// 取消回复交互
pub fn cancel_reply_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CancelReplyButton>)>,
    mut comments_state: ResMut<CommentsState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            comments_state.reply_to = None;
            comments_state.reply_to_name = None;
            comments_state.needs_rebuild = true;
        }
    }
}

/// 展开/折叠子评论交互
pub fn expand_children_interaction(
    interaction_query: Query<(&Interaction, &ExpandChildrenButton), Changed<Interaction>>,
    mut comments_state: ResMut<CommentsState>,
    mut load_children_messages: MessageWriter<LoadChildCommentsRequest>,
) {
    for (interaction, expand_btn) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

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
}

/// 加载更多子评论交互
pub fn load_more_children_interaction(
    interaction_query: Query<(&Interaction, &LoadMoreChildrenButton), Changed<Interaction>>,
    mut comments_state: ResMut<CommentsState>,
    mut load_children_messages: MessageWriter<LoadChildCommentsRequest>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Some(child_state) = comments_state.children_map.get_mut(&btn.comment_id)
        {
            let next_page = child_state.page + 1;
            child_state.is_loading = true;

            load_children_messages.write(LoadChildCommentsRequest {
                comment_id: btn.comment_id.clone(),
                page: next_page,
            });
        }
    }
}

/// 发送评论交互
pub fn comment_send_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CommentSendButton>)>,
    mut input_query: Query<&mut TextInput, With<CommentInputField>>,
    mut comments_state: ResMut<CommentsState>,
    mut post_comment_messages: MessageWriter<PostCommentRequest>,
    mut post_reply_messages: MessageWriter<PostCommentReplyRequest>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok(mut input) = input_query.single_mut() else {
            continue;
        };
        let content = input.value.trim().to_string();
        if content.is_empty() {
            continue;
        }

        submit_comment(
            &mut comments_state,
            content,
            &mut post_comment_messages,
            &mut post_reply_messages,
        );
        input.set_value("");
    }
}

/// 输入框动作键（Enter 发送 / Escape 失焦）
///
/// 字符编辑、光标、剪贴板、IME 全归通用 TextInput 系统，这里只认动作键，
/// 且只在焦点确实落在评论输入框上时才响应。
pub fn comment_input_action_keys(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut input_focus: ResMut<InputFocus>,
    mut input_query: Query<&mut TextInput, With<CommentInputField>>,
    mut comments_state: ResMut<CommentsState>,
    mut post_comment_messages: MessageWriter<PostCommentRequest>,
    mut post_reply_messages: MessageWriter<PostCommentReplyRequest>,
) {
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut input) = input_query.get_mut(focused) else {
        return;
    };

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Enter => {
                let content = input.value.trim().to_string();
                if content.is_empty() {
                    continue;
                }
                submit_comment(
                    &mut comments_state,
                    content,
                    &mut post_comment_messages,
                    &mut post_reply_messages,
                );
                input.set_value("");
            }
            Key::Escape => input_focus.clear(),
            _ => {}
        }
    }
}

/// 提交评论或回复（发送按钮与 Enter 共用）
fn submit_comment(
    comments_state: &mut CommentsState,
    content: String,
    post_comment_messages: &mut MessageWriter<PostCommentRequest>,
    post_reply_messages: &mut MessageWriter<PostCommentReplyRequest>,
) {
    match comments_state.reply_to {
        Some(ref comment_id) => {
            post_reply_messages.write(PostCommentReplyRequest {
                comment_id: comment_id.clone(),
                content,
            });
            tracing::info!("发送回复: comment_id={}", comment_id);
        }
        None => {
            post_comment_messages.write(PostCommentRequest {
                comic_id: comments_state.comic_id.clone(),
                content,
            });
            tracing::info!("发表评论: comic_id={}", comments_state.comic_id);
        }
    }

    comments_state.reply_to = None;
    comments_state.reply_to_name = None;
    comments_state.needs_rebuild = true;
}

/// 输入内容 → 发送按钮启用态
///
/// 底部输入栏建页后不重建，发送按钮的配色只能在这里跟着 `TextInput` 内容走。
pub fn update_comment_send_enabled(
    input_query: Query<&TextInput, (With<CommentInputField>, Changed<TextInput>)>,
    mut send_style_query: Query<&mut ButtonStyle, With<CommentSendButton>>,
) {
    let Ok(input) = input_query.single() else {
        return;
    };

    // 比较后写：避免每个按键都触发 ButtonStyle 变更检测
    let send_enabled = !input.value.trim().is_empty();
    for mut send_style in send_style_query.iter_mut() {
        if send_style.selected != send_enabled {
            send_style.selected = send_enabled;
        }
    }
}

/// 消费分页控件状态变化（翻页边界与按钮行为已内联在控件观察者里）
pub fn comments_pagination_changed(
    pagination_query: Query<
        &Pagination,
        (With<PaginationControl<CommentsPage>>, Changed<Pagination>),
    >,
    mut comments_state: ResMut<CommentsState>,
    mut load_messages: MessageWriter<LoadCommentsRequest>,
    mut scroll_query: Query<&mut ScrollPosition, With<CommentsScrollContainer>>,
) {
    let Ok(pagination) = pagination_query.single() else {
        return;
    };
    // 只响应真实翻页（控件重建后的同值回填在此被过滤）
    let new_page = pagination.current_page as i32;
    if new_page == comments_state.page.max(1) {
        return;
    }

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

    tracing::debug!("切换到评论第 {} 页", new_page);
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
        for child_state in comments_state.children_map.values_mut() {
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
