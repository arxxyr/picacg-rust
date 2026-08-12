//! 漫画详情系统
//!
//! 实现漫画详情页面的 UI 和交互

#![allow(dead_code)]

use bevy::prelude::*;
use picacg_api::models::{Comic, Episode};

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        navigation::NavigationHistory,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        widgets::ButtonStyle,
    },
    utils::icons::*,
};

/// 章节卡片布局常量
mod episode_layout {
    /// 卡片宽度
    pub const CARD_WIDTH: f32 = 120.0;
    /// 卡片高度
    pub const CARD_HEIGHT: f32 = 40.0;
    /// 列间距
    pub const COLUMN_GAP: f32 = 10.0;
    /// 行间距
    pub const ROW_GAP: f32 = 10.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距
    pub const PADDING_RIGHT: f32 = 20.0;
    /// 上内边距
    pub const PADDING_TOP: f32 = 15.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 15.0;
}

/// 详情页返回按钮
#[derive(Component, Default, Clone)]
pub struct DetailBackButton;

/// 下载按钮组件
#[derive(Component, Default, Clone)]
pub struct DownloadButton;

/// 作者按钮（点击搜索该作者）
#[derive(Component, Default, Clone)]
pub struct AuthorButton {
    pub author: String,
}

/// 详情页点赞数文本
#[derive(Component, Default, Clone)]
pub struct DetailLikesText;

/// 详情页收藏按钮文本
#[derive(Component, Default, Clone)]
pub struct DetailFavoriteText;

/// 分类标签组件（可点击）
#[derive(Component, Default, Clone)]
pub struct CategoryTag {
    pub category: String,
}

/// 标签按钮组件（可点击搜索）
#[derive(Component, Default, Clone)]
pub struct TagButton {
    pub tag: String,
}

// ==================== 场景函数 ====================

/// 详情页面场景（标题栏 + 滚动区域，内容由调用方传入）
fn detail_page(title: String, content: Box<dyn SceneList>) -> impl Scene {
    // 滚动区内边距（右侧额外让出滚动条宽度）
    let scroll_padding = UiRect {
        left: Val::Px(20.0),
        right: Val::Px(20.0 + SCROLLBAR_WIDTH),
        top: Val::Px(20.0),
        bottom: Val::Px(20.0),
    };

    bsn! {
        ComicDetailRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 标题栏
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // 返回按钮
                        DetailBackButton
                        Button
                        template_value(ButtonStyle::ghost())
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        // 静息底色与 ButtonStyle::ghost() 的 None 态一致
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text(ICON_CHEVRON_LEFT)
                                TextFont { font_size: FontSize::Px(20.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        Text({title})
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    ),
                ]
            ),
            (
                // 滚动区域包装器
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                }
                Children [
                    (
                        // 可滚动内容区域
                        #DetailScroll
                        DetailScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: {scroll_padding},
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [ {content} ]
                    ),
                    // 创建滚动条
                    scrollbar(#DetailScroll),
                ]
            ),
        ]
    }
}

/// 加载指示器场景
fn loading_indicator() -> impl Scene {
    bsn! {
        LoadingIndicator
        Text("加载中...")
        TextFont { font_size: FontSize::Px(16.0) }
        TextColor(AppColors::TEXT)
    }
}

/// 加载失败提示场景
fn error_message(error: &str) -> impl Scene + use<> {
    let error_text = format!("加载失败: {}", error);

    bsn! {
        ErrorMessage
        Text({error_text})
        TextFont { font_size: FontSize::Px(16.0) }
        TextColor(AppColors::ERROR)
    }
}

/// 空状态场景
fn empty_hint() -> impl Scene {
    bsn! {
        Text("暂无数据")
        TextFont { font_size: FontSize::Px(16.0) }
        TextColor(AppColors::TEXT_SECONDARY)
    }
}

/// 底部间距场景，确保最后一行章节卡片不被截断
fn bottom_spacer() -> impl Scene {
    bsn! {
        Node {
            height: Val::Px(30.0),
            min_height: Val::Px(30.0),
        }
    }
}

/// 基本信息区域（左侧封面 + 右侧详情列）
fn info_row(cover: Box<dyn Scene>, details: Box<dyn Scene>) -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(20.0)),
        }
        Children [
            // 左侧：封面图片
            ({cover}),
            // 右侧：详细信息
            ({details}),
        ]
    }
}

