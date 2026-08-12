//! 收藏列表系统
//!
//! 实现我的收藏页面

use bevy::prelude::*;

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        pagination::{Pagination, PaginationControl, pagination_controls},
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        ui_common::{TagColor, comic_time_info, tag_badge},
        widgets::ButtonStyle,
    },
    utils::content_filter::CompiledFilter,
};

/// 收藏页面标记类型（用于分页组件的泛型参数）
pub struct FavoritesPage;

/// 收藏卡片布局常量
mod favorites_layout {
    /// 列间距
    pub const COLUMN_GAP: f32 = 15.0;
    /// 行间距
    pub const ROW_GAP: f32 = 15.0;
    /// 左内边距
    pub const PADDING_LEFT: f32 = 20.0;
    /// 右内边距（包含滚动条宽度）
    pub const PADDING_RIGHT: f32 = 20.0 + super::SCROLLBAR_WIDTH;
    /// 上内边距
    pub const PADDING_TOP: f32 = 20.0;
    /// 下内边距
    pub const PADDING_BOTTOM: f32 = 30.0;
}

/// 收藏页根标记
#[derive(Component, Default, Clone)]
pub struct FavoritesRoot;

/// 收藏滚动容器标记
#[derive(Component, Default, Clone)]
pub struct FavoritesScrollContainer;

/// 收藏卡片标记
#[derive(Component, Default, Clone)]
pub struct FavoriteCard {
    pub comic_id: String,
}

/// 收藏卡片缩略图标记（占位符与实际图片共用，`url` 供替换系统直接取用）
#[derive(Component, Default, Clone)]
pub struct FavoriteThumbnail {
    /// 图片 URL
    pub url: String,
}

/// 收藏空状态提示标记
#[derive(Component, Default, Clone)]
pub struct FavoritesEmptyHint;

/// 收藏卡片瀑布式创建状态
#[derive(Resource, Default)]
pub struct FavoritesCardCreationState {
    /// 是否正在创建
    pub is_creating: bool,
    /// 待创建的卡片总数
    pub total_cards: usize,
    /// 当前已显示的卡片数
    pub visible_count: usize,
    /// 每帧显示的卡片数
    pub cards_per_frame: usize,
    /// 字体句柄
    pub font: Option<Handle<Font>>,
}

impl FavoritesCardCreationState {
    /// 开始预创建模式
    pub fn start_precreate(&mut self, total: usize, font: Handle<Font>) {
        self.is_creating = true;
        self.total_cards = total;
        self.visible_count = 0;
        self.cards_per_frame = 3; // 每帧显示 3 个
        self.font = Some(font);
    }

    /// 清空状态
    pub fn clear(&mut self) {
        self.is_creating = false;
        self.total_cards = 0;
        self.visible_count = 0;
        self.font = None;
    }
}

/// 创建收藏列表界面（如果已存在则只显示）
pub fn setup_favorites_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    favorites_state: Res<FavoritesState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<FavoritesCardCreationState>,
    mut load_favorites_messages: MessageWriter<LoadFavoritesRequest>,
    mut existing_query: Query<&mut Node, With<FavoritesRoot>>,
) {
    // 如果 FavoritesRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        if favorites_state.comics.is_empty() && !favorites_state.is_loading {
            load_favorites_messages.write(LoadFavoritesRequest {
                page: favorites_state.page,
                sort: favorites_state.sort.clone(),
            });
        }
        return;
    }

    // 字体句柄只作为瀑布式预创建的启动门闸，BSN 场景统一走默认字体句柄
    let font: Handle<Font> = get_font();

    // 清空之前的创建状态
    creation_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    let favorites_root = commands.spawn_scene(favorites_page(&favorites_state)).id();

    // 如果有 ContentArea，将收藏列表作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(favorites_root);
    }

    // 如果收藏列表为空且没有在加载，发送加载请求
    if favorites_state.comics.is_empty() && !favorites_state.is_loading {
        load_favorites_messages.write(LoadFavoritesRequest {
            page: favorites_state.page,
            sort: favorites_state.sort.clone(),
        });
    } else if !favorites_state.comics.is_empty() && !favorites_state.is_loading {
        // 启动预创建模式
        creation_state.start_precreate(favorites_state.comics.len(), font);
    }

    tracing::info!("收藏页面 UI 已创建");
}

