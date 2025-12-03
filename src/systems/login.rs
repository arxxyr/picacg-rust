//! 登录相关系统

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

use crate::{components::*, config::settings::AppSettings, events::*, resources::*};

/// 字体路径常量
pub const FONT_PATH: &str = "fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf";

/// 应用颜色常量
pub struct AppColors;

impl AppColors {
    pub const BACKGROUND: Color = Color::srgb(0.1, 0.1, 0.15);
    pub const SURFACE: Color = Color::srgb(0.15, 0.15, 0.2);
    pub const CARD_BG: Color = Color::srgb(0.18, 0.18, 0.25);
    pub const PRIMARY: Color = Color::srgb(0.2, 0.4, 0.8);
    pub const PRIMARY_HOVER: Color = Color::srgb(0.25, 0.45, 0.85);
    pub const PRIMARY_PRESSED: Color = Color::srgb(0.15, 0.35, 0.7);
    pub const SECONDARY: Color = Color::srgb(0.3, 0.3, 0.4);
    pub const SECONDARY_HOVER: Color = Color::srgb(0.35, 0.35, 0.45);
    pub const TEXT: Color = Color::WHITE;
    pub const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.7);
    pub const ERROR: Color = Color::srgb(1.0, 0.3, 0.3);
    pub const BORDER: Color = Color::srgb(0.3, 0.3, 0.4);
}

/// 登录输入框类型
/// 登录页面焦点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginInputType {
    Email,
    Password,
    LoginButton,
    ProxySettingsButton,
}

/// 登录输入框组件
#[derive(Component)]
pub struct LoginInputField {
    pub input_type: LoginInputType,
}

/// 当前焦点
#[derive(Resource, Default)]
pub struct LoginInputFocus {
    pub focused: Option<LoginInputType>,
}

