//! 登录相关系统

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
use picacg_config::AppSettings;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::widgets::ButtonStyle,
    utils::{
        icons::*,
        text_input::{TextInput, TextInputDisplay},
    },
};

/// 应用颜色常量
pub struct AppColors;

impl AppColors {
    pub const BACKGROUND: Color = Color::srgb(0.1, 0.1, 0.15);
    pub const SURFACE: Color = Color::srgb(0.15, 0.15, 0.2);
    /// 下沉表面（输入框/未选分段按钮）——收编此前 54 处裸值
    pub const SURFACE_SUNKEN: Color = Color::srgb(0.12, 0.12, 0.16);
    /// 通用悬停背景——收编此前 29 处裸值
    pub const SURFACE_HOVER: Color = Color::srgb(0.2, 0.2, 0.25);
    /// 页头/输入栏底色——收编此前 8 处裸值
    pub const HEADER_BG: Color = Color::srgb(0.08, 0.08, 0.12);
    pub const CARD_BG: Color = Color::srgb(0.18, 0.18, 0.25);
    pub const PRIMARY: Color = Color::srgb(0.2, 0.4, 0.8);
    pub const SECONDARY: Color = Color::srgb(0.3, 0.3, 0.4);
    pub const TEXT: Color = Color::WHITE;
    pub const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.7);
    pub const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.6);
    pub const ERROR: Color = Color::srgb(1.0, 0.3, 0.3);
    pub const BORDER: Color = Color::srgb(0.3, 0.3, 0.4);
}

/// 登录表单字段标识
///
/// 焦点由 `InputFocus` 单独仲裁、Tab 顺序由 `TabIndex` 组件决定，
/// 此枚举只用于区分「这个输入框对应表单的哪个字段」。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoginInputType {
    #[default]
    Email,
    Password,
}

/// 登录输入框组件
#[derive(Component, Default, Clone)]
pub struct LoginInputField {
    pub input_type: LoginInputType,
}

/// 显示/隐藏密码切换按钮
#[derive(Component, Default, Clone)]
pub struct ShowPasswordToggle;

/// 显示/隐藏密码按钮内的图标文本
#[derive(Component, Default, Clone)]
pub struct ShowPasswordIcon;

/// 创建登录界面
pub fn setup_login_ui(mut commands: Commands, login_state: Res<LoginFormState>) {
    commands.spawn_scene(login_page(&login_state));
}

/// 登录页面场景
fn login_page(login_state: &LoginFormState) -> impl Scene + use<> {
    let login_label = if login_state.is_loading {
        "登录中..."
    } else {
        "登录"
    };
    let (error_text, error_display) = match login_state.error {
        Some(ref error) => (error.clone(), Display::Flex),
        None => (String::new(), Display::None),
    };
    let email = login_state.email.clone();
    let password = login_state.password.clone();
    let save_password = login_state.save_password;
    let auto_login = login_state.auto_login;
    let auto_punch_in = login_state.auto_punch_in;

    bsn! {
        LoginRoot
        // Tab 环的作用域：子树内所有带 TabIndex 的实体参与循环
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
                Text("PicACG 漫画客户端")
                TextFont { font_size: FontSize::Px(32.0) }
                TextColor(AppColors::PRIMARY)
            ),
            (
                // 副标题
                Text("Rust Bevy 版")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { margin: UiRect::bottom(Val::Px(30.0)) }
            ),
            (
                // 表单容器
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(400.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(15.0),
                }
                Children [
                    // 用户名行
                    input_row("用户名:", email, LoginInputType::Email),
                    // 密码行
                    input_row("密码:", password, LoginInputType::Password),
                    (
                        // 复选框行
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            margin: UiRect::vertical(Val::Px(5.0)),
                        }
                        Children [
                            checkbox("保存密码", save_password, LoginCheckboxType::SavePassword),
                            checkbox("自动登录", auto_login, LoginCheckboxType::AutoLogin),
                            checkbox("自动打卡", auto_punch_in, LoginCheckboxType::AutoPunchIn),
                        ]
                    ),
                    (
                        // 登录按钮
                        LoginButton
                        TabIndex(2)
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                        }
                        template_value(BorderColor::all(Color::NONE))
                        BackgroundColor(AppColors::PRIMARY)
                        Children [
                            (
                                Text({login_label})
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 代理设置按钮
                        ProxySettingsButton
                        TabIndex(3)
                        Button
                        template_value(ButtonStyle::secondary())
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                        }
                        template_value(BorderColor::all(Color::NONE))
                        BackgroundColor(AppColors::SECONDARY)
                        Children [
                            (
                                Text("代理设置")
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 注册提示行
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.0),
                            margin: UiRect::top(Val::Px(10.0)),
                        }
                        Children [
                            (
                                Text("还没有账号？")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            ),
                            (
                                // "立即注册" 链接按钮
                                RegisterButton
                                Button
                                template_value(ButtonStyle::ghost())
                                Node { padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)) }
                                BackgroundColor(Color::NONE)
                                Children [
                                    (
                                        Text("立即注册")
                                        TextFont { font_size: FontSize::Px(14.0) }
                                        TextColor(AppColors::PRIMARY)
                                    )
                                ]
                            ),
                            (
                                // 分隔符
                                Text("|")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT_MUTED)
                            ),
                            (
                                // "忘记密码" 链接按钮
                                ForgotPasswordLink
                                Button
                                template_value(ButtonStyle::ghost())
                                Node { padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)) }
                                BackgroundColor(Color::NONE)
                                Children [
                                    (
                                        Text("忘记密码")
                                        TextFont { font_size: FontSize::Px(14.0) }
                                        TextColor(AppColors::PRIMARY)
                                    )
                                ]
                            ),
                        ]
                    ),
                ]
            ),
            (
                // 提示信息
                Text("提示: 点击输入框后使用键盘输入")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { margin: UiRect::top(Val::Px(10.0)) }
            ),
            (
                // 错误信息（始终创建，按需显示/隐藏）
                LoginErrorText
                Text({error_text})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::ERROR)
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    display: {error_display},
                }
            ),
        ]
    }
}

