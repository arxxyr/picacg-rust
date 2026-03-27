//! 忘记密码页面系统
//!
//! 两步流程：
//! 1. 输入邮箱，调用 forgot-password API 获取安全问题
//! 2. 选择安全问题并输入答案，调用 reset-password API 重置密码

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    window::PrimaryWindow,
};

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::login::AppColors,
    utils::text_input::{TextInput, TextInputDisplay},
};

/// 当前忘记密码页面焦点
#[derive(Resource, Default)]
pub struct ForgotPasswordInputFocus {
    pub focused: Option<ForgotPasswordInputType>,
}

/// 创建忘记密码界面
pub fn setup_forgot_password_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    fp_state: Res<ForgotPasswordState>,
) {
    let font: Handle<Font> = get_font();

    commands
        .spawn((
            ForgotPasswordRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("忘记密码"),
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
            let hint_text = match fp_state.step {
                0 => "请输入注册时使用的邮箱/用户名",
                _ => "请选择安全问题并输入答案",
            };
            parent.spawn((
                Text::new(hint_text),
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

            // 表单容器
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(450.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(12.0),
                    ..default()
                },))
                .with_children(|form| {
                    // 邮箱输入行
                    spawn_fp_input_row(
                        form,
                        &font,
                        "邮箱/用户名:",
                        &fp_state.email,
                        ForgotPasswordInputType::Email,
                        "请输入注册邮箱或用户名",
                        fp_state.step > 0, // 步骤1时邮箱不可编辑
                    );

                    // 步骤1：显示安全问题和答案输入
                    if fp_state.step >= 1 {
                        // 安全问题选择区域
                        form.spawn((
                            ForgotPasswordQuestionsArea,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            },
                        ))
                        .with_children(|area| {
                            // 区域标题
                            area.spawn((
                                Text::new("选择一个安全问题:"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                                Node {
                                    margin: UiRect::bottom(Val::Px(4.0)),
                                    ..default()
                                },
                            ));

                            // 三个安全问题按钮
                            let questions = [
                                (1, &fp_state.question1),
                                (2, &fp_state.question2),
                                (3, &fp_state.question3),
                            ];
                            for (no, question) in questions {
                                let is_selected = fp_state.question_no == no;
                                let label = format!("{}. {}", no, question);
                                area.spawn((
                                    ForgotPasswordQuestionButton { question_no: no },
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        width: Val::Percent(100.0),
                                        min_height: Val::Px(36.0),
                                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                        justify_content: JustifyContent::FlexStart,
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
                                        Text::new(label),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(AppColors::TEXT),
                                    ));
                                });
                            }

                            // 答案输入行
                            spawn_fp_input_row(
                                area,
                                &font,
                                "答案:",
                                &fp_state.answer,
                                ForgotPasswordInputType::Answer,
                                "请输入安全问题的答案",
                                false,
                            );
                        });
                    }

                    // 底部间距
                    form.spawn(Node {
                        height: Val::Px(5.0),
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
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },))
                .with_children(|btns| {
                    // 返回登录按钮
                    btns.spawn((
                        ForgotPasswordBackButton,
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

                    // 提交按钮
                    let submit_text = match fp_state.step {
                        0 => {
                            if fp_state.is_loading {
                                "查询中..."
                            } else {
                                "获取安全问题"
                            }
                        }
                        _ => {
                            if fp_state.is_loading {
                                "重置中..."
                            } else {
                                "重置密码"
                            }
                        }
                    };
                    btns.spawn((
                        ForgotPasswordSubmitButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if fp_state.is_loading {
                            AppColors::SECONDARY
                        } else {
                            AppColors::PRIMARY
                        }),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(submit_text),
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
            if let Some(ref error) = fp_state.error {
                parent.spawn((
                    ForgotPasswordErrorText,
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
            if let Some(ref success) = fp_state.success {
                parent.spawn((
                    ForgotPasswordSuccessText,
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
    commands.insert_resource(ForgotPasswordInputFocus::default());
}

/// 创建忘记密码输入行
fn spawn_fp_input_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    value: &str,
    input_type: ForgotPasswordInputType,
    placeholder: &str,
    disabled: bool,
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
            let text_input = TextInput::new(placeholder).with_value(value);

            let bg_color = if disabled {
                Color::srgb(0.12, 0.12, 0.17) // 略暗，表示禁用
            } else {
                AppColors::CARD_BG
            };

            row.spawn((
                ForgotPasswordInputField { input_type },
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
                BackgroundColor(bg_color),
                bevy::ui::RelativeCursorPosition::default(),
            ))
            .with_children(|input| {
                let display_text = if value.is_empty() { placeholder } else { value };
                input.spawn((
                    TextInputDisplay,
                    Text::new(display_text),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(if value.is_empty() {
                        AppColors::TEXT_MUTED
                    } else {
                        AppColors::TEXT
                    }),
                ));
            });
        });
}

/// 清理忘记密码界面
pub fn cleanup_forgot_password_ui(
    mut commands: Commands,
    query: Query<Entity, With<ForgotPasswordRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ForgotPasswordInputFocus>();
}

/// 忘记密码输入框交互
pub fn forgot_password_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &ForgotPasswordInputField, &mut BorderColor),
        Changed<Interaction>,
    >,
    mut focus: ResMut<ForgotPasswordInputFocus>,
    mut all_inputs: Query<(&ForgotPasswordInputField, &mut BorderColor), Without<Interaction>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    fp_state: Res<ForgotPasswordState>,
) {
    for (interaction, input, mut border) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // 步骤1时邮箱不可编辑
            if input.input_type == ForgotPasswordInputType::Email && fp_state.step > 0 {
                continue;
            }

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

/// 同步 ForgotPasswordInputFocus → TextInput.focused
pub fn forgot_password_sync_focus(
    focus: Res<ForgotPasswordInputFocus>,
    mut query: Query<(&ForgotPasswordInputField, &mut TextInput)>,
) {
    if !focus.is_changed() {
        return;
    }
    for (field, mut input) in query.iter_mut() {
        input.focused = focus.focused == Some(field.input_type);
    }
}

/// 同步 TextInput.value → ForgotPasswordState
pub fn forgot_password_sync_text_values(
    mut fp_state: ResMut<ForgotPasswordState>,
    query: Query<(&ForgotPasswordInputField, &TextInput), Changed<TextInput>>,
) {
    for (field, input) in query.iter() {
        match field.input_type {
            ForgotPasswordInputType::Email if fp_state.email != input.value => {
                fp_state.email.clone_from(&input.value);
            }
            ForgotPasswordInputType::Answer if fp_state.answer != input.value => {
                fp_state.answer.clone_from(&input.value);
            }
            _ => {}
        }
    }
}

/// 键盘输入（仅 Enter 提交，编辑由通用 TextInput 系统处理）
pub fn forgot_password_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut fp_state: ResMut<ForgotPasswordState>,
    mut forgot_messages: MessageWriter<ForgotPasswordRequestEvent>,
    mut reset_messages: MessageWriter<ResetPasswordRequestEvent>,
) {
    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        if matches!(&event.logical_key, Key::Enter) {
            trigger_forgot_password_action(
                &mut fp_state,
                &mut forgot_messages,
                &mut reset_messages,
            );
        }
    }
}

/// 提交按钮交互
pub fn forgot_password_submit_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ForgotPasswordSubmitButton>),
    >,
    mut fp_state: ResMut<ForgotPasswordState>,
    mut forgot_messages: MessageWriter<ForgotPasswordRequestEvent>,
    mut reset_messages: MessageWriter<ResetPasswordRequestEvent>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                trigger_forgot_password_action(
                    &mut fp_state,
                    &mut forgot_messages,
                    &mut reset_messages,
                );
            }
            Interaction::Hovered => {
                if !fp_state.is_loading {
                    *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
                }
            }
            Interaction::None => {
                if !fp_state.is_loading {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                }
            }
        }
    }
}

/// 触发忘记密码操作（根据当前步骤）
fn trigger_forgot_password_action(
    fp_state: &mut ForgotPasswordState,
    forgot_messages: &mut MessageWriter<ForgotPasswordRequestEvent>,
    reset_messages: &mut MessageWriter<ResetPasswordRequestEvent>,
) {
    if fp_state.is_loading {
        return;
    }

    match fp_state.step {
        // 步骤0：获取安全问题
        0 => {
            if fp_state.email.is_empty() {
                fp_state.error = Some("请输入邮箱/用户名".to_string());
                return;
            }
            fp_state.error = None;
            fp_state.success = None;
            fp_state.is_loading = true;

            forgot_messages.write(ForgotPasswordRequestEvent {
                email: fp_state.email.clone(),
            });
        }
        // 步骤1：重置密码
        _ => {
            if fp_state.question_no == 0 {
                fp_state.error = Some("请选择一个安全问题".to_string());
                return;
            }
            if fp_state.answer.is_empty() {
                fp_state.error = Some("请输入安全问题的答案".to_string());
                return;
            }
            fp_state.error = None;
            fp_state.success = None;
            fp_state.is_loading = true;

            reset_messages.write(ResetPasswordRequestEvent {
                email: fp_state.email.clone(),
                question_no: fp_state.question_no,
                answer: fp_state.answer.clone(),
            });
        }
    }
}

/// 返回登录按钮交互
pub fn forgot_password_back_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ForgotPasswordBackButton>),
    >,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut fp_state: ResMut<ForgotPasswordState>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
                // 清空状态
                *fp_state = ForgotPasswordState::default();
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

