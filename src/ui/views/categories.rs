use iced::{
    Alignment, Color, Element, Length,
    widget::{button, column, container, image, row, scrollable, text},
};

use crate::ui::{message::Message, state::CategoriesState};

/// 分类浏览界面视图
pub fn view<'a>(state: &'a CategoriesState) -> Element<'a, Message> {
    // 如果正在加载
    if state.is_loading {
        let loading_text = text("正在加载分类列表...")
            .size(18)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        return container(loading_text)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    // 如果有错误
    if let Some(ref error) = state.error {
        let error_text = text(error.as_str())
            .size(16)
            .color(Color::from_rgb(1.0, 0.3, 0.3));

        let retry_button = button(text("重试").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::LoadCategories)
            .padding(10)
            .width(Length::Fixed(120.0));

        let error_column = column![error_text, retry_button]
            .spacing(20)
            .align_x(Alignment::Center);

        return container(error_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    // 如果分类列表为空
    if state.categories.is_empty() {
        let empty_text = text("暂无分类")
            .size(18)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        return container(empty_text)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    // 显示分类网格
    let title = text("浏览分类")
        .size(24)
        .color(Color::from_rgb(0.9, 0.9, 0.9));

    // 创建分类网格（每行 3 个）
    let mut grid = column![title].spacing(20).padding(20);
    let mut current_row = vec![];

    for (index, category) in state.categories.iter().enumerate() {
        // 获取图片URL
        let img_url = category.thumb.url();

        // 创建分类卡片内容
        let mut card_content = column![].spacing(10).align_x(Alignment::Center);

        // 如果有缩略图，显示图片
        if let Some(handle) = state.thumbnails.get(&img_url) {
            let img = image(handle.clone())
                .width(Length::Fixed(180.0))
                .height(Length::Fixed(120.0));
            card_content = card_content.push(img);
        } else {
            // 占位符
            let placeholder = container(
                text("加载中...")
                    .size(14)
                    .color(Color::from_rgb(0.6, 0.6, 0.6))
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fixed(180.0))
            .height(Length::Fixed(120.0));
            card_content = card_content.push(placeholder);
        }

        // 添加标题
        card_content = card_content.push(
            text(&category.title)
                .size(14)
                .align_x(iced::alignment::Horizontal::Center),
        );

        // 创建按钮
        let category_button = button(card_content.padding(10))
            .on_press(Message::CategoryClicked(category.title.clone()))
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(170.0));

        current_row.push(category_button.into());

        // 每 3 个一行，或者是最后一个
        if (index + 1) % 3 == 0 || index == state.categories.len() - 1 {
            let row_widget = row(std::mem::take(&mut current_row))
                .spacing(20)
                .align_y(Alignment::Center);
            grid = grid.push(row_widget);
        }
    }

    // 添加可滚动容器
    // 注意：scrollable 本身不能设置 height(Fill)，需要用 container 包裹
    container(scrollable(grid))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
