//! 漫画详情系统
//!
//! 实现漫画详情页面的 UI 和交互

#![allow(dead_code)]

use bevy::prelude::*;

use crate::{
    api::models::Episode,
    components::*,
    events::*,
    resources::*,
    systems::{
        login::{AppColors, FONT_PATH},
        navigation::NavigationHistory,
        scrollbar::scrollbar_config::*,
    },
};

/// 滚动条宽度
const SCROLLBAR_WIDTH: f32 = 12.0;

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
#[derive(Component)]
pub struct DetailBackButton;

/// 下载按钮组件
#[derive(Component)]
pub struct DownloadButton;

/// 详情页点赞数文本
#[derive(Component)]
pub struct DetailLikesText;

/// 详情页收藏按钮文本
#[derive(Component)]
pub struct DetailFavoriteText;

/// 分类标签组件（可点击）
#[derive(Component)]
pub struct CategoryTag {
    pub category: String,
}

/// 标签按钮组件（可点击搜索）
#[derive(Component)]
pub struct TagButton {
    pub tag: String,
}

/// 创建漫画详情界面
pub fn setup_detail_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    detail_state: Res<ComicDetailState>,
    image_cache: Res<ImageCache>,
    content_area_query: Query<Entity, With<ContentArea>>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);
    let content_area = content_area_query.single().ok();

    let detail_root = commands
        .spawn((
            ComicDetailRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|root| {
            // 标题栏
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BorderColor::all(AppColors::BORDER))
            .with_children(|header| {
                // 返回按钮
                header
                    .spawn((
                        DetailBackButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("\u{F0141}"), // nf-md-arrow_left
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                header.spawn((
                    Text::new("漫画详情"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // 滚动区域包装器
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                ..default()
            })
            .with_children(|wrapper| {
                // 可滚动内容区域
                let scroll_container_id = wrapper
                    .spawn((
                        DetailScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect {
                                left: Val::Px(20.0),
                                right: Val::Px(20.0 + SCROLLBAR_WIDTH),
                                top: Val::Px(20.0),
                                bottom: Val::Px(20.0),
                            },
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|content| {
                        if detail_state.is_loading {
                            content.spawn((
                                LoadingIndicator,
                                Text::new("加载中..."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        } else if let Some(ref error) = detail_state.error {
                            content.spawn((
                                ErrorMessage,
                                Text::new(format!("加载失败: {}", error)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.4, 0.4)),
                            ));
                        } else if let Some(ref comic) = detail_state.comic {
                            // 基本信息区域（封面 + 详情）
                            content
                                .spawn(Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(20.0),
                                    margin: UiRect::bottom(Val::Px(20.0)),
                                    ..default()
                                })
                                .with_children(|info_row| {
                                    // 左侧：封面图片
                                    let thumb_url = comic.thumb.url();
                                    if let Some(handle) = image_cache.get(&thumb_url) {
                                        info_row.spawn((
                                            CoverImage,
                                            ImageNode::new(handle.clone()),
                                            Node {
                                                width: Val::Px(200.0),
                                                height: Val::Px(280.0),
                                                ..default()
                                            },
                                        ));
                                    } else {
                                        info_row
                                            .spawn((
                                                CoverImage,
                                                PlaceholderImage,
                                                Node {
                                                    width: Val::Px(200.0),
                                                    height: Val::Px(280.0),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                },
                                                BackgroundColor(AppColors::SURFACE),
                                            ))
                                            .with_children(|placeholder| {
                                                placeholder.spawn((
                                                    Text::new("加载中..."),
                                                    TextFont {
                                                        font: font.clone(),
                                                        font_size: 14.0,
                                                        ..default()
                                                    },
                                                    TextColor(AppColors::TEXT_SECONDARY),
                                                ));
                                            });
                                    }

                                    // 右侧：详细信息
                                    info_row
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            flex_grow: 1.0,
                                            row_gap: Val::Px(10.0),
                                            ..default()
                                        })
                                        .with_children(|details| {
                                            // 标题
                                            details.spawn((
                                                Text::new(&comic.title),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 20.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT),
                                            ));

                                            // 作者
                                            details.spawn((
                                                Text::new(format!("作者: {}", comic.author)),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT_SECONDARY),
                                            ));

                                            // 分类
                                            if !comic.categories.is_empty() {
                                                details.spawn((
                                                    Text::new(format!(
                                                        "分类: {}",
                                                        comic.categories.join(", ")
                                                    )),
                                                    TextFont {
                                                        font: font.clone(),
                                                        font_size: 14.0,
                                                        ..default()
                                                    },
                                                    TextColor(AppColors::TEXT_SECONDARY),
                                                ));
                                            }

                                            // 统计信息
                                            details.spawn((
                                                Text::new(format!(
                                                    "章节: {} | 页数: {}",
                                                    comic.eps_count, comic.pages_count
                                                )),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT_SECONDARY),
                                            ));

                                            details.spawn((
                                                DetailLikesText,
                                                Text::new(format!(
                                                    "点赞: {} | 浏览: {}",
                                                    comic.likes_count, comic.views_count
                                                )),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT_SECONDARY),
                                            ));

                                            // 完结状态
                                            let status = if comic.finished {
                                                "已完结"
                                            } else {
                                                "连载中"
                                            };
                                            details.spawn((
                                                Text::new(format!("状态: {}", status)),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(if comic.finished {
                                                    Color::srgb(0.4, 0.8, 0.4)
                                                } else {
                                                    Color::srgb(0.8, 0.6, 0.2)
                                                }),
                                            ));

                                            // 描述
                                            if let Some(ref desc) = comic.description {
                                                details.spawn(Node {
                                                    margin: UiRect::top(Val::Px(10.0)),
                                                    ..default()
                                                });
                                                details.spawn((
                                                    Text::new(desc.clone()),
                                                    TextFont {
                                                        font: font.clone(),
                                                        font_size: 13.0,
                                                        ..default()
                                                    },
                                                    TextColor(AppColors::TEXT_SECONDARY),
                                                    Node {
                                                        max_width: Val::Px(500.0),
                                                        ..default()
                                                    },
                                                ));
                                            }
                                        });
                                });

                            // 操作按钮栏
                            content
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(15.0),
                                        margin: UiRect::bottom(Val::Px(20.0)),
                                        ..default()
                                    },
                                    Transform::default(), /* 必须添加，否则子实体的
                                                           * GlobalTransform 会报警告 */
                                ))
                                .with_children(|buttons| {
                                    // 开始阅读按钮
                                    spawn_action_button(
                                        buttons,
                                        &font,
                                        "开始阅读",
                                        AppColors::PRIMARY,
                                        StartReadButton,
                                    );

                                    // 点赞按钮
                                    let like_text = if detail_state.is_liked {
                                        "已点赞"
                                    } else {
                                        "点赞"
                                    };
                                    let like_color = if detail_state.is_liked {
                                        Color::srgb(0.8, 0.4, 0.4)
                                    } else {
                                        AppColors::SECONDARY
                                    };
                                    spawn_action_button(
                                        buttons, &font, like_text, like_color, LikeButton,
                                    );

                                    // 收藏按钮
                                    let fav_text = if detail_state.is_favorite {
                                        "已收藏"
                                    } else {
                                        "收藏"
                                    };
                                    let fav_color = if detail_state.is_favorite {
                                        Color::srgb(0.8, 0.6, 0.2)
                                    } else {
                                        AppColors::SECONDARY
                                    };
                                    spawn_action_button(
                                        buttons,
                                        &font,
                                        fav_text,
                                        fav_color,
                                        FavoriteButton,
                                    );

                                    // 下载按钮
                                    spawn_action_button(
                                        buttons,
                                        &font,
                                        "下载",
                                        AppColors::SECONDARY,
                                        DownloadButton,
                                    );
                                });

                            // 章节列表标题
                            content
                                .spawn(Node {
                                    width: Val::Percent(100.0),
                                    margin: UiRect::bottom(Val::Px(10.0)),
                                    ..default()
                                })
                                .with_children(|title_row| {
                                    title_row.spawn((
                                        Text::new(format!(
                                            "章节列表 ({})",
                                            detail_state.episodes.len()
                                        )),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(AppColors::TEXT),
                                    ));

                                    if detail_state.is_loading_episodes {
                                        title_row.spawn((
                                            Text::new(" 加载中..."),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(AppColors::TEXT_SECONDARY),
                                        ));
                                    }
                                });

                            // 章节网格
                            content
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_wrap: FlexWrap::Wrap,
                                        column_gap: Val::Px(episode_layout::COLUMN_GAP),
                                        row_gap: Val::Px(episode_layout::ROW_GAP),
                                        padding: UiRect {
                                            left: Val::Px(0.0),
                                            right: Val::Px(0.0),
                                            top: Val::Px(episode_layout::PADDING_TOP),
                                            bottom: Val::Px(episode_layout::PADDING_BOTTOM),
                                        },
                                        ..default()
                                    },
                                    Transform::default(), /* 必须添加，否则子实体的
                                                           * GlobalTransform 会报警告 */
                                ))
                                .with_children(|grid| {
                                    for episode in &detail_state.episodes {
                                        spawn_episode_card(grid, episode, &font);
                                    }
                                });
                        } else {
                            // 空状态
                            content.spawn((
                                Text::new("暂无数据"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        }
                    })
                    .id();

                // 创建滚动条
                spawn_scrollbar_inline(wrapper, scroll_container_id);
            });
        })
        .id();

    // 如果有 ContentArea，将详情页作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(detail_root);
    }
}

