//! 个人资料系统
//!
//! 显示用户个人信息，包括头像、用户名、等级、经验值、称号、注册日期等

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

/// 个人资料刷新按钮
#[derive(Component)]
pub struct ProfileRefreshButton;

/// 个人资料信息区域容器（数据加载后填充）
#[derive(Component)]
pub struct ProfileInfoContainer;

/// 个人资料信息文本（固定节点，只更新内容）
#[derive(Component)]
pub struct ProfileInfoText;

/// 个人资料打卡按钮（预留，后续实现打卡功能时使用）
#[derive(Component)]
#[allow(dead_code)]
pub struct ProfilePunchInButton;

// ==================== 布局常量 ====================

mod profile_layout {
    /// 头像尺寸
    pub const AVATAR_SIZE: f32 = 120.0;
    /// 左右外边距
    pub const MARGIN_H: f32 = 20.0;
}

// ==================== 辅助函数 ====================

/// 确保文本不为空，避免 Bevy text_system 在渲染空字符串时 panic
/// （index out of bounds: the len is 0 but the index is 0）
fn non_empty_text(s: &str, fallback: &str) -> String {
    if s.trim().is_empty() {
        fallback.to_string()
    } else {
        s.to_string()
    }
}

// ==================== 系统函数 ====================

/// 创建个人资料页面 UI（如果已存在则只显示）
pub fn setup_profile_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    profile_state: Res<UserProfileState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut existing_query: Query<&mut Node, With<ProfileRoot>>,
    mut load_profile_messages: MessageWriter<LoadUserProfileRequest>,
) {
    // 如果 ProfileRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        // 仍然触发加载（刷新数据）
        if profile_state.user.is_none() && !profile_state.is_loading {
            load_profile_messages.write(LoadUserProfileRequest);
        }
        return;
    }

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let profile_root = commands
        .spawn((
            ProfileRoot,
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
                    justify_content: JustifyContent::SpaceBetween,
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|header| {
                // 左侧标题
                header.spawn((
                    Text::new("个人资料"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 右侧刷新按钮
                header
                    .spawn((
                        ProfileRefreshButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(12.0),
                                Val::Px(6.0),
                                Val::Px(6.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                        BorderColor::all(AppColors::BORDER),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_REFRESH),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::PRIMARY),
                        ));
                        btn.spawn((
                            Text::new("刷新"),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(AppColors::PRIMARY),
                        ));
                    });
            });

            // 滚动区域包装器
            root.spawn((Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                ..default()
            },))
                .with_children(|wrapper| {
                    // 内容区域（可滚动）
                    let scroll_container_id = wrapper
                        .spawn((
                            ProfileScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect {
                                    left: Val::Px(profile_layout::MARGIN_H),
                                    right: Val::Px(profile_layout::MARGIN_H + SCROLLBAR_WIDTH),
                                    top: Val::Px(20.0),
                                    bottom: Val::Px(30.0),
                                },
                                row_gap: Val::Px(20.0),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            Scrollable,
                            ScrollPosition::default(),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|content| {
                            if profile_state.is_loading {
                                content.spawn((
                                    LoadingIndicator,
                                    Text::new("加载中..."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT),
                                ));
                            }

                            // 信息容器（包含固定的文本节点）
                            content
                                .spawn((
                                    ProfileInfoContainer,
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(20.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|info| {
                                    // 固定文本节点，初始显示"加载中..."，数据到达后更新
                                    info.spawn((
                                        ProfileInfoText,
                                        Text::new("加载中..."),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 15.0,
                                            ..default()
                                        },
                                        TextColor(AppColors::TEXT),
                                        Node {
                                            margin: UiRect::all(Val::Px(15.0)),
                                            ..default()
                                        },
                                    ));
                                });
                        })
                        .id();

                    // 创建滚动条
                    spawn_scrollbar(wrapper, scroll_container_id);
                });
        })
        .id();

    // 如果有 ContentArea，将个人资料页面作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(profile_root);
    }

    // 发送加载请求
    if profile_state.user.is_none() && !profile_state.is_loading {
        load_profile_messages.write(LoadUserProfileRequest);
    }

    tracing::info!("个人资料页面 UI 已创建");
}

