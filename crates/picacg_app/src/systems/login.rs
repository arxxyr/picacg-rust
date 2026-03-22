//! 登录相关系统

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    window::PrimaryWindow,
};
use picacg_config::AppSettings;

use super::font_loader::get_font;
use crate::{components::*, events::*, resources::*};

/// 等宽字符宽度估算（SarasaTermSCNerd, font_size 14.0）
const MONO_CHAR_WIDTH: f32 = 8.4;

/// 获取字符索引对应的字节偏移
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// 输入框内文本标记（用于原地更新文本内容）
#[derive(Component)]
pub struct LoginInputText {
    pub input_type: LoginInputType,
}

/// 光标闪烁计时器资源
#[derive(Resource)]
pub struct LoginCursorBlink {
    pub timer: Timer,
    pub visible: bool,
}

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
    pub const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.6);
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
    /// 是否显示密码明文
    pub show_password: bool,
    /// 用户名光标位置（字符索引）
    pub email_cursor: usize,
    /// 密码光标位置（字符索引）
    pub password_cursor: usize,
}

impl LoginInputFocus {
    /// 获取当前聚焦字段的光标位置
    fn cursor_pos(&self) -> usize {
        match self.focused {
            Some(LoginInputType::Email) => self.email_cursor,
            Some(LoginInputType::Password) => self.password_cursor,
            _ => 0,
        }
    }

    /// 设置当前聚焦字段的光标位置
    fn set_cursor_pos(&mut self, pos: usize) {
        match self.focused {
            Some(LoginInputType::Email) => self.email_cursor = pos,
            Some(LoginInputType::Password) => self.password_cursor = pos,
            _ => {}
        }
    }
}

/// 显示/隐藏密码切换按钮
#[derive(Component)]
pub struct ShowPasswordToggle;

/// 显示/隐藏密码按钮内的图标文本
#[derive(Component)]
pub struct ShowPasswordIcon;