/// 创建操作按钮
fn spawn_action_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    text: &str,
    color: Color,
    marker: M,
) {
    parent
        .spawn((
            marker,
            Button,
            Interaction::default(), // 必须添加！否则按钮无法点击
            Node {
                width: Val::Px(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 创建章节卡片
fn spawn_episode_card(parent: &mut ChildSpawnerCommands, episode: &Episode, font: &Handle<Font>) {
    parent
        .spawn((
            EpisodeCard {
                episode_order: episode.order,
            },
            Button,
            Interaction::default(), // 必须添加！否则卡片无法点击
            Node {
                width: Val::Px(episode_layout::CARD_WIDTH),
                height: Val::Px(episode_layout::CARD_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(AppColors::SURFACE),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(&episode.title),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 内联创建滚动条（在详情页内）
fn spawn_scrollbar_inline(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
    parent
        .spawn((
            ScrollbarContainer { scroll_container },
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(SCROLLBAR_WIDTH),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|scrollbar| {
            // 轨道
            scrollbar
                .spawn((
                    ScrollbarTrack { scroll_container },
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    BackgroundColor(TRACK_COLOR),
                    ZIndex(0),
                    Transform::default(),
                ))
                .with_children(|track| {
                    // 滑块
                    track.spawn((
                        ScrollbarThumb { scroll_container },
                        Button,
                        Interaction::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            top: Val::Px(0.0),
                            left: Val::Px(0.0),
                            ..default()
                        },
                        BackgroundColor(THUMB_COLOR),
                        ZIndex(1),
                    ));
                });
        });
}

/// 清理漫画详情界面
pub fn cleanup_detail_ui(mut commands: Commands, root_query: Query<Entity, With<ComicDetailRoot>>) {
    for entity in root_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 刷新详情页 UI（当状态变化时）
pub fn refresh_detail_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
    let font: Handle<Font> = asset_server.load(FONT_PATH);
    let content_area = content_area_query.single().ok();

    // 调用 setup_detail_ui 的逻辑（内联）
    let detail_root = create_detail_ui_internal(&mut commands, &font, &detail_state, &image_cache);

    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(detail_root);
    }
}

/// 内部创建详情 UI（供 refresh 使用）
fn create_detail_ui_internal(
    commands: &mut Commands,
    font: &Handle<Font>,
    detail_state: &ComicDetailState,
    image_cache: &ImageCache,
) -> Entity {
    commands
        .spawn((
            ComicDetailRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // 标题栏
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BorderColor::all(AppColors::BORDER))
            .with_children(|header| {
                // 返回按钮
                header
                    .spawn((
                        DetailBackButton,
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("\u{F0141}"), // nf-md-arrow_left
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                let title = if let Some(ref comic) = detail_state.comic {
                    comic.title.clone()
                } else {
                    "漫画详情".to_string()
                };
                header.spawn((
                    Text::new(title),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // 滚动区域
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                ..default()
            })
            .with_children(|wrapper| {
                let scroll_container_id = wrapper
                    .spawn((
                        DetailScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect {
                                left: Val::Px(20.0),
                                right: Val::Px(20.0 + SCROLLBAR_WIDTH),
                                top: Val::Px(20.0),
                                bottom: Val::Px(20.0),
                            },
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ContentSizeInfo::default(),
                    ))
                    .with_children(|content| {
                        build_detail_content(content, font, detail_state, image_cache);
                    })
                    .id();

                spawn_scrollbar_inline(wrapper, scroll_container_id);
            });
        })
        .id()
}

/// 构建详情内容
fn build_detail_content(
    content: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    detail_state: &ComicDetailState,
    image_cache: &ImageCache,
) {
    if detail_state.is_loading {
        content.spawn((
            LoadingIndicator,
            Text::new("加载中..."),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(AppColors::TEXT),
        ));
        return;
    }

    if let Some(ref error) = detail_state.error {
        content.spawn((
            ErrorMessage,
            Text::new(format!("加载失败: {}", error)),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.4, 0.4)),
        ));
        return;
    }

    let Some(ref comic) = detail_state.comic else {
        content.spawn((
            Text::new("暂无数据"),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
        ));
        return;
    };

    // 基本信息区域
    content
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        })
        .with_children(|info_row| {
            // 封面图片
            let thumb_url = comic.thumb.url();
            if let Some(handle) = image_cache.get(&thumb_url) {
                info_row.spawn((
                    CoverImage,
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(280.0),
                        ..default()
                    },
                ));
            } else {
                info_row
                    .spawn((
                        CoverImage,
                        PlaceholderImage,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(280.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(AppColors::SURFACE),
                    ))
                    .with_children(|placeholder| {
                        placeholder.spawn((
                            Text::new("加载中..."),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });
            }

            // 详细信息
            info_row
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|details| {
                    // 标题
                    details.spawn((
                        Text::new(&comic.title),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));

                    // 作者
                    details.spawn((
                        Text::new(format!("作者: {}", comic.author)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));

                    // 汉化组
                    if let Some(ref team) = comic.chinese_team {
                        if !team.is_empty() {
                            details.spawn((
                                Text::new(format!("汉化: {}", team)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                            ));
                        }
                    }

                    // 分类标签（可点击）
                    if !comic.categories.is_empty() {
                        details
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(8.0),
                                row_gap: Val::Px(6.0),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    Text::new("分类: "),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT_SECONDARY),
                                ));
                                for cat in &comic.categories {
                                    row.spawn((
                                        CategoryTag {
                                            category: cat.clone(),
                                        },
                                        Button,
                                        Interaction::default(),
                                        Node {
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.2, 0.3, 0.4)),
                                    ))
                                    .with_children(|tag| {
                                        tag.spawn((
                                            Text::new(cat),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.6, 0.8, 1.0)),
                                        ));
                                    });
                                }
                            });
                    }

                    // 标签（tags）- 可点击搜索
                    if !comic.tags.is_empty() {
                        details
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(6.0),
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    Text::new("标签: "),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT_SECONDARY),
                                ));
                                for tag in &comic.tags {
                                    row.spawn((
                                        TagButton { tag: tag.clone() },
                                        Button,
                                        Interaction::default(),
                                        Node {
                                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BorderColor::all(Color::srgb(0.5, 0.4, 0.6)),
                                        BackgroundColor(Color::srgb(0.15, 0.12, 0.2)),
                                    ))
                                    .with_children(
                                        |tag_box| {
                                            tag_box.spawn((
                                                Text::new(tag),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 11.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.8, 0.6, 0.9)),
                                            ));
                                        },
                                    );
                                }
                            });
                    }

                    // 统计信息
                    details.spawn((
                        Text::new(format!(
                            "章节: {} | 页数: {}",
                            comic.eps_count, comic.pages_count
                        )),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));

                    details.spawn((
                        DetailLikesText,
                        Text::new(format!(
                            "点赞: {} | 浏览: {} | 评论: {}",
                            comic.likes_count, comic.views_count, comic.comments_count
                        )),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));

                    // 更新时间
                    if let Some(ref updated_at) = comic.updated_at {
                        // 格式化时间：2023-01-01T12:00:00.000Z -> 2023-01-01
                        let date = updated_at.split('T').next().unwrap_or(updated_at);
                        details.spawn((
                            Text::new(format!("更新: {}", date)),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    }

                    // 完结状态
                    let status = if comic.finished {
                        "已完结"
                    } else {
                        "连载中"
                    };
                    details.spawn((
                        Text::new(format!("状态: {}", status)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(if comic.finished {
                            Color::srgb(0.4, 0.8, 0.4)
                        } else {
                            Color::srgb(0.8, 0.6, 0.2)
                        }),
                    ));

                    // 描述
                    if let Some(ref desc) = comic.description {
                        if !desc.is_empty() {
                            details.spawn(Node {
                                margin: UiRect::top(Val::Px(8.0)),
                                ..default()
                            });
                            details.spawn((
                                Text::new(desc.clone()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT_SECONDARY),
                                Node {
                                    max_width: Val::Px(500.0),
                                    ..default()
                                },
                            ));
                        }
                    }
                });
        });

    // 操作按钮栏
    content
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(15.0),
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|buttons| {
            spawn_action_button(
                buttons,
                font,
                "开始阅读",
                AppColors::PRIMARY,
                StartReadButton,
            );

            let like_text = if detail_state.is_liked {
                "已点赞"
            } else {
                "点赞"
            };
            let like_color = if detail_state.is_liked {
                Color::srgb(0.8, 0.4, 0.4)
            } else {
                AppColors::SECONDARY
            };
            spawn_action_button(buttons, font, like_text, like_color, LikeButton);

            let fav_text = if detail_state.is_favorite {
                "已收藏"
            } else {
                "收藏"
            };
            let fav_color = if detail_state.is_favorite {
                Color::srgb(0.8, 0.6, 0.2)
            } else {
                AppColors::SECONDARY
            };
            spawn_action_button(buttons, font, fav_text, fav_color, FavoriteButton);

            spawn_action_button(buttons, font, "下载", AppColors::SECONDARY, DownloadButton);
        });

    // 章节列表标题
    content
        .spawn(Node {
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|title_row| {
            title_row.spawn((
                Text::new(format!("章节列表 ({})", detail_state.episodes.len())),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            if detail_state.is_loading_episodes {
                title_row.spawn((
                    Text::new(" 加载中..."),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            }
        });

    // 章节网格
    content
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(episode_layout::COLUMN_GAP),
                row_gap: Val::Px(episode_layout::ROW_GAP),
                padding: UiRect {
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(episode_layout::PADDING_TOP),
                    bottom: Val::Px(episode_layout::PADDING_BOTTOM),
                },
                ..default()
            },
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|grid| {
            for episode in &detail_state.episodes {
                spawn_episode_card(grid, episode, font);
            }
        });
}

/// 章节卡片交互
pub fn episode_card_interaction(
    mut interaction_query: Query<
        (&Interaction, &EpisodeCard, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    detail_state: Res<ComicDetailState>,
    mut navigate_messages: MessageWriter<NavigateToReaderEvent>,
) {
    for (interaction, card, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
                // 导航到阅读器
                navigate_messages.write(NavigateToReaderEvent {
                    comic_id: detail_state.comic_id.clone(),
                    episode_order: card.episode_order,
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.30));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::SURFACE);
            }
        }
    }
}

