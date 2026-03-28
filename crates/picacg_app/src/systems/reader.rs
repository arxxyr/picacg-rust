//! 阅读器系统
//!
//! 核心设计：
//! - 单页模式：三实体预加载池（SlotPrev / SlotCurrent /
//!   SlotNext），翻页时旋转，零延迟切换
//! - 条漫模式：窗口化加载，只维护当前可见区域 ± PRELOAD_RANGE
//!   张图片，远离窗口的释放纹理
//! - 共享：图片加载本地优先 → ImageCache → 网络请求，章节末尾自动切换下一章

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

/// 阅读器图片容器（单页模式的三 slot 或条漫滚动容器的父级）
#[derive(Component)]
pub struct ReaderImageContainer;

/// 加载指示器
#[derive(Component)]
pub struct ReaderLoadingIndicator;

/// 错误提示
#[derive(Component)]
pub struct ReaderErrorText;

/// 图片加载中指示器（通用，等待 ImageCache 回调）
#[derive(Component)]
pub struct ReaderImageLoading {
    pub url: String,
}

// ---------- 工具栏 ----------

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

/// 模式切换按钮
#[derive(Component)]
pub struct ReaderModeButton;

/// 缩放显示文本
#[derive(Component)]
pub struct ReaderScaleText;

// ---------- 单页模式三 slot ----------

/// 单页模式的图片槽位类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotType {
    /// 前一页预加载（Display::None）
    Prev,
    /// 当前显示
    Current,
    /// 后一页预加载（Display::None）
    Next,
}

/// 单页模式的图片槽位
#[derive(Component)]
pub struct ImageSlot {
    pub slot_type: SlotType,
    /// 该 slot 对应的页码（0-indexed），None 表示空 slot
    pub page_index: Option<usize>,
}

// ---------- 条漫模式 ----------

/// Webtoon 模式滚动容器
#[derive(Component)]
pub struct WebtoonScrollContainer;

/// 条漫模式固定槽位（共 5 个，图片在槽位间流动）
#[derive(Component)]
pub struct WebtoonSlot {
    /// 槽位编号（0-4，固定不变）
    pub slot_index: usize,
    /// 当前绑定的页码（None = 空槽）
    pub page_index: Option<usize>,
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
    /// 条漫模式固定槽位数量
    pub const WEBTOON_SLOT_COUNT: usize = 5;
    /// 条漫模式滚动速度（每行滚动像素）
    pub const WEBTOON_SCROLL_SPEED: f32 = 60.0;
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
            tracing::debug!("找到本地文件: {}", path.display());
            return Some(path);
        }
    }

    // 尝试序号文件名 (如: 0001.jpg)
    let numbered_name = format!("{:04}.jpg", page_index + 1);
    let path = ep_folder.join(&numbered_name);
    if path.exists() {
        tracing::debug!("找到本地文件: {}", path.display());
        return Some(path);
    }

    None
}

// ==================== 图片样式 ====================

/// 单页模式图片节点样式（根据缩放比例）
fn single_page_image_style(scale: f32) -> Node {
    let scale_percent = scale * 100.0;
    Node {
        width: Val::Auto,
        height: Val::Percent(scale_percent),
        max_width: Val::Percent(scale_percent),
        ..default()
    }
}

/// 条漫模式图片节点样式（宽度 100%，高度自动保持比例）
#[allow(dead_code)]
fn webtoon_image_style() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Auto,
        ..default()
    }
}

// ==================== 图片加载辅助 ====================

