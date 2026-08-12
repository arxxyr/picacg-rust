//! 注册页面系统

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::{
        InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    prelude::*,
    ui::RelativeCursorPosition,
};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{login::AppColors, scrollbar::ScrollArea, widgets::ButtonStyle},
    utils::text_input::{TextInput, TextInputDisplay},
};

/// 创建注册界面
pub fn setup_register_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    register_state: Res<RegisterFormState>,
) {
    commands.spawn_scene(register_page(&register_state));
}

/// 注册页面场景
fn register_page(register_state: &RegisterFormState) -> impl Scene + use<> {
    // 注册按钮：加载中时使用禁用配色与文案
    let (submit_bg, submit_style) = if register_state.is_loading {
        (AppColors::SECONDARY, ButtonStyle::secondary())
    } else {
        (AppColors::PRIMARY, ButtonStyle::primary())
    };
    let submit_label = if register_state.is_loading {
        "注册中..."
    } else {
        "立即注册"
    };

    // 错误信息（仅在存在时创建）
    let error_message: Box<dyn SceneList> = match register_state.error {
        Some(ref error) => {
            let error = error.clone();
            Box::new(bsn_list![(
                RegisterErrorText
                Text({error})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::ERROR)
                Node { margin: UiRect::top(Val::Px(10.0)) }
            )])
        }
        None => Box::new(bsn_list![]),
    };

    // 成功信息（仅在存在时创建）
    let success_message: Box<dyn SceneList> = match register_state.success {
        Some(ref success) => {
            let success = success.clone();
            Box::new(bsn_list![(
                RegisterSuccessText
                Text({success})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::srgb(0.3, 0.9, 0.3))
                Node { margin: UiRect::top(Val::Px(10.0)) }
            )])
        }
        None => Box::new(bsn_list![]),
    };

    bsn! {
        RegisterRoot
        // Tab 环的作用域：子树内所有带 TabIndex 的实体参与循环
        TabGroup
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 标题
                Text("注册 PicACG 账号")
                TextFont { font_size: FontSize::Px(28.0) }
                TextColor(AppColors::PRIMARY)
                Node { margin: UiRect::bottom(Val::Px(10.0)) }
            ),
            (
                // 提示信息
                Text("请填写以下信息完成注册")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { margin: UiRect::bottom(Val::Px(20.0)) }
            ),
            (
                // 滚动容器
                RegisterScrollContainer
                ScrollArea
                Node {
                    width: Val::Px(500.0),
                    max_height: Val::Percent(80.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(12.0),
                }
                BackgroundColor(AppColors::SURFACE)
                Children [
                    // 基本信息区域
                    section_title("基本信息"),
                    // 邮箱
                    register_input_row(
                        "邮箱/用户名:",
                        &register_state.email,
                        RegisterInputType::Email,
                        "只能包含字母、数字、.和_",
                        false,
                    ),
                    // 密码
                    register_input_row(
                        "密码:",
                        &register_state.password,
                        RegisterInputType::Password,
                        "至少8位字符",
                        true,
                    ),
                    // 确认密码
                    register_input_row(
                        "确认密码:",
                        &register_state.confirm_password,
                        RegisterInputType::ConfirmPassword,
                        "再次输入密码",
                        true,
                    ),
                    // 昵称
                    register_input_row(
                        "昵称:",
                        &register_state.name,
                        RegisterInputType::Name,
                        "显示名称",
                        false,
                    ),
                    // 生日
                    register_input_row(
                        "生日:",
                        &register_state.birthday,
                        RegisterInputType::Birthday,
                        "格式: 2000-01-01",
                        false,
                    ),
                    // 性别
                    gender_row(register_state.gender),
                    // 安全问题区域
                    section_title("安全问题（找回密码用）"),
                    // 问题1
                    register_input_row(
                        "问题1:",
                        &register_state.question1,
                        RegisterInputType::Question1,
                        "例如: 你的家乡是哪里",
                        false,
                    ),
                    register_input_row(
                        "答案1:",
                        &register_state.answer1,
                        RegisterInputType::Answer1,
                        "问题1的答案",
                        false,
                    ),
                    // 问题2
                    register_input_row(
                        "问题2:",
                        &register_state.question2,
                        RegisterInputType::Question2,
                        "例如: 你最喜欢的颜色",
                        false,
                    ),
                    register_input_row(
                        "答案2:",
                        &register_state.answer2,
                        RegisterInputType::Answer2,
                        "问题2的答案",
                        false,
                    ),
                    // 问题3
                    register_input_row(
                        "问题3:",
                        &register_state.question3,
                        RegisterInputType::Question3,
                        "例如: 你最喜欢的食物",
                        false,
                    ),
                    register_input_row(
                        "答案3:",
                        &register_state.answer3,
                        RegisterInputType::Answer3,
                        "问题3的答案",
                        false,
                    ),
                    (
                        // 底部间距
                        Node { height: Val::Px(20.0) }
                    ),
                ]
            ),
            (
                // 按钮区域
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(20.0)),
                }
                Children [
                    (
                        // 返回登录按钮
                        BackToLoginButton
                        Button
                        template_value(ButtonStyle::secondary())
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        BackgroundColor(AppColors::SECONDARY)
                        Children [
                            (
                                Text("返回登录")
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 注册按钮
                        RegisterSubmitButton
                        Button
                        template_value(submit_style)
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        BackgroundColor(submit_bg)
                        Children [
                            (
                                Text({submit_label})
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
            // 错误信息
            {error_message},
            // 成功信息
            {success_message},
        ]
    }
}

/// 区域标题场景
fn section_title(title: &str) -> impl Scene + use<> {
    let title = title.to_string();
    // 区域标题外边距（上 10 / 下 5）
    let title_margin = UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(10.0), Val::Px(5.0));

    bsn! {
        Text({title})
        TextFont { font_size: FontSize::Px(16.0) }
        TextColor(AppColors::PRIMARY)
        Node { margin: {title_margin} }
    }
}

/// 输入框的 Tab 环顺序
///
/// 按页面视觉顺序编号（问题/答案交替），与 `RegisterInputType` 的声明顺序不同，
/// 所以这里显式写死而不是用 `as i32`。
fn register_tab_index(input_type: RegisterInputType) -> i32 {
    match input_type {
        RegisterInputType::Email => 0,
        RegisterInputType::Password => 1,
        RegisterInputType::ConfirmPassword => 2,
        RegisterInputType::Name => 3,
        RegisterInputType::Birthday => 4,
        RegisterInputType::Question1 => 5,
        RegisterInputType::Answer1 => 6,
        RegisterInputType::Question2 => 7,
        RegisterInputType::Answer2 => 8,
        RegisterInputType::Question3 => 9,
        RegisterInputType::Answer3 => 10,
    }
}

/// 注册输入行场景（标签 + TextInput 输入框）
fn register_input_row(
    label: &str,
    value: &str,
    input_type: RegisterInputType,
    placeholder: &str,
    is_password: bool,
) -> impl Scene + use<> {
    // 掩码按字符数而非字节数：非 ASCII 密码不会多出星号
    let display_value = if is_password && !value.is_empty() {
        "*".repeat(value.chars().count())
    } else {
        value.to_string()
    };

    // 输入框（TextInput 通用组件）
    let mut text_input = TextInput::new(placeholder).with_value(value);
    if is_password {
        text_input = text_input.with_password();
    }

    let label = label.to_string();
    let tab_index = register_tab_index(input_type);
    let is_placeholder = display_value.is_empty();
    let display_text = if is_placeholder {
        placeholder.to_string()
    } else {
        display_value
    };
    let display_color = if is_placeholder {
        AppColors::TEXT_MUTED
    } else {
        AppColors::TEXT
    };

    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
        }
        Children [
            (
                // 标签
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(100.0) }
            ),
            (
                // 输入框（TextInput 通用组件）
                RegisterInputField { input_type: {input_type} }
                TabIndex({tab_index})
                template_value(text_input)
                Button
                Node {
                    flex_grow: 1.0,
                    height: Val::Px(36.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                }
                template_value(BorderColor::all(AppColors::BORDER))
                BackgroundColor(AppColors::CARD_BG)
                RelativeCursorPosition
                Children [
                    (
                        TextInputDisplay
                        Text({display_text})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(display_color)
                    )
                ]
            ),
        ]
    }
}