/// 开始阅读按钮交互
pub fn start_read_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartReadButton>),
    >,
    detail_state: Res<ComicDetailState>,
    mut navigate_messages: MessageWriter<NavigateToReaderEvent>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.5, 0.8));
                // 从第一章开始阅读
                navigate_messages.write(NavigateToReaderEvent {
                    comic_id: detail_state.comic_id.clone(),
                    episode_order: 1,
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.55, 0.85));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
        }
    }
}

/// 点赞按钮交互
pub fn like_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<LikeButton>),
    >,
    detail_state: Res<ComicDetailState>,
    mut like_messages: MessageWriter<LikeComicRequest>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                like_messages.write(LikeComicRequest {
                    comic_id: detail_state.comic_id.clone(),
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.40));
            }
            Interaction::None => {
                let color = if detail_state.is_liked {
                    Color::srgb(0.8, 0.4, 0.4)
                } else {
                    AppColors::SECONDARY
                };
                *bg_color = BackgroundColor(color);
            }
        }
    }
}

/// 收藏按钮交互
pub fn favorite_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<FavoriteButton>),
    >,
    detail_state: Res<ComicDetailState>,
    mut favorite_messages: MessageWriter<FavoriteComicRequest>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                favorite_messages.write(FavoriteComicRequest {
                    comic_id: detail_state.comic_id.clone(),
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.40));
            }
            Interaction::None => {
                let color = if detail_state.is_favorite {
                    Color::srgb(0.8, 0.6, 0.2)
                } else {
                    AppColors::SECONDARY
                };
                *bg_color = BackgroundColor(color);
            }
        }
    }
}

