//! 代理设置系统

use bevy::{
    input_focus::{
        InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    prelude::*,
    ui::RelativeCursorPosition,
};
use picacg_config::{AppSettings, ProxyType};

use crate::{
    resources::*,
    systems::{login::AppColors, widgets::ButtonStyle},
    utils::text_input::{TextInput, TextInputDisplay},
};

/// 代理设置页面根组件
#[derive(Component, Default, Clone)]
pub struct ProxySettingsRoot;

/// 返回按钮
#[derive(Component, Default, Clone)]
pub struct BackToLoginButton;

/// 保存按钮
#[derive(Component, Default, Clone)]
pub struct SaveProxyButton;

/// 代理启用切换按钮
#[derive(Component, Default, Clone)]
pub struct ProxyEnabledToggle;

/// 代理认证切换按钮
#[derive(Component, Default, Clone)]
pub struct ProxyAuthToggle;

/// 代理类型按钮
#[derive(Component, Default, Clone)]
pub struct ProxyTypeButton {
    pub proxy_type: ProxyType,
}

/// 输入框组件
#[derive(Component, Default, Clone)]
pub struct ProxyInputField {
    pub field_type: ProxyFieldType,
}

/// 代理输入框字段类型
///
/// `Default` 仅用于满足 BSN 组件补丁的 `Default` 约束，无业务含义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyFieldType {
    #[default]
    Host,
    Port,
    Username,
    Password,
}

/// 创建代理设置界面
pub fn setup_proxy_settings_ui(mut commands: Commands, proxy_state: Res<ProxySettingsState>) {
    commands.spawn_scene(proxy_settings_page(&proxy_state));
}

