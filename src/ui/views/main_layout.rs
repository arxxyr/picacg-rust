use crate::ui::message::Message;
use crate::ui::state::{AppState, Route};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

/// 主界面布局视图（包含侧边栏和内容区域）
pub fn view<'a>(state: &'a AppState, content: Element<'a, Message>) -> Element<'a, Message> {
    let sidebar = create_sidebar(&state.route);
    let main_content = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20);

    row![sidebar, main_content]
        .spacing(0)
        .into()
}

/// 创建侧边栏
fn create_sidebar(current_route: &Route) -> Element<'static, Message> {
    let home_button = create_nav_button(
        "主页",
        matches!(current_route, Route::Home),
        Message::NavigateToHome,
    );

    let categories_button = create_nav_button(
        "分类",
        matches!(current_route, Route::Categories),
        Message::NavigateToCategories,
    );

    let search_button = create_nav_button(
        "搜索",
        matches!(current_route, Route::Search),
        Message::NavigateToSearch,
    );

    let favorites_button = create_nav_button(
        "收藏",
        matches!(current_route, Route::Favorites),
        Message::NavigateToFavorites,
    );

    let downloads_button = create_nav_button(
        "下载",
        matches!(current_route, Route::Downloads),
        Message::NavigateToDownloads,
    );

    let settings_button = create_nav_button(
        "设置",
        matches!(current_route, Route::Settings),
        Message::NavigateToSettings,
    );

    let sidebar_content = column![
        text("PicACG").size(24),
        home_button,
        categories_button,
        search_button,
        favorites_button,
        downloads_button,
        settings_button,
    ]
    .spacing(10)
    .padding(20);

    container(sidebar_content)
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .into()
}

/// 创建导航按钮
fn create_nav_button(
    label: &'static str,
    is_active: bool,
    message: Message,
) -> Element<'static, Message> {
    let btn = button(text(label).size(16))
        .on_press(message)
        .width(Length::Fill)
        .padding(10);

    if is_active {
        container(btn).into()
    } else {
        btn.into()
    }
}