/// 安全问题按钮交互
pub fn forgot_password_question_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ForgotPasswordQuestionButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut fp_state: ResMut<ForgotPasswordState>,
    mut all_question_buttons: Query<
        (
            &ForgotPasswordQuestionButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Without<Interaction>,
    >,
) {
    for (interaction, question_btn, mut bg_color, mut border_color) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            fp_state.question_no = question_btn.question_no;

            // 更新当前按钮样式
            *bg_color = BackgroundColor(AppColors::PRIMARY);
            *border_color = BorderColor::all(AppColors::PRIMARY);

            // 取消其他按钮选中状态
            for (other_btn, mut other_bg, mut other_border) in &mut all_question_buttons {
                if other_btn.question_no != question_btn.question_no {
                    *other_bg = BackgroundColor(AppColors::CARD_BG);
                    *other_border = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 处理忘记密码响应（获取安全问题）
pub fn handle_forgot_password_response(
    mut response_events: MessageReader<ForgotPasswordResponseEvent>,
    mut fp_state: ResMut<ForgotPasswordState>,
    mut commands: Commands,
    root_query: Query<Entity, With<ForgotPasswordRoot>>,
) {
    for event in response_events.read() {
        fp_state.is_loading = false;
        match &event.result {
            Ok((q1, q2, q3)) => {
                tracing::info!("获取安全问题成功");
                fp_state.question1 = q1.clone();
                fp_state.question2 = q2.clone();
                fp_state.question3 = q3.clone();
                fp_state.step = 1;
                fp_state.question_no = 1; // 默认选中第一个问题
                fp_state.error = None;

                // 重建 UI 以显示安全问题
                for entity in root_query.iter() {
                    commands.entity(entity).despawn();
                }
                // OnEnter 不会再次触发，需要手动重建 — 通过设置标志让下一帧处理
                // 实际做法：直接 despawn 后通过 rebuild 系统重建
            }
            Err(e) => {
                tracing::error!("获取安全问题失败: {}", e);
                fp_state.error = Some(format!("获取安全问题失败: {}", e));
            }
        }
    }
}

/// 处理重置密码响应
pub fn handle_reset_password_response(
    mut response_events: MessageReader<ResetPasswordResponseEvent>,
    mut fp_state: ResMut<ForgotPasswordState>,
) {
    for event in response_events.read() {
        fp_state.is_loading = false;
        match &event.result {
            Ok(msg) => {
                tracing::info!("重置密码成功: {}", msg);
                fp_state.success = Some(msg.clone());
                fp_state.error = None;
            }
            Err(e) => {
                tracing::error!("重置密码失败: {}", e);
                fp_state.error = Some(format!("重置密码失败: {}", e));
                fp_state.success = None;
            }
        }
    }
}

/// 检测状态变化后重建 UI
///
/// 当 ForgotPasswordState 发生变化（如步骤切换、错误/成功消息更新）时，
/// 自动重建 UI 以反映最新状态。
pub fn rebuild_forgot_password_ui(
    fp_state: Res<ForgotPasswordState>,
    mut commands: Commands,
    root_query: Query<Entity, With<ForgotPasswordRoot>>,
    asset_server: Res<AssetServer>,
) {
    if !fp_state.is_changed() {
        return;
    }

    // 如果已经没有根节点（正在重建中），创建新的 UI
    if root_query.is_empty() {
        setup_forgot_password_ui(commands, asset_server, fp_state);
        return;
    }

    // 有根节点时，销毁旧 UI 并重建
    for entity in root_query.iter() {
        commands.entity(entity).despawn();
    }
    // 下一帧 root_query 为空，会触发上面的分支重建 UI
}

/// 取消焦点（点击空白区域）
pub fn unfocus_forgot_password_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut focus: ResMut<ForgotPasswordInputFocus>,
    mut input_query: Query<(&ForgotPasswordInputField, &mut BorderColor, &Interaction)>,
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