/// 代理设置页面场景
fn proxy_settings_page(proxy_state: &ProxySettingsState) -> impl Scene + use<> {
    let enabled = proxy_state.enabled;
    let proxy_type = proxy_state.proxy_type;
    let host = proxy_state.host.clone();
    let port = proxy_state.port.clone();
    let use_auth = proxy_state.use_auth;
    let username = proxy_state.username.clone();
    let password = proxy_state.password.clone();

    bsn! {
        ProxySettingsRoot
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
                Text("代理设置")
                TextFont { font_size: FontSize::Px(28.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::bottom(Val::Px(30.0)) }
            ),
            (
                // 设置容器
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(400.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(15.0),
                }
                Children [
                    // 启用代理开关
                    toggle_row("启用代理:", enabled),
                    // 代理类型选择
                    proxy_type_row(proxy_type),
                    // 主机地址
                    input_field_row("主机地址:", host, ProxyFieldType::Host, 1, false),
                    // 端口
                    input_field_row("端口:", port, ProxyFieldType::Port, 2, false),
                    // 代理认证（用户名/密码留空即匿名代理）
                    auth_toggle_row(use_auth),
                    input_field_row("用户名:", username, ProxyFieldType::Username, 3, false),
                    input_field_row("密码:", password, ProxyFieldType::Password, 4, true),
                    (
                        // 按钮行
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            column_gap: Val::Px(15.0),
                            margin: UiRect::top(Val::Px(20.0)),
                        }
                        Children [
                            (
                                // 返回按钮
                                BackToLoginButton
                                Button
                                template_value(ButtonStyle::secondary())
                                Node {
                                    width: Val::Px(180.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                }
                                BackgroundColor(AppColors::SECONDARY)
                                Children [
                                    (
                                        Text("返回")
                                        TextFont { font_size: FontSize::Px(16.0) }
                                        TextColor(AppColors::TEXT)
                                    )
                                ]
                            ),
                            (
                                // 保存按钮
                                SaveProxyButton
                                Button
                                template_value(ButtonStyle::primary())
                                Node {
                                    width: Val::Px(180.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                }
                                BackgroundColor(AppColors::PRIMARY)
                                Children [
                                    (
                                        Text("保存")
                                        TextFont { font_size: FontSize::Px(16.0) }
                                        TextColor(AppColors::TEXT)
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
                Node { margin: UiRect::top(Val::Px(20.0)) }
            ),
        ]
    }
}

/// 开关行场景（标签 + 启用/关闭按钮）
fn toggle_row(label: &str, enabled: bool) -> impl Scene + use<> {
    let label = label.to_string();
    // 开/关是双态单选，统一走 segment：关闭态 surface_sunken，开启态钉 primary
    let toggle_bg = if enabled {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let toggle_text = if enabled { "开启" } else { "关闭" };

    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
        }
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(100.0) }
            ),
            (
                ProxyEnabledToggle
                Button
                template_value(ButtonStyle::segment(enabled))
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor({toggle_bg})
                Children [
                    (
                        Text({toggle_text})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
        ]
    }
}

/// 认证开关行场景（与 `toggle_row` 同款外观，标记组件不同）
///
/// 认证凭据的**生效**由 config 层 `to_proxy_url` 决定（`use_auth &&
/// username 非空` 才拼 `user:pass@`），这里只管把三个值收进状态。
fn auth_toggle_row(use_auth: bool) -> impl Scene + use<> {
    let toggle_bg = if use_auth {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let toggle_text = if use_auth { "开启" } else { "关闭" };

    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
        }
        Children [
            (
                Text("代理认证:")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(100.0) }
            ),
            (
                ProxyAuthToggle
                Button
                template_value(ButtonStyle::segment(use_auth))
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor({toggle_bg})
                Children [
                    (
                        Text({toggle_text})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
        ]
    }
}

/// 代理类型选择行场景（标签 + 三个类型按钮）
fn proxy_type_row(current: ProxyType) -> impl Scene {
    let type_buttons: Vec<_> = [ProxyType::Http, ProxyType::Https, ProxyType::Socks5]
        .into_iter()
        .map(|proxy_type| proxy_type_button(proxy_type, current))
        .collect();

    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
        }
        Children [
            (
                Text("代理类型:")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(100.0) }
            ),
            {type_buttons},
        ]
    }
}

/// 单个代理类型按钮场景
fn proxy_type_button(proxy_type: ProxyType, current: ProxyType) -> impl Scene {
    let is_selected = proxy_type == current;
    let button_bg = if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let button_text = match proxy_type {
        ProxyType::Http => "HTTP",
        ProxyType::Https => "HTTPS",
        ProxyType::Socks5 => "SOCKS5",
    };

    bsn! {
        ProxyTypeButton { proxy_type: {proxy_type} }
        Button
        template_value(ButtonStyle::segment(is_selected))
        Node {
            width: Val::Px(70.0),
            height: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor({button_bg})
        Children [
            (
                Text({button_text})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 输入行场景（标签 + 输入框，TextInput 通用组件）
///
/// 聚焦、边框、IME、光标由 `utils::text_input` 的通用系统
/// 按 `InputFocus` 接管，这里只负责布局与 Tab 次序。
fn input_field_row(
    label: &str,
    value: String,
    field_type: ProxyFieldType,
    tab_index: i32,
    password: bool,
) -> impl Scene + use<> {
    let label = label.to_string();
    let mut text_input = TextInput::new("点击输入...").with_value(&value);
    if password {
        text_input = text_input.with_password();
    }
    let display_color = if value.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };
    // 初始渲染与共享层 display_value() 的掩码规则保持一致，
    // 否则第一帧密码会以明文闪现
    let display = if value.is_empty() {
        "点击输入...".to_string()
    } else if password {
        "*".repeat(value.chars().count())
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
                Text({label})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { width: Val::Px(100.0) }
            ),
            (
                ProxyInputField { field_type: {field_type} }
                template_value(text_input)
                Button
                TabIndex({tab_index})
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
                        TextColor({display_color})
                    )
                ]
            ),
        ]
    }
}

/// 清理代理设置界面
pub fn cleanup_proxy_settings_ui(
    mut commands: Commands,
    query: Query<Entity, With<ProxySettingsRoot>>,
    mut input_focus: ResMut<InputFocus>,
    focusables: Query<Entity, With<ProxyInputField>>,
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

/// 返回按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn back_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackToLoginButton>)>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_route.set(AppRoute::Login);
        }
    }
}

/// 保存按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn save_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SaveProxyButton>)>,
    proxy_state: Res<ProxySettingsState>,
    mut next_route: ResMut<NextState<AppRoute>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 保存设置
        let mut settings = AppSettings::global().write();
        settings.proxy.enabled = proxy_state.enabled;
        settings.proxy.proxy_type = proxy_state.proxy_type;
        settings.proxy.host = proxy_state.host.clone();
        settings.proxy.port = proxy_state.port.parse().unwrap_or(1080);
        settings.proxy.use_auth = proxy_state.use_auth;
        settings.proxy.username = proxy_state.username.clone();
        settings.proxy.password = proxy_state.password.clone();
        drop(settings);

        tracing::info!("代理设置已保存");
        next_route.set(AppRoute::Login);
    }
}

