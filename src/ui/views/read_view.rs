use iced::{
    Alignment, Color, Element, Length,
    widget::{button, column, container, horizontal_space, image, row, scrollable, text},
};

use crate::ui::{
    message::Message,
    state::{ReadMode, ReadViewState},
};

/// 阅读界面视图
pub fn view<'a>(state: &'a ReadViewState) -> Element<'a, Message> {
    // 如果正在加载
    if state.is_loading {
        let loading_text = text("正在加载图片...")
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
            .on_press(Message::LoadPictures {
                comic_id: state.comic_id.clone(),
                episode_order: state.episode_order,
                page: 1,
            })
            .padding(10)
            .width(Length::Fixed(120.0));

        let back_button = button(text("返回").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::BackToDetail)
            .padding(10)
            .width(Length::Fixed(120.0));

        let error_column = column![error_text, retry_button, back_button]
            .spacing(20)
            .align_x(Alignment::Center);

        return container(error_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    // 如果没有图片
    if state.pictures.is_empty() {
        let empty_text = text("暂无图片")
            .size(18)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        return container(empty_text)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    // 创建顶部控制栏
    let top_controls = create_top_controls(state);

    // 创建图片显示区域
    let image_area = create_image_area(state);

    // 创建底部控制栏
    let bottom_controls = create_bottom_controls(state);

    // 组装完整界面
    let content = column![top_controls, image_area, bottom_controls]
        .spacing(10)
        .padding(10);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// 创建顶部控制栏
fn create_top_controls<'a>(state: &'a ReadViewState) -> Element<'a, Message> {
    let back_button = button(text("返回").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::BackToDetail)
        .padding(8)
        .width(Length::Fixed(80.0));

    let page_info = text(format!(
        "第 {} 章 - 第 {}/{} 页",
        state.episode_order, state.current_page, state.total_pages
    ))
    .size(16)
    .color(Color::from_rgb(0.9, 0.9, 0.9));

    let mode_text = match state.read_mode {
        ReadMode::SinglePage => "单页",
        ReadMode::DoublePage => "双页",
        ReadMode::Scroll => "滚动",
    };

    let mode_button =
        button(text(format!("模式: {}", mode_text)).align_x(iced::alignment::Horizontal::Center))
            .padding(8)
            .width(Length::Fixed(100.0));

    row![
        back_button,
        horizontal_space(),
        page_info,
        horizontal_space(),
        mode_button
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

/// 创建图片显示区域
fn create_image_area<'a>(state: &'a ReadViewState) -> Element<'a, Message> {
    let image_widget = if let Some(ref handle) = state.current_image {
        // 创建图片 widget
        let img = image(handle.clone());

        // 根据缩放比例设置图片尺寸
        // iced 的 image widget 需要明确的尺寸才能缩放
        // 使用 FillPortion 来实现缩放效果
        let scaled_img = if state.scale == 1.0 {
            // 100% 缩放：填充容器
            img.width(Length::Fill).height(Length::Fill)
        } else if state.scale < 1.0 {
            // 缩小：使用 FillPortion
            let portion = (state.scale * 10.0) as u16;
            img.width(Length::FillPortion(portion))
                .height(Length::FillPortion(portion))
        } else {
            // 放大：需要 scrollable 支持
            // 使用固定像素值来实现放大
            img.width(Length::Fill).height(Length::Fill)
        };

        // 对于放大的情况，使用 scrollable 包裹
        if state.scale > 1.0 {
            let scaled_container = container(scaled_img)
                .width(Length::FillPortion((state.scale * 10.0) as u16))
                .height(Length::FillPortion((state.scale * 10.0) as u16));
            // 注意：scrollable 内部的元素不能使用 center_x/center_y(Length::Fill)
            // 否则会导致 "scrollable content must not fill its vertical scrolling axis" 崩溃

            container(scrollable(scaled_container))
                .width(Length::Fill)
                .height(Length::Fill)
        } else {
            container(scaled_img)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        }
    } else {
        container(
            text("加载中...")
                .size(16)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
                .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    };

    container(image_widget)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// 创建底部控制栏
fn create_bottom_controls<'a>(state: &'a ReadViewState) -> Element<'a, Message> {
    // 章节导航
    let prev_episode_button = button(text("上一章").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::PrevEpisode)
        .padding(8)
        .width(Length::Fixed(80.0));

    let next_episode_button = button(text("下一章").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::NextEpisode)
        .padding(8)
        .width(Length::Fixed(80.0));

    // 页面导航
    let prev_button = button(text("◀").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::PrevPicturePage)
        .padding(8)
        .width(Length::Fixed(50.0));

    let next_button = button(text("▶").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::NextPicturePage)
        .padding(8)
        .width(Length::Fixed(50.0));

    // 缩放控制
    let zoom_out_button = button(text("-").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::ZoomOut)
        .padding(8)
        .width(Length::Fixed(40.0));

    let zoom_text = text(format!("{}%", (state.scale * 100.0) as i32))
        .size(14)
        .color(Color::from_rgb(0.9, 0.9, 0.9));

    let zoom_in_button = button(text("+").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::ZoomIn)
        .padding(8)
        .width(Length::Fixed(40.0));

    let reset_zoom_button = button(text("重置").align_x(iced::alignment::Horizontal::Center))
        .on_press(Message::ResetZoom)
        .padding(8)
        .width(Length::Fixed(60.0));

    row![
        prev_episode_button,
        horizontal_space(),
        prev_button,
        next_button,
        horizontal_space(),
        zoom_out_button,
        zoom_text,
        zoom_in_button,
        reset_zoom_button,
        next_episode_button,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
