//! 注册页面系统

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    ui::RelativeCursorPosition,
    window::PrimaryWindow,
};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{login::AppColors, ui_common::Scrollable},
    utils::text_input::{TextInput, TextInputDisplay},
};

/// 当前注册页面焦点
#[derive(Resource, Default)]
pub struct RegisterInputFocus {
    pub focused: Option<RegisterInputType>,
}

/// 创建注册界面
pub fn setup_register_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    register_state: Res<RegisterFormState>,
) {
    let font: Handle<Font> = get_font();

    commands
        .spawn((
            RegisterRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("注册 PicACG 账号"),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(AppColors::PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // 提示信息
            parent.spawn((
                Text::new("请填写以下信息完成注册"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 滚动容器
            parent
                .spawn((
                    RegisterScrollContainer,
                    Scrollable,
                    ScrollPosition::default(),
                    Node {
                        width: Val::Px(500.0),
                        max_height: Val::Percent(80.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(AppColors::SURFACE),
                ))
                .with_children(|form| {
                    // 基本信息区域
                    spawn_section_title(form, &font, "基本信息");

                    // 邮箱
                    spawn_register_input_row(
                        form,
                        &font,
                        "邮箱/用户名:",
                        &register_state.email,
                        RegisterInputType::Email,
                        "只能包含字母、数字、.和_",
                        false,
                    );

                    // 密码
                    spawn_register_input_row(
                        form,
                        &font,
                        "密码:",
                        &register_state.password,
                        RegisterInputType::Password,
                        "至少8位字符",
                        true,
                    );

                    // 确认密码
                    spawn_register_input_row(
                        form,
                        &font,
                        "确认密码:",
                        &register_state.confirm_password,
                        RegisterInputType::ConfirmPassword,
                        "再次输入密码",
                        true,
                    );

                    // 昵称
                    spawn_register_input_row(
                        form,
                        &font,
                        "昵称:",
                        &register_state.name,
                        RegisterInputType::Name,
                        "显示名称",
                        false,
                    );

                    // 生日
                    spawn_register_input_row(
                        form,
                        &font,
                        "生日:",
                        &register_state.birthday,
                        RegisterInputType::Birthday,
                        "格式: 2000-01-01",
                        false,
                    );

                    // 性别
                    spawn_gender_row(form, &font, register_state.gender);

                    // 安全问题区域
                    spawn_section_title(form, &font, "安全问题（找回密码用）");

                    // 问题1
                    spawn_register_input_row(
                        form,
                        &font,
                        "问题1:",
                        &register_state.question1,
                        RegisterInputType::Question1,
                        "例如: 你的家乡是哪里",
                        false,
                    );
                    spawn_register_input_row(
                        form,
                        &font,
                        "答案1:",
                        &register_state.answer1,
                        RegisterInputType::Answer1,
                        "问题1的答案",
                        false,
                    );

                    // 问题2
                    spawn_register_input_row(
                        form,
                        &font,
                        "问题2:",
                        &register_state.question2,
                        RegisterInputType::Question2,
                        "例如: 你最喜欢的颜色",
                        false,
                    );
                    spawn_register_input_row(
                        form,
                        &font,
                        "答案2:",
                        &register_state.answer2,
                        RegisterInputType::Answer2,
                        "问题2的答案",
                        false,
                    );

                    // 问题3
                    spawn_register_input_row(
                        form,
                        &font,
                        "问题3:",
                        &register_state.question3,
                        RegisterInputType::Question3,
                        "例如: 你最喜欢的食物",
                        false,
                    );
                    spawn_register_input_row(
                        form,
                        &font,
                        "答案3:",
                        &register_state.answer3,
                        RegisterInputType::Answer3,
                        "问题3的答案",
                        false,
                    );

                    // 底部间距
                    form.spawn(Node {
                        height: Val::Px(20.0),
                        ..default()
                    });
                });

            // 按钮区域
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },))
                .with_children(|btns| {
                    // 返回登录按钮
                    btns.spawn((
                        BackToLoginButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(AppColors::SECONDARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("返回登录"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                    // 注册按钮
                    btns.spawn((
                        RegisterSubmitButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if register_state.is_loading {
                            AppColors::SECONDARY
                        } else {
                            AppColors::PRIMARY
                        }),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(if register_state.is_loading {
                                "注册中..."
                            } else {
                                "立即注册"
                            }),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });
                });

            // 错误信息
            if let Some(ref error) = register_state.error {
                parent.spawn((
                    RegisterErrorText,
                    Text::new(error.clone()),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::ERROR),
                    Node {
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }

            // 成功信息
            if let Some(ref success) = register_state.success {
                parent.spawn((
                    RegisterSuccessText,
                    Text::new(success.clone()),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.3, 0.9, 0.3)),
                    Node {
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }
        });

    // 初始化焦点资源
    commands.insert_resource(RegisterInputFocus::default());
}

/// 创建区域标题
fn spawn_section_title(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, title: &str) {
    parent.spawn((
        Text::new(title),
        TextFont {
            font: font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(AppColors::PRIMARY),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(10.0), Val::Px(5.0)),
            ..default()
        },
    ));
}

/// 创建注册输入行
fn spawn_register_input_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    value: &str,
    input_type: RegisterInputType,
    placeholder: &str,
    is_password: bool,
) {
    let display_value = if is_password && !value.is_empty() {
        "*".repeat(value.len())
    } else {
        value.to_string()
    };

    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },))
        .with_children(|row| {
            // 标签
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));

            // 输入框（TextInput 通用组件）
            {
                let mut text_input = TextInput::new(placeholder).with_value(value);
                if is_password {
                    text_input = text_input.with_password();
                }
                row.spawn((
                    RegisterInputField { input_type },
                    text_input,
                    Button,
                    Interaction::default(),
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(36.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(AppColors::BORDER),
                    BackgroundColor(AppColors::CARD_BG),
                    RelativeCursorPosition::default(),
                ))
                .with_children(|input| {
                    input.spawn((
                        TextInputDisplay,
                        Text::new(if display_value.is_empty() {
                            placeholder
                        } else {
                            &display_value
                        }),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(if display_value.is_empty() {
                            AppColors::TEXT_MUTED
                        } else {
                            AppColors::TEXT
                        }),
                    ));
                });
            }
        });
}

/// 创建性别选择行
fn spawn_gender_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_gender: Gender,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },))
        .with_children(|row| {
            // 标签
            row.spawn((
                Text::new("性别:"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));

            // 性别按钮组
            for gender in [Gender::Male, Gender::Female, Gender::Bot] {
                let is_selected = gender == current_gender;
                row.spawn((
                    RegisterGenderButton { gender },
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(if is_selected {
                        AppColors::PRIMARY
                    } else {
                        AppColors::BORDER
                    }),
                    BackgroundColor(if is_selected {
                        AppColors::PRIMARY
                    } else {
                        AppColors::CARD_BG
                    }),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(gender.display_name()),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                });
            }
        });
}

