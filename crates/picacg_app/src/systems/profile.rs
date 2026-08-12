//! 个人资料系统
//!
//! 显示用户个人信息，包括头像、用户名、等级、经验值、称号、注册日期等

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        widgets::{ButtonStyle, ButtonVariant},
    },
};

// ==================== 组件定义 ====================

/// 个人资料刷新按钮
#[derive(Component, Default, Clone)]
pub struct ProfileRefreshButton;

/// 各字段文本标记（用枚举区分，避免 N 个 Query 冲突）
///
/// `Default` 仅用于满足 BSN `template_value` 的 `Default + Clone`
/// 约束，无业务含义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum ProfileField {
    #[default]
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
#[derive(Component, Default, Clone)]
pub struct ProfileAvatarContainer;

/// 个人资料打卡按钮
#[derive(Component, Default, Clone)]
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

// ==================== 场景函数 ====================

/// 个人资料页面场景
fn profile_page() -> impl Scene {
    // 刷新按钮内边距（左右 12 / 上下 6）
    let refresh_padding = UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0));
    // 滚动区内边距（右侧额外让出滚动条宽度）
    let scroll_padding = UiRect {
        left: Val::Px(layout::MARGIN_H),
        right: Val::Px(layout::MARGIN_H + SCROLLBAR_WIDTH),
        top: Val::Px(20.0),
        bottom: Val::Px(30.0),
    };

    bsn! {
        ProfileRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // ── 标题栏 ──
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    border: UiRect::bottom(Val::Px(1.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        Text("个人资料")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        // 刷新按钮
                        ProfileRefreshButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            padding: {refresh_padding},
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                        }
                        template_value(BorderColor::all(AppColors::BORDER))
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text("↻")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::PRIMARY)
                            ),
                            (
                                Text("刷新")
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::PRIMARY)
                            ),
                        ]
                    ),
                ]
            ),
            (
                // ── 滚动区域 ──
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
                        #ProfileScroll
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: {scroll_padding},
                            row_gap: Val::Px(16.0),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [
                            header_card(),
                            stats_row(),
                            info_card(),
                            punch_in_section(),
                        ]
                    ),
                    scrollbar(#ProfileScroll),
                ]
            ),
        ]
    }
}

/// ── 头部卡片：头像 + 用户名 + 称号 + 签名 ──
fn header_card() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(4.0),
            border_radius: BorderRadius::all(Val::Px(layout::HEADER_CARD_RADIUS)),
        }
        BackgroundColor(AppColors::CARD_BG)
        Children [
            (
                // 圆形头像容器
                ProfileAvatarContainer
                Node {
                    width: Val::Px(layout::AVATAR_SIZE),
                    height: Val::Px(layout::AVATAR_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    overflow: Overflow::clip(),
                    margin: UiRect::bottom(Val::Px(8.0)),
                }
                BackgroundColor(Color::srgb(0.12, 0.12, 0.18))
                template_value(BorderColor::all(AppColors::PRIMARY))
                Children [
                    (
                        ProfileAvatarImage
                        Text("👤")
                        TextFont { font_size: FontSize::Px(40.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    )
                ]
            ),
            (
                // 用户名
                template_value(ProfileField::Name)
                Text("加载中...")
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::top(Val::Px(4.0)) }
            ),
            (
                // 称号
                template_value(ProfileField::Title)
                Text(" ")
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::PRIMARY)
            ),
            (
                // 签名
                template_value(ProfileField::Slogan)
                Text(" ")
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_MUTED)
                Node { margin: UiRect::top(Val::Px(6.0)) }
            ),
        ]
    }
}

/// ── 统计行：等级 / 经验 / 性别 ──
fn stats_row() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
        }
        Children [
            stat_card("等级", ProfileField::Level),
            stat_card("经验", ProfileField::Exp),
            stat_card("性别", ProfileField::Gender),
        ]
    }
}

/// 单个统计卡片场景
fn stat_card(label: &str, field: ProfileField) -> impl Scene + use<> {
    let label = label.to_string();

    bsn! {
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(6.0),
            border_radius: BorderRadius::all(Val::Px(layout::CARD_RADIUS)),
        }
        BackgroundColor(AppColors::CARD_BG)
        Children [
            (
                // 标签
                Text({label})
                TextFont { font_size: FontSize::Px(layout::STAT_LABEL_SIZE) }
                TextColor(AppColors::TEXT_MUTED)
            ),
            (
                // 值
                template_value(field)
                Text("--")
                TextFont { font_size: FontSize::Px(layout::STAT_VALUE_SIZE) }
                TextColor(AppColors::TEXT)
            ),
        ]
    }
}