/// 清理个人资料页面（用 Display::None 隐藏，不占布局空间，避免 despawn 触发
/// bevy_text panic）
pub fn cleanup_profile_ui(mut query: Query<&mut Node, With<ProfileRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新个人资料 UI（只更新固定文本节点的内容，不 spawn/despawn）
pub fn refresh_profile_ui(
    profile_state: Res<UserProfileState>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<ProfileInfoText>>,
    mut image_messages: MessageWriter<LoadImageRequest>,
    mut last_text: Local<String>,
) {
    let Ok((mut text, mut text_color)) = text_query.single_mut() else {
        return;
    };

    // 正在加载
    if profile_state.is_loading {
        let new_text = "加载中...".to_string();
        if *last_text != new_text {
            **text = new_text.clone();
            *last_text = new_text;
            *text_color = TextColor(AppColors::TEXT_SECONDARY);
        }
        return;
    }

    // 加载失败
    if let Some(ref error) = profile_state.error {
        let new_text = format!("加载失败: {}", error);
        if *last_text != new_text {
            **text = new_text.clone();
            *last_text = new_text;
            *text_color = TextColor(Color::srgb(0.9, 0.3, 0.3));
        }
        return;
    }

    // 没有数据
    let Some(ref user) = profile_state.user else {
        return;
    };

    // 构建信息文本
    let gender = match user.gender.as_str() {
        "m" => "男",
        "f" => "女",
        "bot" => "扶她",
        _ => "未设置",
    };
    let title = if user.title.trim().is_empty() {
        "无".to_string()
    } else {
        user.title.clone()
    };
    let created = if user.created_at.is_empty() {
        "-".to_string()
    } else {
        user.created_at.chars().take(10).collect()
    };

    let new_text = format!(
        "用户名：{}\n称号：{}\n等级：Lv.{}\n经验值：{} EXP\n性别：{}\n注册日期：{}\n用户 ID：{}",
        non_empty_text(&user.name, "未知用户"),
        title,
        user.level,
        user.exp,
        gender,
        created,
        non_empty_text(&user.id, "-"),
    );

    // 只在内容变化时更新（避免每帧触发 text layout）
    if *last_text != new_text {
        // 触发头像图片加载
        let avatar_url = user.avatar.as_ref().map(|a| a.url()).unwrap_or_default();
        if !avatar_url.is_empty() {
            image_messages.write(LoadImageRequest {
                url: avatar_url.clone(),
            });
        }

        **text = new_text.clone();
        *last_text = new_text;
        *text_color = TextColor(AppColors::TEXT);
    }
}

/// 更新头像图片（从缓存加载后替换占位符）
pub fn update_profile_avatar(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    avatar_query: Query<(Entity, &ProfileAvatarImage), Without<ImageNode>>,
) {
    for (entity, avatar) in avatar_query.iter() {
        if let Some(handle) = image_cache.handles.get(&avatar.url) {
            // 图片已缓存，移除文本占位符组件，添加图片组件
            commands.entity(entity).remove::<Text>();
            commands.entity(entity).remove::<TextFont>();
            commands.entity(entity).remove::<TextColor>();
            commands.entity(entity).insert((
                ImageNode::new(handle.clone()),
                Node {
                    width: Val::Px(profile_layout::AVATAR_SIZE),
                    height: Val::Px(profile_layout::AVATAR_SIZE),
                    ..default()
                },
            ));
        }
    }
}

/// 刷新按钮交互
pub fn profile_refresh_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ProfileRefreshButton>),
    >,
    mut profile_state: ResMut<UserProfileState>,
    mut load_messages: MessageWriter<LoadUserProfileRequest>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                *border_color = BorderColor::all(AppColors::PRIMARY);

                if !profile_state.is_loading {
                    // 清空旧数据，重新加载
                    profile_state.user = None;
                    profile_state.is_loading = true;
                    profile_state.error = None;
                    load_messages.write(LoadUserProfileRequest);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
                *border_color = BorderColor::all(AppColors::BORDER);
            }
        }
    }
}

/// 处理加载完成事件
pub fn handle_profile_loaded(
    mut loaded_messages: MessageReader<UserProfileLoadedEvent>,
    mut failed_messages: MessageReader<UserProfileLoadFailedEvent>,
    mut profile_state: ResMut<UserProfileState>,
) {
    for event in loaded_messages.read() {
        profile_state.user = Some(event.user.clone());
        profile_state.is_loading = false;
        profile_state.error = None;
        tracing::info!("个人资料加载完成: name={}", event.user.name);
    }

    for event in failed_messages.read() {
        profile_state.is_loading = false;
        profile_state.error = Some(event.error.clone());
        tracing::warn!("个人资料加载失败: {}", event.error);
    }
}

/// 处理滚动事件
pub fn handle_profile_scroll(
    _scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<ProfileScrollContainer>,
    >,
    mut _mouse_wheel_events: MessageReader<MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// 更新内容尺寸信息
pub fn update_profile_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<ProfileScrollContainer>,
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
