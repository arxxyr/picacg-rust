//! 代理设置系统

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

use crate::{
    components::*,
    config::settings::{AppSettings, ProxyType},
    resources::*,
    systems::login::{AppColors, FONT_PATH},
};

/// 代理设置页面根组件
#[derive(Component)]
pub struct ProxySettingsRoot;

/// 返回按钮
#[derive(Component)]
pub struct BackToLoginButton;

/// 保存按钮
#[derive(Component)]
pub struct SaveProxyButton;

/// 代理启用切换按钮
#[derive(Component)]
pub struct ProxyEnabledToggle;

/// 代理类型按钮
#[derive(Component)]
pub struct ProxyTypeButton {
    pub proxy_type: ProxyType,
}

/// 输入框组件
#[derive(Component)]
pub struct ProxyInputField {
    pub field_type: ProxyFieldType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyFieldType {
    Host,
    Port,
}

/// 当前焦点的输入框
#[derive(Resource, Default)]
pub struct ProxyInputFocus {
    pub focused: Option<ProxyFieldType>,
}

/// 创建代理设置界面
pub fn setup_proxy_settings_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    proxy_state: Res<ProxySettingsState>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    commands
        .spawn((
            ProxySettingsRoot,
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
                Text::new("代理设置"),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 设置容器
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(400.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|form| {
                    // 启用代理开关
                    spawn_toggle_row(form, &font, "启用代理:", proxy_state.enabled);

                    // 代理类型选择
                    spawn_proxy_type_row(form, &font, proxy_state.proxy_type);

                    // 主机地址
                    spawn_input_field(
                        form,
                        &font,
                        "主机地址:",
                        &proxy_state.host,
                        ProxyFieldType::Host,
                    );

                    // 端口
                    spawn_input_field(
                        form,
                        &font,
                        "端口:",
                        &proxy_state.port,
                        ProxyFieldType::Port,
                    );

                    // 按钮行
                    form.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            column_gap: Val::Px(15.0),
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },
                        Transform::default(),
                    ))
                    .with_children(|buttons| {
                        // 返回按钮
                        buttons
                            .spawn((
                                BackToLoginButton,
                                Button,
                                Node {
                                    width: Val::Px(180.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(AppColors::SECONDARY),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("返回"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT),
                                ));
                            });

                        // 保存按钮
                        buttons
                            .spawn((
                                SaveProxyButton,
                                Button,
                                Node {
                                    width: Val::Px(180.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(AppColors::PRIMARY),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("保存"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT),
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
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });

    // 初始化焦点资源
    commands.insert_resource(ProxyInputFocus::default());
}

fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    enabled: bool,
) {
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
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));

            row.spawn((
                ProxyEnabledToggle,
                Button,
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(if enabled {
                    AppColors::PRIMARY
                } else {
                    AppColors::SECONDARY
                }),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(if enabled { "开启" } else { "关闭" }),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });
        });
}

fn spawn_proxy_type_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current: ProxyType,
) {
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
            row.spawn((
                Text::new("代理类型:"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));

            for proxy_type in [ProxyType::Http, ProxyType::Https, ProxyType::Socks5] {
                let is_selected = proxy_type == current;
                row.spawn((
                    ProxyTypeButton { proxy_type },
                    Button,
                    Node {
                        width: Val::Px(70.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if is_selected {
                        AppColors::PRIMARY
                    } else {
                        AppColors::SECONDARY
                    }),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(match proxy_type {
                            ProxyType::Http => "HTTP",
                            ProxyType::Https => "HTTPS",
                            ProxyType::Socks5 => "SOCKS5",
                        }),
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

fn spawn_input_field(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    value: &str,
    field_type: ProxyFieldType,
) {
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
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));

            row.spawn((
                ProxyInputField { field_type },
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
                    Text::new(if value.is_empty() {
                        "点击输入..."
                    } else {
                        value
                    }),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(if value.is_empty() {
                        AppColors::TEXT_SECONDARY
                    } else {
                        AppColors::TEXT
                    }),
                ));
            });
        });
}

/// 清理代理设置界面
pub fn cleanup_proxy_settings_ui(
    mut commands: Commands,
    query: Query<Entity, With<ProxySettingsRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ProxyInputFocus>();
}

/// 返回按钮交互
pub fn back_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToLoginButton>),
    >,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
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

