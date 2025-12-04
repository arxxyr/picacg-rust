//! UI 插件
//!
//! 管理应用的用户界面

use bevy::prelude::*;

use crate::{components::*, events::*, resources::*, systems::*};

/// UI 插件
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册状态
            .init_state::<AppRoute>()
            // 注册资源
            .init_resource::<AuthState>()
            .init_resource::<LoginFormState>()
            .init_resource::<CategoriesState>()
            .init_resource::<ComicsListState>()
            .init_resource::<ComicDetailState>()
            .init_resource::<ReaderState>()
            .init_resource::<ProxySettingsState>()
            .init_resource::<GlobalMessageState>()
            .init_resource::<ImageCache>()
            .init_resource::<NavigationHistory>()
            .init_resource::<AppFont>()
            .init_resource::<ScrollbarDragState>()
            .init_resource::<SearchState>()
            .init_resource::<RankingsState>()
            .init_resource::<RankingsCardCreationState>()
            .init_resource::<CategoriesCardCreationState>()
            .init_resource::<ComicsCardCreationState>()
            .init_resource::<SearchCardCreationState>()
            // 注册 UI 消息 (Bevy 0.17 使用 add_message)
            .add_message::<NavigateToCategoriesEvent>()
            .add_message::<NavigateToComicsListEvent>()
            .add_message::<NavigateToComicDetailEvent>()
            .add_message::<NavigateToReaderEvent>()
            .add_message::<NavigateToProxySettingsEvent>()
            .add_message::<NavigateBackEvent>()
            .add_message::<NavigateForwardEvent>()
            .add_message::<NavigateToLoginEvent>()
            .add_message::<PrevPageEvent>()
            .add_message::<NextPageEvent>()
            .add_message::<ShowErrorEvent>()
            .add_message::<ShowSuccessEvent>()
            // 启动系统 (字体在 PreStartup 加载确保先于 UI)
            .add_systems(PreStartup, setup_fonts)
            .add_systems(Startup, setup_camera)
            // 登录页面
            .add_systems(OnEnter(AppRoute::Login), setup_login_ui)
            .add_systems(OnExit(AppRoute::Login), cleanup_login_ui)
            .add_systems(
                Update,
                (
                    login_button_interaction,
                    proxy_settings_button_interaction,
                    login_input_interaction,
                    login_keyboard_input,
                    login_checkbox_interaction,
                )
                    .run_if(in_state(AppRoute::Login)),
            )
            // 代理设置页面
            .add_systems(OnEnter(AppRoute::ProxySettings), setup_proxy_settings_ui)
            .add_systems(OnExit(AppRoute::ProxySettings), cleanup_proxy_settings_ui)
            .add_systems(
                Update,
                (
                    back_button_interaction,
                    save_button_interaction,
                    proxy_toggle_interaction,
                    proxy_type_interaction,
                    proxy_input_interaction,
                    proxy_keyboard_input,
                )
                    .run_if(in_state(AppRoute::ProxySettings)),
            )
            // 分类页面（进入时先确保主布局存在）
            .add_systems(
                OnEnter(AppRoute::Categories),
                (
                    ensure_main_layout,
                    setup_categories_ui,
                    trigger_load_categories,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppRoute::Categories), cleanup_categories_ui)
            .add_systems(
                Update,
                (
                    category_card_interaction,
                    refresh_categories_ui,
                    waterfall_create_category_cards,
                    update_categories_images,
                    handle_categories_scroll,
                    clamp_categories_scroll,
                    // 滚动条系统
                    update_all_scrollbar_thumbs,
                    scrollbar_thumb_interaction,
                    scrollbar_track_click,
                    scrollbar_thumb_drag,
                    reset_drag_state_on_release,
                )
                    .run_if(in_state(AppRoute::Categories)),
            )
            // 分类页面内容尺寸更新
            .add_systems(
                Update,
                update_categories_content_size.run_if(in_state(AppRoute::Categories)),
            )
            // 漫画列表页面
            .add_systems(
                OnEnter(AppRoute::ComicsList),
                (ensure_main_layout, setup_comics_list_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::ComicsList), cleanup_comics_list_ui)
            .add_systems(
                Update,
                (
                    comic_card_interaction,
                    pagination_interaction,
                    refresh_comics_list_ui,
                    waterfall_create_comic_cards,
                    update_comics_images,
                    handle_comics_scroll,
                    clamp_comics_scroll,
                    // 滚动条系统
                    update_all_scrollbar_thumbs,
                    scrollbar_thumb_interaction,
                    scrollbar_track_click,
                    scrollbar_thumb_drag,
                    reset_drag_state_on_release,
                )
                    .run_if(in_state(AppRoute::ComicsList)),
            )
            // 漫画列表内容尺寸更新
            .add_systems(
                Update,
                update_comics_content_size.run_if(in_state(AppRoute::ComicsList)),
            )
            // 漫画详情页面
            .add_systems(
                OnEnter(AppRoute::ComicDetail),
                (
                    ensure_main_layout,
                    setup_detail_ui,
                    trigger_load_comic_detail,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppRoute::ComicDetail), cleanup_detail_ui)
            .add_systems(
                Update,
                (
                    episode_card_interaction,
                    start_read_button_interaction,
                    like_button_interaction,
                    favorite_button_interaction,
                    download_button_interaction,
                    refresh_detail_ui,
                    update_cover_image,
                    handle_detail_scroll,
                    clamp_detail_scroll,
                    // 滚动条系统
                    update_all_scrollbar_thumbs,
                    scrollbar_thumb_interaction,
                    scrollbar_track_click,
                    scrollbar_thumb_drag,
                    reset_drag_state_on_release,
                )
                    .run_if(in_state(AppRoute::ComicDetail)),
            )
            // 设置页面
            .add_systems(
                OnEnter(AppRoute::Settings),
                (ensure_main_layout, setup_settings_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Settings), cleanup_settings_ui)
            .add_systems(
                Update,
                (
                    download_path_input_interaction,
                    download_path_keyboard_input,
                    update_download_path_display,
                    clear_cache_button_interaction,
                    save_settings_button_interaction,
                    handle_settings_scroll,
                    clamp_settings_scroll,
                    update_settings_content_size,
                    // 代理设置交互
                    proxy_enabled_checkbox_interaction,
                    proxy_type_button_interaction,
                    proxy_host_input_interaction,
                    proxy_port_input_interaction,
                    proxy_input_keyboard,
                    // 日志等级交互
                    log_level_button_interaction,
                    // 滚动条系统
                    update_all_scrollbar_thumbs,
                    scrollbar_thumb_interaction,
                    scrollbar_track_click,
                    scrollbar_thumb_drag,
                    reset_drag_state_on_release,
                )
                    .run_if(in_state(AppRoute::Settings)),
            )
            // 下载管理页面
            .add_systems(
                OnEnter(AppRoute::Downloads),
                (
                    ensure_main_layout,
                    load_incomplete_downloads,
                    setup_downloads_ui,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppRoute::Downloads), cleanup_downloads_ui)
            .add_systems(
                Update,
                (
                    open_download_folder_interaction,
                    completed_download_item_interaction,
                    // 下载控制按钮交互
                    pause_download_button_interaction,
                    resume_download_button_interaction,
                    delete_download_button_interaction,
                    // 已下载项按钮交互
                    redownload_button_interaction,
                    open_completed_folder_button_interaction,
                    refresh_downloads_ui,
                    add_new_task_ui,
                    handle_download_completed_ui,
                    update_download_titles,
                    handle_downloads_scroll,
                    update_downloads_content_size,
                    // 滚动条系统
                    update_all_scrollbar_thumbs,
                    scrollbar_thumb_interaction,
                    scrollbar_track_click,
                    scrollbar_thumb_drag,
                    reset_drag_state_on_release,
                )
                    .run_if(in_state(AppRoute::Downloads)),
            )
            // 首页（占位）
            .add_systems(
                OnEnter(AppRoute::Home),
                (ensure_main_layout, setup_home_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Home), cleanup_home_ui)
            // 搜索页
            .add_systems(
                OnEnter(AppRoute::Search),
                (ensure_main_layout, setup_search_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Search), cleanup_search_ui)
            .add_systems(
                Update,
                (
                    search_input_interaction,
                    handle_search_keyboard_input,
                    handle_search_ime_input,
                    search_button_interaction,
                    search_result_card_interaction,
                    search_pagination_interaction,
                    handle_search_scroll,
                    update_search_content_size,
                    update_search_images,
                    refresh_search_ui,
                    waterfall_create_search_cards,
                    unfocus_search_input,
                    // 滚动条系统
                    update_all_scrollbar_thumbs,
                    scrollbar_thumb_interaction,
                    scrollbar_track_click,
                    scrollbar_thumb_drag,
                    reset_drag_state_on_release,
                )
                    .run_if(in_state(AppRoute::Search)),
            )
            // 排行榜页
            .add_systems(
                OnEnter(AppRoute::Rankings),
                (ensure_main_layout, setup_rankings_ui, trigger_load_rankings).chain(),
            )
            .add_systems(OnExit(AppRoute::Rankings), cleanup_rankings_ui)
            .add_systems(
                Update,
                (
                    rankings_tab_interaction,
                    rankings_card_interaction,
                    refresh_rankings_ui,
                    waterfall_create_cards,
                    update_rankings_images,
                    handle_rankings_scroll,
                    update_rankings_content_size,
                    // 滚动条系统
                    update_all_scrollbar_thumbs,
                    scrollbar_thumb_interaction,
                    scrollbar_track_click,
                    scrollbar_thumb_drag,
                    reset_drag_state_on_release,
                )
                    .run_if(in_state(AppRoute::Rankings)),
            )
            // 收藏页（占位）
            .add_systems(
                OnEnter(AppRoute::Favorites),
                (ensure_main_layout, setup_favorites_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Favorites), cleanup_favorites_ui)
            // 侧边栏交互（在主布局存在时运行）
            .add_systems(
                Update,
                (sidebar_button_interaction, update_sidebar_active_state)
                    .run_if(any_with_component::<MainLayoutRoot>),
            )
            // 全局导航（handle_back_navigation 处理键盘输入，handle_navigation_messages
            // 处理导航消息，track_route_changes 追踪路由变化）
            .add_systems(
                Update,
                (
                    handle_back_navigation,
                    handle_navigation_messages,
                    track_route_changes,
                ),
            );
    }
}