/// 下载按钮交互
pub fn download_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DownloadButton>),
    >,
    detail_state: Res<ComicDetailState>,
    mut download_messages: MessageWriter<DownloadComicRequest>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));

                // 发送下载请求
                if let Some(ref comic) = detail_state.comic {
                    tracing::info!("开始下载漫画: {} ({})", comic.title, detail_state.comic_id);
                    download_messages.write(DownloadComicRequest {
                        comic_id: detail_state.comic_id.clone(),
                        comic_title: comic.title.clone(),
                        episodes: vec![], // 空表示下载所有章节
                    });
                } else {
                    tracing::warn!("漫画信息未加载，无法下载");
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.40));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::SECONDARY);
            }
        }
    }
}

/// 更新封面图片（当图片加载完成时）
pub fn update_cover_image(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    detail_state: Res<ComicDetailState>,
    placeholder_query: Query<Entity, (With<CoverImage>, With<PlaceholderImage>)>,
) {
    let Some(ref comic) = detail_state.comic else {
        return;
    };

    let thumb_url = comic.thumb.url();

    for placeholder_entity in placeholder_query.iter() {
        if let Some(handle) = image_cache.get(&thumb_url) {
            // 替换占位符为实际图片
            commands
                .entity(placeholder_entity)
                .remove::<PlaceholderImage>();
            commands
                .entity(placeholder_entity)
                .remove::<BackgroundColor>();
            commands
                .entity(placeholder_entity)
                .insert(ImageNode::new(handle.clone()));
        }
    }
}