/// 封面图片场景（已缓存显示图片，未缓存显示占位符）
fn cover_image(image_cache: &ImageCache, thumb_url: &str) -> Box<dyn Scene> {
    // URL 存进组件，供 update_cover_image 直接取用
    let url = thumb_url.to_string();

    match image_cache.get(thumb_url) {
        Some(handle) => {
            let handle = handle.clone();
            Box::new(bsn! {
                CoverImage { url: {url} }
                ImageNode { image: {handle} }
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(280.0),
                }
            })
        }
        None => Box::new(bsn! {
            CoverImage { url: {url} }
            PlaceholderImage
            Node {
                width: Val::Px(200.0),
                height: Val::Px(280.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            BackgroundColor(AppColors::SURFACE)
            Children [
                (
                    Text("加载中...")
                    TextFont { font_size: FontSize::Px(14.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                )
            ]
        }),
    }
}

/// 详情列（首次进入版：分类为纯文本，不含汉化组/标签/评论数/更新时间）
fn basic_info_column(comic: &Comic) -> impl Scene + use<> {
    let title = comic.title.clone();
    let author = comic.author.clone();
    let author_label = format!("作者: {}", comic.author);

    // 分类
    let categories: Box<dyn SceneList> = if comic.categories.is_empty() {
        Box::new(bsn_list![])
    } else {
        let categories_label = format!("分类: {}", comic.categories.join(", "));
        Box::new(bsn_list![(
            Text({categories_label})
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    };

    // 统计信息
    let counts_label = format!("章节: {} | 页数: {}", comic.eps_count, comic.pages_count);
    let likes_label = format!("点赞: {} | 浏览: {}", comic.likes_count, comic.views_count);

    // 完结状态
    let status = if comic.finished {
        "已完结"
    } else {
        "连载中"
    };
    let status_label = format!("状态: {}", status);
    let status_color = if comic.finished {
        Color::srgb(0.4, 0.8, 0.4)
    } else {
        Color::srgb(0.8, 0.6, 0.2)
    };

    // 描述（上方间距 + 正文）
    let description: Box<dyn SceneList> = match comic.description {
        Some(ref desc) => {
            let desc = desc.clone();
            Box::new(bsn_list![
                (Node { margin: UiRect::top(Val::Px(10.0)) }),
                (
                    Text({desc})
                    TextFont { font_size: FontSize::Px(13.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                    Node { max_width: Val::Px(500.0) }
                ),
            ])
        }
        None => Box::new(bsn_list![]),
    };

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            row_gap: Val::Px(10.0),
        }
        Children [
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 作者（可点击搜索）
                AuthorButton { author: {author} }
                Button
                template_value(ButtonStyle::ghost())
                Node { padding: UiRect::all(Val::Px(0.0)) }
                // 静息底色与 ButtonStyle::ghost() 的 None 态一致
                BackgroundColor(Color::NONE)
                Children [
                    (
                        Text({author_label})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::PRIMARY)
                    )
                ]
            ),
            // 分类
            {categories},
            (
                // 统计信息
                Text({counts_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                DetailLikesText
                Text({likes_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 完结状态
                Text({status_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor({status_color})
            ),
            // 描述
            {description},
        ]
    }
}

/// 详情列（完整版：汉化组 + 可点击分类/标签 + 评论数 + 更新时间）
fn full_info_column(comic: &Comic) -> impl Scene + use<> {
    let title = comic.title.clone();
    let author = comic.author.clone();
    let author_label = format!("作者: {}", comic.author);

    // 汉化组
    let chinese_team: Box<dyn SceneList> = match comic.chinese_team {
        Some(ref team) if !team.is_empty() => {
            let team_label = format!("汉化: {}", team);
            Box::new(bsn_list![(
                Text({team_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            )])
        }
        _ => Box::new(bsn_list![]),
    };

    // 分类标签（可点击）
    let categories: Box<dyn SceneList> = if comic.categories.is_empty() {
        Box::new(bsn_list![])
    } else {
        let category_tags: Vec<_> = comic
            .categories
            .iter()
            .map(|cat| category_tag(cat))
            .collect();
        Box::new(bsn_list![(
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(6.0),
                align_items: AlignItems::Center,
            }
            Children [
                (
                    Text("分类: ")
                    TextFont { font_size: FontSize::Px(14.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                ),
                {category_tags},
            ]
        )])
    };

    // 标签（tags）- 可点击搜索
    let tags: Box<dyn SceneList> = if comic.tags.is_empty() {
        Box::new(bsn_list![])
    } else {
        let tag_buttons: Vec<_> = comic.tags.iter().map(|tag| tag_button(tag)).collect();
        Box::new(bsn_list![(
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(4.0),
                align_items: AlignItems::Center,
            }
            Children [
                (
                    Text("标签: ")
                    TextFont { font_size: FontSize::Px(14.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                ),
                {tag_buttons},
            ]
        )])
    };

    // 统计信息
    let counts_label = format!("章节: {} | 页数: {}", comic.eps_count, comic.pages_count);
    let likes_label = format!(
        "点赞: {} | 浏览: {} | 评论: {}",
        comic.likes_count, comic.views_count, comic.comments_count
    );

    // 更新时间
    let updated: Box<dyn SceneList> = match comic.updated_at {
        Some(ref updated_at) => {
            // 格式化时间：2023-01-01T12:00:00.000Z -> 2023-01-01
            let date = updated_at.split('T').next().unwrap_or(updated_at);
            let updated_label = format!("更新: {}", date);
            Box::new(bsn_list![(
                Text({updated_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            )])
        }
        None => Box::new(bsn_list![]),
    };

    // 完结状态
    let status = if comic.finished {
        "已完结"
    } else {
        "连载中"
    };
    let status_label = format!("状态: {}", status);
    let status_color = if comic.finished {
        Color::srgb(0.4, 0.8, 0.4)
    } else {
        Color::srgb(0.8, 0.6, 0.2)
    };

    // 描述（上方间距 + 正文）
    let description: Box<dyn SceneList> = match comic.description {
        Some(ref desc) if !desc.is_empty() => {
            let desc = desc.clone();
            Box::new(bsn_list![
                (Node { margin: UiRect::top(Val::Px(8.0)) }),
                (
                    Text({desc})
                    TextFont { font_size: FontSize::Px(13.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                    Node { max_width: Val::Px(500.0) }
                ),
            ])
        }
        _ => Box::new(bsn_list![]),
    };

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 作者（可点击搜索）
                AuthorButton { author: {author} }
                Button
                template_value(ButtonStyle::ghost())
                Node { padding: UiRect::all(Val::Px(0.0)) }
                // 静息底色与 ButtonStyle::ghost() 的 None 态一致
                BackgroundColor(Color::NONE)
                Children [
                    (
                        Text({author_label})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::PRIMARY)
                    )
                ]
            ),
            // 汉化组
            {chinese_team},
            // 分类标签
            {categories},
            // 标签
            {tags},
            (
                // 统计信息
                Text({counts_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                DetailLikesText
                Text({likes_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            // 更新时间
            {updated},
            (
                // 完结状态
                Text({status_label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor({status_color})
            ),
            // 描述
            {description},
        ]
    }
}

/// 分类标签场景（点击跳转到该分类的漫画列表）
fn category_tag(category: &str) -> impl Scene + use<> {
    let category = category.to_string();
    let label = category.clone();

    bsn! {
        CategoryTag { category: {category} }
        Button
        template_value(ButtonStyle::card())
        Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)) }
        // 静息底色与 ButtonStyle::card() 的 None 态一致
        BackgroundColor(AppColors::SURFACE)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(Color::srgb(0.6, 0.8, 1.0))
            )
        ]
    }
}

/// 标签场景（点击跳转到搜索页搜索该标签）
fn tag_button(tag: &str) -> impl Scene + use<> {
    let tag = tag.to_string();
    let label = tag.clone();

    bsn! {
        TagButton { tag: {tag} }
        Button
        template_value(ButtonStyle::ghost())
        Node {
            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
            border: UiRect::all(Val::Px(1.0)),
        }
        template_value(BorderColor::all(Color::srgb(0.5, 0.4, 0.6)))
        // 静息底色与 ButtonStyle::ghost() 的 None 态一致（描边保留标签色）
        BackgroundColor(Color::NONE)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(Color::srgb(0.8, 0.6, 0.9))
            )
        ]
    }
}

/// 操作按钮栏（开始阅读 / 点赞 / 收藏 / 下载）
fn action_buttons_row(is_liked: bool, is_favorite: bool) -> impl Scene {
    // 点赞按钮：有选中态，走 segment（已点赞钉主色，未点赞为下沉底）
    let like_text = if is_liked { "已点赞" } else { "点赞" };
    let like_color = if is_liked {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let like_style = ButtonStyle::segment(is_liked);

    // 收藏按钮：同上
    let fav_text = if is_favorite { "已收藏" } else { "收藏" };
    let fav_color = if is_favorite {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let fav_style = ButtonStyle::segment(is_favorite);

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(15.0),
            margin: UiRect::bottom(Val::Px(20.0)),
        }
        Children [
            // 开始阅读按钮
            action_button(
                "开始阅读",
                AppColors::PRIMARY,
                ButtonStyle::primary(),
                StartReadButton
            ),
            // 点赞按钮
            action_button(like_text, like_color, like_style, LikeButton),
            // 收藏按钮
            action_button(fav_text, fav_color, fav_style, FavoriteButton),
            // 下载按钮
            action_button(
                "下载",
                AppColors::SECONDARY,
                ButtonStyle::secondary(),
                DownloadButton
            ),
        ]
    }
}

/// 操作按钮场景
///
/// `color` 只是首帧静息底色（与 `style` 的 None 态一致，避免闪烁），
/// 三态配色由全局 `apply_button_interaction` 接管。
///
/// `Unpin` 是 `template_value` 的 `Template` blanket impl 要求
/// （`Clone + Default + Unpin`），泛型参数不会自动带上。
fn action_button<M: Component + Default + Clone + Unpin>(
    text: &str,
    color: Color,
    style: ButtonStyle,
    marker: M,
) -> impl Scene + use<M> {
    let label = text.to_string();

    bsn! {
        template_value(marker)
        Button
        template_value(style)
        Node {
            width: Val::Px(100.0),
            height: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor({color})
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 章节列表标题行
fn episodes_title_row(episode_count: usize, is_loading_episodes: bool) -> impl Scene {
    let title = format!("章节列表 ({})", episode_count);

    // 章节加载提示
    let loading_hint: Box<dyn SceneList> = if is_loading_episodes {
        Box::new(bsn_list![(
            Text(" 加载中...")
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(10.0)),
        }
        Children [
            (
                Text({title})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
            ),
            {loading_hint},
        ]
    }
}

/// 章节网格
fn episode_grid(episodes: &[Episode]) -> impl Scene + use<> {
    let cards: Vec<_> = episodes.iter().map(episode_card).collect();
    let grid_padding = UiRect {
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(episode_layout::PADDING_TOP),
        bottom: Val::Px(episode_layout::PADDING_BOTTOM),
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(episode_layout::COLUMN_GAP),
            row_gap: Val::Px(episode_layout::ROW_GAP),
            padding: {grid_padding},
        }
        Children [ {cards} ]
    }
}

/// 章节卡片场景
fn episode_card(episode: &Episode) -> impl Scene + use<> {
    let episode_order = episode.order;
    let title = episode.title.clone();

    bsn! {
        EpisodeCard { episode_order: {episode_order} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Px(episode_layout::CARD_WIDTH),
            height: Val::Px(episode_layout::CARD_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
        }
        // 静息底色与 ButtonStyle::card() 的 None 态一致
        BackgroundColor(AppColors::SURFACE)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                Text({title})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 首次进入详情页的滚动区内容
fn detail_content_initial(
    detail_state: &ComicDetailState,
    image_cache: &ImageCache,
) -> Box<dyn SceneList> {
    if detail_state.is_loading {
        return Box::new(bsn_list![loading_indicator()]);
    }

    if let Some(ref error) = detail_state.error {
        return Box::new(bsn_list![error_message(error)]);
    }

    let Some(ref comic) = detail_state.comic else {
        // 空状态
        return Box::new(bsn_list![empty_hint()]);
    };

    let thumb_url = comic.thumb.url();
    let cover = cover_image(image_cache, &thumb_url);
    let details: Box<dyn Scene> = Box::new(basic_info_column(comic));

    Box::new(bsn_list![
        // 基本信息区域（封面 + 详情）
        info_row(cover, details),
        // 操作按钮栏
        action_buttons_row(detail_state.is_liked, detail_state.is_favorite),
        // 章节列表标题
        episodes_title_row(
            detail_state.episodes.len(),
            detail_state.is_loading_episodes
        ),
        // 章节网格
        episode_grid(&detail_state.episodes),
        // 底部间距，确保最后一行章节卡片不被截断
        bottom_spacer(),
    ])
}

/// 状态刷新后的滚动区内容（完整信息）
fn detail_content_full(
    detail_state: &ComicDetailState,
    image_cache: &ImageCache,
) -> Box<dyn SceneList> {
    if detail_state.is_loading {
        return Box::new(bsn_list![loading_indicator()]);
    }

    if let Some(ref error) = detail_state.error {
        return Box::new(bsn_list![error_message(error)]);
    }

    let Some(ref comic) = detail_state.comic else {
        // 空状态
        return Box::new(bsn_list![empty_hint()]);
    };

    let thumb_url = comic.thumb.url();
    let cover = cover_image(image_cache, &thumb_url);
    let details: Box<dyn Scene> = Box::new(full_info_column(comic));

    Box::new(bsn_list![
        // 基本信息区域（封面 + 详情）
        info_row(cover, details),
        // 操作按钮栏
        action_buttons_row(detail_state.is_liked, detail_state.is_favorite),
        // 章节列表标题
        episodes_title_row(
            detail_state.episodes.len(),
            detail_state.is_loading_episodes
        ),
        // 章节网格
        episode_grid(&detail_state.episodes),
        // 底部间距，确保最后一行章节卡片不被截断
        bottom_spacer(),
    ])
}

// ==================== 系统函数 ====================

/// 创建漫画详情界面
pub fn setup_detail_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    detail_state: Res<ComicDetailState>,
    image_cache: Res<ImageCache>,
    content_area_query: Query<Entity, With<ContentArea>>,
    existing_query: Query<Entity, With<ComicDetailRoot>>,
) {
    // 参数化页面：每次进入都可能是不同漫画，直接 despawn 重建
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    let content_area = content_area_query.single().ok();

    let detail_root = commands
        .spawn_scene(detail_page(
            "漫画详情".to_string(),
            detail_content_initial(&detail_state, &image_cache),
        ))
        .id();

    // 如果有 ContentArea，将详情页作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(detail_root);
    }
}

/// 清理漫画详情界面（隐藏而非销毁）
pub fn cleanup_detail_ui(mut commands: Commands, query: Query<Entity, With<ComicDetailRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 刷新详情页 UI（当状态变化时）
pub fn refresh_detail_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    detail_state: Res<ComicDetailState>,
    image_cache: Res<ImageCache>,
    root_query: Query<Entity, With<ComicDetailRoot>>,
    content_area_query: Query<Entity, With<ContentArea>>,
) {
    // 只在状态变化时刷新
    if !detail_state.is_changed() {
        return;
    }

    // 如果还在初始加载且没有数据，不刷新
    if detail_state.is_loading && detail_state.comic.is_none() {
        return;
    }

    // 删除旧 UI
    for entity in root_query.iter() {
        commands.entity(entity).despawn();
    }

    // 重新创建 UI
    let content_area = content_area_query.single().ok();

    // 标题栏显示漫画名，未加载时回退到默认标题
    let title = match detail_state.comic {
        Some(ref comic) => comic.title.clone(),
        None => "漫画详情".to_string(),
    };

    let detail_root = commands
        .spawn_scene(detail_page(
            title,
            detail_content_full(&detail_state, &image_cache),
        ))
        .id();

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(detail_root);
    }
}

/// 章节卡片交互
pub fn episode_card_interaction(
    interaction_query: Query<(&Interaction, &EpisodeCard), Changed<Interaction>>,
    detail_state: Res<ComicDetailState>,
    mut navigate_messages: MessageWriter<NavigateToReaderEvent>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // 导航到阅读器
            navigate_messages.write(NavigateToReaderEvent {
                comic_id: detail_state.comic_id.clone(),
                episode_order: card.episode_order,
            });
        }
    }
}

/// 开始阅读按钮交互
pub fn start_read_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<StartReadButton>)>,
    detail_state: Res<ComicDetailState>,
    mut navigate_messages: MessageWriter<NavigateToReaderEvent>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // 从第一章开始阅读
            navigate_messages.write(NavigateToReaderEvent {
                comic_id: detail_state.comic_id.clone(),
                episode_order: 1,
            });
        }
    }
}

/// 点赞按钮交互
pub fn like_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LikeButton>)>,
    detail_state: Res<ComicDetailState>,
    mut like_messages: MessageWriter<LikeComicRequest>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            like_messages.write(LikeComicRequest {
                comic_id: detail_state.comic_id.clone(),
            });
        }
    }
}

/// 收藏按钮交互
pub fn favorite_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<FavoriteButton>)>,
    detail_state: Res<ComicDetailState>,
    mut favorite_messages: MessageWriter<FavoriteComicRequest>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            favorite_messages.write(FavoriteComicRequest {
                comic_id: detail_state.comic_id.clone(),
            });
        }
    }
}

/// 下载按钮交互
pub fn download_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<DownloadButton>)>,
    detail_state: Res<ComicDetailState>,
    mut download_messages: MessageWriter<DownloadComicRequest>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 发送下载请求
        match detail_state.comic {
            Some(ref comic) => {
                tracing::info!("开始下载漫画: {} ({})", comic.title, detail_state.comic_id);
                download_messages.write(DownloadComicRequest {
                    comic_id: detail_state.comic_id.clone(),
                    comic_title: comic.title.clone(),
                    episodes: vec![], // 空表示下载所有章节
                });
            }
            None => tracing::warn!("漫画信息未加载，无法下载"),
        }
    }
}