/// 收藏页面场景
fn favorites_page(state: &FavoritesState) -> impl Scene + use<> {
    let current_page = state.page.max(0) as u32;
    let total_pages = state.total_pages.max(0) as u32;
    // 网格内边距（右侧额外让出滚动条宽度）
    let grid_padding = UiRect {
        left: Val::Px(favorites_layout::PADDING_LEFT),
        right: Val::Px(favorites_layout::PADDING_RIGHT),
        top: Val::Px(favorites_layout::PADDING_TOP),
        bottom: Val::Px(favorites_layout::PADDING_BOTTOM),
    };

    // 网格初始占位内容：加载指示器 / 空状态提示 / 两者皆无
    let grid_placeholder: Box<dyn SceneList> = if state.is_loading {
        Box::new(bsn_list![(
            LoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT)
        )])
    } else if state.comics.is_empty() && state.error.is_none() {
        // 空状态提示（初始状态，数据加载后会被移除）
        Box::new(bsn_list![(
            FavoritesEmptyHint
            Text("暂无收藏，去添加一些喜欢的漫画吧~")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        Box::new(bsn_list![])
    };

    bsn! {
        FavoritesRoot
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
                        Text("我的收藏")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    )
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
                        // 收藏网格（可滚动）
                        #FavoritesScroll
                        FavoritesScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: {grid_padding},
                            column_gap: Val::Px(favorites_layout::COLUMN_GAP),
                            row_gap: Val::Px(favorites_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [ {grid_placeholder} ]
                    ),
                    // 创建滚动条
                    scrollbar(#FavoritesScroll),
                ]
            ),
            // 分页控件（使用通用分页组件）
            pagination_controls::<FavoritesPage>(current_page, total_pages),
        ]
    }
}

/// 收藏封面缩略图场景（图片已缓存时使用）
fn favorite_thumbnail(url: String, handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        FavoriteThumbnail { url: {url} }
        ImageNode { image: {handle} }
        Node {
            width: Val::Px(164.0),
            height: Val::Px(220.0),
        }
    }
}

/// 收藏卡片场景（`hidden` 用于瀑布式预创建，先隐藏后分批显示）
fn favorite_card(
    comic: &picacg_api::models::Comic,
    image_cache: &ImageCache,
    hidden: bool,
) -> impl Scene + use<> {
    let card_comic_id = comic.id.clone();
    let menu_comic_id = comic.id.clone();
    let menu_comic_title = comic.title.clone();
    let title = comic.title.clone();
    let author = comic.author.clone();

    let visibility = if hidden {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    // 封面图片（已缓存直接显示，否则先放占位符）
    let thumb_url = comic.thumb.url();
    let cover: Box<dyn SceneList> = match image_cache.get(&thumb_url) {
        Some(handle) => Box::new(bsn_list![favorite_thumbnail(
            thumb_url.clone(),
            handle.clone()
        )]),
        None => {
            // 占位符自带 URL：图片就绪时无需回查 FavoritesState
            let placeholder_url = thumb_url.clone();
            Box::new(bsn_list![(
                PlaceholderImage
                FavoriteThumbnail { url: {placeholder_url} }
                Node {
                    width: Val::Px(164.0),
                    height: Val::Px(220.0),
                }
                BackgroundColor(AppColors::SURFACE_HOVER)
            )])
        }
    };

    // 分类和标签容器（两者皆空时不创建）
    let tags_container: Box<dyn SceneList> =
        if !comic.categories.is_empty() || !comic.tags.is_empty() {
            // 分类（蓝色）
            let categories = comic
                .categories
                .iter()
                .take(2)
                .map(|category| tag_badge(category, TagColor::Category));
            // 标签（绿色）
            let tags = comic
                .tags
                .iter()
                .take(2)
                .map(|tag| tag_badge(tag, TagColor::Tag));
            let badges: Vec<_> = categories.chain(tags).collect();

            Box::new(bsn_list![(
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                }
                Children [ {badges} ]
            )])
        } else {
            Box::new(bsn_list![])
        };

    // 创建/更新时间
    let time_info = comic_time_info(comic.created_at.as_deref(), comic.updated_at.as_deref());

    bsn! {
        FavoriteCard { comic_id: {card_comic_id} }
        ContextMenuTarget { comic_id: {menu_comic_id}, comic_title: {menu_comic_title} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Px(180.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        BackgroundColor(AppColors::SURFACE)
        template_value(visibility)
        Children [
            // 封面图片
            {cover},
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    max_width: Val::Px(164.0),
                    overflow: Overflow::clip(),
                }
            ),
            (
                // 作者
                Text({author})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { margin: UiRect::bottom(Val::Px(4.0)) }
            ),
            // 分类和标签容器
            {tags_container},
            // 创建/更新时间
            {time_info},
        ]
    }
}