/// 保存按钮交互
pub fn save_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SaveProxyButton>),
    >,
    proxy_state: Res<ProxySettingsState>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);

                // 保存设置
                let mut settings = AppSettings::global().write();
                settings.proxy.enabled = proxy_state.enabled;
                settings.proxy.proxy_type = proxy_state.proxy_type;
                settings.proxy.host = proxy_state.host.clone();
                settings.proxy.port = proxy_state.port.parse().unwrap_or(1080);
                drop(settings);

                tracing::info!("代理设置已保存");
                next_route.set(AppRoute::Login);
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

/// 代理启用切换交互
pub fn proxy_toggle_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<ProxyEnabledToggle>),
    >,
    mut text_query: Query<&mut Text>,
    mut proxy_state: ResMut<ProxySettingsState>,
) {
    for (interaction, mut bg_color, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            proxy_state.enabled = !proxy_state.enabled;

            *bg_color = BackgroundColor(if proxy_state.enabled {
                AppColors::PRIMARY
            } else {
                AppColors::SECONDARY
            });

            // 更新按钮文字
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = if proxy_state.enabled {
                        "开启".to_string()
                    } else {
                        "关闭".to_string()
                    };
                }
            }
        }
    }
}

/// 代理类型按钮交互
pub fn proxy_type_interaction(
    mut interaction_query: Query<
        (&Interaction, &ProxyTypeButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut proxy_state: ResMut<ProxySettingsState>,
    mut all_buttons: Query<(&ProxyTypeButton, &mut BackgroundColor), Without<Interaction>>,
) {
    for (interaction, button, mut bg_color) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            proxy_state.proxy_type = button.proxy_type;
            *bg_color = BackgroundColor(AppColors::PRIMARY);

            // 更新其他按钮
            for (other_button, mut other_bg) in &mut all_buttons {
                if other_button.proxy_type != button.proxy_type {
                    *other_bg = BackgroundColor(AppColors::SECONDARY);
                }
            }
        }
    }
}

/// 输入框点击交互
pub fn proxy_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &ProxyInputField, &mut BorderColor),
        Changed<Interaction>,
    >,
    mut focus: ResMut<ProxyInputFocus>,
    mut all_inputs: Query<(&ProxyInputField, &mut BorderColor), Without<Interaction>>,
) {
    for (interaction, input, mut border) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            focus.focused = Some(input.field_type);
            *border = BorderColor::all(AppColors::PRIMARY);

            // 取消其他输入框的焦点
            for (other_input, mut other_border) in &mut all_inputs {
                if other_input.field_type != input.field_type {
                    *other_border = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 处理键盘输入
pub fn proxy_keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    focus: Res<ProxyInputFocus>,
    mut proxy_state: ResMut<ProxySettingsState>,
    mut input_query: Query<(&ProxyInputField, &Children)>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    let Some(focused_type) = focus.focused else {
        return;
    };

    for event in keyboard_events.read() {
        // 只处理按下事件
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Backspace => {
                match focused_type {
                    ProxyFieldType::Host => {
                        proxy_state.host.pop();
                    }
                    ProxyFieldType::Port => {
                        proxy_state.port.pop();
                    }
                }
                update_input_text(
                    &mut input_query,
                    &mut text_query,
                    &proxy_state,
                    focused_type,
                );
            }
            Key::Character(input) => {
                // 跳过控制字符
                if input.chars().any(|c| c.is_control()) {
                    continue;
                }

                match focused_type {
                    ProxyFieldType::Host => {
                        proxy_state.host.push_str(input);
                    }
                    ProxyFieldType::Port => {
                        // 端口只接受数字
                        for c in input.chars() {
                            if c.is_ascii_digit() {
                                proxy_state.port.push(c);
                            }
                        }
                    }
                }
                update_input_text(
                    &mut input_query,
                    &mut text_query,
                    &proxy_state,
                    focused_type,
                );
            }
            _ => {}
        }
    }
}

fn update_input_text(
    input_query: &mut Query<(&ProxyInputField, &Children)>,
    text_query: &mut Query<(&mut Text, &mut TextColor)>,
    proxy_state: &ProxySettingsState,
    field_type: ProxyFieldType,
) {
    for (input, children) in input_query.iter() {
        if input.field_type == field_type {
            let value = match field_type {
                ProxyFieldType::Host => &proxy_state.host,
                ProxyFieldType::Port => &proxy_state.port,
            };

            for child in children.iter() {
                if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                    if value.is_empty() {
                        **text = "点击输入...".to_string();
                        *color = TextColor(AppColors::TEXT_SECONDARY);
                    } else {
                        **text = value.clone();
                        *color = TextColor(AppColors::TEXT);
                    }
                }
            }
        }
    }
}
