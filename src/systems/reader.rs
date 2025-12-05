//! 阅读器系统
//!
//! 实现漫画阅读功能

use bevy::prelude::*;

use crate::{events::*, resources::ReaderState, systems::login::FONT_PATH};

// ==================== 组件定义 ====================

/// 阅读器根节点
#[derive(Component)]
pub struct ReaderRoot;

/// 阅读器图片容器
#[derive(Component)]
pub struct ReaderImageContainer;

/// 当前显示的图片
#[derive(Component)]
pub struct ReaderCurrentImage;

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

// ==================== 常量 ====================

mod consts {
    pub const TOOLBAR_HEIGHT: f32 = 50.0;
    pub const BOTTOM_BAR_HEIGHT: f32 = 40.0;
}

// ==================== Setup/Cleanup ====================

/// 创建阅读器 UI
pub fn setup_reader_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    reader_state: Res<ReaderState>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    // 根节点 - 全屏黑色背景
    commands
        .spawn((
            ReaderRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            // 顶部工具栏
            spawn_toolbar(root, &font, &reader_state);

            // 图片显示区域
            spawn_image_area(root, &font);

            // 底部信息栏
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
                width: Val::Percent(100.0),
                height: Val::Px(consts::TOOLBAR_HEIGHT),
                padding: UiRect::horizontal(Val::Px(15.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
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
                        Text::new("\u{F0141}"), // nf-md-arrow_left
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

            // 右侧：占位（保持居中）
            toolbar.spawn(Node {
                width: Val::Px(40.0),
                ..default()
            });
        });
}

/// 创建图片显示区域
fn spawn_image_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            ReaderImageContainer,
            Node {
                flex_grow: 1.0,
                width: Val::Percent(100.0),
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
                width: Val::Percent(100.0),
                height: Val::Px(consts::BOTTOM_BAR_HEIGHT),
                padding: UiRect::horizontal(Val::Px(15.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
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
                    Text::new("\u{F0141} 上一页"), // nf-md-arrow_left
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
                    Text::new("下一页 \u{F0142}"), // nf-md-arrow_right
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
    reader_state: Res<ReaderState>,
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
) {
    for event in pictures_events.read() {
        tracing::info!(
            "图片加载完成: {} 张, 共 {} 页",
            event.pictures.len(),
            event.total_pages
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

        // 显示第一张图片
        if let Some(picture) = reader_state.pictures.first() {
            if let Ok(container) = container_query.single() {
                let image_url = picture.media.url();
                tracing::info!("显示图片: {}", image_url);

                commands.entity(container).with_children(|parent| {
                    parent.spawn((
                        ReaderCurrentImage,
                        ImageNode {
                            image: asset_server.load(&image_url),
                            ..default()
                        },
                        Node {
                            max_width: Val::Percent(100.0),
                            max_height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                });
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
    asset_server: Res<AssetServer>,
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
            let font: Handle<Font> = asset_server.load(FONT_PATH);
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
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            go_to_prev_page(
                &mut commands,
                &mut reader_state,
                &current_image_query,
                &container_query,
                &mut page_text_query,
                &asset_server,
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
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            go_to_next_page(
                &mut commands,
                &mut reader_state,
                &current_image_query,
                &container_query,
                &mut page_text_query,
                &asset_server,
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
    mut back_events: MessageWriter<NavigateBackEvent>,
) {
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
) {
    let page_index = (reader_state.current_page - 1) as usize;
    if let Some(picture) = reader_state.pictures.get(page_index) {
        // 移除旧图片
        for entity in current_image_query.iter() {
            commands.entity(entity).despawn();
        }

        // 添加新图片
        if let Ok(container) = container_query.single() {
            let image_url = picture.media.url();
            tracing::debug!("切换到第 {} 页: {}", reader_state.current_page, image_url);

            commands.entity(container).with_children(|parent| {
                parent.spawn((
                    ReaderCurrentImage,
                    ImageNode {
                        image: asset_server.load(&image_url),
                        ..default()
                    },
                    Node {
                        max_width: Val::Percent(100.0),
                        max_height: Val::Percent(100.0),
                        ..default()
                    },
                ));
            });
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
