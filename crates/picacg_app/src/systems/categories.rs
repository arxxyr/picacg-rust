//! 分类浏览系统

use bevy::prelude::*;

use super::font_loader::get_font;
use crate::{
    components::*,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar, scrollbar_config::SCROLLBAR_WIDTH},
        waterfall::CategoriesCardCreationState,
        widgets::ButtonStyle,
    },
    utils::content_filter::CompiledFilter,
};

/// 分类卡片布局常量
mod category_layout {
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
    pub const PADDING_BOTTOM: f32 = 20.0;
}

/// 创建分类界面（如果已存在则只显示）
pub fn setup_categories_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    categories_state: Res<CategoriesState>,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut creation_state: ResMut<CategoriesCardCreationState>,
    mut existing_query: Query<&mut Node, With<CategoriesRoot>>,
) {
    // 如果 CategoriesRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        return;
    }

    // 清空之前的创建状态
    creation_state.clear();

    // 尝试找到 ContentArea
    let content_area = content_area_query.single().ok();

    // 创建分类内容
    let categories_root = commands
        .spawn_scene(categories_page(&categories_state))
        .id();

    // 如果有 ContentArea，将分类内容作为其子实体
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(categories_root);
    }

    // 注意：预创建由 waterfall_create_category_cards 的自动检测来启动
    // 不在这里启动，避免与延迟执行的 commands 冲突
}

/// 分类页面场景
fn categories_page(state: &CategoriesState) -> impl Scene + use<> {
    // 网格内边距（右侧额外让出滚动条宽度）
    let grid_padding = UiRect {
        left: Val::Px(category_layout::PADDING_LEFT),
        right: Val::Px(category_layout::PADDING_RIGHT),
        top: Val::Px(category_layout::PADDING_TOP),
        bottom: Val::Px(category_layout::PADDING_BOTTOM),
    };

    // 卡片通过瀑布式创建系统添加，这里只放错误信息 / 加载中占位
    let placeholder: Box<dyn SceneList> = match state.error.as_ref() {
        // 错误信息
        Some(error) => {
            let error_text = error.clone();
            Box::new(bsn_list![(
                ErrorMessage
                Text({error_text})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::ERROR)
            )])
        }
        // 加载中（categories 为空时显示）
        None if state.categories.is_empty() => Box::new(bsn_list![(
            LoadingIndicator
            Text("加载中...")
            TextFont { font_size: FontSize::Px(16.0) }
            TextColor(AppColors::TEXT)
        )]),
        None => Box::new(bsn_list![]),
    };

    bsn! {
        CategoriesRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 页面标题栏
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    border: UiRect::bottom(Val::Px(1.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        Text("分类浏览")
                        TextFont { font_size: FontSize::Px(20.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
            (
                // 滚动区域包装器（用于放置滚动条）
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
                        // 分类网格容器（可滚动）
                        #CategoriesScroll
                        CategoriesScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::FlexStart,
                            align_content: AlignContent::FlexStart,
                            padding: {grid_padding},
                            column_gap: Val::Px(category_layout::COLUMN_GAP),
                            row_gap: Val::Px(category_layout::ROW_GAP),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        ScrollPosition
                        Children [ {placeholder} ]
                    ),
                    // 创建滚动条
                    scrollbar(#CategoriesScroll),
                ]
            ),
        ]
    }
}

/// 分类缩略图场景（图片已缓存时使用）
fn category_image(url: String, handle: Handle<Image>) -> impl Scene + use<> {
    bsn! {
        CategoryImage { url: {url} }
        ImageNode { image: {handle} }
        Node {
            width: Val::Px(134.0),
            height: Val::Px(134.0),
        }
    }
}

/// 分类卡片场景（`hidden` 用于瀑布式预创建，先隐藏后分批显示）
fn category_card(
    category: &picacg_api::models::Category,
    image_cache: &ImageCache,
    hidden: bool,
) -> impl Scene + use<> {
    let card_title = category.title.clone();
    let title = category.title.clone();

    let visibility = if hidden {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    // 图片区域（已缓存直接显示，否则先放占位符）
    let thumb_url = category.thumb.url();
    let cover: Box<dyn SceneList> = match image_cache.get(&thumb_url) {
        Some(handle) => Box::new(bsn_list![category_image(thumb_url.clone(), handle.clone())]),
        // 占位符自带 URL，图片替换系统据此直接取缓存，无需反查分类列表
        None => {
            let placeholder_url = thumb_url.clone();
            Box::new(bsn_list![(
                PlaceholderImage
                CategoryImage { url: {placeholder_url} }
                Node {
                    width: Val::Px(134.0),
                    height: Val::Px(134.0),
                }
                BackgroundColor(AppColors::SURFACE_HOVER)
            )])
        }
    };

    bsn! {
        CategoryCard { title: {card_title} }
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Px(150.0),
            height: Val::Px(180.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
        }
        template_value(BorderColor::all(AppColors::BORDER))
        // 静息底色与 ButtonStyle::card() 的 None 态一致，避免首帧闪烁
        BackgroundColor(AppColors::SURFACE)
        template_value(visibility)
        Children [
            // 图片区域
            {cover},
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::top(Val::Px(8.0)) }
            ),
        ]
    }
}