/// 创建登录界面
pub fn setup_login_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    login_state: Res<LoginFormState>,
) {
    // 直接加载字体（和参考项目一样）
    let font: Handle<Font> = get_font();

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
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
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
                        Interaction::default(),
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
                        Transform::default(),
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
                        Interaction::default(),
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
                        Transform::default(),
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

                    // 注册提示行
                    form.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.0),
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        Transform::default(),
                    ))
                    .with_children(|row| {
                        // "还没有账号？" 文本
                        row.spawn((
                            Text::new("还没有账号？"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));

                        // "立即注册" 链接按钮
                        row.spawn((
                            RegisterButton,
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            Transform::default(),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("立即注册"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::PRIMARY),
                            ));
                        });
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

            // 错误信息（始终创建，按需显示/隐藏）
            {
                let (error_text, error_display) = match login_state.error {
                    Some(ref error) => (error.clone(), Display::Flex),
                    None => (String::new(), Display::None),
                };
                parent.spawn((
                    LoginErrorText,
                    Text::new(error_text),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::ERROR),
                    Node {
                        margin: UiRect::top(Val::Px(10.0)),
                        display: error_display,
                        ..default()
                    },
                ));
            }
        });

    // 初始化焦点资源
    commands.insert_resource(LoginInputFocus::default());
    // 初始化光标闪烁计时器
    commands.insert_resource(LoginCursorBlink {
        timer: Timer::from_seconds(0.53, TimerMode::Repeating),
        visible: true,
    });
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
            Transform::default(),
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
                Transform::default(), // 必须！点击定位需要 GlobalTransform
            ))
            .with_children(|input| {
                input.spawn((
                    LoginInputText { input_type },
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

            // 密码行：追加显示/隐藏按钮
            if is_password {
                row.spawn((
                    ShowPasswordToggle,
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(AppColors::BORDER),
                    BackgroundColor(AppColors::SURFACE),
                    Transform::default(),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        ShowPasswordIcon,
                        Text::new("◉"), // 隐藏状态图标
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });
            }
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
    commands.remove_resource::<LoginCursorBlink>();
}

/// 输入框和按钮点击交互
pub fn login_input_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &LoginInputField,
            &mut BorderColor,
            &GlobalTransform,
            &ComputedNode,
        ),
        Changed<Interaction>,
    >,
    mut focus: ResMut<LoginInputFocus>,
    mut all_inputs: Query<(&LoginInputField, &mut BorderColor), Without<Interaction>>,
    login_state: Res<LoginFormState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for (interaction, input, mut border, transform, computed) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            focus.focused = Some(input.input_type);
            *border = BorderColor::all(AppColors::PRIMARY);

            // 点击输入框时，根据鼠标位置计算光标位置
            if matches!(
                input.input_type,
                LoginInputType::Email | LoginInputType::Password
            ) {
                let text_len = match input.input_type {
                    LoginInputType::Email => login_state.email.chars().count(),
                    LoginInputType::Password => login_state.password.chars().count(),
                    _ => 0,
                };

                let cursor_pos = window_query
                    .single()
                    .ok()
                    .and_then(|window| {
                        let cursor = window.cursor_position()?;
                        let scale = window.scale_factor();
                        let node_w = computed.size().x / scale;
                        let node_cx = transform.translation().x;
                        let node_left = node_cx - node_w / 2.0;
                        // 屏幕 X → Bevy UI X，减去左侧 padding(10) + border(2)
                        let click_x = cursor.x - window.width() / 2.0;
                        let relative_x = (click_x - node_left - 12.0).max(0.0);
                        let char_pos = (relative_x / MONO_CHAR_WIDTH).round() as usize;
                        Some(char_pos.min(text_len))
                    })
                    .unwrap_or(text_len); // 无法获取位置时定位到末尾

                focus.set_cursor_pos(cursor_pos);
            }

            // 取消其他元素的焦点
            for (other_input, mut other_border) in &mut all_inputs {
                if other_input.input_type != input.input_type {
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
    mut input_query: Query<(&LoginInputField, &mut BorderColor)>,
    mut login_messages: MessageWriter<LoginRequestEvent>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for event in keyboard_events.read() {
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

                // Tab 切换时光标移到末尾
                match new_focus {
                    Some(LoginInputType::Email) => {
                        focus.email_cursor = login_state.email.chars().count();
                    }
                    Some(LoginInputType::Password) => {
                        focus.password_cursor = login_state.password.chars().count();
                    }
                    _ => {}
                }

                for (input, mut border) in &mut input_query {
                    if Some(input.input_type) == new_focus {
                        *border = BorderColor::all(AppColors::PRIMARY);
                    } else {
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
            Key::Enter => match focus.focused {
                Some(LoginInputType::LoginButton)
                | Some(LoginInputType::Email)
                | Some(LoginInputType::Password)
                | None => {
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
                    next_route.set(AppRoute::ProxySettings);
                }
            },
            // 退格：删除光标前一个字符
            Key::Backspace => {
                let Some(focused_type) = focus.focused else {
                    continue;
                };
                let (text, cursor) = match focused_type {
                    LoginInputType::Email => (&mut login_state.email, &mut focus.email_cursor),
                    LoginInputType::Password => {
                        (&mut login_state.password, &mut focus.password_cursor)
                    }
                    _ => continue,
                };
                if *cursor > 0 {
                    let start = char_to_byte_index(text, *cursor - 1);
                    let end = char_to_byte_index(text, *cursor);
                    text.replace_range(start..end, "");
                    *cursor -= 1;
                }
            }
            // Delete：删除光标后一个字符
            Key::Delete => {
                let Some(focused_type) = focus.focused else {
                    continue;
                };
                let (text, cursor) = match focused_type {
                    LoginInputType::Email => (&mut login_state.email, &mut focus.email_cursor),
                    LoginInputType::Password => {
                        (&mut login_state.password, &mut focus.password_cursor)
                    }
                    _ => continue,
                };
                let len = text.chars().count();
                if *cursor < len {
                    let start = char_to_byte_index(text, *cursor);
                    let end = char_to_byte_index(text, *cursor + 1);
                    text.replace_range(start..end, "");
                }
            }
            // 方向键
            Key::ArrowLeft => {
                let Some(focused_type) = focus.focused else {
                    continue;
                };
                match focused_type {
                    LoginInputType::Email => {
                        focus.email_cursor = focus.email_cursor.saturating_sub(1);
                    }
                    LoginInputType::Password => {
                        focus.password_cursor = focus.password_cursor.saturating_sub(1);
                    }
                    _ => {}
                }
            }
            Key::ArrowRight => {
                let Some(focused_type) = focus.focused else {
                    continue;
                };
                match focused_type {
                    LoginInputType::Email => {
                        focus.email_cursor =
                            (focus.email_cursor + 1).min(login_state.email.chars().count());
                    }
                    LoginInputType::Password => {
                        focus.password_cursor =
                            (focus.password_cursor + 1).min(login_state.password.chars().count());
                    }
                    _ => {}
                }
            }
            Key::Home => match focus.focused {
                Some(LoginInputType::Email) => focus.email_cursor = 0,
                Some(LoginInputType::Password) => focus.password_cursor = 0,
                _ => {}
            },
            Key::End => match focus.focused {
                Some(LoginInputType::Email) => {
                    focus.email_cursor = login_state.email.chars().count();
                }
                Some(LoginInputType::Password) => {
                    focus.password_cursor = login_state.password.chars().count();
                }
                _ => {}
            },
            // 字符输入：在光标位置插入
            Key::Character(input) => {
                let Some(focused_type) = focus.focused else {
                    continue;
                };
                if input.chars().any(|c| c.is_control()) {
                    continue;
                }

                let (text, cursor) = match focused_type {
                    LoginInputType::Email => (&mut login_state.email, &mut focus.email_cursor),
                    LoginInputType::Password => {
                        (&mut login_state.password, &mut focus.password_cursor)
                    }
                    _ => continue,
                };
                let byte_idx = char_to_byte_index(text, *cursor);
                text.insert_str(byte_idx, input);
                *cursor += input.chars().count();
            }
            _ => {}
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

/// 显示/隐藏密码切换按钮交互
pub fn show_password_toggle_interaction(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ShowPasswordToggle>)>,
    mut focus: ResMut<LoginInputFocus>,
    mut icon_query: Query<(&mut Text, &mut TextColor), With<ShowPasswordIcon>>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            focus.show_password = !focus.show_password;
            // 更新图标：◎ 显示明文，◉ 隐藏密码
            for (mut text, mut color) in icon_query.iter_mut() {
                if focus.show_password {
                    **text = "◎".to_string();
                    *color = TextColor(AppColors::PRIMARY);
                } else {
                    **text = "◉".to_string();
                    *color = TextColor(AppColors::TEXT_SECONDARY);
                }
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

/// 光标闪烁系统 —— 在聚焦的输入框文本末尾显示/隐藏闪烁光标
pub fn login_cursor_blink(
    time: Res<Time>,
    mut blink: ResMut<LoginCursorBlink>,
    focus: Res<LoginInputFocus>,
    login_state: Res<LoginFormState>,
    mut text_query: Query<(&LoginInputText, &mut Text, &mut TextColor)>,
) {
    let prev_visible = blink.visible;
    blink.timer.tick(time.delta());
    if blink.timer.just_finished() {
        blink.visible = !blink.visible;
    }

    // 输入内容变化时，重置光标为可见状态（打字时光标常亮）
    if login_state.is_changed() {
        blink.visible = true;
        blink.timer.reset();
    }

    // 焦点变化时也重置光标
    if focus.is_changed() {
        blink.visible = true;
        blink.timer.reset();
    }

    // 判断是否需要更新文本
    let blink_changed = prev_visible != blink.visible;
    if !focus.is_changed() && !blink_changed && !login_state.is_changed() {
        return;
    }

    for (input_text, mut text, mut color) in text_query.iter_mut() {
        let is_focused = focus.focused == Some(input_text.input_type);
        let (value, should_mask) = match input_text.input_type {
            LoginInputType::Email => (&login_state.email, false),
            LoginInputType::Password => (&login_state.password, !focus.show_password),
            _ => continue,
        };

        let display_value = if should_mask && !value.is_empty() {
            "*".repeat(value.len())
        } else {
            value.clone()
        };

        if display_value.is_empty() && !is_focused {
            // 无焦点且为空 → 显示占位符
            **text = "点击输入...".to_string();
            *color = TextColor(AppColors::TEXT_SECONDARY);
        } else if is_focused {
            // 有焦点 → 在光标位置插入闪烁光标
            let cursor_char = if blink.visible { "|" } else { " " };
            let cursor_pos = focus.cursor_pos().min(display_value.chars().count());
            let byte_idx = display_value
                .char_indices()
                .nth(cursor_pos)
                .map(|(i, _)| i)
                .unwrap_or(display_value.len());
            let (before, after) = display_value.split_at(byte_idx);
            **text = format!("{}{}{}", before, cursor_char, after);
            *color = TextColor(AppColors::TEXT);
        } else {
            // 无焦点但有内容 → 显示内容（无光标）
            **text = display_value;
            *color = TextColor(AppColors::TEXT);
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

/// 注册按钮交互系统
pub fn register_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &Children),
        (Changed<Interaction>, With<RegisterButton>),
    >,
    mut text_query: Query<&mut TextColor>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for (interaction, children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // 导航到注册页面
                next_route.set(AppRoute::Register);
            }
            Interaction::Hovered => {
                // 悬停时改变颜色
                for child in children.iter() {
                    if let Ok(mut color) = text_query.get_mut(child) {
                        *color = TextColor(AppColors::PRIMARY_HOVER);
                    }
                }
            }
            Interaction::None => {
                // 恢复原始颜色
                for child in children.iter() {
                    if let Ok(mut color) = text_query.get_mut(child) {
                        *color = TextColor(AppColors::PRIMARY);
                    }
                }
            }
        }
    }
}