/// 更新封面图片（当图片加载完成时）
///
/// URL 取自 `CoverImage` 组件，不再每帧回查详情状态重算；
/// 加载失败的占位符摘掉标记退出扫描集，避免永久残留。
pub fn update_cover_image(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    placeholder_query: Query<(Entity, &CoverImage), (With<PlaceholderImage>, Without<ImageNode>)>,
) {
    for (placeholder_entity, cover) in placeholder_query.iter() {
        match image_cache.get(&cover.url) {
            Some(handle) => {
                // 替换占位符为实际图片
                commands
                    .entity(placeholder_entity)
                    .remove::<PlaceholderImage>()
                    .remove::<BackgroundColor>()
                    .insert(ImageNode::new(handle.clone()));
            }
            None if image_cache.is_failed(&cover.url) => {
                commands
                    .entity(placeholder_entity)
                    .remove::<PlaceholderImage>();
            }
            None => {}
        }
    }
}

/// 返回按钮交互
pub fn detail_back_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<DetailBackButton>)>,
    mut navigate_back_messages: MessageWriter<NavigateBackEvent>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            navigate_back_messages.write(NavigateBackEvent);
            tracing::debug!("详情页返回按钮点击");
        }
    }
}

/// 分类标签点击交互
pub fn category_tag_interaction(
    interaction_query: Query<(&Interaction, &CategoryTag), Changed<Interaction>>,
    mut navigate_messages: MessageWriter<NavigateToComicsListEvent>,
) {
    for (interaction, tag) in &interaction_query {
        if *interaction == Interaction::Pressed {
            navigate_messages.write(NavigateToComicsListEvent {
                category: tag.category.clone(),
            });
            tracing::info!("点击分类标签: {}", tag.category);
        }
    }
}