/// 性别选择行场景
fn gender_row(current_gender: Gender) -> impl Scene {
    // 性别按钮组
    let gender_buttons: Vec<_> = [Gender::Male, Gender::Female, Gender::Bot]
        .into_iter()
        .map(|gender| gender_button(gender, gender == current_gender))
        .collect();

    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
        }
        Children [
            (
                // 标签
                Text("性别:")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(100.0) }
            ),
            {gender_buttons},
        ]
    }
}

/// 单个性别按钮场景
fn gender_button(gender: Gender, is_selected: bool) -> impl Scene {
    let border = BorderColor::all(if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    });
    // 单选组统一走 segment：未选 surface_sunken，选中钉 primary
    let style = ButtonStyle::segment(is_selected);
    let bg_color = if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let label = gender.display_name();

    bsn! {
        RegisterGenderButton { gender: {gender} }
        Button
        template_value(style)
        Node {
            width: Val::Px(60.0),
            height: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
        }
        template_value(border)
        BackgroundColor(bg_color)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 清理注册界面
pub fn cleanup_register_ui(
    mut commands: Commands,
    query: Query<Entity, With<RegisterRoot>>,
    mut input_focus: ResMut<InputFocus>,
    focusables: Query<Entity, With<RegisterInputField>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    // 焦点若停在本页输入框上，随页面一并清掉，避免留下悬空实体
    if input_focus.get().is_some_and(|e| focusables.contains(e)) {
        input_focus.clear();
    }
}

/// 同步 TextInput.value → RegisterFormState
pub fn register_sync_text_values(
    mut register_state: ResMut<RegisterFormState>,
    query: Query<(&RegisterInputField, &TextInput), Changed<TextInput>>,
) {
    for (field, input) in query.iter() {
        let target = get_field_mut(&mut register_state, field.input_type);
        if *target != input.value {
            target.clone_from(&input.value);
        }
    }
}

/// 注册页面动作键（仅 Enter 提交），编辑由通用 TextInput 处理
pub fn register_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut register_state: ResMut<RegisterFormState>,
    mut register_messages: MessageWriter<RegisterRequestEvent>,
) {
    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        if matches!(&event.logical_key, Key::Enter) {
            trigger_register(&mut register_state, &mut register_messages);
        }
    }
}