/// 输入行场景（标签 + 输入框，密码行追加显示/隐藏按钮）
fn input_row(label: &str, value: String, input_type: LoginInputType) -> impl Scene + use<> {
    let is_password = input_type == LoginInputType::Password;
    // Tab 环顺序：邮箱 → 密码 → 登录按钮(2) → 代理设置按钮(3)
    let tab_index = match input_type {
        LoginInputType::Email => 0,
        LoginInputType::Password => 1,
    };
    let text_input = if is_password {
        TextInput::new("点击输入...")
            .with_value(&value)
            .with_password()
    } else {
        TextInput::new("点击输入...").with_value(&value)
    };

    let display = if value.is_empty() {
        "点击输入...".to_string()
    } else if is_password {
        // 初始显示用掩码（后续由通用系统更新）
        String::new()
    } else {
        value.clone()
    };
    let display_color = if value.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };
    let label = label.to_string();

    // 密码行专属的显示/隐藏按钮（非密码行为空列表）
    let password_toggle: Box<dyn SceneList> = if is_password {
        Box::new(bsn_list![(
            ShowPasswordToggle
            Button
            template_value(ButtonStyle::card())
            Node {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
            }
            template_value(BorderColor::all(AppColors::BORDER))
            BackgroundColor(AppColors::SURFACE)
            Children [
                (
                    ShowPasswordIcon
                    Text(ICON_EYE_OFF)
                    TextFont { font_size: FontSize::Px(16.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                )
            ]
        )])
    } else {
        Box::new(bsn_list![])
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
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(80.0) }
            ),
            (
                // 输入框（TextInput 通用组件）
                LoginInputField { input_type: {input_type} }
                TabIndex({tab_index})
                template_value(text_input)
                Button
                Node {
                    flex_grow: 1.0,
                    height: Val::Px(40.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                }
                template_value(BorderColor::all(AppColors::BORDER))
                BackgroundColor(AppColors::SURFACE)
                RelativeCursorPosition
                Children [
                    (
                        TextInputDisplay
                        Text({display})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(display_color)
                    )
                ]
            ),
            {password_toggle},
        ]
    }
}

/// 复选框场景
fn checkbox(label: &str, checked: bool, checkbox_type: LoginCheckboxType) -> impl Scene + use<> {
    let icon = if checked { "[X]" } else { "[ ]" };
    let icon_color = if checked {
        AppColors::PRIMARY
    } else {
        AppColors::TEXT_SECONDARY
    };
    let label = label.to_string();

    bsn! {
        LoginCheckbox { checkbox_type: {checkbox_type} }
        Button
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(5.0),
        }
        Children [
            (
                // 复选框图标（使用方框字符模拟）
                CheckboxIcon { checkbox_type: {checkbox_type} }
                Text({icon})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(icon_color)
            ),
            (
                // 标签
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
        ]
    }
}