/// 加载单张图片到指定实体的子节点（本地优先 → 缓存 → 网络）
///
/// 返回 true 表示图片已立即可用（本地或缓存命中），false 表示需要异步加载
fn load_image_for_slot(
    commands: &mut Commands,
    parent_entity: Entity,
    url: &str,
    original_name: &str,
    comic_title: &str,
    episode_order: i32,
    page_index: usize,
    node_style: Node,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) -> bool {
    // 1. 检查本地文件
    if let Some(local_path) =
        try_get_local_image_path(comic_title, episode_order, original_name, page_index)
    {
        let handle: Handle<Image> = asset_server.load(local_path);
        commands.entity(parent_entity).with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: handle,
                    ..default()
                },
                node_style,
            ));
        });
        return true;
    }

    // 2. 检查内存缓存
    if let Some(handle) = image_cache.get(url) {
        commands.entity(parent_entity).with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: handle.clone(),
                    ..default()
                },
                node_style,
            ));
        });
        return true;
    }

    // 3. 网络请求 + 加载中占位
    load_image_messages.write(LoadImageRequest {
        url: url.to_string(),
    });
    commands.entity(parent_entity).with_children(|parent| {
        parent.spawn((
            ReaderImageLoading {
                url: url.to_string(),
            },
            Node {
                width: Val::Percent(50.0),
                height: Val::Px(400.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
        ));
    });
    false
}

// ==================== Setup / Cleanup ====================

/// 创��阅读器 UI
pub fn setup_reader_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    reader_state: Res<ReaderState>,
) {
    let font: Handle<Font> = get_font();

    // 根节点 - 全屏黑色背景
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
            ZIndex(10),
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
            let episode_title = reader_state
                .episodes
                .get(reader_state.current_episode_idx)
                .map(|ep| ep.title.as_str())
                .unwrap_or("未知章节");
            toolbar.spawn((
                ReaderEpisodeText,
                Text::new(format!(
                    "第 {} 章 - {}",
                    reader_state.episode_order, episode_title
                )),
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

                    // 模式切换按钮
                    let mode_label = match reader_state.read_mode {
                        ReadMode::SinglePage => "��页",
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
            ZIndex(10),
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

            // 页码显示（1-indexed 展示）
            let display_page = reader_state.current_page + 1;
            bar.spawn((
                ReaderPageText,
                Text::new(format!("{} / {}", display_page, reader_state.total_pages)),
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

/// 触发加载图片（进入阅读器时自动执行）
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

/// 处理图片加载完成事件 —— 初始化显示
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
) {
    for event in pictures_events.read() {
        // 如果是下一章预加载的结果
        if reader_state.is_loading_next_chapter {
            reader_state.is_loading_next_chapter = false;
            reader_state.next_chapter_pictures = event.pictures.clone();
            tracing::info!(
                "下一章图片预加载完成: {} 张",
                reader_state.next_chapter_pictures.len()
            );
            continue;
        }

        tracing::info!(
            "图片加载完成: {} 张, API 页数={}, 模式={:?}",
            event.pictures.len(),
            event.total_pages,
            reader_state.read_mode
        );

        // 更新状态
        reader_state.pictures = event.pictures.clone();
        reader_state.total_pages = reader_state.pictures.len();
        reader_state.is_loading = false;
        reader_state.current_page = 0;

        // 移除加载指示器
        for entity in loading_query.iter() {
            commands.entity(entity).despawn();
        }

        // 更新页码显示
        update_page_text(&reader_state, &mut page_text_query);

        let Ok(container) = container_query.single() else {
            continue;
        };

        // 根据阅读模式创建视图
        match reader_state.read_mode {
            ReadMode::SinglePage => {
                create_single_page_slots(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
                );
            }
            ReadMode::Webtoon => {
                create_webtoon_view(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
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
) {
    for event in error_events.read() {
        tracing::error!("图片加载失败: {}", event.error);

        reader_state.is_loading = false;
        reader_state.is_loading_next_chapter = false;
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

// ==================== 单页模式：三 slot 预加载池 ====================

/// 创建三个图片 slot（prev / current / next）
fn create_single_page_slots(
    commands: &mut Commands,
    container: Entity,
    reader_state: &ReaderState,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) {
    if reader_state.pictures.is_empty() {
        return;
    }

    let current_page = reader_state.current_page;
    let total = reader_state.total_pages;

    // Prev slot（隐藏）
    let prev_page = if current_page > 0 {
        Some(current_page - 1)
    } else {
        None
    };
    spawn_image_slot(
        commands,
        container,
        SlotType::Prev,
        prev_page,
        reader_state,
        image_cache,
        asset_server,
        load_image_messages,
        true, // hidden
    );

    // Current slot（可见）
    spawn_image_slot(
        commands,
        container,
        SlotType::Current,
        Some(current_page),
        reader_state,
        image_cache,
        asset_server,
        load_image_messages,
        false,
    );

    // Next slot（隐藏）
    let next_page = if current_page + 1 < total {
        Some(current_page + 1)
    } else {
        None
    };
    spawn_image_slot(
        commands,
        container,
        SlotType::Next,
        next_page,
        reader_state,
        image_cache,
        asset_server,
        load_image_messages,
        true, // hidden
    );

    tracing::info!(
        "单页模式：创建三 slot, prev={:?}, current={}, next={:?}",
        prev_page,
        current_page,
        next_page
    );
}

/// 创建单个图片 slot 实体
fn spawn_image_slot(
    commands: &mut Commands,
    container: Entity,
    slot_type: SlotType,
    page_index: Option<usize>,
    reader_state: &ReaderState,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
    hidden: bool,
) {
    let display = if hidden { Display::None } else { Display::Flex };

    // 先 spawn slot 实体并设为 container 的子节点
    let slot_entity = commands
        .spawn((
            ImageSlot {
                slot_type,
                page_index,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display,
                ..default()
            },
        ))
        .id();
    commands.entity(container).add_child(slot_entity);

    // 如果有图片数据，加载图片到 slot 子节点
    if let Some(idx) = page_index
        && let Some(picture) = reader_state.pictures.get(idx)
    {
        let url = picture.media.url();
        load_image_for_slot(
            commands,
            slot_entity,
            &url,
            &picture.media.original_name,
            &reader_state.comic_title,
            reader_state.episode_order,
            idx,
            single_page_image_style(reader_state.scale),
            image_cache,
            asset_server,
            load_image_messages,
        );
    }
}

/// 单页模式翻页：旋转三个 slot
///
/// `direction`: 正数 = 下一页, 负数 = 上一页
fn rotate_slots(
    commands: &mut Commands,
    reader_state: &ReaderState,
    slot_query: &Query<(Entity, &ImageSlot, &Children)>,
    image_cache: &ImageCache,
    asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) {
    let current_page = reader_state.current_page;
    let total = reader_state.total_pages;

    // 收集现有 slot 信息
    let mut slots: Vec<(Entity, SlotType, Option<usize>)> = slot_query
        .iter()
        .map(|(e, slot, _)| (e, slot.slot_type, slot.page_index))
        .collect();

    // 按类型排序确保顺序一致
    slots.sort_by_key(|(_, t, _)| match t {
        SlotType::Prev => 0,
        SlotType::Current => 1,
        SlotType::Next => 2,
    });

    // 更新每个 slot 的角色和内容
    for (entity, old_type, old_page) in &slots {
        let (new_type, new_page) = match old_type {
            SlotType::Prev => {
                let prev = if current_page > 0 {
                    Some(current_page - 1)
                } else {
                    None
                };
                (SlotType::Prev, prev)
            }
            SlotType::Current => (SlotType::Current, Some(current_page)),
            SlotType::Next => {
                let next = if current_page + 1 < total {
                    Some(current_page + 1)
                } else {
                    None
                };
                (SlotType::Next, next)
            }
        };

        // 更新 slot 组件
        let is_hidden = new_type != SlotType::Current;
        commands.entity(*entity).insert((
            ImageSlot {
                slot_type: new_type,
                page_index: new_page,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: if is_hidden {
                    Display::None
                } else {
                    Display::Flex
                },
                ..default()
            },
        ));

        // 如果页码没变，不需要重新加载图片
        if new_page == *old_page {
            continue;
        }

        // 清空旧子节点
        // 获取 slot 的 children，通过 query 已有
        if let Ok((_, _, children)) = slot_query.get(*entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 加载新图片
        if let Some(idx) = new_page
            && let Some(picture) = reader_state.pictures.get(idx)
        {
            let url = picture.media.url();
            load_image_for_slot(
                commands,
                *entity,
                &url,
                &picture.media.original_name,
                &reader_state.comic_title,
                reader_state.episode_order,
                idx,
                single_page_image_style(reader_state.scale),
                image_cache,
                asset_server,
                load_image_messages,
            );
        }
    }
}

/// 更新单页模式三 slot（ReaderState 变化时调用）
pub fn update_single_page_slots(
    mut commands: Commands,
    reader_state: Res<ReaderState>,
    slot_query: Query<(Entity, &ImageSlot, &Children)>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
) {
    if !reader_state.is_changed()
        || reader_state.read_mode != ReadMode::SinglePage
        || reader_state.pictures.is_empty()
    {
        return;
    }

    // 如果没有 slot 存在（首次或模式切换后），不在此处创建
    if slot_query.is_empty() {
        return;
    }

    rotate_slots(
        &mut commands,
        &reader_state,
        &slot_query,
        &image_cache,
        &asset_server,
        &mut load_image_messages,
    );
}

// ==================== 条漫模式：窗口化加载 ====================

/// 创建条漫视图（初始化所有 slot，只加载窗口内的图片）
fn create_webtoon_view(
    commands: &mut Commands,
    container: Entity,
    reader_state: &ReaderState,
    image_cache: &ImageCache,
    _asset_server: &AssetServer,
    load_image_messages: &mut MessageWriter<LoadImageRequest>,
) {
    let total = reader_state.total_pages;
    if total == 0 {
        return;
    }

    let current = reader_state.current_page;

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
                // 只创建 5 个固定槽位，图片在槽位间流动
                for i in 0..consts::WEBTOON_SLOT_COUNT {
                    let page = if current >= 2 {
                        current - 2 + i
                    } else {
                        i.saturating_sub(2 - current)
                    };
                    let page_index = if page < total { Some(page) } else { None };

                    scroll.spawn((
                        WebtoonSlot {
                            slot_index: i,
                            page_index,
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
                    ));
                }
            });
    });

    // 初始加载前几页图片
    let load_end = consts::WEBTOON_SLOT_COUNT.min(total);
    for i in 0..load_end {
        let page = if current >= 2 { current - 2 + i } else { i };
        if page < total
            && let Some(picture) = reader_state.pictures.get(page)
        {
            let url = picture.media.url();
            if image_cache.get(&url).is_none()
                && !image_cache.is_loading(&url)
                && try_get_local_image_path(
                    &reader_state.comic_title,
                    reader_state.episode_order,
                    &picture.media.original_name,
                    page,
                )
                .is_none()
            {
                load_image_messages.write(LoadImageRequest { url });
            }
        }
    }

    tracing::info!(
        "条漫模式：创建 {} 个固定槽位，从第 {} 页开始",
        consts::WEBTOON_SLOT_COUNT,
        current + 1
    );
}

/// 条漫模式：根据滚动位置更新窗口内的图片加载/释放
pub fn update_webtoon_window(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    slot_query: Query<(Entity, &mut WebtoonSlot, &ComputedNode, Option<&ImageNode>)>,
    scroll_query: Query<(&ScrollPosition, &ComputedNode), With<WebtoonScrollContainer>>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    mut load_pictures_messages: MessageWriter<LoadPicturesRequest>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if reader_state.read_mode != ReadMode::Webtoon || reader_state.pictures.is_empty() {
        return;
    }

    let Ok((scroll_pos, scroll_computed)) = scroll_query.single() else {
        return;
    };

    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    let viewport_h = scroll_computed.size().y / scale_factor;

    // 收集槽位按 slot_index 排序
    let mut slots: Vec<(Entity, usize, Option<usize>, f32, bool)> = slot_query
        .iter()
        .map(|(e, slot, cn, img)| {
            (e, slot.slot_index, slot.page_index, cn.size().y / scale_factor, img.is_some())
        })
        .collect();
    slots.sort_by_key(|(_, idx, _, _, _)| *idx);

    // 计算当前可见页码（基于中间槽位的页码）
    let mid_slot = consts::WEBTOON_SLOT_COUNT / 2;
    let current_page = slots
        .get(mid_slot)
        .and_then(|(_, _, page, _, _)| *page)
        .unwrap_or(0);

    if reader_state.current_page != current_page {
        reader_state.current_page = current_page;
    }

    // 检查是否需要向下滚动回收（顶部槽位完全离开视口）
    let scroll_y = scroll_pos.y;
    let mut top_height = 0.0_f32;
    if let Some((_, _, _, h, _)) = slots.first() {
        top_height = if *h > 1.0 { *h } else { viewport_h };
    }

    if scroll_y > top_height && slots.len() == consts::WEBTOON_SLOT_COUNT {
        // 顶部槽位滚出视口，回收到底部
        if let Some(&(entity, _, _, _h, _)) = slots.first() {
            let last_page = slots.last().and_then(|(_, _, p, _, _)| *p).unwrap_or(0);
            let next_page = last_page + 1;

            if next_page < reader_state.total_pages {
                // 更新槽位页码
                commands.entity(entity).insert(WebtoonSlot {
                    slot_index: slots.last().map(|(_, i, _, _, _)| *i).unwrap_or(0) + 1,
                    page_index: Some(next_page),
                });
                // 移除旧图片，等待加载
                commands.entity(entity).remove::<ImageNode>();
                commands.entity(entity).insert(BackgroundColor(Color::srgb(0.05, 0.05, 0.08)));

                // 调整滚动位置补偿（减去回收槽位的高度）
                // 这样画面不会跳动
                // 注意：由于 Bevy 的 ScrollPosition 是 Bevy 管理的，我们不能直接修改
                // 改为：通过 DOM 顺序让 Bevy 自然处理
            }
        }
    }

    // 为每个槽位加载图片（如果还没有加载）
    for (entity, _, page_index, _, has_image) in &slots {
        let Some(page) = page_index else { continue };
        if *has_image { continue; } // 已有图片

        let Some(picture) = reader_state.pictures.get(*page) else { continue };
        let url = picture.media.url();

        // 本地文件优先
        if let Some(local_path) = try_get_local_image_path(
            &reader_state.comic_title,
            reader_state.episode_order,
            &picture.media.original_name,
            *page,
        ) {
            let handle: Handle<Image> = asset_server.load(local_path);
            commands.entity(*entity).remove::<BackgroundColor>();
            commands.entity(*entity).insert((
                ImageNode { image: handle, ..default() },
                Node { width: Val::Percent(100.0), height: Val::Auto, ..default() },
            ));
        } else if let Some(handle) = image_cache.get(&url) {
            commands.entity(*entity).remove::<BackgroundColor>();
            commands.entity(*entity).insert((
                ImageNode { image: handle.clone(), ..default() },
                Node { width: Val::Percent(100.0), height: Val::Auto, ..default() },
            ));
        } else if !image_cache.is_loading(&url) {
            load_image_messages.write(LoadImageRequest { url: url.clone() });
            commands.entity(*entity).insert(ReaderImageLoading { url });
        }
    }

    // 条漫模式：接近最后几页时预加载下一章
    if current_page + 3 >= reader_state.total_pages
        && !reader_state.is_loading
        && !reader_state.is_loading_next_chapter
    {
        let next_ep = reader_state.next_episode().cloned();
        if let Some(ep) = next_ep {
            reader_state.is_loading_next_chapter = true;
            load_pictures_messages.write(LoadPicturesRequest {
                comic_id: reader_state.comic_id.clone(),
                episode_order: ep.order,
                page: 1,
            });
            tracing::info!("条漫模式：预加载下一章 {}", ep.title);
        }
    }

    // 下一章图片就绪，追加到图片列表（槽位会自然加载）
    if !reader_state.next_chapter_pictures.is_empty()
        && current_page + 1 >= reader_state.total_pages
    {
        let next_pics = std::mem::take(&mut reader_state.next_chapter_pictures);
        let next_idx = reader_state.current_episode_idx + 1;
        let next_order = reader_state.episodes.get(next_idx).map(|e| e.order).unwrap_or(0);
        let new_count = next_pics.len();
        reader_state.pictures.extend(next_pics);
        reader_state.total_pages = reader_state.pictures.len();
        reader_state.current_episode_idx = next_idx;
        reader_state.episode_order = next_order;
        reader_state.is_loading_next_chapter = false;
        tracing::info!("条漫模式：衔接第 {} 章，+{} 页", next_order, new_count);
    }
}

/// 条漫模式：ImageCache 加载回调，为等待中的槽位填入图片
pub fn update_webtoon_images_from_cache(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    reader_state: Res<ReaderState>,
    loading_query: Query<(Entity, &WebtoonSlot, &ReaderImageLoading)>,
) {
    if reader_state.read_mode != ReadMode::Webtoon {
        return;
    }

    for (entity, _slot, loading) in loading_query.iter() {
        if let Some(handle) = image_cache.get(&loading.url) {
            commands.entity(entity).remove::<ReaderImageLoading>();
            commands.entity(entity).remove::<BackgroundColor>();
            commands.entity(entity).insert((
                ImageNode { image: handle.clone(), ..default() },
                Node { width: Val::Percent(100.0), height: Val::Auto, ..default() },
            ));
        }
    }
}

/// 条漫模式缩放更新
pub fn update_webtoon_scale(
    reader_state: Res<ReaderState>,
    mut webtoon_images_query: Query<(&mut Node, &WebtoonSlot, Option<&ImageNode>)>,
) {
    if !reader_state.is_changed() || reader_state.read_mode != ReadMode::Webtoon {
        return;
    }

    // 所有已加载图片的宽度设为 100%
    for (mut node, _slot, img) in webtoon_images_query.iter_mut() {
        if img.is_some() {
            node.width = Val::Percent(100.0);
            node.height = Val::Auto;
        }
    }
}

// ==================== 单页模式：缓存回调 ====================

/// 单页模式：检查 ImageCache 更新 slot 中的加载指示器
pub fn update_reader_image_from_cache(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    reader_state: Res<ReaderState>,
    image_loading_query: Query<(Entity, &ReaderImageLoading, &ChildOf)>,
) {
    if reader_state.read_mode != ReadMode::SinglePage {
        return;
    }

    for (entity, loading, child_of) in image_loading_query.iter() {
        if let Some(handle) = image_cache.get(&loading.url) {
            let parent = child_of.parent();

            // 移除加载指示器
            commands.entity(entity).despawn();

            // 添加图片到 slot
            commands.entity(parent).with_children(|p| {
                p.spawn((
                    ImageNode {
                        image: handle.clone(),
                        ..default()
                    },
                    single_page_image_style(reader_state.scale),
                ));
            });
        }
    }
}

// ==================== 翻页与导航 ====================

/// 翻页方向
enum PageDirection {
    Prev,
    Next,
}

/// 执行翻页操作（核心翻页逻辑）
///
/// 处理单页模式和条漫模式的翻页，支持跨章节切换
fn navigate_page(
    reader_state: &mut ResMut<ReaderState>,
    direction: PageDirection,
    load_messages: &mut MessageWriter<LoadPicturesRequest>,
) -> bool {
    if reader_state.pictures.is_empty() || reader_state.is_loading {
        return false;
    }

    match direction {
        PageDirection::Next => {
            if reader_state.current_page + 1 < reader_state.total_pages {
                // 同章节下一页
                reader_state.current_page += 1;
                true
            } else {
                // 章节末尾 → 尝试切换下一章
                try_switch_chapter(reader_state, true, load_messages)
            }
        }
        PageDirection::Prev => {
            if reader_state.current_page > 0 {
                // 同章节上一页
                reader_state.current_page -= 1;
                true
            } else {
                // 章节开头 → 尝试切换上一章
                try_switch_chapter(reader_state, false, load_messages)
            }
        }
    }
}

/// 尝试切换到下一/上一章
///
/// 返回 true 表示成功发起切换，false 表示没有更多章节
fn try_switch_chapter(
    reader_state: &mut ResMut<ReaderState>,
    is_next: bool,
    load_messages: &mut MessageWriter<LoadPicturesRequest>,
) -> bool {
    let target_idx = if is_next {
        if reader_state.current_episode_idx + 1 >= reader_state.episodes.len() {
            tracing::info!("已到最后一章，无法前进");
            return false;
        }
        reader_state.current_episode_idx + 1
    } else {
        if reader_state.current_episode_idx == 0 {
            tracing::info!("已到第一章，无法后退");
            return false;
        }
        reader_state.current_episode_idx - 1
    };

    let episode = reader_state.episodes[target_idx].clone();
    tracing::info!(
        "切换到{}: 第 {} 章 ({})",
        if is_next { "下一章" } else { "上一章" },
        episode.order,
        episode.title
    );

    // 更新状态
    reader_state.current_episode_idx = target_idx;
    reader_state.episode_order = episode.order;
    reader_state.pictures.clear();
    reader_state.total_pages = 0;
    reader_state.current_page = 0;
    reader_state.is_loading = true;
    reader_state.error = None;
    reader_state.next_chapter_pictures.clear();

    // 如果是上一章，从最后一页开始（标记，在 pictures 加载完后处理）
    // 这里先设置 current_page = usize::MAX 作为标记
    if !is_next {
        reader_state.current_page = usize::MAX; // 标记：加载完后跳到最后一页
    }

    // 发起图片加载
    load_messages.write(LoadPicturesRequest {
        comic_id: reader_state.comic_id.clone(),
        episode_order: episode.order,
        page: 1,
    });

    true
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
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderPrevButton>)>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            navigate_page(&mut reader_state, PageDirection::Prev, &mut load_messages);
        }
    }
}

/// 下一页按钮交互
pub fn reader_next_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReaderNextButton>)>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            navigate_page(&mut reader_state, PageDirection::Next, &mut load_messages);
        }
    }
}

/// 键盘控制
pub fn reader_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
    mut back_events: MessageWriter<NavigateBackEvent>,
) {
    // 左方向键 / A 键 - 上一页
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) || keyboard_input.just_pressed(KeyCode::KeyA)
    {
        navigate_page(&mut reader_state, PageDirection::Prev, &mut load_messages);
    }

    // 右方向键 / D 键 / 空格键 - 下一页
    if keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::KeyD)
        || keyboard_input.just_pressed(KeyCode::Space)
    {
        navigate_page(&mut reader_state, PageDirection::Next, &mut load_messages);
    }

    // Escape 键 - 返回
    if keyboard_input.just_pressed(KeyCode::Escape) {
        back_events.write(NavigateBackEvent);
    }
}

