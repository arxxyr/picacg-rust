use iced::{
    Alignment, Element, Length,
    widget::{column, container, text},
};

use crate::ui::message::Message;

/// 主页视图
pub fn view() -> Element<'static, Message> {
    let title = text("欢迎使用 PicACG").size(28);
    let subtitle = text("Rust 重写版本").size(18);
    let description = text("使用左侧导航栏浏览不同功能").size(14);

    let content = column![title, subtitle, description]
        .spacing(20)
        .padding(40)
        .align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