/// 清理收藏页面
pub fn cleanup_favorites_ui(
    mut query: Query<&mut Node, With<FavoritesRoot>>,
    mut creation_state: ResMut<FavoritesCardCreationState>,
) {
    creation_state.clear();

    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 收藏卡片交互系统（配色由 `apply_button_interaction` 统一接管）
pub fn favorite_card_interaction(
    interaction_query: Query<(&Interaction, &FavoriteCard), Changed<Interaction>>,
    mut detail_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // 通过导航消息跳转到详情页（保留导航历史）
            detail_messages.write(NavigateToComicDetailEvent {
                comic_id: card.comic_id.clone(),
            });
        }
    }
}

/// 消费分页控件状态变化（翻页行为已内联在控件观察者里）
pub fn favorites_pagination_changed(
    mut commands: Commands,
    pagination_query: Query<
        &Pagination,
        (With<PaginationControl<FavoritesPage>>, Changed<Pagination>),
    >,
    card_query: Query<Entity, With<FavoriteCard>>,
    mut favorites_state: ResMut<FavoritesState>,
    mut load_favorites_messages: MessageWriter<LoadFavoritesRequest>,
    mut creation_state: ResMut<FavoritesCardCreationState>,
    mut scroll_query: Query<&mut ScrollPosition, With<FavoritesScrollContainer>>,
) {
    let Ok(pagination) = pagination_query.single() else {
        return;
    };
    // 只响应真实翻页（total_pages 回写等非翻页变化在此被过滤）
    if pagination.current_page as i32 == favorites_state.page {
        return;
    }
    favorites_state.page = pagination.current_page as i32;

    // 删除所有旧卡片
    for entity in card_query.iter() {
        commands.entity(entity).despawn();
    }

    // 清除数据和状态
    favorites_state.comics.clear();
    favorites_state.is_loading = true;
    creation_state.clear();

    // 重置滚动位置
    for mut scroll_pos in scroll_query.iter_mut() {
        scroll_pos.y = 0.0;
    }

    // 发送加载请求
    load_favorites_messages.write(LoadFavoritesRequest {
        page: favorites_state.page,
        sort: favorites_state.sort.clone(),
    });

    tracing::debug!("切换到收藏第 {} 页", favorites_state.page);
}

/// 瀑布式创建收藏卡片
pub fn waterfall_create_favorite_cards(
    mut commands: Commands,
    mut creation_state: ResMut<FavoritesCardCreationState>,
    favorites_state: Res<FavoritesState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<FavoritesScrollContainer>>,
    card_query: Query<&FavoriteCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    empty_hint_query: Query<Entity, With<FavoritesEmptyHint>>,
    image_cache: Res<ImageCache>,
    _asset_server: Res<AssetServer>,
) {
    // 如果没有滚动容器，退出
    let Ok((scroll_entity, children)) = scroll_container_query.single() else {
        return;
    };

    // 自动检测：数据存在但没有卡片，启动预创建
    if !creation_state.is_creating
        && !favorites_state.comics.is_empty()
        && favorites_state.error.is_none()
    {
        let has_cards = children
            .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
            .unwrap_or(false);

        if !has_cards {
            // 惰性过滤：仅启动预创建时才用得上，避免每帧全量扫描
            let filter = CompiledFilter::from_settings();
            let filtered_count = favorites_state
                .comics
                .iter()
                .filter(|c| !filter.should_block_comic(c))
                .count();

            if filtered_count > 0 {
                // 删除加载指示器和空状态提示
                for entity in loading_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in empty_hint_query.iter() {
                    commands.entity(entity).despawn();
                }

                let font: Handle<Font> = get_font();
                creation_state.start_precreate(filtered_count, font);
            }
        }
    }

    // 如果不在创建模式，退出
    if !creation_state.is_creating {
        return;
    }

    // 字体句柄只作为"预创建已启动"的门闸，BSN 场景统一走默认字体句柄
    if creation_state.font.is_none() {
        return;
    }

    // 阶段1：预创建所有卡片（隐藏状态）
    let has_cards = children
        .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
        .unwrap_or(false);

    if !has_cards && creation_state.visible_count == 0 {
        // 惰性过滤：只有真正建卡这一帧才构建过滤器，与上面的启动检测各算各的
        let filter = CompiledFilter::from_settings();
        // 一次性创建所有卡片（隐藏），跳过被屏蔽的漫画
        for comic in favorites_state.comics.iter() {
            if !filter.should_block_comic(comic) {
                commands
                    .spawn_scene(favorite_card(comic, &image_cache, true))
                    .insert(ChildOf(scroll_entity));
            }
        }
        return;
    }

    // 阶段2：逐帧显示卡片
    if creation_state.visible_count < creation_state.total_cards {
        let cards_to_show = creation_state.cards_per_frame;
        let start = creation_state.visible_count;
        let end = (start + cards_to_show).min(creation_state.total_cards);

        if let Some(children) = children {
            let card_entities: Vec<Entity> = children
                .iter()
                .filter(|e| card_query.get(*e).is_ok())
                .collect();

            for i in start..end {
                if let Some(entity) = card_entities.get(i) {
                    commands.entity(*entity).insert(Visibility::Inherited);
                }
            }
        }

        creation_state.visible_count = end;

        if creation_state.visible_count >= creation_state.total_cards {
            creation_state.is_creating = false;
            tracing::debug!("收藏卡片瀑布式创建完成: {} 个", creation_state.total_cards);
        }
    }
}