/// 鼠标滚轮控制（单页翻页 / 条漫滚动 / Ctrl+缩放）
pub fn reader_mouse_wheel_control(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    mut load_messages: MessageWriter<LoadPicturesRequest>,
    mut scale_text_query: Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
    mut webtoon_scroll_query: Query<&mut ScrollPosition, With<WebtoonScrollContainer>>,
) {
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    for event in mouse_wheel_events.read() {
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
                // 更新缩放文本
                for mut text in scale_text_query.iter_mut() {
                    **text = format!("{}%", (reader_state.scale * 100.0) as i32);
                }
            }
        } else {
            match reader_state.read_mode {
                ReadMode::SinglePage => {
                    if scroll_delta < 0.0 {
                        navigate_page(&mut reader_state, PageDirection::Next, &mut load_messages);
                    } else if scroll_delta > 0.0 {
                        navigate_page(&mut reader_state, PageDirection::Prev, &mut load_messages);
                    }
                }
                ReadMode::Webtoon => {
                    // 手动更新 ScrollPosition（MessageReader 已消费事件）
                    for mut scroll_pos in webtoon_scroll_query.iter_mut() {
                        let scroll_amount = -scroll_delta * consts::WEBTOON_SCROLL_SPEED;
                        scroll_pos.y = (scroll_pos.y + scroll_amount).max(0.0);
                    }
                }
            }
        }
    }
}

