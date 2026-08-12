//! 锅贴社区系统
//!
//! 展示锅贴帖子列表，支持分页浏览

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        pagination::{Pagination, PaginationControl, pagination_controls},
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        widgets::ButtonStyle,
    },
    utils::icons::*,
};

/// 锅贴页面标记类型（用于分页组件的泛型参数）
pub struct FriedPage;

// ==================== 组件定义 ====================

/// 锅贴社区根节点
#[derive(Component, Default, Clone)]
pub struct FriedRoot;

/// 锅贴社区滚动容器
#[derive(Component, Default, Clone)]
pub struct FriedScrollContainer;

/// 锅贴帖子卡片
#[derive(Component, Default, Clone)]
pub struct FriedPostCard {
    #[allow(dead_code)]
    pub post_id: String,
}

/// 刷新按钮
#[derive(Component, Default, Clone)]
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

    let content_area = content_area_query.single().ok();

    let fried_root = commands.spawn_scene(fried_page(&fried_state)).id();

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

/// 锅贴社区页面场景
fn fried_page(fried_state: &FriedState) -> impl Scene + use<> {
    let scroll_padding = UiRect {
        left: Val::Px(fried_layout::PADDING_LEFT),
        right: Val::Px(fried_layout::PADDING_RIGHT),
        top: Val::Px(fried_layout::PADDING_TOP),
        bottom: Val::Px(fried_layout::PADDING_BOTTOM),
    };
    let refresh_label = format!("{} 刷新", ICON_REFRESH);
    let content = fried_content(fried_state);

    bsn! {
        FriedRoot
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
                        // 图标
                        Text(ICON_FORUM)
                        TextFont { font_size: FontSize::Px(20.0) }
                        TextColor(AppColors::PRIMARY)
                    ),
                    (
                        Text("锅贴社区")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 弹性占位
                        Node { flex_grow: 1.0 }
                    ),
                    (
                        // 刷新按钮
                        FriedRefreshButton
                        Button
                        template_value(ButtonStyle::primary())
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
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        Children [
                            (
                                Text({refresh_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(Color::WHITE)
                            )
                        ]
                    ),
                ]
            ),
            (
                // 滚动区域包装器
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
                        // 可滚动内容区域
                        #FriedScroll
                        FriedScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: {scroll_padding},
                            row_gap: Val::Px(fried_layout::CARD_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [ {content} ]
                    ),
                    // 滚动条
                    scrollbar(#FriedScroll),
                ]
            ),
        ]
    }
}