/// 创建登录界面
pub fn setup_login_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    login_state: Res<LoginFormState>,
) {
    // 直接加载字体（和参考项目一样）
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    commands
        .spawn((
            LoginRoot,
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
                Text::new("PicACG 漫画客户端"),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(AppColors::PRIMARY),
            ));

            // 副标题
            parent.spawn((
                Text::new("Rust Bevy 版"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 表单容器
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(400.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
                ))
                .with_children(|form| {
                    // 用户名行
                    spawn_input_row(
                        form,
                        &font,
                        "用户名:",
                        &login_state.email,
                        LoginInputType::Email,
                    );

                    // 密码行
                    spawn_input_row(
                        form,
                        &font,
                        "密码:",
                        &login_state.password,
                        LoginInputType::Password,
                    );

                    // 复选框行
                    form.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            margin: UiRect::vertical(Val::Px(5.0)),
                            ..default()
                        },
                        Transform::default(), // 必须添加
                    ))
                    .with_children(|row| {
                        // 保存密码复选框
                        spawn_checkbox(
                            row,
                            &font,
                            "保存密码",
                            login_state.save_password,
                            LoginCheckboxType::SavePassword,
                        );

                        // 自动登录复选框
                        spawn_checkbox(
                            row,
                            &font,
                            "自动登录",
                            login_state.auto_login,
                            LoginCheckboxType::AutoLogin,
                        );

                        // 自动打卡复选框
                        spawn_checkbox(
                            row,
                            &font,
                            "自动打卡",
                            login_state.auto_punch_in,
                            LoginCheckboxType::AutoPunchIn,
                        );
                    });

                    // 登录按钮
                    form.spawn((
                        LoginButton,
                        LoginInputField {
                            input_type: LoginInputType::LoginButton,
                        },
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(Color::NONE),
                        BackgroundColor(AppColors::PRIMARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(if login_state.is_loading {
                                "登录中..."
                            } else {
                                "登录"
                            }),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                    // 代理设置按钮
                    form.spawn((
                        ProxySettingsButton,
                        LoginInputField {
                            input_type: LoginInputType::ProxySettingsButton,
                        },
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(Color::NONE),
                        BackgroundColor(AppColors::SECONDARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("代理设置"),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });
                });

            // 提示信息
            parent.spawn((
                Text::new("提示: 点击输入框后使用键盘输入"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));

            // 错误信息
            if let Some(ref error) = login_state.error {
                parent.spawn((
                    LoginErrorText,
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
        });

    // 初始化焦点资源
    commands.insert_resource(LoginInputFocus::default());
}

/// 创建输入行
fn spawn_input_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    value: &str,
    input_type: LoginInputType,
) {
    let is_password = input_type == LoginInputType::Password;
    let display_value = if is_password && !value.is_empty() {
        "*".repeat(value.len())
    } else {
        value.to_string()
    };

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            Transform::default(), // 必须添加
        ))
        .with_children(|row| {
            // 标签
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    width: Val::Px(80.0),
                    ..default()
                },
            ));

            // 输入框（使用按钮模拟点击）
            row.spawn((
                LoginInputField { input_type },
                Button,
                Node {
                    flex_grow: 1.0,
                    height: Val::Px(40.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
                BackgroundColor(AppColors::SURFACE),
            ))
            .with_children(|input| {
                input.spawn((
                    Text::new(if display_value.is_empty() {
                        "点击输入..."
                    } else {
                        &display_value
                    }),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(if display_value.is_empty() {
                        AppColors::TEXT_SECONDARY
                    } else {
                        AppColors::TEXT
                    }),
                ));
            });
        });
}

/// 创建复选框
fn spawn_checkbox(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    checked: bool,
    checkbox_type: LoginCheckboxType,
) {
    parent
        .spawn((
            LoginCheckbox { checkbox_type },
            Button,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            },
        ))
        .with_children(|row| {
            // 复选框图标（使用方框字符模拟）
            row.spawn((
                CheckboxIcon { checkbox_type },
                Text::new(if checked { "[X]" } else { "[ ]" }),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if checked {
                    AppColors::PRIMARY
                } else {
                    AppColors::TEXT_SECONDARY
                }),
            ));

            // 标签
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 清理登录界面
pub fn cleanup_login_ui(mut commands: Commands, query: Query<Entity, With<LoginRoot>>) {
    for entity in query.iter() {
        // Bevy 0.17: despawn() 自动递归删除子实体
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<LoginInputFocus>();
}

/// 输入框和按钮点击交互
pub fn login_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &LoginInputField, &mut BorderColor),
        Changed<Interaction>,
    >,
    mut focus: ResMut<LoginInputFocus>,
    mut all_inputs: Query<(&LoginInputField, &mut BorderColor), Without<Interaction>>,
) {
    for (interaction, input, mut border) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            focus.focused = Some(input.input_type);
            *border = BorderColor::all(AppColors::PRIMARY);

            // 取消其他元素的焦点
            for (other_input, mut other_border) in &mut all_inputs {
                if other_input.input_type != input.input_type {
                    // 输入框使用边框颜色，按钮使用透明
                    let default_border = match other_input.input_type {
                        LoginInputType::Email | LoginInputType::Password => AppColors::BORDER,
                        LoginInputType::LoginButton | LoginInputType::ProxySettingsButton => {
                            Color::NONE
                        }
                    };
                    *other_border = BorderColor::all(default_border);
                }
            }
        }
    }
}

