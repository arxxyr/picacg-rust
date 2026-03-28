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
};

// ==================== 组件定义 ====================

/// 个人资料刷新按钮
#[derive(Component)]
pub struct ProfileRefreshButton;

/// 各字段文本标记（用枚举区分，避免 N 个 Query 冲突）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum ProfileField {
    Name,
    Title,
    Slogan,
    Level,
    Exp,
    Gender,
    Email,
    CreatedAt,
    UserId,
    Verified,
    Characters,
    PunchIn,
}

/// 个人资料头像容器
#[derive(Component)]
pub struct ProfileAvatarContainer;

/// 个人资料打卡按钮
#[derive(Component)]
pub struct ProfilePunchInButton;

// ==================== 布局常量 ====================

mod layout {
    pub const AVATAR_SIZE: f32 = 100.0;
    pub const AVATAR_IMAGE_SIZE: f32 = 96.0;
    pub const MARGIN_H: f32 = 20.0;
    pub const CARD_RADIUS: f32 = 8.0;
    pub const CARD_PADDING: f32 = 16.0;
    pub const HEADER_CARD_RADIUS: f32 = 12.0;
    pub const STAT_LABEL_SIZE: f32 = 11.0;
    pub const STAT_VALUE_SIZE: f32 = 20.0;
    pub const INFO_LABEL_SIZE: f32 = 13.0;
    pub const INFO_VALUE_SIZE: f32 = 13.0;
}

// ==================== 辅助函数 ====================

/// 确保文本不为空
fn non_empty(s: &str, fallback: &str) -> String {
    if s.trim().is_empty() {
        fallback.to_string()
    } else {
        s.to_string()
    }
}

/// 性别文本
fn gender_text(g: &str) -> &'static str {
    match g {
        "m" => "♂ 男",
        "f" => "♀ 女",
        "bot" => "⚥ 扶她",
        _ => "未设置",
    }
}

/// 计算当前等级所需经验（简单公式：100 * level^2）
fn exp_for_level(level: i32) -> i64 {
    100 * (level as i64) * (level as i64)
}

