//! UI 插件
//!
//! 管理应用的用户界面

use bevy::prelude::*;
use picacg_config::AppSettings;

use crate::{
    components::*,
    events::*,
    resources::*,
    systems::*,
    utils::{
        i18n::I18n,
        text_input::{self, TextInputCursorBlink},
    },
};

/// UI 插件
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 从配置加载界面语言，初始化 I18n 资源
        let language = AppSettings::global().read().language;
        app
            // 注册状态
            .init_state::<AppRoute>()
            // 注册多语言资源
            .insert_resource(I18n::new(language))
            // 注册资源
            .init_resource::<AuthState>()
            .init_resource::<LoginFormState>()
            .init_resource::<CategoriesState>()
            .init_resource::<CachedTagsState>()
            .init_resource::<ComicsListState>()
            .init_resource::<ComicDetailState>()
            .init_resource::<ReaderState>()
            .init_resource::<ProxySettingsState>()
            .init_resource::<ImageCache>()
            .init_resource::<NavigationHistory>()
            .init_resource::<DownloadedComicsIndex>()
            .add_systems(
                Update,
                (
                    crate::systems::pagination::refresh_pagination_widgets,
                    crate::systems::scrollbar::update_scrollbar_thumb_colors,
                    crate::systems::widgets::apply_button_interaction,
                    // 封面下载角标：索引变化时全局刷新，卡片无需重建
                    crate::systems::ui_common::refresh_download_status_badges,
                    // 图片占位块的骨架屏微光（图片就位后自动退出查询）
                    crate::systems::ui_common::animate_loading_shimmer,
                    // 错误页重试按钮（games/fried/comments 共用，按路由分发）
                    crate::systems::ui_common::error_retry_button_interaction,
                    // 下载飞行动画（右键下载 → 圆点飞向侧边栏下载按钮）
                    crate::systems::ui_common::animate_download_fly,
                ),
            )
            .init_resource::<SearchState>()
            .init_resource::<RankingsState>()
            .init_resource::<RankingsCardCreationState>()
            .init_resource::<CategoriesCardCreationState>()
            .init_resource::<crate::systems::ComicsVirtualState>()
            .init_resource::<ComicsSelectionState>()
            .init_resource::<SearchCardCreationState>()
            .init_resource::<FavoritesState>()
            .init_resource::<FavoritesCardCreationState>()
            .init_resource::<HomeState>()
            .init_resource::<HomeCardCreationState>()
            .init_resource::<PunchInState>()
            .init_resource::<DownloadSectionCollapseState>()
            .init_resource::<DownloadsToastState>()
            .init_resource::<RegisterFormState>()
            .init_resource::<ForgotPasswordState>()
            .init_resource::<HistoryState>()
            .init_resource::<LikeRecordsState>()
            .init_resource::<CommentsState>()
            .init_resource::<UserProfileState>()
            .init_resource::<LocalReadState>()
            .init_resource::<GamesState>()
            .init_resource::<GameDetailState>()
            .init_resource::<FriedState>()
            .init_resource::<ImageConvertState>()
            .init_resource::<ImageConvertPickerResult>()
            .init_resource::<ImageConvertProgressResult>()
            .init_resource::<Waifu2xState>()
            .init_resource::<Waifu2xPickerResult>()
            .init_resource::<Waifu2xProgressResult>()
            .init_resource::<NasState>()
            .init_resource::<ChatState>()
            .init_resource::<ChatRoomState>()
            .init_resource::<NetworkDiagState>()
            .init_resource::<UpdateCheckState>()
            // 注册 UI 消息 (Bevy 0.17 使用 add_message)
            .add_message::<NavigateToCategoriesEvent>()
            .add_message::<NavigateToComicsListEvent>()
            .add_message::<NavigateToComicDetailEvent>()
            .add_message::<NavigateToReaderEvent>()
            .add_message::<NavigateToProxySettingsEvent>()
            .add_message::<NavigateBackEvent>()
            .add_message::<NavigateForwardEvent>()
            .add_message::<NavigateToLoginEvent>()
            .add_message::<NavigateToGameDetailEvent>()
            // 本地阅读消息
            .add_message::<ScanLocalComicsRequest>()
            .add_message::<ScanLocalComicsCompletedEvent>()
            .add_message::<ScanLocalComicsFailedEvent>()
            // 通用文本输入框系统（全局注册，所有页面共享）
            .init_resource::<TextInputCursorBlink>()
            .add_systems(
                Update,
                (
                    text_input::text_input_keyboard,
                    text_input::text_input_ime,
                    text_input::text_input_click_focus,
                    text_input::text_input_blur,
                    text_input::text_input_focus_visuals,
                    text_input::text_input_cursor_blink,
                ),
            )
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
                    login_sync_text_values,
                    login_keyboard_input,
                    login_checkbox_interaction,
                    register_button_interaction,
                    show_password_toggle_interaction,
                    update_login_error,
                    forgot_password_link_interaction,
                    login_focus_ring,
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
                    proxy_auth_toggle_interaction,
                    proxy_type_interaction,
                    proxy_sync_text_values,
                )
                    .run_if(in_state(AppRoute::ProxySettings)),
            )
            // 注册页面
            .add_systems(OnEnter(AppRoute::Register), setup_register_ui)
            .add_systems(OnExit(AppRoute::Register), cleanup_register_ui)
            .add_systems(
                Update,
                (
                    register_sync_text_values,
                    register_keyboard_input,
                    register_gender_interaction,
                    back_to_login_interaction,
                    register_submit_interaction,
                    handle_register_response,
                )
                    .run_if(in_state(AppRoute::Register)),
            )
            // 忘记密码页面
            .add_systems(OnEnter(AppRoute::ForgotPassword), setup_forgot_password_ui)
            .add_systems(OnExit(AppRoute::ForgotPassword), cleanup_forgot_password_ui)
            .add_systems(
                Update,
                (
                    forgot_password_sync_text_values,
                    forgot_password_keyboard_input,
                    forgot_password_submit_interaction,
                    forgot_password_back_interaction,
                    forgot_password_question_interaction,
                    handle_forgot_password_response,
                    handle_reset_password_response,
                    rebuild_forgot_password_ui,
                )
                    .run_if(in_state(AppRoute::ForgotPassword)),
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
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Categories)),
            )
            // 漫画列表页面
            .add_systems(
                OnEnter(AppRoute::ComicsList),
                (
                    ensure_main_layout,
                    setup_comics_list_ui,
                    trigger_load_comics,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppRoute::ComicsList), cleanup_comics_list_ui)
            .add_systems(
                Update,
                (
                    comic_card_interaction,
                    breadcrumb_back_to_categories,
                    auto_load_more_comics,
                    refresh_comics_list_ui,
                    // 顺序要紧：滚动系统排出待改绑清单，重绑系统紧接着消费
                    (comics_virtual_scroll, comics_rebind_cards).chain(),
                    update_comics_images,
                    // 批量选择
                    comics_select_mode_interaction,
                    comics_select_all_interaction,
                    comics_clear_selection_interaction,
                    comics_download_selected_interaction,
                    refresh_comics_selection_ui,
                )
                    .run_if(in_state(AppRoute::ComicsList)),
            )
            // 漫画详情页面
            .add_systems(
                OnEnter(AppRoute::ComicDetail),
                (
                    ensure_main_layout,
                    setup_detail_ui,
                    trigger_load_comic_detail,
                    trigger_load_comic_history,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppRoute::ComicDetail), cleanup_detail_ui)
            .add_systems(
                Update,
                (
                    detail_back_button_interaction,
                    author_button_interaction,
                    episode_card_interaction,
                    start_read_button_interaction,
                    like_button_interaction,
                    favorite_button_interaction,
                    download_button_interaction,
                    category_tag_interaction,
                    tag_button_interaction,
                    handle_comic_history_loaded,
                    refresh_detail_ui,
                    update_cover_image,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::ComicDetail)),
            )
            // 阅读器页面
            .add_systems(OnEnter(AppRoute::ReadView), setup_reader_ui)
            .add_systems(OnExit(AppRoute::ReadView), cleanup_reader_ui)
            .add_systems(
                Update,
                (
                    // 图片加载
                    trigger_load_pictures,
                    handle_pictures_loaded,
                    handle_all_pictures_loaded,
                    handle_pictures_load_failed,
                    // 单页模式
                    update_single_page_slots,
                    update_reader_image_from_cache,
                    // 条漫模式
                    // 顺序要紧：先按锚点定位滚动，再据当前页决定加载哪些图
                    (sync_webtoon_scroll, update_webtoon_window).chain(),
                    webtoon_failed_slot_retry,
                    webtoon_failed_chapters_retry,
                    update_webtoon_images_from_cache,
                    update_webtoon_scale,
                    // 交互（元组超过 20 项会撞上 bevy 的 tuple impl 上限，故分组嵌套）
                    (
                        reader_back_button_interaction,
                        reader_prev_button_interaction,
                        reader_next_button_interaction,
                        reader_keyboard_input,
                        reader_mouse_wheel_control,
                        reader_zoom_keyboard_control,
                        reader_zoom_button_interaction,
                        reader_mode_button_interaction,
                        handle_read_mode_change,
                    ),
                    // 章节切换
                    handle_chapter_switch,
                    // 页码更新
                    update_page_info,
                    // 历史保存
                    save_reading_history,
                )
                    .run_if(in_state(AppRoute::ReadView)),
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
                    download_path_keyboard_input,
                    sync_download_path_value,
                    download_path_picker_interaction,
                    handle_download_path_picker_result,
                    move_all_downloads_button_interaction,
                    clear_cache_button_interaction,
                    auto_save_settings,
                    update_settings_save_status,
                )
                    .run_if(in_state(AppRoute::Settings)),
            )
            .add_systems(
                Update,
                (
                    // 代理设置交互
                    proxy_enabled_checkbox_interaction,
                    proxy_auth_checkbox_interaction,
                    proxy_type_button_interaction,
                    proxy_input_keyboard,
                    sync_proxy_input_values,
                )
                    .run_if(in_state(AppRoute::Settings)),
            )
            .add_systems(
                Update,
                (
                    // 日志等级交互
                    log_level_button_interaction,
                    // 自动恢复下载交互
                    auto_resume_downloads_checkbox_interaction,
                    exit_after_downloads_checkbox_interaction,
                    // 性能追踪分组
                    perf_overlay_checkbox_interaction,
                    profiling_checkbox_interaction,
                    refresh_timings_button_interaction,
                    open_profiling_log_interaction,
                    // 最大并发下载数交互
                    max_concurrent_downloads_decrease_interaction,
                    max_concurrent_downloads_increase_interaction,
                    // CBZ 打包设置交互
                    auto_pack_cbz_checkbox_interaction,
                    delete_images_after_cbz_checkbox_interaction,
                    // 内容过滤交互
                    filter_by_category_checkbox_interaction,
                    filter_by_tag_checkbox_interaction,
                    filter_by_title_checkbox_interaction,
                    remove_keyword_interaction,
                    new_keyword_keyboard_input,
                    sync_keyword_input_value,
                    add_keyword_button_interaction,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Settings)),
            )
            .add_systems(
                Update,
                (
                    refresh_blocked_keywords_ui,
                    keyword_suggestion_toggle_interaction,
                    keyword_suggestion_item_interaction,
                    // 分流设置交互
                    api_channel_button_interaction,
                    image_channel_button_interaction,
                    custom_cdn_ip_keyboard_input,
                    sync_cdn_ip_input_values,
                    // 网络诊断交互
                    speed_test_button_interaction,
                    ping_test_button_interaction,
                    update_network_diag_result,
                )
                    .run_if(in_state(AppRoute::Settings)),
            )
            .add_systems(
                Update,
                (
                    // 高级设置交互
                    ui_scale_button_interaction,
                    sync_custom_font_path_value,
                    custom_font_path_picker_interaction,
                    handle_custom_font_path_picker_result,
                    sni_pretend_checkbox_interaction,
                    prefer_ipv6_checkbox_interaction,
                    // 主题/语言/关闭行为设置交互
                    theme_mode_button_interaction,
                    language_button_interaction,
                    close_behavior_button_interaction,
                    // 检查更新（此前 check_update_button_interaction 与
                    // refresh_update_status 从未注册，按钮点了没有任何反应）
                    (
                        check_update_button_interaction,
                        refresh_update_status,
                        open_release_page_interaction,
                        self_update_button_interaction,
                        auto_check_update_checkbox_interaction,
                    ),
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
                    open_cbz_folder_interaction,
                    completed_download_item_interaction,
                    section_header_collapse_interaction,
                    // 下载控制按钮交互
                    pause_download_button_interaction,
                    resume_download_button_interaction,
                    retry_download_button_interaction,
                    delete_download_button_interaction,
                    start_all_downloads_button_interaction,
                    start_all_header_button_interaction,
                    pause_all_header_button_interaction,
                    update_all_downloads_button_interaction,
                    // 已下载项按钮交互
                    redownload_button_interaction,
                    open_completed_folder_button_interaction,
                    move_completed_button_interaction,
                    // 更新结果反馈（Toast + 已下载列表项摘除）
                    handle_redownload_skipped,
                    remove_completed_item_on_redownload,
                    display_downloads_toast,
                    auto_hide_downloads_toast,
                )
                    .run_if(in_state(AppRoute::Downloads)),
            )
            .add_systems(
                Update,
                (
                    // 标题/分类/标签点击跳转
                    download_title_interaction,
                    download_category_interaction,
                    download_tag_interaction,
                    // 独立设置交互
                    task_settings_button_interaction,
                    task_path_select_interaction,
                    task_cbz_toggle_interaction,
                    refresh_downloads_ui,
                    update_download_stats,
                    add_new_task_ui,
                    handle_download_completed_ui,
                )
                    .run_if(in_state(AppRoute::Downloads)),
            )
            .add_systems(
                Update,
                (
                    move_task_between_sections,
                    update_download_titles,
                    update_download_task_tags,
                    delete_completed_download_interaction,
                    delete_files_checkbox_interaction,
                    confirm_delete_button_interaction,
                    cancel_delete_button_interaction,
                    // 浮动标题系统
                    update_floating_header,
                    floating_header_click_interaction,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Downloads)),
            )
            // 首页
            .add_systems(
                OnEnter(AppRoute::Home),
                (ensure_main_layout, setup_home_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Home), cleanup_home_ui)
            .add_systems(
                Update,
                (
                    home_card_interaction,
                    home_refresh_button_interaction,
                    waterfall_create_home_cards,
                    update_home_images,
                    handle_recommendations_loaded,
                    handle_recommendations_load_failed,
                    display_punch_in_toast,
                    auto_hide_punch_in_toast,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Home)),
            )
            // 搜索页
            .add_systems(
                OnEnter(AppRoute::Search),
                (ensure_main_layout, setup_search_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Search), cleanup_search_ui)
            .add_systems(
                Update,
                (
                    handle_search_keyboard_input,
                    search_button_interaction,
                    search_reset_button_interaction,
                    search_result_card_interaction,
                    search_pagination_changed.before(refresh_search_ui),
                    update_search_images,
                    refresh_search_ui,
                    waterfall_create_search_cards,
                    hot_keyword_tag_interaction,
                )
                    .run_if(in_state(AppRoute::Search)),
            )
            .add_systems(
                Update,
                (
                    // 过滤工具栏交互
                    sort_button_interaction,
                    category_filter_toggle_interaction,
                    category_checkbox_interaction,
                    select_all_categories_interaction,
                    clear_all_categories_interaction,
                    // 滚动条系统
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
                    refresh_knight_rankings_ui,
                    waterfall_create_cards,
                    update_rankings_images,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Rankings)),
            )
            // 游戏列表页
            .add_systems(
                OnEnter(AppRoute::Games),
                (ensure_main_layout, setup_games_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Games), cleanup_games_ui)
            .add_systems(
                Update,
                (
                    game_card_interaction,
                    games_pagination_changed.before(refresh_games_ui),
                    update_games_images,
                    refresh_games_ui,
                    handle_games_loaded,
                    handle_games_load_failed,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Games)),
            )
            // 游戏详情页
            .add_systems(
                OnEnter(AppRoute::GameDetail),
                (ensure_main_layout, setup_game_detail_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::GameDetail), cleanup_game_detail_ui)
            .add_systems(
                Update,
                (
                    game_detail_back_interaction,
                    refresh_game_detail_ui,
                    update_game_detail_images,
                    handle_game_detail_loaded,
                    handle_game_detail_load_failed,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::GameDetail)),
            )
            // 锅贴社区页
            .add_systems(
                OnEnter(AppRoute::Fried),
                (ensure_main_layout, setup_fried_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Fried), cleanup_fried_ui)
            .add_systems(
                Update,
                (
                    fried_refresh_interaction,
                    fried_pagination_changed.before(refresh_fried_ui),
                    refresh_fried_ui,
                    handle_apps_loaded,
                    handle_apps_load_failed,
                    handle_fried_posts_loaded,
                    handle_fried_posts_load_failed,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Fried)),
            )
            // 收藏页
            .add_systems(
                OnEnter(AppRoute::Favorites),
                (ensure_main_layout, setup_favorites_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Favorites), cleanup_favorites_ui)
            .add_systems(
                Update,
                (
                    favorite_card_interaction,
                    favorites_pagination_changed.before(refresh_favorites_ui),
                    waterfall_create_favorite_cards,
                    update_favorites_images,
                    refresh_favorites_ui,
                    handle_favorites_loaded,
                    handle_favorites_load_failed,
                )
                    .run_if(in_state(AppRoute::Favorites)),
            )
            // 阅读历史页
            .add_systems(
                OnEnter(AppRoute::History),
                (ensure_main_layout, setup_history_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::History), cleanup_history_ui)
            .add_systems(
                Update,
                (
                    history_card_interaction,
                    history_delete_interaction,
                    clear_all_history_interaction,
                    update_history_images,
                    refresh_history_ui,
                    handle_history_loaded,
                    handle_history_load_failed,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::History)),
            )
            // 点赞记录页
            .add_systems(
                OnEnter(AppRoute::LikeRecords),
                (ensure_main_layout, setup_like_records_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::LikeRecords), cleanup_like_records_ui)
            .add_systems(
                Update,
                (
                    like_record_card_interaction,
                    like_record_delete_interaction,
                    update_like_records_images,
                    refresh_like_records_ui,
                    handle_like_records_loaded,
                    handle_like_records_load_failed,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::LikeRecords)),
            )
            // 个人资料页面
            .add_systems(
                OnEnter(AppRoute::Profile),
                (ensure_main_layout, setup_profile_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Profile), cleanup_profile_ui)
            .add_systems(
                Update,
                (
                    profile_refresh_interaction,
                    profile_punch_in_interaction,
                    refresh_profile_ui,
                    update_profile_avatar,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Profile)),
            )
            // 评论页面
            .add_systems(
                OnEnter(AppRoute::Comments),
                (ensure_main_layout, setup_comments_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Comments), cleanup_comments_ui)
            .add_systems(
                Update,
                (
                    comments_back_interaction,
                    comment_like_interaction,
                    comment_reply_interaction,
                    cancel_reply_interaction,
                    expand_children_interaction,
                    load_more_children_interaction,
                    comment_send_interaction,
                    comment_input_action_keys,
                    update_comment_send_enabled,
                    comments_pagination_changed.before(refresh_comments_ui),
                )
                    .run_if(in_state(AppRoute::Comments)),
            )
            .add_systems(
                Update,
                (
                    refresh_comments_ui,
                    handle_comments_loaded,
                    handle_child_comments_loaded,
                    handle_post_comment_response,
                    handle_like_comment_response,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Comments)),
            )
            // 本地阅读页面
            .add_systems(
                OnEnter(AppRoute::LocalRead),
                (ensure_main_layout, setup_local_read_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::LocalRead), cleanup_local_read_ui)
            .add_systems(
                Update,
                (
                    local_read_scan_button_interaction,
                    open_local_folder_interaction,
                    handle_scan_local_comics,
                    handle_scan_completed,
                    handle_scan_failed,
                    refresh_local_read_ui,
                    update_local_cover_images,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::LocalRead)),
            )
            // 图片格式转换页面
            .add_systems(
                OnEnter(AppRoute::ImageConvert),
                (ensure_main_layout, setup_image_convert_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::ImageConvert), cleanup_image_convert_ui)
            .add_systems(
                Update,
                (
                    select_source_dir_interaction,
                    handle_source_dir_picker_result,
                    target_format_button_interaction,
                    refresh_format_buttons,
                    start_convert_interaction,
                    refresh_convert_progress,
                )
                    .run_if(in_state(AppRoute::ImageConvert)),
            )
            // Waifu2x 超分辨率页面
            .add_systems(
                OnEnter(AppRoute::Waifu2x),
                (ensure_main_layout, setup_waifu2x_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Waifu2x), cleanup_waifu2x_ui)
            .add_systems(
                Update,
                (
                    waifu2x_select_exe_interaction,
                    waifu2x_select_input_dir_interaction,
                    waifu2x_select_output_dir_interaction,
                    handle_waifu2x_picker_result,
                    waifu2x_scale_interaction,
                    waifu2x_noise_interaction,
                    waifu2x_gpu_interaction,
                    waifu2x_format_interaction,
                    refresh_waifu2x_option_buttons,
                    waifu2x_start_interaction,
                    refresh_waifu2x_progress,
                )
                    .run_if(in_state(AppRoute::Waifu2x)),
            )
            // NAS 远程存储页面
            .add_systems(
                OnEnter(AppRoute::Nas),
                (ensure_main_layout, setup_nas_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Nas), cleanup_nas_ui)
            .add_systems(
                Update,
                (
                    nas_input_interaction,
                    sync_nas_input_values,
                    nas_enabled_checkbox_interaction,
                    nas_test_connection_interaction,
                    nas_upload_button_interaction,
                    nas_browse_button_interaction,
                    auto_save_nas_settings,
                    handle_nas_test_connection,
                    handle_nas_test_response,
                    handle_nas_upload_request,
                    handle_nas_upload_progress,
                    handle_nas_upload_completed,
                )
                    .run_if(in_state(AppRoute::Nas)),
            )
            .add_systems(
                Update,
                (
                    handle_nas_upload_failed,
                    handle_nas_browse_request,
                    handle_nas_browse_response,
                    handle_nas_browse_failed,
                    refresh_nas_status_ui,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Nas)),
            )
            // 聊天大厅页面
            .add_systems(
                OnEnter(AppRoute::Chat),
                (ensure_main_layout, setup_chat_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::Chat), cleanup_chat_ui)
            .add_systems(
                Update,
                (
                    chat_room_card_interaction,
                    chat_refresh_interaction,
                    refresh_chat_ui,
                    handle_chat_rooms_loaded,
                    update_chat_room_icons,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::Chat)),
            )
            // 聊天室页面
            .add_systems(
                OnEnter(AppRoute::ChatRoom),
                (ensure_main_layout, setup_chat_room_ui).chain(),
            )
            .add_systems(OnExit(AppRoute::ChatRoom), cleanup_chat_room_ui)
            .add_systems(
                Update,
                (
                    poll_chat_messages,
                    rebuild_chat_messages_ui,
                    update_connection_status,
                    auto_scroll_chat,
                    chat_room_back_interaction,
                    chat_room_send_interaction,
                    chat_room_input_action_keys,
                    handle_send_chat_message_response,
                    // 滚动条系统
                )
                    .run_if(in_state(AppRoute::ChatRoom)),
            )
            // 侧边栏交互（在主布局存在时运行）
            .add_systems(
                Update,
                (
                    sidebar_button_interaction,
                    update_sidebar_active_state,
                    auto_load_user_profile,
                    handle_profile_loaded,
                    update_sidebar_avatar_url,
                    update_sidebar_avatar_image,
                    update_download_count_badge,
                )
                    .run_if(any_with_component::<MainLayoutRoot>),
            )
            // 全局右键菜单系统（所有带 ContextMenuTarget 的漫画卡片通用）
            .add_systems(
                Update,
                (
                    comic_card_context_menu,
                    comic_context_menu_interaction,
                    dismiss_context_menu,
                )
                    .run_if(any_with_component::<MainLayoutRoot>),
            )
            // 全局滚轮分发（根据鼠标悬停位置分发滚动事件到对应容器）
            // 全局导航（handle_back_navigation 处理键盘输入，handle_navigation_messages
            // 处理导航消息，track_route_changes 追踪路由变化）
            .add_systems(
                Update,
                (
                    handle_back_navigation,
                    // 有导航消息才运行：该系统独占 4 个大状态资源（漫画列表/详情/
                    // 阅读器/游戏详情），空闲帧不启动即不阻塞这些资源的并行读取
                    handle_navigation_messages.run_if(
                        on_message::<NavigateToCategoriesEvent>
                            .or_else(on_message::<NavigateToComicsListEvent>)
                            .or_else(on_message::<NavigateToComicDetailEvent>)
                            .or_else(on_message::<NavigateToReaderEvent>)
                            .or_else(on_message::<NavigateToProxySettingsEvent>)
                            .or_else(on_message::<NavigateBackEvent>)
                            .or_else(on_message::<NavigateForwardEvent>)
                            .or_else(on_message::<NavigateToLoginEvent>)
                            .or_else(on_message::<NavigateToGameDetailEvent>),
                    ),
                    track_route_changes,
                ),
            )
            // 全局窗口管理系统
            .init_resource::<WindowPositionSaveTimer>()
            .init_resource::<ExplicitWindowClose>()
            .add_systems(
                Update,
                (
                    handle_window_close,
                    save_window_position,
                    ensure_primary_window,
                    // 首启按屏幕 78% 设定窗口并居中（等 Monitor 信息就位）
                    apply_first_launch_geometry,
                ),
            );
    }
}

/// 确保主布局存在（如果不存在则创建）
fn ensure_main_layout(
    commands: Commands,
    asset_server: Res<AssetServer>,
    i18n: Res<I18n>,
    main_layout_query: Query<Entity, With<MainLayoutRoot>>,
) {
    // 如果主布局已存在，跳过
    if !main_layout_query.is_empty() {
        return;
    }

    // 创建主布局
    setup_main_layout(commands, asset_server, i18n);
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

/// 触发加载漫画列表（进入漫画列表页面时）
fn trigger_load_comics(
    comics_state: Res<ComicsListState>,
    mut load_messages: MessageWriter<LoadComicsRequest>,
) {
    // 如果有分类且没有数据，触发加载
    if !comics_state.category.is_empty()
        && comics_state.comics.is_empty()
        && !comics_state.is_loading
    {
        tracing::info!(
            "触发加载漫画列表: category={}, page={}",
            comics_state.category,
            comics_state.page
        );
        load_messages.write(LoadComicsRequest {
            category: comics_state.category.clone(),
            page: comics_state.page,
            sort: comics_state.sort.clone(),
        });
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