/// ── 信息详情卡片 ──
fn info_card() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(layout::CARD_PADDING)),
            row_gap: Val::Px(14.0),
            border_radius: BorderRadius::all(Val::Px(layout::CARD_RADIUS)),
        }
        BackgroundColor(AppColors::CARD_BG)
        Children [
            (
                // 标题
                Text("个人信息")
                TextFont { font_size: FontSize::Px(15.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::bottom(Val::Px(2.0)) }
            ),
            info_row("✉  邮箱", ProfileField::Email),
            info_row("📅 注册", ProfileField::CreatedAt),
            info_row("🆔 ID", ProfileField::UserId),
            info_row("✓  认证", ProfileField::Verified),
            info_row("🏷  角色", ProfileField::Characters),
        ]
    }
}

/// 单个信息行场景（label + value）
fn info_row(label: &str, field: ProfileField) -> impl Scene + use<> {
    let label = label.to_string();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
        }
        Children [
            (
                // 标签
                Text({label})
                TextFont { font_size: FontSize::Px(layout::INFO_LABEL_SIZE) }
                TextColor(AppColors::TEXT_MUTED)
                Node { min_width: Val::Px(65.0) }
            ),
            (
                // 值
                template_value(field)
                Text("--")
                TextFont { font_size: FontSize::Px(layout::INFO_VALUE_SIZE) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// ── 签到按钮 ──
fn punch_in_section() -> impl Scene {
    bsn! {
        // 已签到时 refresh_profile_ui 把变体切到 Secondary（灰）
        ProfilePunchInButton
        Button
        template_value(ButtonStyle::primary())
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(14.0), Val::Px(14.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(layout::CARD_RADIUS)),
        }
        BackgroundColor(AppColors::PRIMARY)
        template_value(BorderColor::all(AppColors::PRIMARY))
        Children [
            (
                template_value(ProfileField::PunchIn)
                Text("签到")
                TextFont { font_size: FontSize::Px(15.0) }
                TextColor(AppColors::TEXT)
            )
        ]
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
        if profile_state.user.is_none() && !profile_state.is_loading {
            load_profile_messages.write(LoadUserProfileRequest);
        }
        return;
    }

    let content_area = content_area_query.single().ok();

    let profile_root = commands.spawn_scene(profile_page()).id();

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(profile_root);
    }

    if profile_state.user.is_none() && !profile_state.is_loading {
        load_profile_messages.write(LoadUserProfileRequest);
    }

    tracing::info!("个人资料页面 UI 已创建");
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
    mut punch_btn_query: Query<(&mut ButtonStyle, &mut BorderColor), With<ProfilePunchInButton>>,
    mut image_messages: MessageWriter<LoadImageRequest>,
    mut last_user_name: Local<String>,
) {
    // 资源未变化时零开销（此前每帧走到下方 format! 之后才比较）
    if !profile_state.is_changed() && !punch_in_state.is_changed() {
        return;
    }

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

    // 更新签到按钮配色：底色三态交给 apply_button_interaction 按变体解析，
    // 边框不在 ButtonStyle 管辖内，随变体一并翻转
    let (punch_variant, punch_border) = if is_punched {
        (ButtonVariant::Secondary, AppColors::SECONDARY)
    } else {
        (ButtonVariant::Primary, AppColors::PRIMARY)
    };
    for (mut style, mut border) in punch_btn_query.iter_mut() {
        if style.variant != punch_variant {
            style.variant = punch_variant;
            *border = BorderColor::all(punch_border);
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
        // 加载失败的头像摘掉标记，退出每帧扫描集（此前永久残留）；
        // 占位文本保留，视觉上等同于没有头像
        if image_cache.is_failed(&avatar.url) {
            commands.entity(entity).remove::<ProfileAvatarImage>();
            continue;
        }
        if let Some(handle) = image_cache.get(&avatar.url) {
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
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ProfileRefreshButton>)>,
    mut profile_state: ResMut<UserProfileState>,
    mut load_messages: MessageWriter<LoadUserProfileRequest>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed && !profile_state.is_loading {
            profile_state.user = None;
            profile_state.is_loading = true;
            profile_state.error = None;
            load_messages.write(LoadUserProfileRequest);
        }
    }
}

/// 签到按钮交互
pub fn profile_punch_in_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ProfilePunchInButton>)>,
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

    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed && !is_punched {
            punch_in_messages.write(PunchInRequestEvent);
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
