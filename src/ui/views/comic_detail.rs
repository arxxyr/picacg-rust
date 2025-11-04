use crate::ui::message::Message;
use crate::ui::state::ComicDetailState;
use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Alignment, Color, Element, Length};

/// 漫画详情界面视图
pub fn view<'a>(state: &'a ComicDetailState, image_cache: &'a crate::ui::image_loader::ImageCache) -> Element<'a, Message> {
    // 如果正在加载
    if state.is_loading {
        let loading_text = text("正在加载漫画详情...")
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
            .on_press(Message::LoadComicDetail(state.comic_id.clone()))
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

    // 如果没有漫画数据
    let Some(ref comic) = state.comic else {
        let empty_text = text("暂无数据")
            .size(18)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        return container(empty_text)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    // 创建详情内容
    let mut content = column![].spacing(20).padding(20);

    // 标题
    content = content.push(
        text(&comic.title)
            .size(28)
            .color(Color::from_rgb(0.9, 0.9, 0.9)),
    );

    // 创建左右布局
    let mut left_column = column![].spacing(15);

    // 封面图片
    let cover_widget = if let Some(ref handle) = state.cover_image {
        container(image(handle.clone()))
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(280.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    } else {
        container(
            text("加载中...")
                .size(14)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
                .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fixed(200.0))
        .height(Length::Fixed(280.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    };

    left_column = left_column.push(cover_widget);

    // 统计信息
    let stats = column![
        text(format!("👁 浏览: {}", comic.views_count))
            .size(14)
            .color(Color::from_rgb(0.8, 0.8, 0.8)),
        text(format!("❤ 点赞: {}", comic.likes_count))
            .size(14)
            .color(Color::from_rgb(0.9, 0.5, 0.5)),
        text(format!("💬 评论: {}", comic.comments_count))
            .size(14)
            .color(Color::from_rgb(0.6, 0.8, 0.9)),
    ]
    .spacing(8);

    left_column = left_column.push(stats);

    // 右侧信息列
    let mut right_column = column![].spacing(12);

    // 作者
    right_column = right_column.push(
        text(format!("作者: {}", comic.author))
            .size(16)
            .color(Color::from_rgb(0.8, 0.8, 0.8)),
    );

    // 汉化组
    if let Some(ref team) = comic.chinese_team {
        right_column = right_column.push(
            text(format!("汉化: {}", team))
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
        );
    }

    // 章节和页数
    right_column = right_column.push(
        text(format!("章节: {} | 页数: {}", comic.eps_count, comic.pages_count))
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
    );

    // 完结状态
    let status_text = if comic.finished { "已完结" } else { "连载中" };
    let status_color = if comic.finished {
        Color::from_rgb(0.5, 0.9, 0.5)
    } else {
        Color::from_rgb(0.9, 0.7, 0.3)
    };
    right_column = right_column.push(text(status_text).size(14).color(status_color));

    // 分类标签
    if !comic.categories.is_empty() {
        let categories_text = comic.categories.join(", ");
        right_column = right_column.push(
            text(format!("分类: {}", categories_text))
                .size(14)
                .color(Color::from_rgb(0.6, 0.8, 1.0)),
        );
    }

    // 标签
    if !comic.tags.is_empty() {
        let tags_text = comic.tags.join(", ");
        right_column = right_column.push(
            text(format!("标签: {}", tags_text))
                .size(13)
                .color(Color::from_rgb(0.8, 0.6, 0.9)),
        );
    }

    // 简介
    if !comic.description.is_empty() {
        right_column = right_column.push(
            column![
                text("简介:")
                    .size(16)
                    .color(Color::from_rgb(0.9, 0.9, 0.9)),
                text(&comic.description)
                    .size(14)
                    .color(Color::from_rgb(0.8, 0.8, 0.8)),
            ]
            .spacing(8),
        );
    }

    // 创建左右布局
    let info_row = row![left_column, right_column]
        .spacing(30)
        .align_y(Alignment::Start);

    content = content.push(info_row);

    // 添加操作按钮（暂时占位）
    let buttons_row = row![
        button(text("开始阅读").align_x(iced::alignment::Horizontal::Center))
            .padding(10)
            .width(Length::Fixed(120.0)),
        button(text("收藏").align_x(iced::alignment::Horizontal::Center))
            .padding(10)
            .width(Length::Fixed(100.0)),
        button(text("点赞").align_x(iced::alignment::Horizontal::Center))
            .padding(10)
            .width(Length::Fixed(100.0)),
    ]
    .spacing(10);

    content = content.push(buttons_row);

    // 添加可滚动容器
    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
