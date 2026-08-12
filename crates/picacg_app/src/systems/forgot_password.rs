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
    systems::{login::AppColors, widgets::ButtonStyle},
    utils::text_input::{TextInput, TextInputDisplay},
};

/// 创建忘记密码界面
pub fn setup_forgot_password_ui(mut commands: Commands, fp_state: Res<ForgotPasswordState>) {
    commands.spawn_scene(forgot_password_page(&fp_state));
}

/// 忘记密码页面场景
fn forgot_password_page(fp_state: &ForgotPasswordState) -> impl Scene + use<> {
    let hint_text = match fp_state.step {
        0 => "请输入注册时使用的邮箱/用户名",
        _ => "请选择安全问题并输入答案",
    };

    // 步骤1 起邮箱已确认：降级为只读展示行，既不能点击聚焦也不进 Tab 环
    let email = fp_state.email.clone();
    let email_row: Box<dyn SceneList> = match fp_state.step {
        0 => Box::new(bsn_list![fp_input_row(
            "邮箱/用户名:",
            email,
            ForgotPasswordInputType::Email,
            "请输入注册邮箱或用户名",
            1
        )]),
        _ => Box::new(bsn_list![fp_readonly_row("邮箱/用户名:", email)]),
    };

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
    // 加载中降级为次要配色表达禁用感，三态配色由 ButtonStyle 统一
    let (submit_bg, submit_style) = if fp_state.is_loading {
        (AppColors::SECONDARY, ButtonStyle::secondary())
    } else {
        (AppColors::PRIMARY, ButtonStyle::primary())
    };

    // 步骤1：显示安全问题和答案输入
    let questions_area: Box<dyn SceneList> = if fp_state.step >= 1 {
        // 三个安全问题按钮
        let question_buttons: Vec<_> = [
            (1, fp_state.question1.as_str()),
            (2, fp_state.question2.as_str()),
            (3, fp_state.question3.as_str()),
        ]
        .into_iter()
        .map(|(no, question)| fp_question_button(no, question, fp_state.question_no == no))
        .collect();
        let answer = fp_state.answer.clone();

        Box::new(bsn_list![(
            // 安全问题选择区域
            ForgotPasswordQuestionsArea
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                margin: UiRect::top(Val::Px(5.0)),
            }
            Children [
                (
                    // 区域标题
                    Text("选择一个安全问题:")
                    TextFont { font_size: FontSize::Px(14.0) }
                    TextColor(AppColors::TEXT)
                    Node { margin: UiRect::bottom(Val::Px(4.0)) }
                ),
                {question_buttons},
                // 答案输入行
                fp_input_row(
                    "答案:",
                    answer,
                    ForgotPasswordInputType::Answer,
                    "请输入安全问题的答案",
                    2
                ),
            ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 错误信息
    let error_message: Box<dyn SceneList> = match fp_state.error {
        Some(ref error) => {
            let error = error.clone();
            Box::new(bsn_list![(
                ForgotPasswordErrorText
                Text({error})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::ERROR)
                Node { margin: UiRect::top(Val::Px(10.0)) }
            )])
        }
        None => Box::new(bsn_list![]),
    };

    // 成功信息
    let success_message: Box<dyn SceneList> = match fp_state.success {
        Some(ref success) => {
            let success = success.clone();
            Box::new(bsn_list![(
                ForgotPasswordSuccessText
                Text({success})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::srgb(0.3, 0.9, 0.3))
                Node { margin: UiRect::top(Val::Px(10.0)) }
            )])
        }
        None => Box::new(bsn_list![]),
    };

    bsn! {
        ForgotPasswordRoot
        // Tab 环的根（order 0、非模态）：环内成员由子孙节点上的 TabIndex 决定
        TabGroup
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 标题
                Text("忘记密码")
                TextFont { font_size: FontSize::Px(28.0) }
                TextColor(AppColors::PRIMARY)
                Node { margin: UiRect::bottom(Val::Px(10.0)) }
            ),
            (
                // 提示信息
                Text({hint_text})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { margin: UiRect::bottom(Val::Px(20.0)) }
            ),
            (
                // 表单容器
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(450.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(12.0),
                }
                Children [
                    // 邮箱行（步骤0 可编辑，步骤1 起只读）
                    {email_row},
                    {questions_area},
                    (
                        // 底部间距
                        Node { height: Val::Px(5.0) }
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
                    margin: UiRect::top(Val::Px(15.0)),
                }
                Children [
                    (
                        // 返回登录按钮
                        ForgotPasswordBackButton
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
                        // 提交按钮
                        ForgotPasswordSubmitButton
                        Button
                        template_value(submit_style)
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        BackgroundColor({submit_bg})
                        Children [
                            (
                                Text({submit_text})
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
            {error_message},
            {success_message},
        ]
    }
}

/// 安全问题选择按钮场景
fn fp_question_button(question_no: i32, question: &str, is_selected: bool) -> impl Scene + use<> {
    let label = format!("{}. {}", question_no, question);
    let border_color = if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };
    // 单选组统一走 segment：未选 surface_sunken，选中钉 primary
    let style = ButtonStyle::segment(is_selected);
    let bg_color = if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };

    bsn! {
        ForgotPasswordQuestionButton { question_no: {question_no} }
        Button
        template_value(style)
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(36.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
        }
        template_value(BorderColor::all(border_color))
        BackgroundColor({bg_color})
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 忘记密码输入行场景（标签 + 可编辑输入框）
///
/// 聚焦、边框、IME、光标由 `utils::text_input` 的通用系统
/// 按 `InputFocus` 接管，这里只负责布局与 Tab 次序。
fn fp_input_row(
    label: &str,
    value: String,
    input_type: ForgotPasswordInputType,
    placeholder: &str,
    tab_index: i32,
) -> impl Scene + use<> {
    let label = label.to_string();

    // 输入框（TextInput 通用组件）
    let text_input = TextInput::new(placeholder).with_value(&value);

    let display_color = if value.is_empty() {
        AppColors::TEXT_MUTED
    } else {
        AppColors::TEXT
    };
    let display_text = if value.is_empty() {
        placeholder.to_string()
    } else {
        value
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
                ForgotPasswordInputField { input_type: {input_type} }
                template_value(text_input)
                Button
                TabIndex({tab_index})
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
                        TextColor({display_color})
                    )
                ]
            ),
        ]
    }
}

/// 只读展示行（步骤1 的邮箱：已确认，不可再改）
///
/// 不挂 `TextInput` / `Button`：既不进焦点仲裁，也不进 Tab 环。
fn fp_readonly_row(label: &str, value: String) -> impl Scene + use<> {
    let label = label.to_string();

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
                // 只读值（配色比可编辑输入框更暗，表达"不可改"）
                Node {
                    flex_grow: 1.0,
                    height: Val::Px(36.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                }
                template_value(BorderColor::all(AppColors::BORDER))
                BackgroundColor(AppColors::SURFACE_SUNKEN)
                Children [
                    (
                        Text({value})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    )
                ]
            ),
        ]
    }
}