/// 获取对应字段的可变引用
fn get_field_mut(state: &mut RegisterFormState, field_type: RegisterInputType) -> &mut String {
    match field_type {
        RegisterInputType::Email => &mut state.email,
        RegisterInputType::Password => &mut state.password,
        RegisterInputType::ConfirmPassword => &mut state.confirm_password,
        RegisterInputType::Name => &mut state.name,
        RegisterInputType::Birthday => &mut state.birthday,
        RegisterInputType::Question1 => &mut state.question1,
        RegisterInputType::Question2 => &mut state.question2,
        RegisterInputType::Question3 => &mut state.question3,
        RegisterInputType::Answer1 => &mut state.answer1,
        RegisterInputType::Answer2 => &mut state.answer2,
        RegisterInputType::Answer3 => &mut state.answer3,
    }
}

/// 获取字段值（只读，预留工具函数）
#[allow(dead_code)]
fn get_field_value(state: &RegisterFormState, field_type: RegisterInputType) -> &str {
    match field_type {
        RegisterInputType::Email => &state.email,
        RegisterInputType::Password => &state.password,
        RegisterInputType::ConfirmPassword => &state.confirm_password,
        RegisterInputType::Name => &state.name,
        RegisterInputType::Birthday => &state.birthday,
        RegisterInputType::Question1 => &state.question1,
        RegisterInputType::Question2 => &state.question2,
        RegisterInputType::Question3 => &state.question3,
        RegisterInputType::Answer1 => &state.answer1,
        RegisterInputType::Answer2 => &state.answer2,
        RegisterInputType::Answer3 => &state.answer3,
    }
}

/// 获取字段占位符（预留工具函数）
#[allow(dead_code)]
fn get_field_placeholder(field_type: RegisterInputType) -> &'static str {
    match field_type {
        RegisterInputType::Email => "只能包含字母、数字、.和_",
        RegisterInputType::Password => "至少8位字符",
        RegisterInputType::ConfirmPassword => "再次输入密码",
        RegisterInputType::Name => "显示名称",
        RegisterInputType::Birthday => "格式: 2000-01-01",
        RegisterInputType::Question1 => "例如: 你的家乡是哪里",
        RegisterInputType::Question2 => "例如: 你最喜欢的颜色",
        RegisterInputType::Question3 => "例如: 你最喜欢的食物",
        RegisterInputType::Answer1 => "问题1的答案",
        RegisterInputType::Answer2 => "问题2的答案",
        RegisterInputType::Answer3 => "问题3的答案",
    }
}

/// 是否为密码字段（预留工具函数）
#[allow(dead_code)]
fn is_password_field(field_type: RegisterInputType) -> bool {
    matches!(
        field_type,
        RegisterInputType::Password | RegisterInputType::ConfirmPassword
    )
}

// update_register_input_text 已移除 — 由通用 text_input_cursor_blink 系统处理