/// 清理登录界面
pub fn cleanup_login_ui(
    mut commands: Commands,
    query: Query<Entity, With<LoginRoot>>,
    mut input_focus: ResMut<InputFocus>,
    focusables: Query<
        Entity,
        Or<(
            With<LoginInputField>,
            With<LoginButton>,
            With<ProxySettingsButton>,
        )>,
    >,
) {
    for entity in query.iter() {
        // Bevy 0.17: despawn() 自动递归删除子实体
        commands.entity(entity).despawn();
    }
    // 焦点若停在本页元素上，随页面一并清掉，避免留下悬空实体
    if input_focus.get().is_some_and(|e| focusables.contains(e)) {
        input_focus.clear();
    }
}

/// 焦点环：Tab / 点击落到登录页按钮上时描蓝边
///
/// 输入框自身的边框由通用 `text_input_focus_visuals` 接管，这里只补两个按钮
/// ——否则 Tab 到按钮上没有任何视觉反馈，用户不知道 Enter 会触发哪一个。
pub fn login_focus_ring(
    input_focus: Res<InputFocus>,
    mut buttons: Query<
        (Entity, &mut BorderColor),
        Or<(With<LoginButton>, With<ProxySettingsButton>)>,
    >,
) {
    if !input_focus.is_changed() {
        return;
    }

    let focused = input_focus.get();
    for (entity, mut border) in buttons.iter_mut() {
        let target = if focused == Some(entity) {
            BorderColor::all(AppColors::PRIMARY)
        } else {
            BorderColor::all(Color::NONE)
        };
        if *border != target {
            *border = target;
        }
    }
}

/// 同步 TextInput.value → LoginFormState（保持 LoginFormState
/// 与输入框内容一致）
pub fn login_sync_text_values(
    mut login_state: ResMut<LoginFormState>,
    query: Query<(&LoginInputField, &TextInput), Changed<TextInput>>,
) {
    for (field, input) in query.iter() {
        match field.input_type {
            LoginInputType::Email if login_state.email != input.value => {
                login_state.email.clone_from(&input.value);
            }
            LoginInputType::Password if login_state.password != input.value => {
                login_state.password.clone_from(&input.value);
            }
            _ => {}
        }
    }
}

/// 处理登录页面 Enter 键
///
/// Tab 导航交给上游 `TabNavigationPlugin` + 各实体的 `TabIndex`，
/// 文本编辑交给通用 `TextInput` 系统，这里只剩「按哪个 Enter 干什么」。
pub fn login_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    input_focus: Res<InputFocus>,
    mut login_state: ResMut<LoginFormState>,
    proxy_button_query: Query<Entity, With<ProxySettingsButton>>,
    mut login_messages: MessageWriter<LoginRequestEvent>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        if !matches!(&event.logical_key, Key::Enter) {
            continue;
        }

        // 焦点在代理设置按钮上 → 跳转；其余（邮箱/密码/登录按钮/无焦点）→
        // 提交登录
        let on_proxy_button = input_focus
            .get()
            .is_some_and(|entity| proxy_button_query.contains(entity));
        if on_proxy_button {
            next_route.set(AppRoute::ProxySettings);
            continue;
        }

        let email = login_state.email.clone();
        let password = login_state.password.clone();

        if email.is_empty() || password.is_empty() {
            login_state.error = Some("请输入用户名和密码".to_string());
        } else {
            login_state.is_loading = true;
            login_state.error = None;
            login_messages.write(LoginRequestEvent { email, password });
        }
    }
}

/// 登录按钮交互系统（配色由 `apply_button_interaction` 统一处理）
pub fn login_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoginButton>)>,
    mut login_state: ResMut<LoginFormState>,
    mut login_messages: MessageWriter<LoginRequestEvent>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let email = login_state.email.clone();
        let password = login_state.password.clone();

        if email.is_empty() || password.is_empty() {
            login_state.error = Some("请输入用户名和密码".to_string());
            return;
        }

        login_state.is_loading = true;
        login_state.error = None;
        login_messages.write(LoginRequestEvent { email, password });
    }
}

/// 代理设置按钮交互系统（配色由 `apply_button_interaction` 统一处理）
pub fn proxy_settings_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ProxySettingsButton>)>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_route.set(AppRoute::ProxySettings);
        }
    }
}