/// 清理忘记密码界面
pub fn cleanup_forgot_password_ui(
    mut commands: Commands,
    query: Query<Entity, With<ForgotPasswordRoot>>,
    mut input_focus: ResMut<InputFocus>,
    focusables: Query<Entity, With<ForgotPasswordInputField>>,
) {
    // 焦点若还停在本页输入框上，随页面一并清掉，避免留下悬空实体
    //（IME 由通用 `text_input_focus_visuals` 随之关闭）
    if input_focus.get().is_some_and(|e| focusables.contains(e)) {
        input_focus.clear();
    }

    for entity in query.iter() {
        commands.entity(entity).despawn();
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

/// 提交按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn forgot_password_submit_interaction(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<ForgotPasswordSubmitButton>),
    >,
    mut fp_state: ResMut<ForgotPasswordState>,
    mut forgot_messages: MessageWriter<ForgotPasswordRequestEvent>,
    mut reset_messages: MessageWriter<ResetPasswordRequestEvent>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            trigger_forgot_password_action(
                &mut fp_state,
                &mut forgot_messages,
                &mut reset_messages,
            );
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

/// 返回登录按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn forgot_password_back_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ForgotPasswordBackButton>)>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut fp_state: ResMut<ForgotPasswordState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // 清空状态
            *fp_state = ForgotPasswordState::default();
            next_route.set(AppRoute::Login);
        }
    }
}