/// 代理启用切换交互（开/关配色走 ButtonStyle 的选中态）
pub fn proxy_toggle_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &Children),
        (Changed<Interaction>, With<ProxyEnabledToggle>),
    >,
    mut text_query: Query<&mut Text>,
    mut proxy_state: ResMut<ProxySettingsState>,
) {
    for (interaction, mut style, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            proxy_state.enabled = !proxy_state.enabled;

            if style.selected != proxy_state.enabled {
                style.selected = proxy_state.enabled;
            }

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

/// 代理认证切换交互（与启用开关同款）
pub fn proxy_auth_toggle_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &Children),
        (Changed<Interaction>, With<ProxyAuthToggle>),
    >,
    mut text_query: Query<&mut Text>,
    mut proxy_state: ResMut<ProxySettingsState>,
) {
    for (interaction, mut style, children) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        proxy_state.use_auth = !proxy_state.use_auth;

        if style.selected != proxy_state.use_auth {
            style.selected = proxy_state.use_auth;
        }

        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = if proxy_state.use_auth {
                    "开启".to_string()
                } else {
                    "关闭".to_string()
                };
            }
        }
    }
}

/// 代理类型按钮交互（单选组：只改选中态，配色交给 ButtonStyle）
pub fn proxy_type_interaction(
    interaction_query: Query<(&Interaction, &ProxyTypeButton), Changed<Interaction>>,
    mut proxy_state: ResMut<ProxySettingsState>,
    mut all_buttons: Query<(&ProxyTypeButton, &mut ButtonStyle)>,
) {
    // 一帧内至多一个按钮被按下
    let Some(proxy_type) = interaction_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, button)| button.proxy_type)
    else {
        return;
    };

    proxy_state.proxy_type = proxy_type;

    for (button, mut style) in &mut all_buttons {
        let is_selected = button.proxy_type == proxy_type;
        if style.selected != is_selected {
            style.selected = is_selected;
        }
    }
}

/// 端口输入的数字过滤：返回（剔除非数字后的文本，回退后的光标）
///
/// 原本就全是数字时返回 `None`，避免无谓地触发 `TextInput` 的变更检测。
fn port_digits_only(input: &TextInput) -> Option<(String, usize)> {
    if !input.value.chars().any(|c| !c.is_ascii_digit()) {
        return None;
    }

    // 光标按"它前面被删掉几个"回退，避免落到串尾
    let removed_before = input
        .value
        .chars()
        .take(input.cursor)
        .filter(|c| !c.is_ascii_digit())
        .count();
    let digits: String = input.value.chars().filter(char::is_ascii_digit).collect();
    let cursor = input
        .cursor
        .saturating_sub(removed_before)
        .min(digits.chars().count());

    Some((digits, cursor))
}

/// 同步 TextInput.value → ProxySettingsState（端口只收数字）
pub fn proxy_sync_text_values(
    mut proxy_state: ResMut<ProxySettingsState>,
    mut query: Query<(&ProxyInputField, &mut TextInput), Changed<TextInput>>,
) {
    for (field, mut input) in query.iter_mut() {
        match field.field_type {
            ProxyFieldType::Host => {
                if proxy_state.host != input.value {
                    proxy_state.host.clone_from(&input.value);
                }
            }
            ProxyFieldType::Port => {
                // 键盘/IME/粘贴都汇入 TextInput，统一在这里剔除非数字
                if let Some((digits, cursor)) = port_digits_only(&input) {
                    input.value = digits;
                    input.cursor = cursor;
                }
                if proxy_state.port != input.value {
                    proxy_state.port.clone_from(&input.value);
                }
            }
            ProxyFieldType::Username => {
                if proxy_state.username != input.value {
                    proxy_state.username.clone_from(&input.value);
                }
            }
            ProxyFieldType::Password => {
                if proxy_state.password != input.value {
                    proxy_state.password.clone_from(&input.value);
                }
            }
        }
    }
}
