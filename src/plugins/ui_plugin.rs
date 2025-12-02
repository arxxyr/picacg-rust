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
            // 注册 UI 消息 (Bevy 0.17 使用 add_message)
            .add_message::<NavigateToCategoriesEvent>()
            .add_message::<NavigateToComicsListEvent>()
            .add_message::<NavigateToComicDetailEvent>()
            .add_message::<NavigateToReaderEvent>()
            .add_message::<NavigateToProxySettingsEvent>()
            .add_message::<NavigateBackEvent>()
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
            // 侧边栏交互（在主布局存在时运行）
            .add_systems(
                Update,
                (sidebar_button_interaction, update_sidebar_active_state)
                    .run_if(any_with_component::<MainLayoutRoot>),
            )
            // 全局导航
            .add_systems(Update, handle_back_navigation);
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
    // 如果分类列表为空，触发加载
    if categories_state.categories.is_empty() && !categories_state.is_loading {
        load_messages.write(LoadCategoriesRequest);
    }
}