/// 确保主布局存在（如果不存在则创建）
fn ensure_main_layout(
    commands: Commands,
    asset_server: Res<AssetServer>,
    main_layout_query: Query<Entity, With<MainLayoutRoot>>,
) {
    // 如果主布局已存在，跳过
    if !main_layout_query.is_empty() {
        return;
    }

    // 创建主布局
    setup_main_layout(commands, asset_server);
}

/// 触发加载分类（进入分类页面时）
fn trigger_load_categories(
    categories_state: Res<CategoriesState>,
    mut load_messages: MessageWriter<LoadCategoriesRequest>,
) {
    // 如果数据还没有加载，发送加载请求
    // 预创建由 waterfall_create_category_cards 的自动检测来处理
    if categories_state.categories.is_empty() && !categories_state.is_loading {
        load_messages.write(LoadCategoriesRequest);
    }
}

/// 触发加载漫画详情（进入详情页面时）
fn trigger_load_comic_detail(
    detail_state: Res<ComicDetailState>,
    mut load_messages: MessageWriter<LoadComicDetailRequest>,
) {
    // 如果有漫画 ID 且没有数据，触发加载
    if !detail_state.comic_id.is_empty() && detail_state.comic.is_none() && !detail_state.is_loading
    {
        load_messages.write(LoadComicDetailRequest {
            comic_id: detail_state.comic_id.clone(),
        });
    }
}
