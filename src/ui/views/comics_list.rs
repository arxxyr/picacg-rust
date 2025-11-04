use iced::{
    Alignment, Color, Element, Length,
    widget::{button, column, container, image, row, scrollable, text},
};

use crate::ui::{message::Message, state::ComicsListState};

/// 漫画列表界面视图
pub fn view<'a>(state: &'a ComicsListState) -> Element<'a, Message> {
    // 如果正在加载
    if state.is_loading {
        let loading_text = text("正在加载漫画列表...")
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
            .on_press(Message::LoadComics(state.category.clone()))
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

    // 如果漫画列表为空
    if state.comics.is_empty() {
        let empty_text = text("暂无漫画")
            .size(18)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        return container(empty_text)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    // 显示漫画网格
    let title = text(format!(
        "{} - 第 {} / {} 页",
        state.category, state.page, state.total_pages
    ))
    .size(24)
    .color(Color::from_rgb(0.9, 0.9, 0.9));

    // 创建漫画网格（每行 4 个）
    let mut grid = column![title].spacing(20).padding(20);
    let mut current_row = vec![];

    for (index, comic) in state.comics.iter().enumerate() {
        // 获取图片URL
        let img_url = comic.thumb.url();

        // 创建漫画卡片内容
        let mut card_content = column![].spacing(8).align_x(Alignment::Center);

        // 如果有缩略图，显示图片
        if let Some(handle) = state.thumbnails.get(&img_url) {
            let img = image(handle.clone())
                .width(Length::Fixed(140.0))
                .height(Length::Fixed(200.0));
            card_content = card_content.push(img);
        } else {
            // 占位符
            let placeholder = container(
                text("加载中...")
                    .size(14)
                    .color(Color::from_rgb(0.6, 0.6, 0.6))
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fixed(140.0))
            .height(Length::Fixed(200.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill);
            card_content = card_content.push(placeholder);
        }

        // 添加标题（限制宽度避免过长）
        card_content = card_content.push(
            container(
                text(&comic.title)
                    .size(12)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fixed(140.0)),
        );

        // 添加作者
        card_content = card_content.push(
            text(&comic.author)
                .size(10)
                .color(Color::from_rgb(0.7, 0.7, 0.7))
                .align_x(iced::alignment::Horizontal::Center),
        );

        // 添加统计信息（点赞数）
        card_content = card_content.push(
            text(format!("❤ {}", comic.likes_count))
                .size(10)
                .color(Color::from_rgb(0.9, 0.5, 0.5))
                .align_x(iced::alignment::Horizontal::Center),
        );

        // 创建按钮
        let comic_button = button(card_content.padding(8))
            .on_press(Message::ComicClicked(comic.id.clone()))
            .width(Length::Fixed(156.0))
            .height(Length::Fixed(280.0));

        current_row.push(comic_button.into());

        // 每 4 个一行，或者是最后一个
        if (index + 1) % 4 == 0 || index == state.comics.len() - 1 {
            let row_widget = row(std::mem::take(&mut current_row))
                .spacing(15)
                .align_y(Alignment::Center);
            grid = grid.push(row_widget);
        }
    }

    // 添加分页控制
    let mut pagination_row = row![].spacing(10).align_y(Alignment::Center);

    // 上一页按钮
    if state.page > 1 {
        let prev_button = button(text("上一页").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::PrevPage)
            .padding(8)
            .width(Length::Fixed(80.0));
        pagination_row = pagination_row.push(prev_button);
    }

    // 页码信息
    pagination_row = pagination_row.push(
        text(format!("第 {} / {} 页", state.page, state.total_pages))
            .size(14)
            .color(Color::from_rgb(0.8, 0.8, 0.8)),
    );

    // 下一页按钮
    if state.page < state.total_pages {
        let next_button = button(text("下一页").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::NextPage)
            .padding(8)
            .width(Length::Fixed(80.0));
        pagination_row = pagination_row.push(next_button);
    }

    grid = grid.push(pagination_row);

    // 添加可滚动容器
    scrollable(grid)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
