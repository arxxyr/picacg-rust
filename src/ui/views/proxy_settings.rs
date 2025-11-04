use iced::{
    Alignment, Color, Element, Length,
    widget::{button, checkbox, column, container, pick_list, row, text, text_input},
};

use crate::{
    config::settings::ProxyType,
    ui::{message::Message, state::ProxySettingsState},
};

/// 代理类型选项
const PROXY_TYPES: [ProxyType; 3] = [ProxyType::Http, ProxyType::Https, ProxyType::Socks5];

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Https => write!(f, "HTTPS"),
            ProxyType::Socks5 => write!(f, "SOCKS5"),
        }
    }
}

/// 代理设置视图
pub fn view<'a>(state: &'a ProxySettingsState) -> Element<'a, Message> {
    let title = text("代理设置")
        .size(28)
        .color(Color::from_rgb(0.2, 0.4, 0.8));

    let description = text("配置网络代理，用于访问 API 服务器").size(14);

    // 启用代理开关
    let enable_proxy = checkbox("启用代理", state.enabled)
        .on_toggle(Message::ProxyEnabledToggled)
        .size(18);

    // 代理类型选择
    let proxy_type_label = text("代理类型:").width(100);
    let proxy_type_picker = pick_list(
        &PROXY_TYPES[..],
        Some(state.proxy_type),
        Message::ProxyTypeChanged,
    )
    .width(Length::Fixed(150.0));

    let proxy_type_row = row![proxy_type_label, proxy_type_picker]
        .spacing(10)
        .align_y(Alignment::Center);

    // 代理主机输入
    let host_label = text("代理主机:").width(100);
    let host_input = text_input("例如：127.0.0.1", &state.host)
        .on_input(Message::ProxyHostChanged)
        .padding(10)
        .size(16);

    let host_row = row![host_label, host_input]
        .spacing(10)
        .align_y(Alignment::Center);

    // 代理端口输入
    let port_label = text("代理端口:").width(100);
    let port_input = text_input("例如：7890", &state.port)
        .on_input(Message::ProxyPortChanged)
        .padding(10)
        .size(16);

    let port_row = row![port_label, port_input]
        .spacing(10)
        .align_y(Alignment::Center);

    // 认证开关
    let auth_checkbox = checkbox("需要认证", state.use_auth)
        .on_toggle(Message::ProxyAuthToggled)
        .size(18);

    // 用户名输入（仅在启用认证时显示）
    let username_row = if state.use_auth {
        let username_label = text("用户名:").width(100);
        let username_input = text_input("代理用户名", &state.username)
            .on_input(Message::ProxyUsernameChanged)
            .padding(10)
            .size(16);

        Some(
            row![username_label, username_input]
                .spacing(10)
                .align_y(Alignment::Center),
        )
    } else {
        None
    };

    // 密码输入（仅在启用认证时显示）
    let password_row = if state.use_auth {
        let password_label = text("密码:").width(100);
        let password_input = text_input("代理密码", &state.password)
            .on_input(Message::ProxyPasswordChanged)
            .secure(true)
            .padding(10)
            .size(16);

        Some(
            row![password_label, password_input]
                .spacing(10)
                .align_y(Alignment::Center),
        )
    } else {
        None
    };

    // 测试按钮
    let test_button = if state.is_testing {
        button(text("测试中...").align_x(iced::alignment::Horizontal::Center))
            .width(Length::Fixed(120.0))
            .padding(10)
    } else {
        button(text("测试连接").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::TestProxyConnection)
            .width(Length::Fixed(120.0))
            .padding(10)
    };

    // 保存按钮
    let save_button = button(text("保存设置").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::SaveProxySettings)
        .width(Length::Fixed(120.0))
        .padding(10);

    // 返回登录按钮
    let back_button = button(text("返回登录").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::BackToLogin)
        .width(Length::Fixed(120.0))
        .padding(10);

    let buttons_row = row![test_button, save_button, back_button]
        .spacing(20)
        .align_y(Alignment::Center);

    // 测试结果消息
    let test_message = if let Some(ref msg) = state.test_message {
        let color = if msg.starts_with("成功") {
            Color::from_rgb(0.0, 0.8, 0.0)
        } else {
            Color::from_rgb(1.0, 0.0, 0.0)
        };
        Some(text(msg.as_str()).size(14).color(color))
    } else {
        None
    };

    // 组装表单
    let mut form = column![title, description]
        .spacing(20)
        .padding(40)
        .align_x(Alignment::Start);

    form = form.push(enable_proxy);

    if state.enabled {
        form = form.push(
            column![proxy_type_row, host_row, port_row, auth_checkbox]
                .spacing(15)
                .padding(20),
        );

        if let Some(username) = username_row {
            form = form.push(username);
        }
        if let Some(password) = password_row {
            form = form.push(password);
        }

        form = form.push(buttons_row);

        if let Some(message) = test_message {
            form = form.push(message);
        }
    }

    // 使用说明
    let help_text = text("提示：修改代理设置后需要重新启动应用才能生效").size(12);

    form = form.push(help_text);

    container(form)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