/// 滚动容器内的内容（加载中 / 错误 / 空 / 帖子列表 + 分页，末尾附底部间距）
///
/// `setup_fried_ui` 与 `refresh_fried_ui` 共用，保证首次创建与刷新结构一致。
fn fried_content(fried_state: &FriedState) -> Vec<Box<dyn Scene>> {
    let mut items: Vec<Box<dyn Scene>> = Vec::new();

    if fried_state.is_loading {
        items.push(Box::new(bsn! {
            LoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    } else if let Some(ref error) = fried_state.error {
        let message = format!("加载失败: {}", error);
        items.push(Box::new(bsn! {
            ErrorMessage
            Text({message})
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::ERROR)
        }));
    } else if fried_state.posts.is_empty() {
        items.push(Box::new(bsn! {
            Text("暂无帖子")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        }));
    } else {
        // 显示帖子列表
        for post in &fried_state.posts {
            items.push(Box::new(fried_post_card(post)));
        }

        // 分页控件（共享控件；state.page 是 0 起的偏移量，控件按 1 起显示）
        let total_pages = calculate_total_pages(fried_state);
        if total_pages > 1 {
            items.push(Box::new(pagination_controls::<FriedPage>(
                (fried_state.page + 1).max(1) as u32,
                total_pages.max(0) as u32,
            )));
        }
    }

    // 底部间距
    items.push(Box::new(bsn! {
        Node {
            height: Val::Px(30.0),
            min_height: Val::Px(30.0),
        }
    }));

    items
}

/// 单个帖子卡片场景
fn fried_post_card(post: &picacg_api::endpoints::fried::FriedPost) -> impl Scene + use<> {
    let post_id = post.id.clone();

    // 用户名 + 等级 + 称号（无用户信息时为空列表）
    let user_row: Box<dyn SceneList> = match post.user {
        Some(ref user) => {
            let user_name = user.name.clone();
            let level_label = format!("Lv{}", user.level);

            // 称号（为空时不显示）
            let title_badge: Box<dyn SceneList> = if user.title.is_empty() {
                Box::new(bsn_list![])
            } else {
                let title = user.title.clone();
                Box::new(bsn_list![(
                    Node {
                        padding: UiRect::new(
                            Val::Px(4.0),
                            Val::Px(4.0),
                            Val::Px(1.0),
                            Val::Px(1.0),
                        ),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                    }
                    BackgroundColor(Color::srgb(0.6, 0.3, 0.8))
                    Children [
                        (
                            Text({title})
                            TextFont { font_size: FontSize::Px(10.0) }
                            TextColor(Color::WHITE)
                        )
                    ]
                )])
            };

            Box::new(bsn_list![(
                // 用户名
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                }
                Children [
                    (
                        Text({user_name})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 等级标签
                        Node {
                            padding: UiRect::new(
                                Val::Px(4.0),
                                Val::Px(4.0),
                                Val::Px(1.0),
                                Val::Px(1.0),
                            ),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        Children [
                            (
                                Text({level_label})
                                TextFont { font_size: FontSize::Px(10.0) }
                                TextColor(Color::WHITE)
                            )
                        ]
                    ),
                    // 称号
                    {title_badge},
                ]
            )])
        }
        None => Box::new(bsn_list![]),
    };

    // 时间（为空时不显示）
    let time_row: Box<dyn SceneList> = if post.created_at.is_empty() {
        Box::new(bsn_list![])
    } else {
        let time_display = format_time(&post.created_at);
        Box::new(bsn_list![(
            Text({time_display})
            TextFont { font_size: FontSize::Px(11.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    };

    // 帖子内容（为空时不显示）
    let content_row: Box<dyn SceneList> = if post.content.is_empty() {
        Box::new(bsn_list![])
    } else {
        // 截取前 200 个字符
        let content_text = if post.content.chars().count() > 200 {
            format!("{}...", post.content.chars().take(200).collect::<String>())
        } else {
            post.content.clone()
        };
        Box::new(bsn_list![(
            Text({content_text})
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(AppColors::TEXT)
            Node { max_width: Val::Percent(100.0) }
        )])
    };

    // 媒体附件提示（无附件时不显示）
    let media_hint: Box<dyn SceneList> = if post.medias.is_empty() {
        Box::new(bsn_list![])
    } else {
        let media_label = format!("📷 {} 张图片", post.medias.len());
        Box::new(bsn_list![(
            Node {
                padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(3.0), Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
            }
            BackgroundColor(Color::srgba(0.3, 0.5, 0.8, 0.2))
            Children [
                (
                    Text({media_label})
                    TextFont { font_size: FontSize::Px(12.0) }
                    TextColor(Color::srgb(0.5, 0.7, 1.0))
                )
            ]
        )])
    };

    let like_color = if post.liked {
        AppColors::ERROR
    } else {
        AppColors::TEXT_SECONDARY
    };
    let likes_label = format!("{}", post.total_likes);
    let comments_label = format!("{}", post.total_comments);

    bsn! {
        FriedPostCard { post_id: {post_id} }
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(14.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(AppColors::SURFACE_SUNKEN)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 用户信息行
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                }
                Children [
                    (
                        // 用户头像占位
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            min_width: Val::Px(36.0),
                            min_height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Percent(50.0)),
                        }
                        BackgroundColor(Color::srgb(0.18, 0.18, 0.22))
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text(ICON_USER)
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                    (
                        // 用户名 + 等级
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            row_gap: Val::Px(2.0),
                        }
                        Children [
                            {user_row},
                            // 时间
                            {time_row},
                        ]
                    ),
                ]
            ),
            // 帖子内容
            {content_row},
            // 媒体附件提示
            {media_hint},
            (
                // 底部操作栏（点赞数、评论数）
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(4.0)),
                }
                Children [
                    (
                        // 点赞
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                        }
                        Children [
                            (
                                Text(ICON_HEART)
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(like_color)
                            ),
                            (
                                Text({likes_label})
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                        ]
                    ),
                    (
                        // 评论
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                        }
                        Children [
                            (
                                Text(ICON_FORUM)
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                            (
                                Text({comments_label})
                                TextFont { font_size: FontSize::Px(12.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
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
    for scene in fried_content(&fried_state) {
        commands.spawn_scene(scene).insert(ChildOf(scroll_entity));
    }
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

/// 消费分页控件状态变化（翻页边界与按钮行为已内联在控件观察者里）
pub fn fried_pagination_changed(
    pagination_query: Query<&Pagination, (With<PaginationControl<FriedPage>>, Changed<Pagination>)>,
    mut fried_state: ResMut<FriedState>,
    mut load_messages: MessageWriter<LoadFriedPostsRequest>,
) {
    let Ok(pagination) = pagination_query.single() else {
        return;
    };
    // 控件页码从 1 起，state.page 是 0 起的偏移量
    let new_page = pagination.current_page.max(1) as i32 - 1;
    // 只响应真实翻页（控件重建后的同值回填在此被过滤）
    if new_page == fried_state.page {
        return;
    }

    fried_state.page = new_page;
    fried_state.posts.clear();
    fried_state.is_loading = true;
    fried_state.error = None;
    load_messages.write(LoadFriedPostsRequest { page: new_page });

    tracing::debug!("切换到锅贴第 {} 页", new_page + 1);
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
