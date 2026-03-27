//! 阅读器系统
//!
//! 实现漫画阅读功能

use std::path::PathBuf;

use bevy::{input::mouse::MouseWheel, prelude::*};

use super::font_loader::get_font;
use crate::{
    events::*,
    resources::{ComicDetailState, ImageCache, ReadMode, ReaderState},
    systems::{downloads::get_download_base_path, ui_common::Scrollable},
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 阅读器根节点
#[derive(Component)]
pub struct ReaderRoot;

/// 阅读器图片容器
#[derive(Component)]
pub struct ReaderImageContainer;

/// 当前显示的图片
#[derive(Component)]
pub struct ReaderCurrentImage {
    /// 当前图片的 URL
    #[allow(dead_code)]
    pub url: String,
}

/// 阅读器顶部工具栏
#[derive(Component)]
pub struct ReaderToolbar;

/// 阅读器底部信息栏
#[derive(Component)]
pub struct ReaderBottomBar;

/// 返回按钮
#[derive(Component)]
pub struct ReaderBackButton;

/// 上一页按钮
#[derive(Component)]
pub struct ReaderPrevButton;

/// 下一页按钮
#[derive(Component)]
pub struct ReaderNextButton;

/// 页码显示文本
#[derive(Component)]
pub struct ReaderPageText;

/// 章节标题文本
#[derive(Component)]
pub struct ReaderEpisodeText;

/// 加载指示器
#[derive(Component)]
pub struct ReaderLoadingIndicator;

/// 错误提示
#[derive(Component)]
pub struct ReaderErrorText;

/// 图片加载中指示器（单页图片）
#[derive(Component)]
pub struct ReaderImageLoading {
    pub url: String,
}

/// 模式切换按钮
#[derive(Component)]
pub struct ReaderModeButton;

/// 缩放显示文本
#[derive(Component)]
pub struct ReaderScaleText;

/// Webtoon 模式滚动容器
#[derive(Component)]
pub struct WebtoonScrollContainer;

/// Webtoon 模式图片项
#[derive(Component)]
pub struct WebtoonImageItem {
    #[allow(dead_code)]
    pub index: usize,
}

// ==================== 常量 ====================

mod consts {
    pub const TOOLBAR_HEIGHT: f32 = 50.0;
    pub const BOTTOM_BAR_HEIGHT: f32 = 40.0;
    /// 最小缩放比例
    pub const MIN_SCALE: f32 = 0.5;
    /// 最大缩放比例
    pub const MAX_SCALE: f32 = 3.0;
    /// 缩放步长
    pub const SCALE_STEP: f32 = 0.1;
    /// Webtoon 模式预加载图片数量（向下）
    #[allow(dead_code)]
    pub const WEBTOON_PRELOAD_AHEAD: usize = 5;
    /// Webtoon 模式保留图片数量（向上）
    #[allow(dead_code)]
    pub const WEBTOON_KEEP_BEHIND: usize = 10;
}

/// 创建图片节点的 Node 样式（根据缩放比例）
/// 确保图片居中显示，并保持原始比例缩放到容器内
fn image_node_style(scale: f32) -> Node {
    let scale_percent = scale * 100.0;
    Node {
        // 高度根据缩放比例，宽度自动计算以保持比例
        width: Val::Auto,
        height: Val::Percent(scale_percent),
        // 最大宽度限制，防止横向图片超出
        max_width: Val::Percent(scale_percent),
        ..default()
    }
}

// ==================== 本地文件加载 ====================

/// 清理文件名中的非法字符（与 api_plugin.rs 中的 sanitize_filename 保持一致）
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// 尝试获取本地图片路径
///
/// 返回 Some(path) 如果本地文件存在，否则返回 None
fn try_get_local_image_path(
    comic_title: &str,
    episode_order: i32,
    original_name: &str,
    page_index: usize,
) -> Option<PathBuf> {
    let base_path = get_download_base_path();
    let sanitized_title = sanitize_filename(comic_title);
    let ep_folder = base_path
        .join(&sanitized_title)
        .join(format!("第{}章", episode_order));

    // 尝试原始文件名
    if !original_name.is_empty() {
        let path = ep_folder.join(original_name);
        if path.exists() {
            tracing::info!("找到本地文件: {}", path.display());
            return Some(path);
        }
    }

    // 尝试序号文件名 (如: 0001.jpg)
    let numbered_name = format!("{:04}.jpg", page_index + 1);
    let path = ep_folder.join(&numbered_name);
    if path.exists() {
        tracing::info!("找到本地文件: {}", path.display());
        return Some(path);
    }

    None
}

// ==================== Setup/Cleanup ====================

/// 创建阅读器 UI
pub fn setup_reader_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    reader_state: Res<ReaderState>,
) {
    let font: Handle<Font> = get_font();

    // 根节点 - 全屏黑色背景，使用层叠布局
    commands
        .spawn((
            ReaderRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            // 图片显示区域（底层，全屏）
            spawn_image_area(root, &font);

            // 顶部工具栏（浮动层）
            spawn_toolbar(root, &font, &reader_state);

            // 底部信息栏（浮动层）
            spawn_bottom_bar(root, &font, &reader_state);
        });

    tracing::info!(
        "阅读器 UI 初始化: comic_id={}, episode={}",
        reader_state.comic_id,
        reader_state.episode_order
    );
}