/// 刷新收藏页面 UI（响应数据变化）：把页码/总页数回写进分页控件
pub fn refresh_favorites_ui(
    favorites_state: Res<FavoritesState>,
    mut pagination_query: Query<&mut Pagination, With<PaginationControl<FavoritesPage>>>,
) {
    if !favorites_state.is_changed() {
        return;
    }

    let target = Pagination {
        current_page: favorites_state.page.max(0) as u32,
        total_pages: favorites_state.total_pages.max(0) as u32,
    };
    for mut pagination in pagination_query.iter_mut() {
        // 比较后写入，避免 Changed 循环触发
        if *pagination != target {
            *pagination = target.clone();
        }
    }
}

/// 处理收藏数据加载完成
pub fn handle_favorites_loaded(
    mut favorites_state: ResMut<FavoritesState>,
    mut messages: MessageReader<FavoritesLoadedEvent>,
) {
    for event in messages.read() {
        favorites_state.comics = event.comics.clone();
        favorites_state.total_pages = event.total_pages;
        favorites_state.is_loading = false;
        favorites_state.error = None;
        tracing::info!(
            "收藏列表加载完成: {} 个, 共 {} 页",
            favorites_state.comics.len(),
            favorites_state.total_pages
        );
    }
}

/// 处理收藏数据加载失败
pub fn handle_favorites_load_failed(
    mut favorites_state: ResMut<FavoritesState>,
    mut messages: MessageReader<FavoritesLoadFailedEvent>,
) {
    for event in messages.read() {
        favorites_state.is_loading = false;
        favorites_state.error = Some(event.error.clone());
        tracing::warn!("收藏列表加载失败: {}", event.error);
    }
}

/// 更新收藏封面图片（当图片加载完成时替换占位符）
///
/// 扫描集只含"仍是占位符"的实体：已替换的带 `ImageNode`，加载失败的会被摘掉
/// `PlaceholderImage` 标记，两者都不再进入每帧遍历。
pub fn update_favorites_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    placeholder_query: Query<
        (Entity, &ChildOf, &FavoriteThumbnail),
        (With<PlaceholderImage>, Without<ImageNode>),
    >,
) {
    let mut replaced_count = 0;
    for (placeholder_entity, child_of, thumbnail) in placeholder_query.iter() {
        // 加载失败：摘掉占位标记（灰底保留），让它退出扫描集
        if image_cache.is_failed(&thumbnail.url) {
            commands
                .entity(placeholder_entity)
                .remove::<PlaceholderImage>();
            continue;
        }

        // 检查图片是否已加载
        let Some(handle) = image_cache.get(&thumbnail.url) else {
            continue;
        };

        // 删除占位符，添加实际图片
        let parent_entity: Entity = child_of.parent();
        commands.entity(placeholder_entity).despawn();
        let image_entity = commands
            .spawn_scene(favorite_thumbnail(thumbnail.url.clone(), handle.clone()))
            .id();

        // 插入到第一个位置（在标题之前）
        commands
            .entity(parent_entity)
            .insert_children(0, &[image_entity]);
        replaced_count += 1;
    }

    if replaced_count > 0 {
        tracing::trace!("[Favorites] 替换了 {} 个封面图片", replaced_count);
    }
}