/// 键盘 +/- 缩放控制
pub fn reader_zoom_keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reader_state: ResMut<ReaderState>,
    mut scale_text_query: Query<&mut Text, (With<ReaderScaleText>, Without<ReaderPageText>)>,
) {
    let mut scale_changed = false;

    if keyboard_input.just_pressed(KeyCode::Equal)
        || keyboard_input.just_pressed(KeyCode::NumpadAdd)
    {
        reader_state.scale = (reader_state.scale + consts::SCALE_STEP).min(consts::MAX_SCALE);
        scale_changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::Minus)
        || keyboard_input.just_pressed(KeyCode::NumpadSubtract)
    {
        reader_state.scale = (reader_state.scale - consts::SCALE_STEP).max(consts::MIN_SCALE);
        scale_changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::Digit0) || keyboard_input.just_pressed(KeyCode::Numpad0)
    {
        reader_state.scale = 1.0;
        scale_changed = true;
    }

    if scale_changed {
        for mut text in scale_text_query.iter_mut() {
            **text = format!("{}%", (reader_state.scale * 100.0) as i32);
        }
    }
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
            reader_state.read_mode = match reader_state.read_mode {
                ReadMode::SinglePage => ReadMode::Webtoon,
                ReadMode::Webtoon => ReadMode::SinglePage,
            };

            let new_label = match reader_state.read_mode {
                ReadMode::SinglePage => "单页",
                ReadMode::Webtoon => "条漫",
            };

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

/// 处理阅读模式变化，重建图片视图
pub fn handle_read_mode_change(
    mut commands: Commands,
    reader_state: Res<ReaderState>,
    asset_server: Res<AssetServer>,
    image_cache: Res<ImageCache>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    slot_query: Query<Entity, With<ImageSlot>>,
    webtoon_container_query: Query<Entity, With<WebtoonScrollContainer>>,
    image_loading_query: Query<Entity, With<ReaderImageLoading>>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    mut previous_mode: Local<Option<ReadMode>>,
) {
    let current_mode = reader_state.read_mode;

    let mode_changed = previous_mode
        .map(|prev| prev != current_mode)
        .unwrap_or(false);
    *previous_mode = Some(current_mode);

    if !mode_changed || reader_state.pictures.is_empty() {
        return;
    }

    tracing::info!("阅读模式切换: {:?}", current_mode);

    let Ok(container) = container_query.single() else {
        return;
    };

    match current_mode {
        ReadMode::SinglePage => {
            // 清除条漫视图
            for entity in webtoon_container_query.iter() {
                commands.entity(entity).despawn();
            }

            // 创建三 slot
            if slot_query.is_empty() {
                create_single_page_slots(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
                );
            }
        }
        ReadMode::Webtoon => {
            // 清除单页 slot
            for entity in slot_query.iter() {
                commands.entity(entity).despawn();
            }
            // 清除残留的加载指示器
            for entity in image_loading_query.iter() {
                commands.entity(entity).despawn();
            }

            // 如果没有条漫容器，创建
            if webtoon_container_query.is_empty() {
                create_webtoon_view(
                    &mut commands,
                    container,
                    &reader_state,
                    &image_cache,
                    &asset_server,
                    &mut load_image_messages,
                );
            }
        }
    }
}

// ==================== 章节切换时重建视图 ====================

/// 处理章节切换后图片加载完成 —— 重建当前模式的视图
///
/// 监听 ReaderState 的 pictures 变化：
/// - 当 is_loading 从 true 变为 false 且 pictures 非空时触发
/// - 清除旧视图，根据当前模式重建
pub fn handle_chapter_switch(
    mut commands: Commands,
    mut reader_state: ResMut<ReaderState>,
    container_query: Query<Entity, With<ReaderImageContainer>>,
    slot_query: Query<Entity, With<ImageSlot>>,
    webtoon_container_query: Query<Entity, With<WebtoonScrollContainer>>,
    loading_indicator_query: Query<Entity, With<ReaderLoadingIndicator>>,
    error_text_query: Query<Entity, With<ReaderErrorText>>,
    image_cache: Res<ImageCache>,
    asset_server: Res<AssetServer>,
    mut load_image_messages: MessageWriter<LoadImageRequest>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
    mut episode_text_query: Query<&mut Text, (With<ReaderEpisodeText>, Without<ReaderPageText>)>,
    mut last_episode_order: Local<i32>,
) {
    if !reader_state.is_changed() {
        return;
    }

    // 检测章节切换：episode_order 变了且图片已加载
    let episode_changed = *last_episode_order != 0
        && *last_episode_order != reader_state.episode_order
        && !reader_state.pictures.is_empty()
        && !reader_state.is_loading;

    *last_episode_order = reader_state.episode_order;

    if !episode_changed {
        // 处理上一章跳转到末页的标记
        if reader_state.current_page == usize::MAX && !reader_state.pictures.is_empty() {
            reader_state.current_page = reader_state.total_pages.saturating_sub(1);
        }
        return;
    }

    // 处理上一章跳转到末页的标记
    if reader_state.current_page == usize::MAX {
        reader_state.current_page = reader_state.total_pages.saturating_sub(1);
    }

    tracing::info!(
        "章节切换完成: 第 {} 章, {} 张图片, 页码={}",
        reader_state.episode_order,
        reader_state.total_pages,
        reader_state.current_page
    );

    let Ok(container) = container_query.single() else {
        return;
    };

    // 清除旧视图
    for entity in slot_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in webtoon_container_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in loading_indicator_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in error_text_query.iter() {
        commands.entity(entity).despawn();
    }

    // 根据当前模式重建
    match reader_state.read_mode {
        ReadMode::SinglePage => {
            create_single_page_slots(
                &mut commands,
                container,
                &reader_state,
                &image_cache,
                &asset_server,
                &mut load_image_messages,
            );
        }
        ReadMode::Webtoon => {
            create_webtoon_view(
                &mut commands,
                container,
                &reader_state,
                &image_cache,
                &asset_server,
                &mut load_image_messages,
            );
        }
    }

    // 更新章节标题
    let episode_title = reader_state
        .episodes
        .get(reader_state.current_episode_idx)
        .map(|ep| ep.title.as_str())
        .unwrap_or("未知章节");
    for mut text in episode_text_query.iter_mut() {
        **text = format!("第 {} 章 - {}", reader_state.episode_order, episode_title);
    }

    // 更新页码
    update_page_text(&reader_state, &mut page_text_query);
}

// ==================== 页码更新 ====================

/// 更新底部栏页码/章节信息
pub fn update_page_info(
    reader_state: Res<ReaderState>,
    mut page_text_query: Query<&mut Text, With<ReaderPageText>>,
) {
    if !reader_state.is_changed() || reader_state.pictures.is_empty() {
        return;
    }

    update_page_text(&reader_state, &mut page_text_query);
}

/// 辅助函数：设置页码文本
fn update_page_text(
    reader_state: &ReaderState,
    page_text_query: &mut Query<&mut Text, With<ReaderPageText>>,
) {
    let display_page = reader_state.current_page + 1;
    let total = reader_state.total_pages;
    for mut text in page_text_query.iter_mut() {
        **text = format!("{} / {}", display_page, total);
    }
}

// ==================== 阅读历史保存 ====================

/// 自动保存阅读历史（监听 ReaderState 变化）
pub fn save_reading_history(
    reader_state: Res<ReaderState>,
    comic_detail_state: Res<ComicDetailState>,
    mut save_messages: MessageWriter<SaveHistoryRequest>,
    mut last_saved: Local<(i32, usize)>,
) {
    if !reader_state.is_changed() {
        return;
    }

    if reader_state.comic_id.is_empty() || reader_state.pictures.is_empty() {
        return;
    }

    // 跳过 usize::MAX 标记页（上一章切换中间状态）
    if reader_state.current_page == usize::MAX {
        return;
    }

    let current = (reader_state.episode_order, reader_state.current_page);
    if *last_saved == current {
        return;
    }
    *last_saved = current;

    // 获取漫画信息
    let Some(comic) = &comic_detail_state.comic else {
        return;
    };

    // 获取章节标题
    let eps_title = reader_state
        .episodes
        .get(reader_state.current_episode_idx)
        .map(|ep| ep.title.clone())
        .unwrap_or_else(|| format!("第{}章", reader_state.episode_order));

    // current_page 为 0-indexed，SaveHistoryRequest.last_page 为 1-indexed
    save_messages.write(SaveHistoryRequest {
        comic_id: reader_state.comic_id.clone(),
        comic_title: comic.title.clone(),
        thumb_url: comic.thumb.url(),
        last_eps_order: reader_state.episode_order,
        last_eps_title: eps_title,
        last_page: (reader_state.current_page + 1) as i32,
    });
}