/// 标签点击交互（跳转到搜索页面搜索该标签）
pub fn tag_button_interaction(
    interaction_query: Query<(&Interaction, &TagButton), Changed<Interaction>>,
    mut search_state: ResMut<SearchState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
    current_route: Res<State<AppRoute>>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    for (interaction, tag_btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 设置搜索状态
        search_state.keyword = tag_btn.tag.clone();
        search_state.results.clear();
        search_state.page = 1;
        search_state.total_pages = 0;
        search_state.is_loading = true;
        search_state.has_searched = true;
        search_state.error = None;

        // 记录导航历史
        history.push(current_route.get().clone());

        // 跳转到搜索页面
        next_route.set(AppRoute::Search);

        // 发送搜索请求
        search_messages.write(SearchComicsRequestEvent {
            keyword: tag_btn.tag.clone(),
            page: 1,
            sort: search_state.sort.clone(),
            categories: search_state.selected_categories.clone(),
        });

        tracing::info!("点击标签搜索: {}", tag_btn.tag);
    }
}

/// 作者按钮交互：点击跳转到搜索页并搜索该作者
pub fn author_button_interaction(
    interaction_query: Query<(&Interaction, &AuthorButton), Changed<Interaction>>,
    mut search_state: ResMut<crate::resources::SearchState>,
    mut next_route: ResMut<NextState<crate::resources::AppRoute>>,
    mut search_messages: MessageWriter<crate::events::SearchComicsRequestEvent>,
) {
    for (interaction, btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 设置搜索状态
        search_state.keyword = btn.author.clone();
        search_state.is_loading = true;
        search_state.has_searched = true;
        search_state.needs_rebuild = true;
        search_state.page = 1;
        search_state.results.clear();
        search_state.selected_categories.clear();

        // 发送搜索请求
        search_messages.write(crate::events::SearchComicsRequestEvent {
            keyword: btn.author.clone(),
            page: 1,
            sort: search_state.sort.clone(),
            categories: vec![],
        });

        // 跳转到搜索页
        next_route.set(crate::resources::AppRoute::Search);
        tracing::info!("搜索作者: {}", btn.author);
    }
}
