use iced::{
    Alignment, Color, Element, Length,
    widget::{button, column, container, row, text, text_input},
};

use crate::ui::{message::Message, state::LoginState};

// 定义输入框 ID 常量
pub const USERNAME_INPUT_ID: &str = "username_input";
pub const PASSWORD_INPUT_ID: &str = "password_input";

/// 登录界面视图
pub fn view<'a>(state: &'a LoginState) -> Element<'a, Message> {
    let title = text("PicACG 漫画客户端")
        .size(32)
        .color(Color::from_rgb(0.2, 0.4, 0.8));

    let subtitle = text("Rust 重写版").size(16);

    // 用户名输入框
    let email_input = text_input("请输入用户名", &state.email)
        .on_input(Message::EmailChanged)
        .on_submit(Message::LoginPressed)
        .padding(10)
        .size(16)
        .id(text_input::Id::new(USERNAME_INPUT_ID));

    let email_row = row![text("用户名:").width(80), email_input]
        .spacing(10)
        .align_y(Alignment::Center);

    // 密码输入框 - 使用 secure 参数
    let password_input = text_input("请输入密码", &state.password)
        .on_input(Message::PasswordChanged)
        .on_submit(Message::LoginPressed)
        .secure(true)
        .padding(10)
        .size(16)
        .id(text_input::Id::new(PASSWORD_INPUT_ID));

    let password_row = row![text("密码:").width(80), password_input]
        .spacing(10)
        .align_y(Alignment::Center);

    // 登录按钮
    let login_button = if state.is_loading {
        button(text("登录中...").align_x(iced::alignment::Horizontal::Center))
            .width(Length::Fill)
            .padding(10)
    } else {
        button(text("登录").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::LoginPressed)
            .width(Length::Fill)
            .padding(10)
    };

    // 代理设置按钮
    let proxy_settings_button =
        button(text("代理设置").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::NavigateToProxySettings)
            .width(Length::Fill)
            .padding(10);

    // 错误信息
    let error_message = if let Some(ref error) = state.error {
        Some(
            text(error.as_str())
                .color(Color::from_rgb(1.0, 0.0, 0.0))
                .size(14),
        )
    } else {
        None
    };

    // 组装登录表单
    let mut form = column![title, subtitle]
        .spacing(10)
        .padding(20)
        .align_x(Alignment::Center);

    form = form.push(
        column![email_row, password_row, login_button, proxy_settings_button]
            .spacing(15)
            .padding(20)
            .width(Length::Fixed(400.0)),
    );

    if let Some(error) = error_message {
        form = form.push(error);
    }

    // 居中容器
    container(form)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