/// 清理注册界面
pub fn cleanup_register_ui(mut commands: Commands, query: Query<Entity, With<RegisterRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<RegisterInputFocus>();
}

/// 注册输入框交互
pub fn register_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &RegisterInputField, &mut BorderColor),
        Changed<Interaction>,
    >,
    mut focus: ResMut<RegisterInputFocus>,
    mut all_inputs: Query<(&RegisterInputField, &mut BorderColor), Without<Interaction>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for (interaction, input, mut border) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            focus.focused = Some(input.input_type);
            *border = BorderColor::all(AppColors::PRIMARY);

            if let Ok(mut window) = window_query.single_mut() {
                window.ime_enabled = true;
            }

            for (other_input, mut other_border) in &mut all_inputs {
                if other_input.input_type != input.input_type {
                    *other_border = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 同步 RegisterInputFocus → TextInput.focused
pub fn register_sync_focus(
    focus: Res<RegisterInputFocus>,
    mut query: Query<(&RegisterInputField, &mut TextInput)>,
) {
    if !focus.is_changed() {
        return;
    }
    for (field, mut input) in query.iter_mut() {
        input.focused = focus.focused == Some(field.input_type);
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

/// 性别按钮交互
pub fn register_gender_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &RegisterGenderButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut register_state: ResMut<RegisterFormState>,
    mut all_gender_buttons: Query<
        (
            &RegisterGenderButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Without<Interaction>,
    >,
) {
    for (interaction, gender_btn, mut bg_color, mut border_color) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            register_state.gender = gender_btn.gender;

            // 更新当前按钮样式
            *bg_color = BackgroundColor(AppColors::PRIMARY);
            *border_color = BorderColor::all(AppColors::PRIMARY);

            // 取消其他按钮选中状态
            for (other_btn, mut other_bg, mut other_border) in &mut all_gender_buttons {
                if other_btn.gender != gender_btn.gender {
                    *other_bg = BackgroundColor(AppColors::CARD_BG);
                    *other_border = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 返回登录按钮交互
pub fn back_to_login_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToLoginButton>),
    >,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut register_state: ResMut<RegisterFormState>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
                // 清空注册状态
                *register_state = RegisterFormState::default();
                next_route.set(AppRoute::Login);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::SECONDARY_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::SECONDARY);
            }
        }
    }
}

/// 注册提交按钮交互
pub fn register_submit_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RegisterSubmitButton>),
    >,
    mut register_state: ResMut<RegisterFormState>,
    mut register_messages: MessageWriter<RegisterRequestEvent>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                trigger_register(&mut register_state, &mut register_messages);
            }
            Interaction::Hovered => {
                if !register_state.is_loading {
                    *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
                }
            }
            Interaction::None => {
                if !register_state.is_loading {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                }
            }
        }
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

/// 取消焦点（点击空白区域）
pub fn unfocus_register_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut focus: ResMut<RegisterInputFocus>,
    mut input_query: Query<(&RegisterInputField, &mut BorderColor, &Interaction)>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let any_clicked = input_query
            .iter()
            .any(|(_, _, i)| *i == Interaction::Pressed);

        if !any_clicked {
            focus.focused = None;
            for (_, mut border, _) in input_query.iter_mut() {
                *border = BorderColor::all(AppColors::BORDER);
            }
            if let Ok(mut window) = window_query.single_mut() {
                window.ime_enabled = false;
            }
        }
    }
}