/// 处理详情页滚动
pub fn handle_detail_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<DetailScrollContainer>,
    >,
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        let scroll_delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        };

        for (mut scroll_pos, content_info) in scroll_query.iter_mut() {
            let max_scroll = content_info
                .map(|info| (info.content_height - info.viewport_height).max(0.0))
                .unwrap_or(0.0);
            scroll_pos.y = (scroll_pos.y - scroll_delta).clamp(0.0, max_scroll);
        }
    }
}

/// 限制详情页滚动范围
pub fn clamp_detail_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<DetailScrollContainer>,
    >,
) {
    for (mut scroll_pos, content_info) in scroll_query.iter_mut() {
        if scroll_pos.y < 0.0 {
            scroll_pos.y = 0.0;
        }
        if let Some(info) = content_info {
            let max_scroll = (info.content_height - info.viewport_height).max(0.0);
            if scroll_pos.y > max_scroll {
                scroll_pos.y = max_scroll;
            }
        }
    }
}

/// 返回按钮交互
pub fn detail_back_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DetailBackButton>),
    >,
    mut navigate_back_messages: MessageWriter<NavigateBackEvent>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                navigate_back_messages.write(NavigateBackEvent);
                tracing::debug!("详情页返回按钮点击");
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.30));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

/// 分类标签点击交互
pub fn category_tag_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &CategoryTag),
        Changed<Interaction>,
    >,
    mut navigate_messages: MessageWriter<NavigateToComicsListEvent>,
) {
    for (interaction, mut bg_color, tag) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.4, 0.5));
                navigate_messages.write(NavigateToComicsListEvent {
                    category: tag.category.clone(),
                });
                tracing::info!("点击分类标签: {}", tag.category);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.45));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.3, 0.4));
            }
        }
    }
}

/// 标签点击交互（跳转到搜索页面搜索该标签）
pub fn tag_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &TagButton,
        ),
        Changed<Interaction>,
    >,
    mut search_state: ResMut<SearchState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
    current_route: Res<State<AppRoute>>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    for (interaction, mut bg_color, mut border_color, tag_btn) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.2, 0.35));
                *border_color = BorderColor::all(Color::srgb(0.7, 0.5, 0.8));

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
                });

                tracing::info!("点击标签搜索: {}", tag_btn.tag);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.17, 0.28));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.5, 0.7));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.12, 0.2));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.4, 0.6));
            }
        }
    }
}