/// 清理分类界面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_categories_ui(
    mut query: Query<&mut Node, With<CategoriesRoot>>,
    mut creation_state: ResMut<CategoriesCardCreationState>,
) {
    // 清空瀑布式创建状态（防止对已隐藏的 Entity 操作）
    creation_state.clear();

    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 刷新分类界面（只处理错误状态，卡片由瀑布式系统创建）
///
/// 注意：这个函数**不应该**在数据加载完成后重建整个
/// UI，否则会覆盖瀑布式系统创建的卡片。 它只在出现错误时重建 UI 显示错误信息。
pub fn refresh_categories_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    categories_state: Res<CategoriesState>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<CategoriesScrollContainer>>,
    error_query: Query<Entity, With<ErrorMessage>>,
) {
    // 只在状态变化时检查
    if !categories_state.is_changed() {
        return;
    }

    // 如果有错误，显示错误信息
    if let Some(ref error) = categories_state.error {
        // 如果还没有错误信息 UI，添加它
        if error_query.is_empty()
            && let Ok((container_entity, _)) = scroll_container_query.single()
        {
            let error_text = error.clone();
            commands
                .spawn_scene(bsn! {
                    ErrorMessage
                    Text({error_text})
                    TextFont { font_size: FontSize::Px(14.0) }
                    TextColor(AppColors::ERROR)
                })
                .insert(ChildOf(container_entity));
        }
    }

    // 如果数据存在或已有卡片，让瀑布式系统处理，不干涉
    // 数据为空且没有卡片则保持加载中状态
}