/// 生成一个信息行（label + value），返回 value entity
fn spawn_info_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    field: ProfileField,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            // 标签
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: layout::INFO_LABEL_SIZE,
                    ..default()
                },
                TextColor(AppColors::TEXT_MUTED),
                Node {
                    min_width: Val::Px(65.0),
                    ..default()
                },
            ));
            // 值
            row.spawn((
                field,
                Text::new("--"),
                TextFont {
                    font: font.clone(),
                    font_size: layout::INFO_VALUE_SIZE,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 生成一个统计卡片
fn spawn_stat_card(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    field: ProfileField,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(6.0),
                border_radius: BorderRadius::all(Val::Px(layout::CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(AppColors::CARD_BG),
        ))
        .with_children(|card| {
            // 标签
            card.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: layout::STAT_LABEL_SIZE,
                    ..default()
                },
                TextColor(AppColors::TEXT_MUTED),
            ));
            // 值
            card.spawn((
                field,
                Text::new("--"),
                TextFont {
                    font: font.clone(),
                    font_size: layout::STAT_VALUE_SIZE,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
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
            // ── 标题栏 ──
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
                header.spawn((
                    Text::new("个人资料"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 刷新按钮
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
                            Text::new("↻"),
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

            // ── 滚动区域 ──
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
                let scroll_id = wrapper
                    .spawn((
                        ProfileScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect {
                                left: Val::Px(layout::MARGIN_H),
                                right: Val::Px(layout::MARGIN_H + SCROLLBAR_WIDTH),
                                top: Val::Px(20.0),
                                bottom: Val::Px(30.0),
                            },
                            row_gap: Val::Px(16.0),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        Scrollable,
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|content| {
                        spawn_header_card(content, &font);
                        spawn_stats_row(content, &font);
                        spawn_info_card(content, &font);
                        spawn_punch_in_section(content, &font);
                    })
                    .id();

                spawn_scrollbar(wrapper, scroll_id);
            });
        })
        .id();

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(profile_root);
    }

    if profile_state.user.is_none() && !profile_state.is_loading {
        load_profile_messages.write(LoadUserProfileRequest);
    }

    tracing::info!("个人资料页面 UI 已创建");
}

/// ── 头部卡片：头像 + 用户名 + 称号 + 签名 ──
fn spawn_header_card(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(4.0),
                border_radius: BorderRadius::all(Val::Px(layout::HEADER_CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(AppColors::CARD_BG),
        ))
        .with_children(|card| {
            // 圆形头像容器
            card.spawn((
                ProfileAvatarContainer,
                Node {
                    width: Val::Px(layout::AVATAR_SIZE),
                    height: Val::Px(layout::AVATAR_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    overflow: Overflow::clip(),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
                BorderColor::all(AppColors::PRIMARY),
            ))
            .with_children(|avatar| {
                avatar.spawn((
                    ProfileAvatarImage { url: String::new() },
                    Text::new("👤"),
                    TextFont {
                        font: font.clone(),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });

            // 用户名
            card.spawn((
                ProfileField::Name,
                Text::new("加载中..."),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ));

            // 称号
            card.spawn((
                ProfileField::Title,
                Text::new(" "),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::PRIMARY),
            ));

            // 签名
            card.spawn((
                ProfileField::Slogan,
                Text::new(" "),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_MUTED),
                Node {
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
            ));
        });
}

/// ── 统计行：等级 / 经验 / 性别 ──
fn spawn_stats_row(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            spawn_stat_card(row, font, "等级", ProfileField::Level);
            spawn_stat_card(row, font, "经验", ProfileField::Exp);
            spawn_stat_card(row, font, "性别", ProfileField::Gender);
        });
}

/// ── 信息详情卡片 ──
fn spawn_info_card(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(layout::CARD_PADDING)),
                row_gap: Val::Px(14.0),
                border_radius: BorderRadius::all(Val::Px(layout::CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(AppColors::CARD_BG),
        ))
        .with_children(|card| {
            // 标题
            card.spawn((
                Text::new("个人信息"),
                TextFont {
                    font: font.clone(),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ));

            spawn_info_row(card, font, "✉  邮箱", ProfileField::Email);
            spawn_info_row(card, font, "📅 注册", ProfileField::CreatedAt);
            spawn_info_row(card, font, "🆔 ID", ProfileField::UserId);
            spawn_info_row(card, font, "✓  认证", ProfileField::Verified);
            spawn_info_row(card, font, "🏷  角色", ProfileField::Characters);
        });
}

/// ── 签到按钮 ──
fn spawn_punch_in_section(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            ProfilePunchInButton,
            Button,
            Interaction::default(),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(14.0), Val::Px(14.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(layout::CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(AppColors::PRIMARY),
            BorderColor::all(AppColors::PRIMARY),
        ))
        .with_children(|btn| {
            btn.spawn((
                ProfileField::PunchIn,
                Text::new("签到"),
                TextFont {
                    font: font.clone(),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 清理个人资料页面（隐藏）
pub fn cleanup_profile_ui(mut query: Query<&mut Node, With<ProfileRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新个人资料 UI（更新各字段文本）
pub fn refresh_profile_ui(
    profile_state: Res<UserProfileState>,
    punch_in_state: Res<PunchInState>,
    mut field_query: Query<(&ProfileField, &mut Text, &mut TextColor)>,
    mut avatar_query: Query<&mut ProfileAvatarImage>,
    mut punch_btn_query: Query<
        (&mut BackgroundColor, &mut BorderColor),
        With<ProfilePunchInButton>,
    >,
    mut image_messages: MessageWriter<LoadImageRequest>,
    mut last_user_name: Local<String>,
) {
    // 正在加载
    if profile_state.is_loading {
        if last_user_name.is_empty() || *last_user_name == "__loading__" {
            for (field, mut text, mut color) in field_query.iter_mut() {
                if *field == ProfileField::Name {
                    **text = "加载中...".into();
                    *color = TextColor(AppColors::TEXT_SECONDARY);
                } else if *field == ProfileField::PunchIn {
                    // 保持签到按钮文字不变
                } else {
                    **text = "--".into();
                    *color = TextColor(AppColors::TEXT_SECONDARY);
                }
            }
            *last_user_name = "__loading__".into();
        }
        return;
    }

    // 加载失败
    if let Some(ref error) = profile_state.error {
        let err_tag = format!("__error__{}", error);
        if *last_user_name != err_tag {
            for (field, mut text, mut color) in field_query.iter_mut() {
                if *field == ProfileField::Name {
                    **text = format!("加载失败: {}", error);
                    *color = TextColor(AppColors::ERROR);
                } else if *field == ProfileField::PunchIn {
                    // 保持
                } else {
                    **text = "--".into();
                    *color = TextColor(AppColors::TEXT_SECONDARY);
                }
            }
            *last_user_name = err_tag;
        }
        return;
    }

    let Some(ref user) = profile_state.user else {
        return;
    };

    // 只在数据变化时更新
    let user_tag = format!(
        "{}_{}_{}_{}",
        user.name, user.level, user.exp, punch_in_state.is_punched
    );
    if *last_user_name == user_tag {
        return;
    }

    // 更新头像
    let avatar_url = user.avatar.as_ref().map(|a| a.url()).unwrap_or_default();
    if !avatar_url.is_empty() {
        for mut avatar in avatar_query.iter_mut() {
            if avatar.url != avatar_url {
                avatar.url = avatar_url.clone();
                image_messages.write(LoadImageRequest {
                    url: avatar_url.clone(),
                });
            }
        }
    }

    let is_punched = user.is_punched.unwrap_or(false) || punch_in_state.is_punched;

    // 更新签到按钮颜色
    for (mut bg, mut border) in punch_btn_query.iter_mut() {
        if is_punched {
            *bg = BackgroundColor(AppColors::SECONDARY);
            *border = BorderColor::all(AppColors::SECONDARY);
        } else {
            *bg = BackgroundColor(AppColors::PRIMARY);
            *border = BorderColor::all(AppColors::PRIMARY);
        }
    }

    // 更新所有字段
    let title = if user.title.trim().is_empty() {
        "无称号"
    } else {
        &user.title
    };
    let slogan = user
        .slogan
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("这个人很懒，什么都没写~");
    let created = if user.created_at.is_empty() {
        "--".to_string()
    } else {
        user.created_at.chars().take(10).collect()
    };
    let email = user
        .email
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("未绑定");
    let verified = match user.verified {
        Some(true) => "✓ 已认证",
        Some(false) => "✗ 未认证",
        None => "--",
    };
    let characters = user
        .characters
        .as_ref()
        .filter(|c| !c.is_empty())
        .map(|c| c.join("、"))
        .unwrap_or_else(|| "无".into());
    let punch_text = if is_punched {
        "✓ 今日已签到"
    } else {
        "签到"
    };

    for (field, mut text, mut color) in field_query.iter_mut() {
        match field {
            ProfileField::Name => {
                **text = non_empty(&user.name, "未知用户");
                *color = TextColor(AppColors::TEXT);
            }
            ProfileField::Title => {
                **text = title.to_string();
                *color = TextColor(AppColors::PRIMARY);
            }
            ProfileField::Slogan => {
                **text = slogan.to_string();
                *color = TextColor(AppColors::TEXT_MUTED);
            }
            ProfileField::Level => {
                **text = format!("Lv.{}", user.level);
                *color = TextColor(AppColors::TEXT);
            }
            ProfileField::Exp => {
                let next_level_exp = exp_for_level(user.level + 1);
                **text = format!("{}/{}", user.exp, next_level_exp);
                *color = TextColor(AppColors::TEXT);
            }
            ProfileField::Gender => {
                **text = gender_text(&user.gender).to_string();
                *color = TextColor(AppColors::TEXT);
            }
            ProfileField::Email => {
                **text = email.to_string();
                *color = TextColor(AppColors::TEXT_SECONDARY);
            }
            ProfileField::CreatedAt => {
                **text = created.clone();
                *color = TextColor(AppColors::TEXT_SECONDARY);
            }
            ProfileField::UserId => {
                **text = non_empty(&user.id, "--");
                *color = TextColor(AppColors::TEXT_SECONDARY);
            }
            ProfileField::Verified => {
                **text = verified.to_string();
                *color = if user.verified == Some(true) {
                    TextColor(Color::srgb(0.3, 0.8, 0.4))
                } else {
                    TextColor(AppColors::TEXT_SECONDARY)
                };
            }
            ProfileField::Characters => {
                **text = characters.clone();
                *color = TextColor(AppColors::TEXT_SECONDARY);
            }
            ProfileField::PunchIn => {
                **text = punch_text.to_string();
                *color = TextColor(AppColors::TEXT);
            }
        }
    }

    *last_user_name = user_tag;
}

/// 更新头像图片（从缓存加载后替换占位符）
pub fn update_profile_avatar(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    avatar_query: Query<(Entity, &ProfileAvatarImage, &ChildOf), Without<ImageNode>>,
    mut text_node_query: Query<&mut Node>,
) {
    for (entity, avatar, child_of) in avatar_query.iter() {
        if avatar.url.is_empty() {
            continue;
        }
        if let Some(handle) = image_cache.handles.get(&avatar.url) {
            // 隐藏占位符文本
            if let Ok(mut node) = text_node_query.get_mut(entity) {
                node.display = Display::None;
            }
            // 在父容器中添加圆形图片
            commands.entity(child_of.parent()).with_children(|parent| {
                parent.spawn((
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(layout::AVATAR_IMAGE_SIZE),
                        height: Val::Px(layout::AVATAR_IMAGE_SIZE),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                ));
            });
            // 移除标记防止重复
            commands.entity(entity).remove::<ProfileAvatarImage>();
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

/// 签到按钮交互
pub fn profile_punch_in_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ProfilePunchInButton>),
    >,
    profile_state: Res<UserProfileState>,
    punch_in_state: Res<PunchInState>,
    mut punch_in_messages: MessageWriter<PunchInRequestEvent>,
) {
    let is_punched = profile_state
        .user
        .as_ref()
        .and_then(|u| u.is_punched)
        .unwrap_or(false)
        || punch_in_state.is_punched;

    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if !is_punched {
                    punch_in_messages.write(PunchInRequestEvent);
                }
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::Hovered => {
                if is_punched {
                    *bg_color = BackgroundColor(AppColors::SECONDARY_HOVER);
                    *border_color = BorderColor::all(AppColors::SECONDARY_HOVER);
                } else {
                    *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
                    *border_color = BorderColor::all(AppColors::PRIMARY_HOVER);
                }
            }
            Interaction::None => {
                if is_punched {
                    *bg_color = BackgroundColor(AppColors::SECONDARY);
                    *border_color = BorderColor::all(AppColors::SECONDARY);
                } else {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                }
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