/// 创建顶部工具栏
fn spawn_toolbar(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    reader_state: &ReaderState,
) {
    parent
        .spawn((
            ReaderToolbar,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(consts::TOOLBAR_HEIGHT),
                padding: UiRect::horizontal(Val::Px(15.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ZIndex(10), // 确保在图片上方
        ))
        .with_children(|toolbar| {
            // 左侧：返回按钮
            toolbar
                .spawn((
                    ReaderBackButton,
                    Button,
                    Interaction::default(),
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(ICON_CHEVRON_LEFT),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // 中间：章节标题
            toolbar.spawn((
                ReaderEpisodeText,
                Text::new(format!("第 {} 章", reader_state.episode_order)),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 右侧：缩放显示 + 模式切换
            toolbar
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },))
                .with_children(|right| {
                    // 缩放显示
                    right.spawn((
                        ReaderScaleText,
                        Text::new(format!("{}%", (reader_state.scale * 100.0) as i32)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));

                    // 模式切换按钮（显示当前模式，点击切换）
                    let mode_label = match reader_state.read_mode {
                        ReadMode::SinglePage => "单页",
                        ReadMode::Webtoon => "条漫",
                    };
                    right
                        .spawn((
                            ReaderModeButton,
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8)),
                            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(mode_label),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// 创建图片显示区域
fn spawn_image_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            ReaderImageContainer,
            Node {
                // 全屏显示
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|container| {
            // 加载指示器（默认显示）
            container.spawn((
                ReaderLoadingIndicator,
                Text::new("加载中..."),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

/// 创建底部信息栏
fn spawn_bottom_bar(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    reader_state: &ReaderState,
) {
    parent
        .spawn((
            ReaderBottomBar,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(consts::BOTTOM_BAR_HEIGHT),
                padding: UiRect::horizontal(Val::Px(15.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ZIndex(10), // 确保在图片上方
        ))
        .with_children(|bar| {
            // 上一页按钮
            bar.spawn((
                ReaderPrevButton,
                Button,
                Interaction::default(),
                Node {
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(format!("{ICON_CHEVRON_LEFT} 上一页")),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 页码显示
            bar.spawn((
                ReaderPageText,
                Text::new(format!(
                    "{} / {}",
                    reader_state.current_page, reader_state.total_pages
                )),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 下一页按钮
            bar.spawn((
                ReaderNextButton,
                Button,
                Interaction::default(),
                Node {
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(format!("下一页 {ICON_CHEVRON_RIGHT}")),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 清理阅读器 UI
pub fn cleanup_reader_ui(mut commands: Commands, query: Query<Entity, With<ReaderRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ==================== 图片加载 ====================

/// 触发加载图片
pub fn trigger_load_pictures(
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
) {
    if !reader_state.comic_id.is_empty()
        && reader_state.pictures.is_empty()
        && !reader_state.is_loading
    {
        tracing::info!(
            "触发加载图片: comic_id={}, episode={}",
            reader_state.comic_id,
            reader_state.episode_order
        );
        reader_state.is_loading = true;
        load_messages.write(LoadPicturesRequest {
            comic_id: reader_state.comic_id.clone(),
            episode_order: reader_state.episode_order,
            page: 1,
        });
    }
}

/// 处理图片加载完成
pub fn handle_pictures_loaded(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    mut pictures_events: MessageReader<PicturesLoadedEvent>,
    loading_query: Query<Entity, With<ReaderLoadingIndicator>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    asset_server: Res<AssetServer>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    image_cache: Res<ImageCache>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    comic_detail_state: Res<ComicDetailState>,
) {
    for event in pictures_events.read() {
        tracing::info!(
            "图片加载完成: {} 张, 共 {} 页, 模式: {:?}",
            event.pictures.len(),
            event.total_pages,
            reader_state.read_mode
        );

        // 更新状态
        reader_state.pictures = event.pictures.clone();
        reader_state.total_pages = event.total_pages;
        reader_state.is_loading = false;
        reader_state.current_page = 1;

        // 移除加载指示器
        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }

        // 更新页码显示
        for mut text in page_text_query.iter_mut() {
            **text = format!(
                "{} / {}",
                reader_state.current_page,
                reader_state.pictures.len()
            );
        }

        let Ok(container) = container_query.single() else {
            continue;
        };

        // 获取漫画标题用于构建本地路径
        let comic_title = comic_detail_state
            .comic
            .as_ref()
            .map(|c| c.title.clone())
            .unwrap_or_default();

        // 根据阅读模式创建视图
        match reader_state.read_mode {
            ReadMode::SinglePage => {
                // 单页模式：显示第一张图片
                if let Some(picture) = reader_state.pictures.first() {
                    let image_url = picture.media.url();
                    spawn_single_page_image(
                        &mut commands,
                        container,
                        &image_url,
                        &picture.media.original_name,
                        0,
                        &comic_title,
                        reader_state.episode_order,
                        reader_state.scale,
                        &asset_server,
                        &image_cache,
                        &mut load_image_messages,
                    );
                }
            }
            ReadMode::Webtoon => {
                // Webtoon 模式：创建滚动容器显示所有图片
                let font: Handle<Font> = get_font();
                commands.entity(container).with_children(|parent| {
                    parent
                        .spawn((
                            WebtoonScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                overflow: Overflow::scroll_y(),
                                padding: UiRect::vertical(Val::Px(consts::TOOLBAR_HEIGHT + 10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::BLACK),
                            Scrollable,
                            ScrollPosition::default(),
                        ))
                        .with_children(|scroll| {
                            for (index, picture) in reader_state.pictures.iter().enumerate() {
                                let image_url = picture.media.url();

                                if let Some(local_path) = try_get_local_image_path(
                                    &comic_title,
                                    reader_state.episode_order,
                                    &picture.media.original_name,
                                    index,
                                ) {
                                    let handle: Handle<Image> = asset_server.load(local_path);
                                    scroll.spawn((
                                        WebtoonImageItem { index },
                                        ImageNode {
                                            image: handle,
                                            ..default()
                                        },
                                        webtoon_image_style(reader_state.scale),
                                    ));
                                } else if let Some(handle) = image_cache.get(&image_url) {
                                    scroll.spawn((
                                        WebtoonImageItem { index },
                                        ImageNode {
                                            image: handle.clone(),
                                            ..default()
                                        },
                                        webtoon_image_style(reader_state.scale),
                                    ));
                                } else {
                                    load_image_messages.write(LoadImageRequest {
                                        url: image_url.clone(),
                                    });

                                    scroll.spawn((
                                        WebtoonImageItem { index },
                                        ReaderImageLoading { url: image_url },
                                        Text::new(format!("图片 {} 加载中...", index + 1)),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                        Node {
                                            width: Val::Percent(80.0),
                                            height: Val::Px(200.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            margin: UiRect::vertical(Val::Px(5.0)),
                                            ..default()
                                        },
                                    ));
                                }
                            }
                        });
                });

                tracing::info!(
                    "Webtoon 模式：创建了 {} 张图片",
                    reader_state.pictures.len()
                );
            }
        }
    }
}

/// 处理图片加载失败
pub fn handle_pictures_load_failed(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    mut error_events: MessageReader<PicturesLoadFailedEvent>,
    loading_query: Query<Entity, With<ReaderLoadingIndicator>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    _asset_server: Res<AssetServer>,
) {
    for event in error_events.read() {
        tracing::error!("图片加载失败: {}", event.error);

        reader_state.is_loading = false;
        reader_state.error = Some(event.error.clone());

        // 移除加载指示器
        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }

        // 显示错误信息
        if let Ok(container) = container_query.single() {
            let font: Handle<Font> = get_font();
            commands.entity(container).with_children(|parent| {
                parent.spawn((
                    ReaderErrorText,
                    Text::new(format!("加载失败: {}", event.error)),
                    TextFont {
                        font,
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                ));
            });
        }
    }
}

// ==================== 交互处理 ====================

/// 返回按钮交互
pub fn reader_back_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderBackButton>)>,
    mut back_events: MessageWriter<NavigateBackEvent>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            back_events.write(NavigateBackEvent);
        }
    }
}

/// 上一页按钮交互
pub fn reader_prev_button_interaction(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderPrevButton>)>,
    mut reader_state: ResMut<ReaderState>,
    current_image_query: Query<Entity, With<ReaderCurrentImage>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    image_loading_query: Query<Entity, With<ReaderImageLoading>>,
    comic_detail_state: Res<ComicDetailState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let comic_title = comic_detail_state
                .comic
                .as_ref()
                .map(|c| c.title.clone())
                .unwrap_or_default();
            go_to_prev_page(
                &mut commands,
                &mut reader_state,
                &current_image_query,
                &container_query,
                &mut page_text_query,
                &asset_server,
                &image_cache,
                &mut load_image_messages,
                &image_loading_query,
                &comic_title,
            );
        }
    }
}

/// 下一页按钮交互
pub fn reader_next_button_interaction(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderNextButton>)>,
    mut reader_state: ResMut<ReaderState>,
    current_image_query: Query<Entity, With<ReaderCurrentImage>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    image_loading_query: Query<Entity, With<ReaderImageLoading>>,
    comic_detail_state: Res<ComicDetailState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let comic_title = comic_detail_state
                .comic
                .as_ref()
                .map(|c| c.title.clone())
                .unwrap_or_default();
            go_to_next_page(
                &mut commands,
                &mut reader_state,
                &current_image_query,
                &container_query,
                &mut page_text_query,
                &asset_server,
                &image_cache,
                &mut load_image_messages,
                &image_loading_query,
                &comic_title,
            );
        }
    }
}

/// 键盘控制
pub fn reader_keyboard_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    current_image_query: Query<Entity, With<ReaderCurrentImage>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    image_loading_query: Query<Entity, With<ReaderImageLoading>>,
    mut back_events: MessageWriter<NavigateBackEvent>,
    comic_detail_state: Res<ComicDetailState>,
) {
    let comic_title = comic_detail_state
        .comic
        .as_ref()
        .map(|c| c.title.clone())
        .unwrap_or_default();

    // 左方向键 / A 键 - 上一页
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) || keyboard_input.just_pressed(KeyCode::KeyA)
    {
        go_to_prev_page(
            &mut commands,
            &mut reader_state,
            &current_image_query,
            &container_query,
            &mut page_text_query,
            &asset_server,
            &image_cache,
            &mut load_image_messages,
            &image_loading_query,
            &comic_title,
        );
    }

    // 右方向键 / D 键 / 空格键 - 下一页
    if keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::KeyD)
        || keyboard_input.just_pressed(KeyCode::Space)
    {
        go_to_next_page(
            &mut commands,
            &mut reader_state,
            &current_image_query,
            &container_query,
            &mut page_text_query,
            &asset_server,
            &image_cache,
            &mut load_image_messages,
            &image_loading_query,
            &comic_title,
        );
    }

    // Escape 键 - 返回
    if keyboard_input.just_pressed(KeyCode::Escape) {
        back_events.write(NavigateBackEvent);
    }
}

// ==================== 翻页逻辑 ====================

/// 跳转到上一页
fn go_to_prev_page(
    commands: &mut Commands,
    reader_state: &mut ResMut<ReaderState>,
    current_image_query: &Query<Entity, With<ReaderCurrentImage>>,
    container_query: &Query<Entity, With<ReaderImageContainer>>,
    page_text_query: &mut Query<&mut Text, With<ReaderPageText>>,
    asset_server: &Res<AssetServer>,
    image_cache: &Res<ImageCache>,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
    image_loading_query: &Query<Entity, With<ReaderImageLoading>>,
    comic_title: &str,
) {
    if reader_state.current_page <= 1 {
        return;
    }

    reader_state.current_page -= 1;
    update_current_image(
        commands,
        reader_state,
        current_image_query,
        container_query,
        page_text_query,
        asset_server,
        image_cache,
        load_image_messages,
        image_loading_query,
        comic_title,
    );
}

/// 跳转到下一页
fn go_to_next_page(
    commands: &mut Commands,
    reader_state: &mut ResMut<ReaderState>,
    current_image_query: &Query<Entity, With<ReaderCurrentImage>>,
    container_query: &Query<Entity, With<ReaderImageContainer>>,
    page_text_query: &mut Query<&mut Text, With<ReaderPageText>>,
    asset_server: &Res<AssetServer>,
    image_cache: &Res<ImageCache>,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
    image_loading_query: &Query<Entity, With<ReaderImageLoading>>,
    comic_title: &str,
) {
    if reader_state.current_page >= reader_state.pictures.len() as i32 {
        return;
    }

    reader_state.current_page += 1;
    update_current_image(
        commands,
        reader_state,
        current_image_query,
        container_query,
        page_text_query,
        asset_server,
        image_cache,
        load_image_messages,
        image_loading_query,
        comic_title,
    );
}

/// 更新当前显示的图片
fn update_current_image(
    commands: &mut Commands,
    reader_state: &ResMut<ReaderState>,
    current_image_query: &Query<Entity, With<ReaderCurrentImage>>,
    container_query: &Query<Entity, With<ReaderImageContainer>>,
    page_text_query: &mut Query<&mut Text, With<ReaderPageText>>,
    asset_server: &Res<AssetServer>,
    image_cache: &Res<ImageCache>,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
    image_loading_query: &Query<Entity, With<ReaderImageLoading>>,
    comic_title: &str,
) {
    let page_index = (reader_state.current_page - 1) as usize;
    if let Some(picture) = reader_state.pictures.get(page_index) {
        // 移除旧图片
        for entity in current_image_query.iter() {
            commands.entity(entity).despawn();
        }
        // 移除旧的加载指示器
        for entity in image_loading_query.iter() {
            commands.entity(entity).despawn();
        }

        // 添加新图片
        if let Ok(container) = container_query.single() {
            let image_url = picture.media.url();
            tracing::debug!("切换到第 {} 页: {}", reader_state.current_page, image_url);

            // 1. 首先检查本地文件是否存在
            if let Some(local_path) = try_get_local_image_path(
                comic_title,
                reader_state.episode_order,
                &picture.media.original_name,
                page_index,
            ) {
                // 从本地加载
                let handle: Handle<Image> = asset_server.load(local_path);
                commands.entity(container).with_children(|parent| {
                    parent.spawn((
                        ReaderCurrentImage {
                            url: image_url.clone(),
                        },
                        ImageNode {
                            image: handle,
                            ..default()
                        },
                        image_node_style(reader_state.scale),
                    ));
                });
            }
            // 2. 检查图片是否已在内存缓存中
            else if let Some(handle) = image_cache.get(&image_url) {
                // 图片已加载，直接显示
                commands.entity(container).with_children(|parent| {
                    parent.spawn((
                        ReaderCurrentImage {
                            url: image_url.clone(),
                        },
                        ImageNode {
                            image: handle.clone(),
                            ..default()
                        },
                        image_node_style(reader_state.scale),
                    ));
                });
            } else {
                // 3. 从网络加载
                load_image_messages.write(LoadImageRequest {
                    url: image_url.clone(),
                });

                let font: Handle<Font> = get_font();
                commands.entity(container).with_children(|parent| {
                    parent.spawn((
                        ReaderImageLoading {
                            url: image_url.clone(),
                        },
                        Text::new("图片加载中..."),
                        TextFont {
                            font,
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
            }
        }

        // 更新页码显示
        for mut text in page_text_query.iter_mut() {
            **text = format!(
                "{} / {}",
                reader_state.current_page,
                reader_state.pictures.len()
            );
        }
    }
}

/// 检查图片缓存并更新显示
pub fn update_reader_image_from_cache(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    reader_state: Res<ReaderState>,
    image_loading_query: Query<(Entity, &ReaderImageLoading, &ChildOf)>,
) {
    for (entity, loading, child_of) in image_loading_query.iter() {
        // 检查图片是否已加载
        if let Some(handle) = image_cache.get(&loading.url) {
            let parent = child_of.parent();
            let url = loading.url.clone();

            // 移除加载指示器
            commands.entity(entity).despawn();

            // 添加图片
            commands.entity(parent).with_children(|p| {
                p.spawn((
                    ReaderCurrentImage { url },
                    ImageNode {
                        image: handle.clone(),
                        ..default()
                    },
                    image_node_style(reader_state.scale),
                ));
            });
        }
    }
}

// ==================== 鼠标滚轮和缩放控制 ====================

/// Webtoon 模式滚动速度（每行滚动像素）
const WEBTOON_SCROLL_SPEED: f32 = 60.0;

/// 鼠标滚轮控制（翻页 / Ctrl+滚轮缩放）
pub fn reader_mouse_wheel_control(
    mut commands: Commands,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    current_image_query: Query<Entity, With<ReaderCurrentImage>>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    image_loading_query: Query<Entity, With<ReaderImageLoading>>,
    comic_detail_state: Res<ComicDetailState>,
    mut scale_text_query: Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
    mut webtoon_scroll_query: Query<&mut ScrollPosition, With<WebtoonScrollContainer>>,
) {
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    let comic_title = comic_detail_state
        .comic
        .as_ref()
        .map(|c| c.title.clone())
        .unwrap_or_default();

    for event in mouse_wheel_events.read() {
        // 根据单位计算滚动量
        let scroll_delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y / 40.0,
        };

        if ctrl_pressed {
            // Ctrl + 滚轮：缩放
            let new_scale = if scroll_delta > 0.0 {
                (reader_state.scale + consts::SCALE_STEP).min(consts::MAX_SCALE)
            } else {
                (reader_state.scale - consts::SCALE_STEP).max(consts::MIN_SCALE)
            };

            if (new_scale - reader_state.scale).abs() > 0.001 {
                reader_state.scale = new_scale;
                update_image_scale(
                    &mut commands,
                    &reader_state,
                    &current_image_query,
                    &mut scale_text_query,
                );
            }
        } else {
            // 普通滚轮：翻页（单页模式）或滚动（Webtoon 模式）
            match reader_state.read_mode {
                ReadMode::SinglePage => {
                    if scroll_delta < 0.0 {
                        // 向下滚动 = 下一页
                        go_to_next_page(
                            &mut commands,
                            &mut reader_state,
                            &current_image_query,
                            &container_query,
                            &mut page_text_query,
                            &asset_server,
                            &image_cache,
                            &mut load_image_messages,
                            &image_loading_query,
                            &comic_title,
                        );
                    } else if scroll_delta > 0.0 {
                        // 向上滚动 = 上一页
                        go_to_prev_page(
                            &mut commands,
                            &mut reader_state,
                            &current_image_query,
                            &container_query,
                            &mut page_text_query,
                            &asset_server,
                            &image_cache,
                            &mut load_image_messages,
                            &image_loading_query,
                            &comic_title,
                        );
                    }
                }
                ReadMode::Webtoon => {
                    // Webtoon 模式：手动更新 ScrollPosition
                    // MessageReader 已消费事件，Bevy 原生滚动不会收到，需手动处理
                    for mut scroll_pos in webtoon_scroll_query.iter_mut() {
                        // 向上滚动 (scroll_delta > 0) -> scroll_pos.y 减小
                        // 向下滚动 (scroll_delta < 0) -> scroll_pos.y 增大
                        let scroll_amount = -scroll_delta * WEBTOON_SCROLL_SPEED;
                        scroll_pos.y = (scroll_pos.y + scroll_amount).max(0.0);
                    }
                }
            }
        }
    }
}

/// 键盘 +/- 缩放控制
pub fn reader_zoom_keyboard_control(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    current_image_query: Query<Entity, With<ReaderCurrentImage>>,
    mut scale_text_query: Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
) {
    let mut scale_changed = false;

    // + 键或 = 键（放大）
    if keyboard_input.just_pressed(KeyCode::Equal)
        || keyboard_input.just_pressed(KeyCode::NumpadAdd)
    {
        reader_state.scale = (reader_state.scale + consts::SCALE_STEP).min(consts::MAX_SCALE);
        scale_changed = true;
    }

    // - 键（缩小）
    if keyboard_input.just_pressed(KeyCode::Minus)
        || keyboard_input.just_pressed(KeyCode::NumpadSubtract)
    {
        reader_state.scale = (reader_state.scale - consts::SCALE_STEP).max(consts::MIN_SCALE);
        scale_changed = true;
    }

    // 0 键（重置缩放）
    if keyboard_input.just_pressed(KeyCode::Digit0) || keyboard_input.just_pressed(KeyCode::Numpad0)
    {
        reader_state.scale = 1.0;
        scale_changed = true;
    }

    if scale_changed {
        update_image_scale(
            &mut commands,
            &reader_state,
            &current_image_query,
            &mut scale_text_query,
        );
    }
}

/// 更新图片缩放
fn update_image_scale(
    commands: &mut Commands,
    reader_state: &ReaderState,
    current_image_query: &Query<Entity, With<ReaderCurrentImage>>,
    scale_text_query: &mut Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
) {
    // 更新图片节点的样式
    for entity in current_image_query.iter() {
        commands
            .entity(entity)
            .insert(image_node_style(reader_state.scale));
    }

    // 更新缩放显示文本
    for mut text in scale_text_query.iter_mut() {
        **text = format!("{}%", (reader_state.scale * 100.0) as i32);
    }

    tracing::debug!("缩放比例: {}%", (reader_state.scale * 100.0) as i32);
}

/// 模式切换按钮交互
pub fn reader_mode_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderModeButton>)>,
    mut reader_state: ResMut<ReaderState>,
    mode_btn_children_query: Query<&Children, With<ReaderModeButton>>,
    mut all_text_query: Query<&mut Text, Without<ReaderModeButton>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // 切换模式
            reader_state.read_mode = match reader_state.read_mode {
                ReadMode::SinglePage => ReadMode::Webtoon,
                ReadMode::Webtoon => ReadMode::SinglePage,
            };

            // 更新按钮文字
            let new_label = match reader_state.read_mode {
                ReadMode::SinglePage => "单页",
                ReadMode::Webtoon => "条漫",
            };

            // 查找模式按钮的子文本并更新
            for children in mode_btn_children_query.iter() {
                for child in children.iter() {
                    if let Ok(mut text) = all_text_query.get_mut(child) {
                        **text = new_label.to_string();
                    }
                }
            }

            tracing::info!("切换阅读模式: {:?}", reader_state.read_mode);
        }
    }
}

// ==================== Webtoon 模式 ====================

/// 处理阅读模式变化，重建 UI
pub fn handle_read_mode_change(
    mut commands: Commands,
    reader_state: Res<ReaderState>,
    asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    current_image_query: Query<Entity, With<ReaderCurrentImage>>,
    webtoon_container_query: Query<Entity, With<WebtoonScrollContainer>>,
    image_loading_query: Query<Entity, With<ReaderImageLoading>>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    comic_detail_state: Res<ComicDetailState>,
    mut previous_mode: Local<Option<ReadMode>>,
) {
    // 只在模式实际变化时执行（使用 Local 追踪上一次模式，避免与
    // handle_pictures_loaded 冲突）
    let current_mode = reader_state.read_mode;

    // 检查模式是否真正变化
    let mode_changed = previous_mode
        .map(|prev| prev != current_mode)
        .unwrap_or(false);

    // 更新上一次模式
    *previous_mode = Some(current_mode);

    // 只在模式变化时执行，且图片不为空
    if !mode_changed || reader_state.pictures.is_empty() {
        return;
    }

    tracing::info!("阅读模式切换: {:?}", current_mode);

    let comic_title = comic_detail_state
        .comic
        .as_ref()
        .map(|c| c.title.clone())
        .unwrap_or_default();

    let Ok(container) = container_query.single() else {
        return;
    };

    match reader_state.read_mode {
        ReadMode::SinglePage => {
            // 切换到单页模式：移除 Webtoon 容器，显示当前页
            for entity in webtoon_container_query.iter() {
                commands.entity(entity).despawn();
            }

            // 如果当前没有显示图片，显示当前页
            if current_image_query.is_empty() && image_loading_query.is_empty() {
                let page_index = (reader_state.current_page - 1) as usize;
                if let Some(picture) = reader_state.pictures.get(page_index) {
                    let image_url = picture.media.url();
                    spawn_single_page_image(
                        &mut commands,
                        container,
                        &image_url,
                        &picture.media.original_name,
                        page_index,
                        &comic_title,
                        reader_state.episode_order,
                        reader_state.scale,
                        &asset_server,
                        &image_cache,
                        &mut load_image_messages,
                    );
                }
            }
        }
        ReadMode::Webtoon => {
            // 切换到 Webtoon 模式：移除单页图片，创建滚动容器
            for entity in current_image_query.iter() {
                commands.entity(entity).despawn();
            }
            for entity in image_loading_query.iter() {
                commands.entity(entity).despawn();
            }

            // 如果已有 Webtoon 容器，不重复创建
            if !webtoon_container_query.is_empty() {
                return;
            }

            // 创建 Webtoon 滚动容器
            let font: Handle<Font> = get_font();
            commands.entity(container).with_children(|parent| {
                parent
                    .spawn((
                        WebtoonScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            overflow: Overflow::scroll_y(),
                            padding: UiRect::vertical(Val::Px(consts::TOOLBAR_HEIGHT + 10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::BLACK),
                        Scrollable,
                        ScrollPosition::default(),
                    ))
                    .with_children(|scroll| {
                        // 添加所有图片
                        for (index, picture) in reader_state.pictures.iter().enumerate() {
                            let image_url = picture.media.url();

                            // 检查本地文件
                            if let Some(local_path) = try_get_local_image_path(
                                &comic_title,
                                reader_state.episode_order,
                                &picture.media.original_name,
                                index,
                            ) {
                                let handle: Handle<Image> = asset_server.load(local_path);
                                scroll.spawn((
                                    WebtoonImageItem { index },
                                    ImageNode {
                                        image: handle,
                                        ..default()
                                    },
                                    webtoon_image_style(reader_state.scale),
                                ));
                            } else if let Some(handle) = image_cache.get(&image_url) {
                                scroll.spawn((
                                    WebtoonImageItem { index },
                                    ImageNode {
                                        image: handle.clone(),
                                        ..default()
                                    },
                                    webtoon_image_style(reader_state.scale),
                                ));
                            } else {
                                // 请求加载图片
                                load_image_messages.write(LoadImageRequest {
                                    url: image_url.clone(),
                                });

                                // 显示占位符
                                scroll.spawn((
                                    WebtoonImageItem { index },
                                    ReaderImageLoading { url: image_url },
                                    Text::new(format!("图片 {} 加载中...", index + 1)),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                    Node {
                                        width: Val::Percent(80.0),
                                        height: Val::Px(200.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::vertical(Val::Px(5.0)),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    });
            });

            tracing::info!(
                "Webtoon 模式：创建了 {} 张图片",
                reader_state.pictures.len()
            );
        }
    }
}

/// 创建单页图片的辅助函数
fn spawn_single_page_image(
    commands: &mut Commands,
    container: Entity,
    image_url: &str,
    original_name: &str,
    page_index: usize,
    comic_title: &str,
    episode_order: i32,
    scale: f32,
    asset_server: &Res<AssetServer>,
    image_cache: &Res<ImageCache>,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) {
    // 检查本地文件
    if let Some(local_path) =
        try_get_local_image_path(comic_title, episode_order, original_name, page_index)
    {
        let handle: Handle<Image> = asset_server.load(local_path);
        commands.entity(container).with_children(|parent| {
            parent.spawn((
                ReaderCurrentImage {
                    url: image_url.to_string(),
                },
                ImageNode {
                    image: handle,
                    ..default()
                },
                image_node_style(scale),
            ));
        });
    } else if let Some(handle) = image_cache.get(image_url) {
        commands.entity(container).with_children(|parent| {
            parent.spawn((
                ReaderCurrentImage {
                    url: image_url.to_string(),
                },
                ImageNode {
                    image: handle.clone(),
                    ..default()
                },
                image_node_style(scale),
            ));
        });
    } else {
        load_image_messages.write(LoadImageRequest {
            url: image_url.to_string(),
        });

        let font: Handle<Font> = get_font();
        commands.entity(container).with_children(|parent| {
            parent.spawn((
                ReaderImageLoading {
                    url: image_url.to_string(),
                },
                Text::new("图片加载中..."),
                TextFont {
                    font,
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
    }
}

/// Webtoon 模式图片样式
fn webtoon_image_style(scale: f32) -> Node {
    let scale_percent = scale * 100.0;
    Node {
        width: Val::Percent(scale_percent.min(100.0)),
        height: Val::Auto,
        margin: UiRect::vertical(Val::Px(2.0)),
        ..default()
    }
}

/// 更新 Webtoon 模式下加载完成的图片
pub fn update_webtoon_images_from_cache(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    reader_state: Res<ReaderState>,
    loading_query: Query<(Entity, &WebtoonImageItem, &ReaderImageLoading)>,
) {
    // 只在 Webtoon 模式下执行
    if reader_state.read_mode != ReadMode::Webtoon {
        return;
    }

    for (entity, _item, loading) in loading_query.iter() {
        if let Some(handle) = image_cache.get(&loading.url) {
            // 替换为实际图片
            commands.entity(entity).remove::<ReaderImageLoading>();
            commands.entity(entity).remove::<Text>();
            commands.entity(entity).remove::<TextFont>();
            commands.entity(entity).remove::<TextColor>();
            commands.entity(entity).insert((
                ImageNode {
                    image: handle.clone(),
                    ..default()
                },
                webtoon_image_style(reader_state.scale),
            ));
        }
    }
}

/// 更新 Webtoon 模式下的图片缩放
pub fn update_webtoon_scale(
    reader_state: Res<ReaderState>,
    mut webtoon_images_query: Query<&mut Node, With<WebtoonImageItem>>,
) {
    if !reader_state.is_changed() || reader_state.read_mode != ReadMode::Webtoon {
        return;
    }

    let new_style = webtoon_image_style(reader_state.scale);
    for mut node in webtoon_images_query.iter_mut() {
        node.width = new_style.width;
    }
}

// ==================== 阅读历史保存 ====================

/// 自动保存阅读历史（监听 ReaderState 变化）
///
/// 当页码或章节发生变化时，将当前阅读进度保存到数据库。
/// 使用 `Local<(i32, i32)>` 追踪上一次保存的 (episode, page)，
/// 避免在状态未实际改变时重复写入。
pub fn save_reading_history(
    reader_state: Res<ReaderState>,
    comic_detail_state: Res<ComicDetailState>,
    mut save_messages: MessageWriter<SaveHistoryRequest>,
    mut last_saved: Local<(i32, i32)>,
) {
    // 只在状态变化时检查
    if !reader_state.is_changed() {
        return;
    }

    // 确保有漫画信息且图片已加载
    if reader_state.comic_id.is_empty() || reader_state.pictures.is_empty() {
        return;
    }

    let current = (reader_state.episode_order, reader_state.current_page);

    // 如果和上次保存的一样，跳过
    if *last_saved == current {
        return;
    }
    *last_saved = current;

    // 获取漫画信息
    let Some(comic) = &comic_detail_state.comic else {
        return;
    };

    // 获取章节标题
    let eps_title = comic_detail_state
        .episodes
        .iter()
        .find(|ep| ep.order == reader_state.episode_order)
        .map(|ep| ep.title.clone())
        .unwrap_or_else(|| format!("第{}章", reader_state.episode_order));

    save_messages.write(SaveHistoryRequest {
        comic_id: reader_state.comic_id.clone(),
        comic_title: comic.title.clone(),
        thumb_url: comic.thumb.url(),
        last_eps_order: reader_state.episode_order,
        last_eps_title: eps_title,
        last_page: reader_state.current_page,
    });
}