/// 安全问题按钮交互（单选组：只改选中态，配色交给 ButtonStyle）
pub fn forgot_password_question_interaction(
    interaction_query: Query<(&Interaction, &ForgotPasswordQuestionButton), Changed<Interaction>>,
    mut fp_state: ResMut<ForgotPasswordState>,
    mut all_question_buttons: Query<(
        &ForgotPasswordQuestionButton,
        &mut ButtonStyle,
        &mut BorderColor,
    )>,
) {
    // 一帧内至多一个按钮被按下
    let Some(question_no) = interaction_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, button)| button.question_no)
    else {
        return;
    };

    fp_state.question_no = question_no;

    for (button, mut style, mut border_color) in &mut all_question_buttons {
        let is_selected = button.question_no == question_no;
        if style.selected != is_selected {
            style.selected = is_selected;
        }
        *border_color = BorderColor::all(if is_selected {
            AppColors::PRIMARY
        } else {
            AppColors::BORDER
        });
    }
}

/// 处理忘记密码响应（获取安全问题）
///
/// 只改状态，UI 由 `rebuild_forgot_password_ui` 按场景指纹重建。
pub fn handle_forgot_password_response(
    mut response_events: MessageReader<ForgotPasswordResponseEvent>,
    mut fp_state: ResMut<ForgotPasswordState>,
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

/// 场景指纹 —— 只有这些字段变化才需要重建 UI
///
/// 邮箱/答案由 `TextInput` 自身持有并渲染，`question_no` 的选中态由
/// `forgot_password_question_interaction` 原地更新：三者都不进指纹。
/// 于是输入过程中不再重建整页，`InputFocus` 指向的输入框实体得以存活。
pub struct ForgotPasswordSceneKey {
    step: u8,
    is_loading: bool,
    error: Option<String>,
    success: Option<String>,
    questions: [String; 3],
}

impl ForgotPasswordSceneKey {
    fn of(state: &ForgotPasswordState) -> Self {
        Self {
            step: state.step,
            is_loading: state.is_loading,
            error: state.error.clone(),
            success: state.success.clone(),
            questions: [
                state.question1.clone(),
                state.question2.clone(),
                state.question3.clone(),
            ],
        }
    }

    /// 与当前状态是否一致（逐字段比较，不分配）
    fn matches(&self, state: &ForgotPasswordState) -> bool {
        self.step == state.step
            && self.is_loading == state.is_loading
            && self.error == state.error
            && self.success == state.success
            && self.questions[0] == state.question1
            && self.questions[1] == state.question2
            && self.questions[2] == state.question3
    }
}

/// 状态变化后重建 UI
///
/// 步骤切换、加载态、安全问题、错误/成功提示改变时整页重建；纯文本输入不重建。
pub fn rebuild_forgot_password_ui(
    fp_state: Res<ForgotPasswordState>,
    mut commands: Commands,
    root_query: Query<Entity, With<ForgotPasswordRoot>>,
    mut last_built: Local<Option<(Entity, ForgotPasswordSceneKey)>>,
) {
    // 根节点缺失（上一帧刚被销毁）→ 按当前状态重建
    let Some(root) = root_query.iter().next() else {
        let root = commands.spawn_scene(forgot_password_page(&fp_state)).id();
        *last_built = Some((root, ForgotPasswordSceneKey::of(&fp_state)));
        return;
    };

    // 认得这棵 UI：指纹变了就销毁（下一帧走上面的分支重建），没变则原地不动
    if let Some((built, key)) = last_built.as_ref()
        && *built == root
    {
        if !key.matches(&fp_state) {
            for entity in root_query.iter() {
                commands.entity(entity).despawn();
            }
        }
        return;
    }

    // 陌生的 UI（OnEnter 刚建好）→ 只补记指纹
    *last_built = Some((root, ForgotPasswordSceneKey::of(&fp_state)));
}