/// 瀑布式显示分类卡片（预创建所有隐藏卡片，然后分批显示）
#[allow(clippy::too_many_arguments)]
pub fn waterfall_create_category_cards(
    mut commands: Commands,
    mut creation_state: ResMut<CategoriesCardCreationState>,
    categories_state: Res<CategoriesState>,
    image_cache: Res<ImageCache>,
    scroll_container_query: Query<(Entity, Option<&Children>), With<CategoriesScrollContainer>>,
    card_query: Query<&CategoryCard>,
    loading_query: Query<Entity, With<LoadingIndicator>>,
    _time: Res<Time>,
    _asset_server: Res<AssetServer>,
) {
    // 如果数据已加载但 creation_state 未启动，主动启动预创建
    // （解决系统执行顺序导致 is_changed() 检测失败的问题）
    if !creation_state.is_creating
        && !categories_state.categories.is_empty()
        && categories_state.error.is_none()
    {
        // 检查当前容器中是否有卡片
        if let Ok((container_entity, children)) = scroll_container_query.single() {
            // 检查容器的子元素中是否有 CategoryCard
            let has_cards = children
                .map(|c| c.iter().any(|child| card_query.get(child).is_ok()))
                .unwrap_or(false);

            if !has_cards {
                // 惰性过滤：仅启动预创建时才用得上，避免每帧全量扫描
                let filtered_indices = CompiledFilter::from_settings()
                    .filter_category_indices(&categories_state.categories);

                if !filtered_indices.is_empty() {
                    // 删除"加载中..."指示器（安全删除，
                    // 实体可能已被其他系统删除）
                    for entity in loading_query.iter() {
                        if let Ok(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.despawn();
                        }
                    }
                    let font: Handle<Font> = get_font();
                    creation_state.start_precreate(filtered_indices.len(), font);
                    tracing::debug!(
                        "自动启动分类卡片预创建: {} 个（过滤后）",
                        filtered_indices.len()
                    );
                }
            }
            let _ = container_entity; // suppress warning
        }
    }

    // 检查是否需要预创建
    if creation_state.needs_precreate() {
        let Ok((container_entity, _)) = scroll_container_query.single() else {
            return;
        };

        // 字体句柄只作为"预创建已启动"的门闸，BSN 场景统一走默认字体句柄
        if creation_state.font_handle.is_none() {
            return;
        }

        let categories = &categories_state.categories;
        let count = creation_state.get_precreate_count();

        // 惰性过滤：预创建是单帧一次性事件，与上面的启动检测各算各的
        let filtered_indices = CompiledFilter::from_settings().filter_category_indices(categories);

        if filtered_indices.is_empty() || count == 0 {
            creation_state.clear();
            return;
        }

        // 分类数量少（~30个），直接全部创建为可见（不用瀑布流动画）
        for i in 0..count {
            if let Some(&original_index) = filtered_indices.get(i)
                && let Some(category) = categories.get(original_index)
            {
                commands
                    .spawn_scene(category_card(category, &image_cache, false))
                    .insert(ChildOf(container_entity));
            }
        }

        creation_state.clear();
        tracing::debug!("分类卡片直接创建完成: {} 个（过滤后）", count);
    }
}

/// 分类卡片交互系统（配色由全局 `apply_button_interaction` 统一处理）
pub fn category_card_interaction(
    interaction_query: Query<(&Interaction, &CategoryCard), Changed<Interaction>>,
    mut comics_list_state: ResMut<ComicsListState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut load_comics_messages: MessageWriter<LoadComicsRequest>,
) {
    for (interaction, card) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 设置当前分类并导航
        comics_list_state.category = card.title.clone();
        comics_list_state.page = 1;
        comics_list_state.comics.clear();

        next_route.set(AppRoute::ComicsList);

        // 触发加载漫画列表
        load_comics_messages.write(LoadComicsRequest {
            category: card.title.clone(),
            page: 1,
            sort: comics_list_state.sort.clone(),
        });
    }
}

/// 更新分类页面图片（当图片加载完成时）
///
/// 不使用 `is_changed()` 检查，因为系统执行顺序可能导致检测失败。已换成图片
/// 的实体不带 `PlaceholderImage`，加载失败的实体会被摘掉该标记，两者都自然
/// 退出扫描集。
pub fn update_categories_images(
    mut commands: Commands,
    image_cache: Res<ImageCache>,
    placeholder_query: Query<
        (Entity, &ChildOf, &CategoryImage),
        (With<PlaceholderImage>, Without<ImageNode>),
    >,
) {
    for (placeholder_entity, child_of, image) in placeholder_query.iter() {
        // 加载失败：摘掉占位标记，保留灰底方块，但不再每帧重扫
        if image_cache.is_failed(&image.url) {
            commands
                .entity(placeholder_entity)
                .remove::<PlaceholderImage>();
            continue;
        }

        // 检查图片是否已加载
        let Some(handle) = image_cache.get(&image.url) else {
            continue;
        };

        // 删除占位符，添加实际图片
        let parent_entity: Entity = child_of.parent();
        commands.entity(placeholder_entity).despawn();
        // 创建新的图片实体并插入到父卡片的第一个位置（在标题之前）
        let image_entity = commands
            .spawn_scene(category_image(image.url.clone(), handle.clone()))
            .id();
        commands
            .entity(parent_entity)
            .insert_children(0, &[image_entity]);
    }
}