/// 复选框交互系统
pub fn login_checkbox_interaction(
    mut interaction_query: Query<(&Interaction, &LoginCheckbox), Changed<Interaction>>,
    mut login_state: ResMut<LoginFormState>,
    mut icon_query: Query<(&CheckboxIcon, &mut Text, &mut TextColor)>,
) {
    for (interaction, checkbox) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // 切换对应的复选框状态
            let new_value = match checkbox.checkbox_type {
                LoginCheckboxType::SavePassword => {
                    login_state.save_password = !login_state.save_password;
                    login_state.save_password
                }
                LoginCheckboxType::AutoLogin => {
                    login_state.auto_login = !login_state.auto_login;
                    // 如果启用自动登录，必须启用保存密码
                    if login_state.auto_login && !login_state.save_password {
                        login_state.save_password = true;
                    }
                    login_state.auto_login
                }
                LoginCheckboxType::AutoPunchIn => {
                    login_state.auto_punch_in = !login_state.auto_punch_in;
                    login_state.auto_punch_in
                }
            };

            // 更新复选框图标
            for (icon, mut text, mut color) in &mut icon_query {
                if icon.checkbox_type == checkbox.checkbox_type {
                    **text = if new_value { "[X]" } else { "[ ]" }.to_string();
                    *color = TextColor(if new_value {
                        AppColors::PRIMARY
                    } else {
                        AppColors::TEXT_SECONDARY
                    });
                }
                // 如果自动登录被启用，同时更新保存密码的图标
                if checkbox.checkbox_type == LoginCheckboxType::AutoLogin
                    && icon.checkbox_type == LoginCheckboxType::SavePassword
                    && login_state.save_password
                {
                    **text = "[X]".to_string();
                    *color = TextColor(AppColors::PRIMARY);
                }
            }

            // 保存设置到配置文件
            save_login_settings(&login_state);
        }
    }
}

/// 显示/隐藏密码切换按钮交互
///
/// 明文标志直接写在密码框的 `TextInput` 上（原先寄存在页面 Focus 资源里），
/// 写入触发 `Changed<TextInput>`，通用渲染系统随即重刷掩码/明文。
pub fn show_password_toggle_interaction(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ShowPasswordToggle>)>,
    mut password_query: Query<(&LoginInputField, &mut TextInput)>,
    mut icon_query: Query<(&mut Text, &mut TextColor), With<ShowPasswordIcon>>,
) {
    for interaction in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut show_password = false;
        for (field, mut input) in password_query.iter_mut() {
            if field.input_type != LoginInputType::Password {
                continue;
            }
            input.show_password = !input.show_password;
            show_password = input.show_password;
        }

        // 更新图标：ICON_EYE 显示明文，ICON_EYE_OFF 隐藏密码
        for (mut text, mut color) in icon_query.iter_mut() {
            if show_password {
                **text = ICON_EYE.to_string();
                *color = TextColor(AppColors::PRIMARY);
            } else {
                **text = ICON_EYE_OFF.to_string();
                *color = TextColor(AppColors::TEXT_SECONDARY);
            }
        }
    }
}

/// 监听 LoginFormState 变化，动态更新错误提示
pub fn update_login_error(
    login_state: Res<LoginFormState>,
    mut error_query: Query<(&mut Text, &mut Node), With<LoginErrorText>>,
) {
    if !login_state.is_changed() {
        return;
    }

    for (mut text, mut node) in error_query.iter_mut() {
        match &login_state.error {
            Some(error) => {
                **text = error.clone();
                node.display = Display::Flex;
            }
            None => {
                node.display = Display::None;
            }
        }
    }
}

/// 保存登录设置到配置文件
fn save_login_settings(login_state: &LoginFormState) {
    let mut settings = AppSettings::global().write();
    settings.login.save_password = login_state.save_password;
    settings.login.auto_login = login_state.auto_login;
    settings.login.auto_punch_in = login_state.auto_punch_in;

    // 如果保存密码，保存用户名和密码
    if login_state.save_password {
        settings.login.saved_email = login_state.email.clone();
        settings.login.saved_password = login_state.password.clone();
    } else {
        settings.login.saved_email.clear();
        settings.login.saved_password.clear();
    }

    // 异步保存配置（忽略错误）
    if let Err(e) = settings.save() {
        tracing::error!("保存登录设置失败: {}", e);
    }
}

/// 登录成功后保存用户名密码（如果启用了保存密码）
pub fn save_credentials_on_login(login_state: &LoginFormState) {
    if login_state.save_password {
        let mut settings = AppSettings::global().write();
        settings.login.saved_email = login_state.email.clone();
        settings.login.saved_password = login_state.password.clone();
        if let Err(e) = settings.save() {
            tracing::error!("保存登录凭据失败: {}", e);
        }
    }
}

/// 注册按钮交互系统（悬停浮起由 ghost 变体统一处理）
pub fn register_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RegisterButton>)>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_route.set(AppRoute::Register);
        }
    }
}

/// 忘记密码链接交互（悬停浮起由 ghost 变体统一处理）
pub fn forgot_password_link_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ForgotPasswordLink>)>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_route.set(AppRoute::ForgotPassword);
        }
    }
}