/// 处理登录页面键盘输入
pub fn login_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut focus: ResMut<LoginInputFocus>,
    mut login_state: ResMut<LoginFormState>,
    mut input_query: Query<(&LoginInputField, &Children, &mut BorderColor)>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
    mut login_messages: MessageWriter<LoginRequestEvent>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for event in keyboard_events.read() {
        // 只处理按下事件
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            // Tab 键切换焦点
            Key::Tab => {
                let new_focus = match focus.focused {
                    None => Some(LoginInputType::Email),
                    Some(LoginInputType::Email) => Some(LoginInputType::Password),
                    Some(LoginInputType::Password) => Some(LoginInputType::LoginButton),
                    Some(LoginInputType::LoginButton) => Some(LoginInputType::ProxySettingsButton),
                    Some(LoginInputType::ProxySettingsButton) => Some(LoginInputType::Email),
                };
                focus.focused = new_focus;

                // 更新所有可聚焦元素的边框颜色
                for (input, _children, mut border) in &mut input_query {
                    if Some(input.input_type) == new_focus {
                        *border = BorderColor::all(AppColors::PRIMARY);
                    } else {
                        // 输入框使用边框颜色，按钮使用透明
                        let default_border = match input.input_type {
                            LoginInputType::Email | LoginInputType::Password => AppColors::BORDER,
                            LoginInputType::LoginButton | LoginInputType::ProxySettingsButton => {
                                Color::NONE
                            }
                        };
                        *border = BorderColor::all(default_border);
                    }
                }
            }
            // Enter 键触发登录或导航
            Key::Enter => {
                match focus.focused {
                    Some(LoginInputType::LoginButton)
                    | Some(LoginInputType::Email)
                    | Some(LoginInputType::Password)
                    | None => {
                        // 触发登录
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
                    Some(LoginInputType::ProxySettingsButton) => {
                        // 导航到代理设置页面
                        next_route.set(AppRoute::ProxySettings);
                    }
                }
            }
            Key::Backspace => {
                let Some(focused_type) = focus.focused else {
                    continue;
                };
                match focused_type {
                    LoginInputType::Email => {
                        login_state.email.pop();
                        update_login_input_text(
                            &mut input_query,
                            &mut text_query,
                            &login_state,
                            focused_type,
                        );
                    }
                    LoginInputType::Password => {
                        login_state.password.pop();
                        update_login_input_text(
                            &mut input_query,
                            &mut text_query,
                            &login_state,
                            focused_type,
                        );
                    }
                    // 按钮不处理退格
                    LoginInputType::LoginButton | LoginInputType::ProxySettingsButton => {}
                }
            }
            Key::Character(input) => {
                let Some(focused_type) = focus.focused else {
                    continue;
                };
                // 跳过控制字符
                if input.chars().any(|c| c.is_control()) {
                    continue;
                }

                match focused_type {
                    LoginInputType::Email => {
                        login_state.email.push_str(input);
                        update_login_input_text(
                            &mut input_query,
                            &mut text_query,
                            &login_state,
                            focused_type,
                        );
                    }
                    LoginInputType::Password => {
                        login_state.password.push_str(input);
                        update_login_input_text(
                            &mut input_query,
                            &mut text_query,
                            &login_state,
                            focused_type,
                        );
                    }
                    // 按钮不处理字符输入
                    LoginInputType::LoginButton | LoginInputType::ProxySettingsButton => {}
                }
            }
            _ => {}
        }
    }
}

fn update_login_input_text(
    input_query: &mut Query<(&LoginInputField, &Children, &mut BorderColor)>,
    text_query: &mut Query<(&mut Text, &mut TextColor)>,
    login_state: &LoginFormState,
    field_type: LoginInputType,
) {
    // 只处理输入框类型，按钮不需要更新文本
    let (value, is_password) = match field_type {
        LoginInputType::Email => (&login_state.email, false),
        LoginInputType::Password => (&login_state.password, true),
        // 按钮不需要更新文本
        LoginInputType::LoginButton | LoginInputType::ProxySettingsButton => return,
    };

    for (input, children, _border) in input_query.iter() {
        if input.input_type == field_type {
            let display_value = if is_password && !value.is_empty() {
                "*".repeat(value.len())
            } else {
                value.clone()
            };

            for child in children.iter() {
                if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                    if display_value.is_empty() {
                        **text = "点击输入...".to_string();
                        *color = TextColor(AppColors::TEXT_SECONDARY);
                    } else {
                        **text = display_value.clone();
                        *color = TextColor(AppColors::TEXT);
                    }
                }
            }
        }
    }
}

/// 登录按钮交互系统
pub fn login_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<LoginButton>),
    >,
    mut login_state: ResMut<LoginFormState>,
    mut login_messages: MessageWriter<LoginRequestEvent>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);

                // 临时：使用硬编码的测试数据（实际应从输入框获取）
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
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
        }
    }
}

/// 代理设置按钮交互系统
pub fn proxy_settings_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ProxySettingsButton>),
    >,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
                next_route.set(AppRoute::ProxySettings);
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