/// 性别按钮交互（选中态写入 `ButtonStyle.selected`，配色由全局系统接管）
pub fn register_gender_interaction(
    interaction_query: Query<(&Interaction, &RegisterGenderButton), Changed<Interaction>>,
    mut register_state: ResMut<RegisterFormState>,
    mut all_gender_buttons: Query<(&RegisterGenderButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    let Some(picked) = interaction_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, btn)| btn.gender)
    else {
        return;
    };

    register_state.gender = picked;

    // 整组同步：比较后写，避免无谓触发变更检测
    for (btn, mut style, mut border) in &mut all_gender_buttons {
        let selected = btn.gender == picked;
        if style.selected != selected {
            style.selected = selected;
        }
        let target = if selected {
            AppColors::PRIMARY
        } else {
            AppColors::BORDER
        };
        *border = BorderColor::all(target);
    }
}

/// 返回登录按钮交互（配色由 `apply_button_interaction` 统一接管）
pub fn back_to_login_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackToLoginButton>)>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut register_state: ResMut<RegisterFormState>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 清空注册状态
        *register_state = RegisterFormState::default();
        next_route.set(AppRoute::Login);
    }
}

/// 注册提交按钮交互（配色由 `apply_button_interaction` 统一接管）
pub fn register_submit_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RegisterSubmitButton>)>,
    mut register_state: ResMut<RegisterFormState>,
    mut register_messages: MessageWriter<RegisterRequestEvent>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        trigger_register(&mut register_state, &mut register_messages);
    }
}

/// 触发注册
fn trigger_register(
    register_state: &mut RegisterFormState,
    register_messages: &mut MessageWriter<RegisterRequestEvent>,
) {
    // 验证表单
    if register_state.email.is_empty() {
        register_state.error = Some("请输入邮箱/用户名".to_string());
        return;
    }
    if register_state.password.len() < 8 {
        register_state.error = Some("密码至少需要8位字符".to_string());
        return;
    }
    if register_state.password != register_state.confirm_password {
        register_state.error = Some("两次输入的密码不一致".to_string());
        return;
    }
    if register_state.name.is_empty() {
        register_state.error = Some("请输入昵称".to_string());
        return;
    }
    if register_state.birthday.is_empty() {
        register_state.error = Some("请输入生日".to_string());
        return;
    }
    // 验证生日格式
    if !is_valid_date(&register_state.birthday) {
        register_state.error = Some("生日格式错误，请使用 yyyy-MM-dd 格式".to_string());
        return;
    }
    if register_state.question1.is_empty()
        || register_state.question2.is_empty()
        || register_state.question3.is_empty()
    {
        register_state.error = Some("请填写所有安全问题".to_string());
        return;
    }
    if register_state.answer1.is_empty()
        || register_state.answer2.is_empty()
        || register_state.answer3.is_empty()
    {
        register_state.error = Some("请填写所有安全问题的答案".to_string());
        return;
    }

    // 清除错误，开始注册
    register_state.error = None;
    register_state.success = None;
    register_state.is_loading = true;

    register_messages.write(RegisterRequestEvent {
        email: register_state.email.clone(),
        password: register_state.password.clone(),
        name: register_state.name.clone(),
        birthday: register_state.birthday.clone(),
        gender: register_state.gender.as_api_str().to_string(),
        question1: register_state.question1.clone(),
        question2: register_state.question2.clone(),
        question3: register_state.question3.clone(),
        answer1: register_state.answer1.clone(),
        answer2: register_state.answer2.clone(),
        answer3: register_state.answer3.clone(),
    });
}

/// 验证日期格式 (yyyy-MM-dd)
fn is_valid_date(date: &str) -> bool {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let year = parts[0].parse::<i32>().ok();
    let month = parts[1].parse::<u32>().ok();
    let day = parts[2].parse::<u32>().ok();

    match (year, month, day) {
        (Some(y), Some(m), Some(d)) => {
            (1900..=2100).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d)
        }
        _ => false,
    }
}

/// 处理注册响应
pub fn handle_register_response(
    mut response_events: MessageReader<RegisterResponseEvent>,
    mut register_state: ResMut<RegisterFormState>,
) {
    for event in response_events.read() {
        register_state.is_loading = false;
        match &event.result {
            Ok(msg) => {
                tracing::info!("注册成功: {}", msg);
                register_state.success = Some("注册成功！请返回登录页面登录".to_string());
                register_state.error = None;
            }
            Err(e) => {
                tracing::error!("注册失败: {}", e);
                register_state.error = Some(format!("注册失败: {}", e));
                register_state.success = None;
            }
        }
    }
}
