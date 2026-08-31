//! API 插件
//!
//! 处理与后端 API 的异步通信

use bevy::{asset::RenderAssetUsages, prelude::*};
use picacg_api::{ApiClient, apply_image_dns_override, transform_image_url};
use picacg_config::AppSettings;

use crate::{
    events::*,
    resources::{DownloadManagerState, DownloadTaskMeta, SharedTaskControl, *},
    systems::login::save_credentials_on_login,
    utils::TokioTasksRuntime,
};

/// API 插件
pub struct ApiPlugin;

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册 API 客户端资源
            .insert_resource(ApiClientResource::new())
            // 注册下载管理状态
            .init_resource::<DownloadManagerState>()
            // CBZ 打包在途计数（「下载完成后退出」依赖它等打包收尾）
            .init_resource::<CbzPackagingState>()
            // 启动时加载未完成的下载任务
            .add_systems(Startup, setup_download_manager)
            // 注册消息 (Bevy 0.17 使用 add_message)
            .add_message::<LoginRequestEvent>()
            .add_message::<LoginResponseEvent>()
            .add_message::<UserLoggedInEvent>()
            .add_message::<LoadCategoriesRequest>()
            .add_message::<CategoriesLoadedEvent>()
            .add_message::<CategoriesLoadFailedEvent>()
            .add_message::<LoadComicsRequest>()
            .add_message::<ComicsLoadedEvent>()
            .add_message::<ComicsLoadFailedEvent>()
            .add_message::<LoadComicDetailRequest>()
            .add_message::<ComicDetailLoadedEvent>()
            .add_message::<ComicDetailLoadFailedEvent>()
            .add_message::<LoadEpisodesRequest>()
            .add_message::<EpisodesLoadedEvent>()
            .add_message::<EpisodesLoadFailedEvent>()
            .add_message::<LoadPicturesRequest>()
            .add_message::<PicturesLoadedEvent>()
            .add_message::<PicturesLoadFailedEvent>()
            .add_message::<LoadAllChapterPicturesRequest>()
            .add_message::<AllChapterPicturesLoadedEvent>()
            .add_message::<LikeComicRequest>()
            .add_message::<LikeComicResponse>()
            .add_message::<FavoriteComicRequest>()
            .add_message::<FavoriteComicResponse>()
            .add_message::<LoadImageRequest>()
            .add_message::<ImageLoadedEvent>()
            .add_message::<ImageLoadFailedEvent>()
            .add_message::<PunchInRequestEvent>()
            .add_message::<PunchInResponseEvent>()
            // 下载相关消息
            .add_message::<DownloadComicRequest>()
            .add_message::<DownloadProgressEvent>()
            .add_message::<DownloadCompletedEvent>()
            .add_message::<DownloadFailedEvent>()
            .add_message::<DownloadPausedEvent>()
            .add_message::<ResumeDownloadRequest>()
            .add_message::<RedownloadRequest>()
            .add_message::<RedownloadConfirmed>()
            .add_message::<RedownloadSkipped>()
            // CBZ 打包相关消息
            .add_message::<CbzPackageRequest>()
            .add_message::<CbzPackageCompletedEvent>()
            .add_message::<CbzPackageFailedEvent>()
            // 搜索相关消息
            .add_message::<SearchComicsRequestEvent>()
            .add_message::<SearchResultsLoadedEvent>()
            .add_message::<SearchFailedEvent>()
            // 热词相关消息
            .add_message::<LoadKeywordsRequest>()
            .add_message::<KeywordsLoadedEvent>()
            .add_message::<KeywordsLoadFailedEvent>()
            // 排行榜相关消息
            .add_message::<LoadRankingsRequest>()
            .add_message::<RankingsLoadedEvent>()
            .add_message::<RankingsLoadFailedEvent>()
            // 骑士榜相关消息
            .add_message::<LoadKnightRankingsRequest>()
            .add_message::<KnightRankingsLoadedEvent>()
            .add_message::<KnightRankingsLoadFailedEvent>()
            // 收藏列表相关消息
            .add_message::<LoadFavoritesRequest>()
            .add_message::<FavoritesLoadedEvent>()
            .add_message::<FavoritesLoadFailedEvent>()
            // 首页相关消息
            .add_message::<LoadRecommendationsRequest>()
            .add_message::<RecommendationsLoadedEvent>()
            .add_message::<RecommendationsLoadFailedEvent>()
            // API 客户端重载消息
            .add_message::<ReloadApiClientEvent>()
            // 注册相关消息
            .add_message::<RegisterRequestEvent>()
            .add_message::<RegisterResponseEvent>()
            // 忘记/重置密码相关消息
            .add_message::<ForgotPasswordRequestEvent>()
            .add_message::<ForgotPasswordResponseEvent>()
            .add_message::<ResetPasswordRequestEvent>()
            .add_message::<ResetPasswordResponseEvent>()
            // 历史记录相关消息
            .add_message::<LoadHistoryRequest>()
            .add_message::<HistoryLoadedEvent>()
            .add_message::<HistoryLoadFailedEvent>()
            .add_message::<SaveHistoryRequest>()
            .add_message::<DeleteHistoryRequest>()
            .add_message::<ClearAllHistoryRequest>()
            // 点赞记录相关消息
            .add_message::<LoadLikeRecordsRequest>()
            .add_message::<LikeRecordsLoadedEvent>()
            .add_message::<LikeRecordsLoadFailedEvent>()
            .add_message::<SaveLikeRecordRequest>()
            .add_message::<DeleteLikeRecordRequest>()
            // 评论相关消息
            .add_message::<LoadCommentsRequest>()
            .add_message::<CommentsLoadedEvent>()
            .add_message::<CommentsLoadFailedEvent>()
            .add_message::<PostCommentRequest>()
            .add_message::<PostCommentResponseEvent>()
            .add_message::<PostCommentReplyRequest>()
            .add_message::<PostCommentReplyResponseEvent>()
            .add_message::<LikeCommentRequestEvent>()
            .add_message::<LikeCommentResponseEvent>()
            .add_message::<LoadChildCommentsRequest>()
            .add_message::<ChildCommentsLoadedEvent>()
            // 个人资料相关消息
            .add_message::<LoadUserProfileRequest>()
            .add_message::<UserProfileLoadedEvent>()
            .add_message::<UserProfileLoadFailedEvent>()
            // 版本更新检查消息
            .add_systems(Startup, auto_check_update_on_startup)
            .add_message::<CheckUpdateRequest>()
            .add_message::<CheckUpdateResponse>()
            .add_message::<CheckUpdateFailedEvent>()
            // 游戏相关消息
            .add_message::<LoadGamesRequest>()
            .add_message::<GamesLoadedEvent>()
            .add_message::<GamesLoadFailedEvent>()
            .add_message::<LoadGameDetailRequest>()
            .add_message::<GameDetailLoadedEvent>()
            .add_message::<GameDetailLoadFailedEvent>()
            // 网络诊断消息
            .add_message::<SpeedTestRequest>()
            .add_message::<SpeedTestResultEvent>()
            .add_message::<PingTestRequest>()
            .add_message::<PingTestResultEvent>()
            .add_message::<NetworkTestFailedEvent>()
            // 锅贴社区消息
            .add_message::<LoadAppsRequest>()
            .add_message::<AppsLoadedEvent>()
            .add_message::<AppsLoadFailedEvent>()
            .add_message::<LoadFriedPostsRequest>()
            .add_message::<FriedPostsLoadedEvent>()
            .add_message::<FriedPostsLoadFailedEvent>()
            // NAS 远程存储消息
            .add_message::<NasTestConnectionRequest>()
            .add_message::<NasTestConnectionResponse>()
            .add_message::<NasUploadRequest>()
            .add_message::<NasUploadProgressEvent>()
            .add_message::<NasUploadCompletedEvent>()
            .add_message::<NasUploadFailedEvent>()
            .add_message::<NasBrowseRequest>()
            .add_message::<NasBrowseResponse>()
            .add_message::<NasBrowseFailedEvent>()
            // 聊天室消息
            .add_message::<LoadChatRoomsRequest>()
            .add_message::<ChatRoomsLoadedEvent>()
            .add_message::<ChatRoomsLoadFailedEvent>()
            .add_message::<ConnectChatRoomRequest>()
            .add_message::<SendChatMessageRequest>()
            .add_message::<SendChatMessageResponse>()
            .add_message::<DisconnectChatRoomRequest>()
            // 注册系统 - 登录和分类
            .add_systems(
                Update,
                (
                    handle_login_request,
                    handle_login_response,
                    handle_load_categories,
                    handle_categories_response,
                    handle_load_comics,
                    handle_comics_response,
                    handle_load_comic_detail,
                    handle_comic_detail_response,
                ),
            )
            // 注册系统 - 章节、点赞、收藏
            .add_systems(
                Update,
                (
                    handle_load_episodes,
                    handle_episodes_response,
                    handle_load_pictures,
                    handle_load_all_chapter_pictures,
                    handle_like_comic,
                    handle_like_response,
                    handle_favorite_comic,
                    handle_favorite_response,
                    // 图片加载：先入队，再按节流处理
                    enqueue_image_requests,
                    process_image_queue,
                    handle_image_response,
                ),
            )
            // 注册系统 - 打卡和下载
            .add_systems(
                Update,
                (
                    handle_punch_in_request,
                    handle_punch_in_response,
                    handle_download_comic,
                    handle_download_progress,
                    handle_download_completed,
                    handle_download_failed,
                    handle_download_paused,
                    handle_resume_download,
                    // 前置检查（普通更新）与实际下载（强制更新 / 检查放行后）
                    handle_redownload_precheck,
                    handle_redownload,
                    download_queue_manager,
                    exit_after_downloads_complete,
                ),
            )
            // 注册系统 - 搜索
            .add_systems(Update, (handle_search_request, handle_search_response))
            // 注册系统 - 热词
            .add_systems(Update, (handle_load_keywords, handle_keywords_response))
            // 注册系统 - 排行榜
            .add_systems(Update, (handle_load_rankings, handle_rankings_response))
            // 注册系统 - 骑士榜
            .add_systems(
                Update,
                (handle_load_knight_rankings, handle_knight_rankings_response),
            )
            // 注册系统 - 收藏列表
            .add_systems(Update, (handle_load_favorites, handle_favorites_response))
            // 注册系统 - 首页推荐
            .add_systems(
                Update,
                (handle_load_recommendations, handle_recommendations_response),
            )
            // 注册系统 - 用户注册
            .add_systems(Update, handle_register_request)
            // 注册系统 - 忘记/重置密码
            .add_systems(
                Update,
                (
                    handle_forgot_password_request,
                    handle_reset_password_request,
                ),
            )
            // 注册系统 - CBZ 打包
            .add_systems(
                Update,
                (
                    handle_cbz_package_request,
                    handle_cbz_package_completed,
                    handle_cbz_package_failed,
                ),
            )
            // 注册系统 - 历史记录
            .add_systems(
                Update,
                (
                    handle_load_history,
                    handle_save_history,
                    handle_delete_history,
                    handle_clear_all_history,
                ),
            )
            // 注册系统 - 点赞记录
            .add_systems(
                Update,
                (
                    handle_load_like_records,
                    handle_save_like_record,
                    handle_delete_like_record,
                ),
            )
            // 注册系统 - 评论
            .add_systems(
                Update,
                (
                    handle_load_comments,
                    handle_post_comment,
                    handle_post_comment_reply,
                    handle_like_comment,
                    handle_load_child_comments,
                ),
            )
            // 注册系统 - 个人资料
            .add_systems(Update, handle_load_user_profile)
            // API 客户端重载（通道/代理变更）
            .add_systems(Update, handle_reload_api_client)
            // 自动收集标签到缓存
            .add_systems(Update, update_cached_tags)
            // 启动时自动登录系统
            .add_systems(Startup, auto_login_on_startup)
            // 检查自动登录计时器（在 Update 中运行）
            .add_systems(Update, check_auto_login_timer)
            // 注册系统 - 游戏
            .add_systems(Update, (handle_load_games, handle_load_game_detail))
            // 注册系统 - 锅贴社区
            .add_systems(Update, (handle_load_apps, handle_load_fried_posts))
            // 注册系统 - 版本更新检查
            // ⚠️ 这三个 handler 曾整体漏注册：CheckUpdateRequest 发出去没人收，
            // is_checking 永远是 false，表现为「点检查更新毫无反应、也不报错」。
            // 本文件顶部的 `#![allow(dead_code)]` 让编译器不会提醒——改这里时留意
            .add_systems(
                Update,
                (
                    handle_check_update,
                    handle_check_update_response,
                    handle_check_update_failed,
                ),
            )
            // 注册系统 - 网络诊断
            .add_systems(
                Update,
                (
                    handle_speed_test,
                    handle_speed_test_result,
                    handle_ping_test,
                    handle_ping_test_result,
                    handle_network_test_failed,
                ),
            )
            // 注册系统 - 聊天室
            .add_systems(
                Update,
                (
                    handle_load_chat_rooms,
                    handle_connect_chat_room,
                    handle_send_chat_message,
                    handle_disconnect_chat_room,
                ),
            );
    }
}

/// API 客户端资源包装
#[derive(Resource)]
pub struct ApiClientResource(pub ApiClient);

/// 自动登录延迟计时器
#[derive(Resource)]
pub struct AutoLoginTimer {
    pub timer: Timer,
    pub should_login: bool,
    pub email: String,
    pub password: String,
}

impl ApiClientResource {
    pub fn new() -> Self {
        // 读取代理、分流、SNI/IPv6 设置
        let (proxy_url, api_channel, custom_cdn_api_ip, use_sni_pretend, prefer_ipv6) = {
            let settings = AppSettings::global().read();
            (
                settings.proxy.to_proxy_url(),
                settings.channel.api_channel,
                settings.channel.custom_cdn_api_ip.clone(),
                settings.use_sni_pretend,
                settings.prefer_ipv6,
            )
        };
        Self(
            ApiClient::with_config(
                proxy_url,
                api_channel,
                &custom_cdn_api_ip,
                use_sni_pretend,
                prefer_ipv6,
            )
            .expect("创建 API 客户端失败"),
        )
    }
}

impl Default for ApiClientResource {
    fn default() -> Self {
        Self::new()
    }
}

/// 处理登录请求
fn handle_login_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoginRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let email = event.email.clone();
        let password = event.password.clone();
        let client = api_client.0.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::LoginRequest;

            let request = LoginRequest { email, password };
            let result = match client.request(request).await {
                Ok(response) => Ok(response.token),
                Err(e) => Err(e.to_string()),
            };

            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(LoginResponseEvent { result });
            })
            .await;
        });
    }
}

/// 处理登录响应
fn handle_login_response(
    mut messages: MessageReader<LoginResponseEvent>,
    mut auth_state: ResMut<AuthState>,
    mut login_form: ResMut<LoginFormState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    api_client: Res<ApiClientResource>,
    mut punch_in_messages: MessageWriter<PunchInRequestEvent>,
    mut user_logged_in_messages: MessageWriter<UserLoggedInEvent>,
) {
    for event in messages.read() {
        login_form.is_loading = false;

        match &event.result {
            Ok(token) => {
                api_client.0.set_token(token.clone());
                auth_state.token = Some(token.clone());
                auth_state.is_logged_in = true;
                login_form.error = None;

                // 保存登录凭据（如果启用了保存密码）
                save_credentials_on_login(&login_form);

                // 如果启用了自动打卡，触发打卡请求
                if login_form.auto_punch_in {
                    punch_in_messages.write(PunchInRequestEvent);
                    tracing::info!("已触发自动打卡");
                }

                // 发送用户登录成功事件（通知需要等待登录的系统）
                user_logged_in_messages.write(UserLoggedInEvent);
                tracing::info!("用户登录成功，已发送 UserLoggedInEvent");

                // 登录成功后直接进入分类页面
                next_route.set(AppRoute::Categories);
            }
            Err(error) => {
                login_form.error = Some(format!("登录失败: {}", error));
            }
        }
    }
}

/// 处理加载分类请求
fn handle_load_categories(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadCategoriesRequest>,
    api_client: Res<ApiClientResource>,
    mut categories_state: ResMut<CategoriesState>,
) {
    for _event in messages.read() {
        // 防止重复请求（预加载和页面进入可能同时触发）
        if categories_state.is_loading || !categories_state.categories.is_empty() {
            continue;
        }
        categories_state.is_loading = true;
        categories_state.error = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::category::GetCategoriesRequest;

            match client.request(GetCategoriesRequest).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(CategoriesLoadedEvent {
                            categories: response.categories,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(CategoriesLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理分类加载响应
fn handle_categories_response(
    mut loaded_messages: MessageReader<CategoriesLoadedEvent>,
    mut failed_messages: MessageReader<CategoriesLoadFailedEvent>,
    mut categories_state: ResMut<CategoriesState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        categories_state.is_loading = false;
        categories_state.categories = event.categories.clone();

        // 触发加载图片
        for category in &event.categories {
            image_messages.write(LoadImageRequest {
                url: category.thumb.url(),
            });
        }
    }

    for event in failed_messages.read() {
        categories_state.is_loading = false;
        categories_state.error = Some(event.error.clone());
    }
}

/// 处理加载漫画列表请求
fn handle_load_comics(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadComicsRequest>,
    api_client: Res<ApiClientResource>,
    mut comics_state: ResMut<ComicsListState>,
) {
    for event in messages.read() {
        // 只有首次加载才设 is_loading，追加页使用 is_loading_more（由
        // auto_load_more_comics 设置）
        if event.page <= 1 {
            comics_state.is_loading = true;
        }
        comics_state.error = None;

        let client = api_client.0.clone();
        let category = event.category.clone();
        let page = event.page;
        let sort = event.sort.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::GetComicsRequest;

            let request = GetComicsRequest {
                category,
                page,
                sort,
            };

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(ComicsLoadedEvent {
                            comics: response.comics.docs,
                            total_pages: response.comics.pages,
                            page,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(ComicsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理漫画列表加载响应（支持追加模式）
fn handle_comics_response(
    mut loaded_messages: MessageReader<ComicsLoadedEvent>,
    mut failed_messages: MessageReader<ComicsLoadFailedEvent>,
    mut comics_state: ResMut<ComicsListState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        comics_state.total_pages = event.total_pages;
        let filtered = apply_block_filter(&event.comics);
        let added = filtered.len();

        if event.page <= 1 {
            // 首次加载：替换数据
            comics_state.is_loading = false;
            comics_state.comics = filtered;
        } else {
            // 无限滚动追加：合并数据
            comics_state.is_loading = false;
            comics_state.is_loading_more = false;
            comics_state.comics.extend(filtered);
        }

        // 触发加载图片（只为过滤后真正会显示的漫画预取，屏蔽的不下载）
        let start = comics_state.comics.len() - added;
        for comic in &comics_state.comics[start..] {
            image_messages.write(LoadImageRequest {
                url: comic.thumb.url(),
            });
        }
    }

    for event in failed_messages.read() {
        comics_state.is_loading = false;
        comics_state.is_loading_more = false;
        comics_state.error = Some(event.error.clone());
    }
}

/// 获取图片缓存目录
fn get_image_cache_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cache")
        .join("images")
}

/// 将 URL 转换为缓存文件路径
fn url_to_cache_path(url: &str) -> std::path::PathBuf {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    // 使用 URL 的 hash 作为文件名，避免特殊字符问题
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    // 提取扩展名
    let ext = url
        .rsplit('.')
        .next()
        .and_then(|e| {
            let e = e.to_lowercase();
            if ["jpg", "jpeg", "png", "gif", "webp"].contains(&e.as_str()) {
                Some(e)
            } else {
                None
            }
        })
        .unwrap_or_else(|| "jpg".to_string());

    get_image_cache_path().join(format!("{:016x}.{}", hash, ext))
}

/// 将图片加载请求加入队列（不立即加载，节流处理）
fn enqueue_image_requests(
    mut messages: MessageReader<LoadImageRequest>,
    mut image_cache: ResMut<ImageCache>,
) {
    for event in messages.read() {
        image_cache.enqueue(event.url.clone());
    }
}

/// 处理图片加载队列（每帧处理有限数量）
fn process_image_queue(runtime: ResMut<TokioTasksRuntime>, mut image_cache: ResMut<ImageCache>) {
    // 退避期已过的失败条目重新排队（无待重试项时 O(1) 直返）
    image_cache.requeue_ready_retries();

    // 获取可以开始加载的批次
    let batch = image_cache.take_pending_batch();
    if batch.is_empty() {
        return;
    }

    for url in batch {
        image_cache.mark_loading(url.clone());

        runtime.spawn_background_task(move |mut ctx| async move {
            let cache_path = url_to_cache_path(&url);

            // 先尝试从本地缓存加载
            let result = if cache_path.exists() {
                match tokio::fs::read(&cache_path).await {
                    Ok(data) => Ok(data),
                    Err(_) => {
                        // 缓存文件读取失败，从网络下载
                        download_image(&url).await
                    }
                }
            } else {
                // 从网络下载
                let data = download_image(&url).await;

                // 下载成功后保存到缓存
                if let Ok(ref image_data) = data {
                    // 确保缓存目录存在
                    if let Some(parent) = cache_path.parent() {
                        let _ = tokio::fs::create_dir_all(parent).await;
                    }
                    // 保存到缓存（忽略保存失败）
                    let _ = tokio::fs::write(&cache_path, image_data).await;
                }

                data
            };

            // 解码走 spawn_blocking：纯 CPU 密集操作不占 tokio worker
            // （此前直接写在 async 块里，最多 15 个解码任务同时饿住网络 I/O）
            let decoded_result = match result {
                Ok(image_data) => {
                    let decoded = tokio::task::spawn_blocking(move || {
                        match image::load_from_memory(&image_data) {
                            Ok(img) => {
                                // 已是 RGBA8 时直接取缓冲，避免整图额外拷贝
                                let rgba = match img {
                                    image::DynamicImage::ImageRgba8(buffer) => buffer,
                                    other => other.to_rgba8(),
                                };
                                let (width, height) = rgba.dimensions();
                                Ok((width, height, rgba.into_raw()))
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    })
                    .await;
                    match decoded {
                        Ok(inner) => inner,
                        Err(e) => Err(format!("解码任务被取消: {e}")),
                    }
                }
                Err(e) => Err(e),
            };

            // 只在主线程创建 Bevy 资源（轻量操作）
            ctx.run_on_main_thread(move |ctx| {
                match decoded_result {
                    Ok((width, height, rgba_data)) => {
                        let image = Image::new(
                            bevy::render::render_resource::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                            bevy::render::render_resource::TextureDimension::D2,
                            rgba_data,
                            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                            RenderAssetUsages::RENDER_WORLD,
                        );

                        // 添加到 Assets<Image>
                        let mut images = ctx.world.resource_mut::<Assets<Image>>();
                        let handle = images.add(image);

                        ctx.world.write_message(ImageLoadedEvent { url, handle });
                    }
                    Err(e) => {
                        ctx.world
                            .write_message(ImageLoadFailedEvent { url, error: e });
                    }
                }
            })
            .await;
        });
    }
}

/// 下载图片数据（使用代理设置和签名头部）
async fn download_image(url: &str) -> Result<Vec<u8>, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    use hmac::{Hmac, KeyInit, Mac};
    use reqwest::Proxy;
    use sha2::Sha256;

    const API_KEY: &str = "C69BAF41DA5ABD1FFEDC6D2FEA56B";
    const SECRET_KEY: &str = r"~d}$Q7$eIni=V)9\RK/P.RM4;9[7|@/CA}b~OW!3?EV`:<>M7pddUBL5n|0/*Cn";
    const VERSION: &str = "2.2.1.3.3.4";
    const BUILD_VERSION: &str = "45";
    const APP_UUID: &str = "defaultUuid";

    type HmacSha256 = Hmac<Sha256>;

    // 使用独立作用域确保锁在 await 之前释放
    let (proxy_url, image_channel, custom_cdn_img_ip) = {
        let settings = AppSettings::global().read();
        (
            settings.proxy.to_proxy_url(),
            settings.channel.image_channel,
            settings.channel.custom_cdn_img_ip.clone(),
        )
    };

    // 转换图片 URL（反代模式下替换域名）
    let actual_url = transform_image_url(url, image_channel);

    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::limited(10));

    // 添加代理配置
    if let Some(ref proxy_url_str) = proxy_url {
        let proxy = Proxy::all(proxy_url_str).map_err(|e| format!("代理配置错误: {}", e))?;
        builder = builder.proxy(proxy);
    }

    // 添加图片 CDN DNS 覆盖
    builder = apply_image_dns_override(builder, image_channel, &custom_cdn_img_ip);

    let client = builder.build().map_err(|e| e.to_string())?;

    // 生成签名头部（始终使用原始 URL 签名）
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let nonce = uuid::Uuid::new_v4().simple().to_string();

    let method = "GET";
    let src = format!("{}{}{}{}{}", url, now, nonce, method, API_KEY);

    let mut mac =
        HmacSha256::new_from_slice(SECRET_KEY.as_bytes()).expect("HMAC can take key of any size");
    mac.update(src.to_lowercase().as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let response = client
        .get(&actual_url)
        .header("api-key", API_KEY)
        .header("accept", "application/vnd.picacomic.com.v1+json")
        .header("app-channel", "3")
        .header("time", &now)
        .header("app-uuid", APP_UUID)
        .header("nonce", &nonce)
        .header("signature", &signature)
        .header("app-version", VERSION)
        .header("image-quality", "original")
        .header("app-platform", "android")
        .header("app-build-version", BUILD_VERSION)
        .header("user-agent", "okhttp/3.8.1")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

/// 处理图片加载响应
fn handle_image_response(
    mut loaded_messages: MessageReader<ImageLoadedEvent>,
    mut failed_messages: MessageReader<ImageLoadFailedEvent>,
    mut image_cache: ResMut<ImageCache>,
) {
    for event in loaded_messages.read() {
        image_cache.set_loaded(event.url.clone(), event.handle.clone());
    }

    for event in failed_messages.read() {
        image_cache.set_failed(event.url.clone(), event.error.clone());
    }
}

/// 处理打卡请求
fn handle_punch_in_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<PunchInRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for _event in messages.read() {
        let client = api_client.0.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::auth::PunchInRequest;

            let result = match client.request(PunchInRequest).await {
                Ok(response) => Ok(response.res.status),
                Err(e) => Err(e.to_string()),
            };

            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(PunchInResponseEvent { result });
            })
            .await;
        });
    }
}

/// 处理打卡响应
fn handle_punch_in_response(
    mut messages: MessageReader<PunchInResponseEvent>,
    mut punch_in_state: ResMut<PunchInState>,
) {
    for event in messages.read() {
        match &event.result {
            Ok(status) => {
                tracing::info!("打卡成功: {}", status);
                punch_in_state.is_punched = true;
                punch_in_state.is_success = true;
                punch_in_state.message = Some("签到成功！".into());
            }
            Err(error) => {
                tracing::warn!("打卡失败: {}", error);
                punch_in_state.is_success = false;
                punch_in_state.message = Some(format!("签到失败: {}", error));
            }
        }
    }
}

/// 处理 API 客户端重载事件（通道/代理变更时重建客户端）
fn handle_reload_api_client(
    mut messages: MessageReader<ReloadApiClientEvent>,
    mut api_client: ResMut<ApiClientResource>,
) {
    for _ in messages.read() {
        let (proxy_url, api_channel, custom_cdn_api_ip, use_sni_pretend, prefer_ipv6) = {
            let settings = AppSettings::global().read();
            (
                settings.proxy.to_proxy_url(),
                settings.channel.api_channel,
                settings.channel.custom_cdn_api_ip.clone(),
                settings.use_sni_pretend,
                settings.prefer_ipv6,
            )
        };

        if let Err(e) = api_client.0.reload_config(
            proxy_url,
            api_channel,
            &custom_cdn_api_ip,
            use_sni_pretend,
            prefer_ipv6,
        ) {
            tracing::error!("重载 API 客户端失败: {}", e);
        } else {
            tracing::info!("API 客户端已重载");
        }
    }
}

/// 启动时初始化自动登录计时器
fn auto_login_on_startup(mut commands: Commands) {
    let settings = AppSettings::global().read();

    // 检查是否启用自动登录且有保存的凭据
    if settings.login.auto_login
        && !settings.login.saved_email.is_empty()
        && !settings.login.saved_password.is_empty()
    {
        let email = settings.login.saved_email.clone();
        let password = settings.login.saved_password.clone();
        drop(settings); // 释放锁

        tracing::info!("启用自动登录，将在 3 秒后自动登录...");

        // 创建 3 秒延迟计时器
        commands.insert_resource(AutoLoginTimer {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
            should_login: true,
            email,
            password,
        });
    }
}

/// 检查自动登录计时器并触发登录
fn check_auto_login_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Option<ResMut<AutoLoginTimer>>,
    mut login_messages: MessageWriter<LoginRequestEvent>,
) {
    let Some(ref mut auto_login) = timer else {
        return;
    };

    if !auto_login.should_login {
        return;
    }

    auto_login.timer.tick(time.delta());

    if auto_login.timer.just_finished() {
        tracing::info!("自动登录计时完成，正在登录...");
        login_messages.write(LoginRequestEvent {
            email: auto_login.email.clone(),
            password: auto_login.password.clone(),
        });
        auto_login.should_login = false;

        // 移除计时器资源
        commands.remove_resource::<AutoLoginTimer>();
    }
}

// ==================== 漫画详情处理 ====================

/// 处理加载漫画详情请求
fn handle_load_comic_detail(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadComicDetailRequest>,
    api_client: Res<ApiClientResource>,
    mut detail_state: ResMut<ComicDetailState>,
) {
    for event in messages.read() {
        detail_state.is_loading = true;
        detail_state.error = None;
        detail_state.comic_id = event.comic_id.clone();

        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::GetComicDetailRequest;

            let request = GetComicDetailRequest { comic_id };

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(ComicDetailLoadedEvent {
                            comic: response.comic,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(ComicDetailLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理漫画详情加载响应
fn handle_comic_detail_response(
    mut loaded_messages: MessageReader<ComicDetailLoadedEvent>,
    mut failed_messages: MessageReader<ComicDetailLoadFailedEvent>,
    mut detail_state: ResMut<ComicDetailState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
    mut episodes_messages: MessageWriter<LoadEpisodesRequest>,
) {
    for event in loaded_messages.read() {
        detail_state.is_loading = false;
        detail_state.is_favorite = event.comic.is_favourite.unwrap_or(false);
        detail_state.is_liked = event.comic.is_liked.unwrap_or(false);
        detail_state.comic = Some(event.comic.clone());

        // 加载封面图片
        image_messages.write(LoadImageRequest {
            url: event.comic.thumb.url(),
        });

        // 加载章节列表（自动获取所有页）
        episodes_messages.write(LoadEpisodesRequest {
            comic_id: detail_state.comic_id.clone(),
        });
    }

    for event in failed_messages.read() {
        detail_state.is_loading = false;
        detail_state.error = Some(event.error.clone());
    }
}

// ==================== 章节列表处理 ====================

/// 处理加载章节列表请求（自动获取所有页）
fn handle_load_episodes(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadEpisodesRequest>,
    api_client: Res<ApiClientResource>,
    mut detail_state: ResMut<ComicDetailState>,
) {
    for event in messages.read() {
        detail_state.is_loading_episodes = true;

        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::GetEpisodesRequest;

            let mut all_episodes = Vec::new();
            let mut page = 1;

            loop {
                let request = GetEpisodesRequest {
                    comic_id: comic_id.clone(),
                    page,
                };

                match client.request(request).await {
                    Ok(response) => {
                        let total_pages = response.eps.pages;
                        all_episodes.extend(response.eps.docs);
                        if page >= total_pages {
                            break;
                        }
                        page += 1;
                    }
                    Err(e) => {
                        let error = e.to_string();
                        ctx.run_on_main_thread(move |ctx| {
                            ctx.world.write_message(EpisodesLoadFailedEvent { error });
                        })
                        .await;
                        return;
                    }
                }
            }

            // 按章节顺序排序
            all_episodes.sort_by_key(|e| e.order);

            let total_pages = page;
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(EpisodesLoadedEvent {
                    episodes: all_episodes,
                    total_pages,
                });
            })
            .await;
        });
    }
}

/// 处理章节列表加载响应
fn handle_episodes_response(
    mut loaded_messages: MessageReader<EpisodesLoadedEvent>,
    mut failed_messages: MessageReader<EpisodesLoadFailedEvent>,
    mut detail_state: ResMut<ComicDetailState>,
) {
    for event in loaded_messages.read() {
        detail_state.is_loading_episodes = false;
        detail_state.episodes = event.episodes.clone();
        detail_state.episodes_total_pages = event.total_pages;
    }

    for event in failed_messages.read() {
        detail_state.is_loading_episodes = false;
        // 不覆盖漫画详情的错误
        tracing::warn!("章节加载失败: {}", event.error);
    }
}

// ==================== 图片列表加载 ====================

/// 处理图片列表加载请求
fn handle_load_pictures(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadPicturesRequest>,
    api_client: Res<ApiClientResource>,
    mut reader_state: ResMut<ReaderState>,
) {
    for event in messages.read() {
        reader_state.is_loading = true;

        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();
        let episode_order = event.episode_order;
        let page = event.page;

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::GetPicturesRequest;

            let request = GetPicturesRequest {
                comic_id,
                episode_order,
                page,
            };

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(PicturesLoadedEvent {
                            pictures: response.pages.docs,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(PicturesLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 并发加载所有章节的图片列表（条漫模式用，DB 缓存优先）
fn handle_load_all_chapter_pictures(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadAllChapterPicturesRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();
        let episodes = event.episodes.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::{endpoints::comic::GetPicturesRequest, models::Picture};

            let pool = picacg_db::get_pool();
            let mut all_pictures: Vec<Picture> = Vec::new();
            let mut all_metas = Vec::new();
            let mut cache_hit = 0_usize;
            let mut cache_miss = 0_usize;

            for ep in &episodes {
                // 1. 先查 DB 缓存
                if let Some(json) =
                    picacg_db::get_episode_pictures_async(&pool, &comic_id, ep.order).await
                    && let Ok(pics) = serde_json::from_str::<Vec<Picture>>(&json)
                {
                    for (i, pic) in pics.into_iter().enumerate() {
                        all_metas.push(crate::events::WebtoonPageMeta {
                            episode_order: ep.order,
                            page_in_chapter: i,
                        });
                        all_pictures.push(pic);
                    }
                    cache_hit += 1;
                    continue;
                }

                // 2. 缓存未命中，从 API 加载
                cache_miss += 1;
                let mut page = 1;
                let mut chapter_pics: Vec<Picture> = Vec::new();

                loop {
                    let request = GetPicturesRequest {
                        comic_id: comic_id.clone(),
                        episode_order: ep.order,
                        page,
                    };

                    match client.request(request).await {
                        Ok(response) => {
                            let total_api_pages = response.pages.pages;
                            chapter_pics.extend(response.pages.docs);
                            if page >= total_api_pages {
                                break;
                            }
                            page += 1;
                        }
                        Err(e) => {
                            tracing::error!("加载第 {} 章图片列表失败: {}", ep.order, e);
                            break;
                        }
                    }
                }

                // 3. 写入 DB 缓存
                if !chapter_pics.is_empty()
                    && let Ok(json) = serde_json::to_string(&chapter_pics)
                {
                    picacg_db::save_episode_pictures_async(&pool, &comic_id, ep.order, &json).await;
                }

                // 4. 追加到总列表
                for (i, pic) in chapter_pics.into_iter().enumerate() {
                    all_metas.push(crate::events::WebtoonPageMeta {
                        episode_order: ep.order,
                        page_in_chapter: i,
                    });
                    all_pictures.push(pic);
                }
            }

            let total = all_pictures.len();
            tracing::info!(
                "全章节图片列表加载完成: {} 章, {} 张图片（缓存命中 {}, API {} 章）",
                episodes.len(),
                total,
                cache_hit,
                cache_miss,
            );

            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(AllChapterPicturesLoadedEvent {
                    pictures: all_pictures,
                    page_metas: all_metas,
                });
            })
            .await;
        });
    }
}

// ==================== 点赞处理 ====================

/// 处理点赞漫画请求
fn handle_like_comic(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LikeComicRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::LikeComicRequest as ApiLikeRequest;

            let request = ApiLikeRequest {
                comic_id: comic_id.clone(),
            };

            match client.request(request).await {
                Ok(response) => {
                    let cid = comic_id;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(LikeComicResponse {
                            comic_id: cid,
                            action: response.action,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    tracing::warn!("点赞失败: {}", e);
                }
            }
        });
    }
}

/// 处理点赞响应
///
/// 更新详情页状态，同时保存/删除本地点赞记录
fn handle_like_response(
    mut messages: MessageReader<LikeComicResponse>,
    mut detail_state: ResMut<ComicDetailState>,
    mut save_messages: MessageWriter<SaveLikeRecordRequest>,
    mut delete_messages: MessageWriter<DeleteLikeRecordRequest>,
) {
    for event in messages.read() {
        let is_liked = event.action == "like";
        detail_state.is_liked = is_liked;

        // 保存/删除本地点赞记录
        if is_liked {
            // 从当前详情页获取漫画信息
            if let Some(ref comic) = detail_state.comic {
                save_messages.write(SaveLikeRecordRequest {
                    comic_id: event.comic_id.clone(),
                    comic_title: comic.title.clone(),
                    thumb_url: comic.thumb.url(),
                });
            }
        } else {
            delete_messages.write(DeleteLikeRecordRequest {
                comic_id: event.comic_id.clone(),
            });
        }

        tracing::info!("点赞操作: {}", event.action);
    }
}

// ==================== 收藏处理 ====================

/// 处理收藏漫画请求
fn handle_favorite_comic(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<FavoriteComicRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::FavoriteComicRequest as ApiFavoriteRequest;

            let request = ApiFavoriteRequest { comic_id };

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(FavoriteComicResponse {
                            action: response.action,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    tracing::warn!("收藏失败: {}", e);
                }
            }
        });
    }
}

/// 处理收藏响应
fn handle_favorite_response(
    mut messages: MessageReader<FavoriteComicResponse>,
    mut detail_state: ResMut<ComicDetailState>,
) {
    for event in messages.read() {
        detail_state.is_favorite = event.action == "favorite";
        tracing::info!("收藏操作: {}", event.action);
    }
}

// ==================== 收藏列表处理 ====================

/// 处理加载收藏列表请求
fn handle_load_favorites(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadFavoritesRequest>,
    api_client: Res<ApiClientResource>,
    mut favorites_state: ResMut<FavoritesState>,
) {
    for event in messages.read() {
        let page = event.page;
        let sort = event.sort.clone();

        favorites_state.is_loading = true;
        favorites_state.error = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::category::GetFavoritesRequest;

            let request = GetFavoritesRequest { page, sort };

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(FavoritesLoadedEvent {
                            comics: response.comics.docs,
                            total_pages: response.comics.pages,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(FavoritesLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理收藏列表响应
fn handle_favorites_response(
    mut loaded_messages: MessageReader<FavoritesLoadedEvent>,
    mut failed_messages: MessageReader<FavoritesLoadFailedEvent>,
    mut favorites_state: ResMut<FavoritesState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        let filtered = apply_block_filter(&event.comics);
        favorites_state.comics = filtered;
        favorites_state.total_pages = event.total_pages;
        favorites_state.is_loading = false;
        favorites_state.error = None;
        tracing::info!(
            "收藏列表加载完成: {} 个, 共 {} 页",
            favorites_state.comics.len(),
            favorites_state.total_pages
        );

        // 触发加载封面图片
        for comic in &favorites_state.comics {
            image_messages.write(LoadImageRequest {
                url: comic.thumb.url(),
            });
        }
    }

    for event in failed_messages.read() {
        favorites_state.is_loading = false;
        favorites_state.error = Some(event.error.clone());
        tracing::warn!("收藏列表加载失败: {}", event.error);
    }
}

// ==================== 首页推荐处理 ====================

/// 处理加载推荐漫画请求
fn handle_load_recommendations(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadRecommendationsRequest>,
    api_client: Res<ApiClientResource>,
    mut home_state: ResMut<HomeState>,
) {
    for _request in messages.read() {
        if home_state.is_loading {
            continue;
        }

        home_state.is_loading = true;
        home_state.error = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::GetRecommendationsRequest;

            let request = GetRecommendationsRequest;

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(RecommendationsLoadedEvent {
                            comics: response.comics,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error: String = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(RecommendationsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理推荐漫画响应
fn handle_recommendations_response(
    mut loaded_messages: MessageReader<RecommendationsLoadedEvent>,
    mut failed_messages: MessageReader<RecommendationsLoadFailedEvent>,
    mut home_state: ResMut<HomeState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        let filtered = apply_block_filter(&event.comics);
        home_state.recommendations = filtered;
        home_state.is_loading = false;
        home_state.error = None;
        tracing::info!("推荐漫画加载完成: {} 个", home_state.recommendations.len());

        // 触发加载封面图片
        for comic in &home_state.recommendations {
            image_messages.write(LoadImageRequest {
                url: comic.thumb.url(),
            });
        }
    }

    for event in failed_messages.read() {
        home_state.is_loading = false;
        home_state.error = Some(event.error.clone());
        tracing::warn!("推荐漫画加载失败: {}", event.error);
    }
}

// ==================== 下载处理 ====================

/// 获取下载根目录
/// 优先使用设置中的自定义路径，否则使用程序目录下的 Downloads 文件夹
fn get_download_base_path() -> std::path::PathBuf {
    // 先检查设置中是否有自定义路径
    let settings = AppSettings::global().read();
    if !settings.download_path.is_empty() {
        return std::path::PathBuf::from(&settings.download_path);
    }
    drop(settings);

    // 使用程序所在目录的 Downloads 文件夹
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Downloads")
}

/// 获取原图下载保存路径
/// 结构：base/image/<漫画标题>/<章节>/
fn get_images_download_path() -> std::path::PathBuf {
    get_download_base_path().join("image")
}

/// 获取 CBZ 文件保存目录
/// 结构：base/cbz/<漫画标题>.cbz
fn get_cbz_output_path() -> std::path::PathBuf {
    get_download_base_path().join("cbz")
}

/// 清理文件名中的非法字符（代理到 utils::sanitize_filename）
fn sanitize_filename(name: &str) -> String {
    crate::utils::sanitize_filename(name)
}

/// 获取本地文件夹中的所有文件名（用于比对）
async fn get_local_filenames(folder: &std::path::Path) -> std::collections::HashSet<String> {
    let mut filenames = std::collections::HashSet::new();
    if let Ok(mut entries) = tokio::fs::read_dir(folder).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(name) = entry.file_name().to_str() {
                // 只收集图片文件
                let name_lower = name.to_lowercase();
                if name_lower.ends_with(".jpg")
                    || name_lower.ends_with(".jpeg")
                    || name_lower.ends_with(".png")
                    || name_lower.ends_with(".gif")
                    || name_lower.ends_with(".webp")
                {
                    filenames.insert(name.to_string());
                }
            }
        }
    }
    filenames
}

/// 处理下载漫画请求（FSM 架构）
fn handle_download_comic(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<DownloadComicRequest>,
    api_client: Res<ApiClientResource>,
    mut download_state: ResMut<DownloadManagerState>,
    detail_state: Res<ComicDetailState>,
) {
    for event in messages.read() {
        let comic_id = event.comic_id.clone();
        let comic_title = event.comic_title.clone();

        // 检查是否已在下载
        if download_state.downloading_ids.contains(&comic_id) {
            tracing::warn!("漫画 {} 已在下载队列中", comic_id);
            continue;
        }

        // 确定要下载的章节
        let mut episodes_to_download: Vec<i32> = if !event.episodes.is_empty() {
            event.episodes.clone()
        } else if !detail_state.episodes.is_empty() && detail_state.comic_id == comic_id {
            // 当前详情页就是这个漫画，直接用已加载的章节
            detail_state.episodes.iter().map(|e| e.order).collect()
        } else {
            // 章节列表未加载（如右键菜单下载），先通过 API 获取
            let client = api_client.0.clone();
            let cid = comic_id.clone();
            let ctitle = comic_title.clone();
            let event_eps_count = event.remote_eps_count;
            runtime.spawn_background_task(move |mut ctx| async move {
                use picacg_api::endpoints::GetEpisodesRequest;
                tracing::info!("快速下载：正在获取 {} 的章节列表...", ctitle);
                let mut all_episodes = Vec::new();
                let mut page = 1;
                loop {
                    match client
                        .request(GetEpisodesRequest {
                            comic_id: cid.clone(),
                            page,
                        })
                        .await
                    {
                        Ok(resp) => {
                            all_episodes.extend(resp.eps.docs.iter().map(|e| e.order));
                            if page >= resp.eps.pages {
                                break;
                            }
                            page += 1;
                        }
                        Err(e) => {
                            tracing::error!("获取章节列表失败: {}", e);
                            return;
                        }
                    }
                }
                if all_episodes.is_empty() {
                    tracing::warn!("快速下载：{} 没有章节", ctitle);
                    return;
                }
                // 重新发送带章节的下载请求
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.write_message(DownloadComicRequest {
                        comic_id: cid,
                        comic_title: ctitle,
                        episodes: all_episodes,
                        remote_eps_count: event_eps_count,
                    });
                })
                .await;
            });
            continue; // 本次跳过，等异步回调重新触发
        };
        episodes_to_download.sort();

        if episodes_to_download.is_empty() {
            tracing::warn!("没有章节可下载");
            continue;
        }

        let save_path = get_images_download_path()
            .join(sanitize_filename(&comic_title))
            .to_string_lossy()
            .to_string();

        // 从漫画详情获取分类、标签与 epsCount 快照（详情未加载时为空）
        let (categories, tags, detail_eps_count) = if detail_state.comic_id == comic_id {
            detail_state
                .comic
                .as_ref()
                .map(|c| (c.categories.clone(), c.tags.clone(), Some(c.eps_count)))
                .unwrap_or_default()
        } else {
            (vec![], vec![], None)
        };
        // 基准优先取请求自带的（列表右键下载走这条），再回落详情页；
        // 两处都没有就留空 = 未知，角标只报已下载、不猜更新
        let remote_eps_count = event
            .remote_eps_count
            .filter(|v| *v > 0)
            .or(detail_eps_count);

        // 创建 FSM 任务元数据
        let mut meta = DownloadTaskMeta::new(
            comic_id.clone(),
            comic_title.clone(),
            episodes_to_download.clone(),
            save_path.clone(),
            categories,
            tags,
        );
        meta.remote_eps_count = remote_eps_count;

        // 保存元数据到文件
        if let Err(e) = meta.save() {
            tracing::error!("保存下载元数据失败: {}", e);
        }

        // 检查是否已达到最大并发数
        let max_concurrent = AppSettings::global().read().max_concurrent_downloads;
        let should_start = download_state.downloading_ids.len() < max_concurrent;

        // 添加到下载状态
        download_state.add_task(meta);

        if !should_start {
            tracing::info!(
                "已达到最大并发下载数 ({})，任务 {} 排队等待",
                max_concurrent,
                comic_title
            );
            continue; // 任务保持 Queued 状态，等待其他任务完成后自动启动
        }

        // 添加到正在下载列表并获取控制器
        download_state.downloading_ids.insert(comic_id.clone());
        let control = if let Some(fsm) = download_state.find_task_mut(&comic_id) {
            // 启动下载
            if let Err(e) = fsm.start() {
                tracing::error!("启动下载失败: {}", e);
            }
            fsm.get_control()
        } else {
            std::sync::Arc::new(SharedTaskControl::new())
        };

        let total_episodes = episodes_to_download.len() as i32;
        tracing::info!(
            "开始下载漫画: {} ({} 章节) -> {}",
            comic_title,
            total_episodes,
            save_path
        );

        let client = api_client.0.clone();

        // 启动后台下载任务
        spawn_download_task(
            runtime.as_ref(),
            client,
            comic_id,
            comic_title,
            save_path,
            episodes_to_download,
            total_episodes,
            control,
        );
    }
}

/// 启动后台下载任务
#[allow(clippy::too_many_arguments)]
fn spawn_download_task(
    runtime: &TokioTasksRuntime,
    client: ApiClient,
    comic_id: String,
    comic_title: String,
    save_path: String,
    episodes_to_download: Vec<i32>,
    total_episodes: i32,
    control: std::sync::Arc<SharedTaskControl>,
) {
    runtime.spawn_background_task(move |mut ctx| async move {
        execute_download_task(
            &mut ctx,
            client,
            comic_id,
            comic_title,
            save_path,
            episodes_to_download,
            total_episodes,
            control,
        )
        .await;
    });
}

/// 执行下载任务（核心下载逻辑）
///
/// 可以从任何有 TaskContext 的异步上下文中调用
#[allow(clippy::too_many_arguments)]
async fn execute_download_task(
    ctx: &mut crate::utils::TaskContext,
    client: ApiClient,
    comic_id: String,
    comic_title: String,
    save_path: String,
    episodes_to_download: Vec<i32>,
    total_episodes: i32,
    control: std::sync::Arc<SharedTaskControl>,
) {
    // clone 一份用于闭包，原始值留给函数末尾的日志
    let comic_title_for_log = comic_title.clone();
    let download_path = std::path::PathBuf::from(&save_path);
    let comic_start = tokio::time::Instant::now();

    // 创建下载目录
    if let Err(e) = tokio::fs::create_dir_all(&download_path).await {
        let comic_id_clone = comic_id.clone();
        ctx.run_on_main_thread(move |ctx| {
            ctx.world.write_message(DownloadFailedEvent {
                comic_id: comic_id_clone,
                error: format!("创建目录失败: {}", e),
            });
        })
        .await;
        return;
    }

    // 逐章节下载
    for (ep_idx, episode_order) in episodes_to_download.iter().enumerate() {
        let episode_order = *episode_order;
        let episode_start = tokio::time::Instant::now();

        // 检查是否已暂停
        if control.is_pause_requested() {
            tracing::info!("下载已暂停: {}", comic_id);
            let comic_id_clone = comic_id.clone();
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(DownloadPausedEvent {
                    comic_id: comic_id_clone,
                });
            })
            .await;
            return;
        }

        // 发送进度更新
        let comic_id_clone = comic_id.clone();
        ctx.run_on_main_thread(move |ctx| {
            ctx.world.write_message(DownloadProgressEvent {
                comic_id: comic_id_clone,
                current_episode: ep_idx as i32 + 1,
                current_page: 0,
                total_pages: 0,
                status: format!(
                    "正在获取第 {}/{} 章图片列表...",
                    episode_order, total_episodes
                ),
            });
        })
        .await;

        // 创建章节目录
        let ep_folder = download_path.join(format!("第{}章", episode_order));
        if let Err(e) = tokio::fs::create_dir_all(&ep_folder).await {
            let comic_id_clone = comic_id.clone();
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(DownloadFailedEvent {
                    comic_id: comic_id_clone,
                    error: format!("创建章节目录失败: {}", e),
                });
            })
            .await;
            return;
        }

        // 获取该章节所有图片
        let mut all_pictures = Vec::new();
        let mut page = 1;
        loop {
            use picacg_api::endpoints::comic::GetPicturesRequest;

            let request = GetPicturesRequest {
                comic_id: comic_id.clone(),
                episode_order,
                page,
            };

            match client.request(request).await {
                Ok(response) => {
                    all_pictures.extend(response.pages.docs);
                    if page >= response.pages.pages {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    let comic_id_clone = comic_id.clone();
                    let error = format!(
                        "[{}] 获取第 {}/{} 章图片列表失败: {}",
                        comic_title, episode_order, total_episodes, e
                    );
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(DownloadFailedEvent {
                            comic_id: comic_id_clone,
                            error,
                        });
                    })
                    .await;
                    return;
                }
            }
        }

        let total_pages = all_pictures.len() as i32;
        tracing::info!(
            "[{}] 第 {}/{} 章共 {} 张图片",
            comic_title,
            episode_order,
            total_episodes,
            total_pages
        );

        // 收集 API 返回的所有 original_name
        let required_files: std::collections::HashSet<String> = all_pictures
            .iter()
            .enumerate()
            .map(|(idx, pic)| {
                if pic.media.original_name.is_empty() {
                    // 如果 original_name 为空，使用序号作为文件名
                    format!("{:04}.jpg", idx + 1)
                } else {
                    pic.media.original_name.clone()
                }
            })
            .collect();

        // 获取本地文件夹中的所有文件名
        let local_files = get_local_filenames(&ep_folder).await;

        // 检查是否所有需要的文件都已存在
        let missing_files: Vec<_> = required_files
            .iter()
            .filter(|f| !local_files.contains(*f))
            .collect();

        if missing_files.is_empty() {
            tracing::info!(
                "[{}] 第 {}/{} 章本地已完整（{} 张），跳过下载",
                comic_title,
                episode_order,
                total_episodes,
                total_pages
            );

            // 在数据库中标记章节完成
            let comic_id_for_complete = comic_id.clone();
            use picacg_db::{add_completed_episode_async, get_pool, run_db_operation};
            let pool = get_pool();
            run_db_operation(async move {
                if let Err(e) =
                    add_completed_episode_async(&pool, &comic_id_for_complete, episode_order).await
                {
                    tracing::warn!("更新章节完成状态到数据库失败: {}", e);
                }
            });

            // 发送进度更新（显示为已跳过）
            let comic_id_clone = comic_id.clone();
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(DownloadProgressEvent {
                    comic_id: comic_id_clone,
                    current_episode: ep_idx as i32 + 1,
                    current_page: total_pages,
                    total_pages,
                    status: format!("第{}/{}章 本地已完整，跳过", episode_order, total_episodes),
                });
            })
            .await;
            continue; // 跳过该章节，继续下一章
        } else {
            tracing::info!(
                "[{}] 第 {}/{} 章缺少 {} 张图片，开始下载",
                comic_title,
                episode_order,
                total_episodes,
                missing_files.len()
            );
        }

        // 并发下载配置（类似 Python 的 DownloadThreadNum）
        let download_workers = AppSettings::global().read().download_workers;
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(download_workers));

        // 准备下载任务列表
        let mut download_tasks: Vec<(usize, String, std::path::PathBuf)> = Vec::new();
        let mut skip_count = 0;

        for (pic_idx, picture) in all_pictures.iter().enumerate() {
            let url = picture.media.url();
            let original_name = &picture.media.original_name;

            // 使用 original_name 作为文件名，如果为空则使用序号
            let file_name = if original_name.is_empty() {
                // 从 URL 提取扩展名
                let ext = url
                    .rsplit('.')
                    .next()
                    .and_then(|e| {
                        let e = e.to_lowercase();
                        if ["jpg", "jpeg", "png", "gif", "webp"].contains(&e.as_str()) {
                            Some(e)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "jpg".to_string());
                format!("{:04}.{}", pic_idx + 1, ext)
            } else {
                original_name.clone()
            };

            let file_path = ep_folder.join(&file_name);

            // 如果文件已存在，跳过（支持断点续传）
            if file_path.exists() {
                tracing::trace!("文件已存在，跳过: {}", file_name);
                skip_count += 1;
                continue;
            }

            download_tasks.push((pic_idx, url, file_path));
        }

        // 发送初始进度
        let comic_id_clone = comic_id.clone();
        let pending_count = download_tasks.len();
        ctx.run_on_main_thread(move |ctx| {
            ctx.world.write_message(DownloadProgressEvent {
                comic_id: comic_id_clone,
                current_episode: ep_idx as i32 + 1,
                current_page: 0,
                total_pages,
                status: format!(
                    "第{}/{}章 并发下载中（{} 个线程，待下载 {} 张）",
                    episode_order, total_episodes, download_workers, pending_count
                ),
            });
        })
        .await;

        // 并发下载（类似 Python 的 _downloadQueue 模式）
        let completed_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(skip_count));
        let failed_images = std::sync::Arc::new(parking_lot::Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for (pic_idx, url, file_path) in download_tasks {
            // 检查是否已暂停
            if control.is_pause_requested() {
                tracing::info!("下载已暂停: {}", comic_id);
                let comic_id_clone = comic_id.clone();
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.write_message(DownloadPausedEvent {
                        comic_id: comic_id_clone,
                    });
                })
                .await;
                return;
            }

            let semaphore = semaphore.clone();
            let control = control.clone();
            let completed_count = completed_count.clone();
            let failed_images = failed_images.clone();
            let comic_title = comic_title.clone();

            // 启动并发下载任务
            let handle = tokio::spawn(async move {
                // 获取信号量许可（控制并发数）
                let _permit = semaphore.acquire().await.unwrap();

                // 检查是否已暂停
                if control.is_pause_requested() {
                    return;
                }

                // 下载图片（30秒超时）
                match download_image_to_file(&url, &file_path, 30).await {
                    Ok(_) => {
                        completed_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                    Err(e) => {
                        // 记录失败的图片，稍后重试
                        failed_images.lock().push((pic_idx, url, file_path));
                        tracing::warn!(
                            "⚠ [{}] 第{}/{}章 {}/{} 首次下载失败（稍后重试）: {}",
                            comic_title,
                            episode_order,
                            total_episodes,
                            pic_idx + 1,
                            total_pages,
                            e
                        );
                    }
                }
            });

            handles.push(handle);
        }

        // 进度监控：每300ms发送一次进度更新
        let mut last_reported = 0usize;
        loop {
            // 检查是否所有任务都完成了
            let mut all_done = true;
            for handle in &handles {
                if !handle.is_finished() {
                    all_done = false;
                    break;
                }
            }

            // 获取当前完成数量
            let current_count = completed_count.load(std::sync::atomic::Ordering::SeqCst);

            // 如果有新进度，发送更新
            if current_count != last_reported {
                last_reported = current_count;
                let comic_id_clone = comic_id.clone();
                let current_page = current_count as i32;
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.write_message(DownloadProgressEvent {
                        comic_id: comic_id_clone,
                        current_episode: ep_idx as i32 + 1,
                        current_page,
                        total_pages,
                        status: format!(
                            "第{}/{}章 下载中 {}/{}",
                            episode_order, total_episodes, current_page, total_pages
                        ),
                    });
                })
                .await;
            }

            if all_done {
                break;
            }

            // 等待一段时间再检查
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        // 等待所有任务完成（确保清理）
        for handle in handles {
            let _ = handle.await;
        }

        let success_count = completed_count.load(std::sync::atomic::Ordering::SeqCst);
        let mut failed_images = std::sync::Arc::try_unwrap(failed_images)
            .map(|mutex| mutex.into_inner())
            .unwrap_or_else(|arc| arc.lock().clone());

        // 并发重试失败的图片（最多重试 3 次）
        const MAX_RETRIES: u32 = 3;
        let mut retry_count = 0;
        let mut success_count = success_count; // 转为可变

        while !failed_images.is_empty() && retry_count < MAX_RETRIES {
            retry_count += 1;
            let retry_delay = std::time::Duration::from_secs(5 * retry_count as u64);

            tracing::info!(
                "[{}] 第 {}/{} 章有 {} 张图片下载失败，{}秒后进行第 {} 次重试...",
                comic_title,
                episode_order,
                total_episodes,
                failed_images.len(),
                retry_delay.as_secs(),
                retry_count
            );

            // 等待一段时间再重试
            tokio::time::sleep(retry_delay).await;

            // 检查是否已暂停
            if control.is_pause_requested() {
                tracing::info!("下载已暂停: {}", comic_id);
                let comic_id_clone = comic_id.clone();
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.write_message(DownloadPausedEvent {
                        comic_id: comic_id_clone,
                    });
                })
                .await;
                return;
            }

            // 发送重试进度
            let comic_id_clone = comic_id.clone();
            let retry_status = format!(
                "第{}/{}章 并发重试 {}/{}（剩余 {} 张）",
                episode_order,
                total_episodes,
                retry_count,
                MAX_RETRIES,
                failed_images.len()
            );
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(DownloadProgressEvent {
                    comic_id: comic_id_clone,
                    current_episode: ep_idx as i32 + 1,
                    current_page: total_pages,
                    total_pages,
                    status: retry_status,
                });
            })
            .await;

            // 并发重试下载
            let retry_success_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let still_failed = std::sync::Arc::new(parking_lot::Mutex::new(Vec::<(
                usize,
                String,
                std::path::PathBuf,
            )>::new()));
            let mut retry_handles = Vec::new();

            for (pic_idx, url, file_path) in failed_images.drain(..) {
                // 如果文件已存在（可能被其他进程下载），跳过
                if file_path.exists() {
                    retry_success_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    tracing::info!(
                        "✓ 第{}/{}章 {}/{} 文件已存在",
                        episode_order,
                        total_episodes,
                        pic_idx + 1,
                        total_pages
                    );
                    continue;
                }

                let semaphore = semaphore.clone();
                let control = control.clone();
                let retry_success_count = retry_success_count.clone();
                let still_failed = still_failed.clone();
                let comic_title = comic_title.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    if control.is_pause_requested() {
                        return;
                    }

                    // 重试时使用更长的超时（60秒）
                    match download_image_to_file(&url, &file_path, 60).await {
                        Ok(_) => {
                            retry_success_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            tracing::info!(
                                "✓ 第{}/{}章 {}/{} 重试成功",
                                episode_order,
                                total_episodes,
                                pic_idx + 1,
                                total_pages
                            );
                        }
                        Err(e) => {
                            still_failed.lock().push((pic_idx, url, file_path));
                            tracing::warn!(
                                "⚠ [{}] 第{}/{}章 {}/{} 重试失败: {}",
                                comic_title,
                                episode_order,
                                total_episodes,
                                pic_idx + 1,
                                total_pages,
                                e
                            );
                        }
                    }
                });

                retry_handles.push(handle);
            }

            // 等待所有重试任务完成
            for handle in retry_handles {
                let _ = handle.await;
            }

            success_count += retry_success_count.load(std::sync::atomic::Ordering::SeqCst);
            failed_images = std::sync::Arc::try_unwrap(still_failed)
                .map(|mutex| mutex.into_inner())
                .unwrap_or_else(|arc| arc.lock().clone());
        }

        // 统计最终失败数量
        let final_fail_count = failed_images.len();
        if final_fail_count > 0 {
            tracing::error!(
                "[{}] ✗ 第 {}/{} 章有 {} 张图片下载失败（已跳过）",
                comic_title,
                episode_order,
                total_episodes,
                final_fail_count
            );
        }

        let episode_elapsed = episode_start.elapsed();
        tracing::info!(
            "[{}] 第 {}/{} 章下载完成: 成功={}, 跳过={}, 失败={}, 耗时 {:.1}s",
            comic_title,
            episode_order,
            total_episodes,
            success_count,
            skip_count,
            final_fail_count,
            episode_elapsed.as_secs_f64()
        );

        // 如果没有失败的图片，在数据库中标记章节完成
        if final_fail_count == 0 {
            let comic_id_for_complete = comic_id.clone();
            use picacg_db::{add_completed_episode_async, get_pool, run_db_operation};
            let pool = get_pool();
            run_db_operation(async move {
                if let Err(e) =
                    add_completed_episode_async(&pool, &comic_id_for_complete, episode_order).await
                {
                    tracing::warn!("更新章节完成状态到数据库失败: {}", e);
                } else {
                    tracing::debug!("章节 {} 已标记为完成（数据库）", episode_order);
                }
            });
        }
    }

    // 下载完成
    let comic_elapsed = comic_start.elapsed();
    let minutes = comic_elapsed.as_secs() / 60;
    let seconds = comic_elapsed.as_secs() % 60;
    tracing::info!(
        "[{}] 全部下载完成，共 {} 章，总耗时 {}m{}s",
        comic_title_for_log,
        episodes_to_download.len(),
        minutes,
        seconds
    );

    let comic_id_clone = comic_id.clone();
    let save_path_clone = save_path.clone();
    ctx.run_on_main_thread(move |ctx| {
        ctx.world.write_message(DownloadCompletedEvent {
            comic_id: comic_id_clone,
            save_path: save_path_clone,
        });
    })
    .await;
}

/// 下载图片到文件（使用代理设置和签名头部）
/// timeout_secs: 下载超时时间（秒），默认 30 秒
async fn download_image_to_file(
    url: &str,
    file_path: &std::path::Path,
    timeout_secs: u64,
) -> Result<(), String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    tracing::trace!("开始下载: {}", file_name);

    use hmac::{Hmac, KeyInit, Mac};
    use reqwest::Proxy;
    use sha2::Sha256;

    const API_KEY: &str = "C69BAF41DA5ABD1FFEDC6D2FEA56B";
    const SECRET_KEY: &str = r"~d}$Q7$eIni=V)9\RK/P.RM4;9[7|@/CA}b~OW!3?EV`:<>M7pddUBL5n|0/*Cn";
    const VERSION: &str = "2.2.1.3.3.4";
    const BUILD_VERSION: &str = "45";
    const APP_UUID: &str = "defaultUuid";

    type HmacSha256 = Hmac<Sha256>;

    // 使用独立作用域确保锁在 await 之前释放
    let (proxy_url, image_channel, custom_cdn_img_ip) = {
        let settings = AppSettings::global().read();
        (
            settings.proxy.to_proxy_url(),
            settings.channel.image_channel,
            settings.channel.custom_cdn_img_ip.clone(),
        )
    };

    // 转换图片 URL（反代模式下替换域名）
    let actual_url = transform_image_url(url, image_channel);

    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .connect_timeout(std::time::Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::limited(10));

    // 添加代理配置
    if let Some(ref proxy_url_str) = proxy_url {
        let proxy = Proxy::all(proxy_url_str).map_err(|e| format!("代理配置错误: {}", e))?;
        builder = builder.proxy(proxy);
    }

    // 添加图片 CDN DNS 覆盖
    builder = apply_image_dns_override(builder, image_channel, &custom_cdn_img_ip);

    let client = builder.build().map_err(|e| e.to_string())?;

    // 生成签名头部（始终使用原始 URL 签名）
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let nonce = uuid::Uuid::new_v4().simple().to_string();

    let method = "GET";
    let src = format!("{}{}{}{}{}", url, now, nonce, method, API_KEY);

    let mut mac =
        HmacSha256::new_from_slice(SECRET_KEY.as_bytes()).expect("HMAC can take key of any size");
    mac.update(src.to_lowercase().as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // 发送带签名头部的请求
    let response = client
        .get(&actual_url)
        .header("api-key", API_KEY)
        .header("accept", "application/vnd.picacomic.com.v1+json")
        .header("app-channel", "3")
        .header("time", &now)
        .header("app-uuid", APP_UUID)
        .header("nonce", &nonce)
        .header("signature", &signature)
        .header("app-version", VERSION)
        .header("image-quality", "original")
        .header("app-platform", "android")
        .header("app-build-version", BUILD_VERSION)
        .header("user-agent", "okhttp/3.8.1")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    tokio::fs::write(file_path, &bytes)
        .await
        .map_err(|e| format!("保存文件失败: {}", e))?;

    tracing::trace!("下载完成: {} ({} bytes)", file_name, bytes.len());
    Ok(())
}

/// 处理下载进度更新（FSM 架构）
fn handle_download_progress(
    mut messages: MessageReader<DownloadProgressEvent>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    for event in messages.read() {
        // 更新 FSM 任务状态
        if let Some(fsm) = download_state.find_task_mut(&event.comic_id) {
            fsm.current_episode_total_pages = event.total_pages;
            // 更新状态并保存到文件
            if let Err(e) =
                fsm.update_progress(event.current_episode, event.current_page, event.total_pages)
            {
                tracing::warn!("更新下载进度失败: {}", e);
            }
        }

        tracing::debug!("下载进度: {} - {}", event.comic_id, event.status);
    }
}

/// 处理下载完成（FSM 架构）
fn handle_download_completed(
    mut messages: MessageReader<DownloadCompletedEvent>,
    mut download_state: ResMut<DownloadManagerState>,
    mut downloaded_index: ResMut<DownloadedComicsIndex>,
    mut cbz_messages: MessageWriter<CbzPackageRequest>,
) {
    for event in messages.read() {
        download_state.downloading_ids.remove(&event.comic_id);

        // 同步封面角标索引：基准取刚下完的任务里记录的 epsCount 快照
        let remote_eps_count = download_state
            .find_task(&event.comic_id)
            .and_then(|fsm| fsm.meta.remote_eps_count);
        downloaded_index.insert(event.comic_id.clone(), remote_eps_count);

        // 获取漫画标题（用于 CBZ 打包）
        let comic_title = download_state
            .find_task(&event.comic_id)
            .map(|fsm| fsm.meta.comic_title.clone())
            .unwrap_or_default();

        // 更新 FSM 状态
        if let Some(fsm) = download_state.find_task_mut(&event.comic_id)
            && let Err(e) = fsm.complete()
        {
            tracing::warn!("更新下载完成状态失败: {}", e);
        }

        tracing::info!("下载完成: {} -> {}", event.comic_id, event.save_path);

        // 检查是否启用了自动打包 CBZ（优先使用任务独立设置）
        let auto_pack_cbz = download_state
            .find_task(&event.comic_id)
            .map(|fsm| fsm.meta.effective_auto_pack_cbz())
            .unwrap_or_else(|| AppSettings::global().read().auto_pack_cbz);
        if auto_pack_cbz && !comic_title.is_empty() {
            tracing::info!("触发 CBZ 打包: {}", comic_title);
            cbz_messages.write(CbzPackageRequest {
                comic_id: event.comic_id.clone(),
                comic_title,
                source_path: event.save_path.clone(),
            });
        }
    }
}

/// 处理下载失败（FSM 架构）
fn handle_download_failed(
    mut messages: MessageReader<DownloadFailedEvent>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    for event in messages.read() {
        download_state.downloading_ids.remove(&event.comic_id);

        // 更新 FSM 状态
        if let Some(fsm) = download_state.find_task_mut(&event.comic_id)
            && let Err(e) = fsm.fail(event.error.clone())
        {
            tracing::warn!("更新下载失败状态失败: {}", e);
        }

        tracing::error!("下载失败: {} - {}", event.comic_id, event.error);
    }
}

/// 处理下载暂停（后台任务通知主线程已暂停）（FSM 架构）
fn handle_download_paused(
    mut messages: MessageReader<DownloadPausedEvent>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    for event in messages.read() {
        download_state.downloading_ids.remove(&event.comic_id);

        // 更新 FSM 状态
        if let Some(fsm) = download_state.find_task_mut(&event.comic_id)
            && let Err(e) = fsm.pause()
        {
            tracing::warn!("更新下载暂停状态失败: {}", e);
        }

        tracing::info!("下载已暂停: {}", event.comic_id);
    }
}

/// 处理恢复下载请求（FSM 架构）
fn handle_resume_download(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<ResumeDownloadRequest>,
    api_client: Res<ApiClientResource>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    for event in messages.read() {
        let comic_id = event.comic_id.clone();

        // 查找 FSM 任务
        let task_info = download_state.find_task(&comic_id).map(|fsm| {
            let mut episode_orders = fsm.meta.episode_orders.clone();
            // 从第一章开始下载（正序）
            episode_orders.sort();
            (
                fsm.meta.comic_title.clone(),
                fsm.meta.effective_download_path().to_string(),
                episode_orders,
                fsm.meta.total_episodes,
            )
        });

        let Some((comic_title, save_path, episode_orders, total_episodes)) = task_info else {
            tracing::warn!("找不到下载任务: {}", comic_id);
            continue;
        };

        // 检查是否已在下载
        if download_state.downloading_ids.contains(&comic_id) {
            tracing::warn!("漫画 {} 已在下载中", comic_id);
            continue;
        }

        // 跳过已完成的任务（防止重复下载）
        if let Some(fsm) = download_state.find_task(&comic_id)
            && matches!(fsm.meta.state, DownloadState::Completed)
        {
            tracing::debug!("漫画 {} 已完成，跳过恢复", comic_title);
            continue;
        }

        // 检查是否已达到最大并发数
        let max_concurrent = AppSettings::global().read().max_concurrent_downloads;
        if download_state.downloading_ids.len() >= max_concurrent {
            tracing::info!(
                "已达到最大并发下载数 ({})，任务 {} 排队等待",
                max_concurrent,
                comic_title
            );
            // 将任务状态更新为 Waiting（排队等待）
            if let Some(fsm) = download_state.find_task_mut(&comic_id)
                && let Err(e) = fsm.queue()
            {
                tracing::warn!("更新任务状态为等待中失败: {}", e);
            }
            continue; // 等待其他任务完成后自动启动
        }

        // 重置控制器并获取新的控制器
        if let Some(fsm) = download_state.find_task_mut(&comic_id) {
            fsm.control.reset();
            // 更新状态为下载中
            if let Err(e) = fsm.start() {
                tracing::warn!("更新下载状态失败: {}", e);
            }
        }

        let control = download_state
            .find_task(&comic_id)
            .map(|fsm| fsm.get_control())
            .unwrap_or_else(|| std::sync::Arc::new(SharedTaskControl::new()));

        // 更新下载中状态
        download_state.downloading_ids.insert(comic_id.clone());

        tracing::info!("恢复下载漫画: {} -> {}", comic_title, save_path);

        let client = api_client.0.clone();

        // 启动后台下载任务（使用元数据中保存的章节信息，
        // 会自动跳过已存在的文件）
        spawn_download_task(
            runtime.as_ref(),
            client,
            comic_id,
            comic_title,
            save_path,
            episode_orders,
            total_episodes,
            control,
        );
    }
}

/// 更新前置检查：只取一次漫画详情比对章节数，判断是否真的有新章节
///
/// 普通「更新」的快路径——章节数与本地记录一致即判定已是最新，不进下载流程
/// （旧实现无论有无更新都要逐章拉图片列表、逐图比对文件名，几十章的漫画一次
/// 检查就是几十个 API 请求）。真有新章节时再走与强制更新相同的完整流程。
fn handle_redownload_precheck(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<RedownloadRequest>,
    api_client: Res<ApiClientResource>,
    download_state: Res<DownloadManagerState>,
) {
    use crate::resources::DownloadTaskMeta;

    for event in messages.read() {
        // 强制更新不做前置检查，由 handle_redownload 直接接手
        if event.force {
            continue;
        }

        let comic_id = event.comic_id.clone();
        let new_base_path = event.new_base_path.clone();

        if download_state.downloading_ids.contains(&comic_id) {
            tracing::warn!("漫画 {} 已在下载中，跳过更新检查", comic_id);
            continue;
        }

        let Ok(old_meta) = DownloadTaskMeta::load_by_comic_id(&comic_id) else {
            tracing::warn!("找不到漫画 {} 的下载记录", comic_id);
            continue;
        };

        // 比对基准是下载当时的 epsCount 快照，不是本地章节数——服务端这个字段
        // 与真实章节列表长期对不上（详见 DownloadedComicsIndex::badge_state），
        // 只有同一字段自比才可靠。老记录没有快照，只能跑完整流程后补上。
        // 比对基准是下载当时的 epsCount 快照，不是本地章节数——服务端这个字段
        // 与真实章节列表长期对不上（详见 DownloadedComicsIndex::badge_state），
        // 只有同一字段自比才可靠。
        let baseline = old_meta.remote_eps_count;
        let local_episodes = if old_meta.episode_orders.is_empty() {
            old_meta.total_episodes
        } else {
            old_meta.episode_orders.len() as i32
        };
        let comic_title = old_meta.comic_title.clone();
        let client = api_client.0.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::{GetComicDetailRequest, GetEpisodesRequest};

            // 失败收口：报一次 RedownloadSkipped 就退出
            macro_rules! bail {
                ($ctx:expr, $err:expr) => {{
                    let error = $err;
                    tracing::error!("[{}] 更新检查失败: {}", comic_title, error);
                    $ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(RedownloadSkipped {
                            comic_title,
                            error: Some(error),
                        });
                    })
                    .await;
                    return;
                }};
            }

            // 第一跳：1 个请求拿当前 epsCount
            let remote_episodes = match client
                .request(GetComicDetailRequest {
                    comic_id: comic_id.clone(),
                })
                .await
            {
                Ok(resp) => resp.comic.eps_count,
                Err(e) => bail!(ctx, e.to_string()),
            };

            // 快路径：有基准且没变大 → 已是最新，一个请求收工
            if matches!((baseline, remote_episodes), (Some(base), now) if now > 0 && now <= base) {
                tracing::info!(
                    "[{}] 已是最新（epsCount 基准 {:?} → 现 {}），跳过更新",
                    comic_title,
                    baseline,
                    remote_episodes
                );
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.write_message(RedownloadSkipped {
                        comic_title,
                        error: None,
                    });
                })
                .await;
                return;
            }

            // 慢路径：epsCount 变大了，或压根没基准（老记录）。
            // 此时**不能**直接进完整下载流程——epsCount 会无缘无故漂移，
            // 光凭它变大就逐章拉图片列表，代价是几十上百个请求却常常一无所获。
            // 改为拉一次真实章节列表（分页，每页 ~40 条）核对：
            //   真实条数 <= 本地已下载章节数 → 确实没新章节，顺手把基准补上
            //   否则                         → 真有新章节，交给完整流程
            let mut real_episodes = 0;
            let mut page = 1;
            loop {
                match client
                    .request(GetEpisodesRequest {
                        comic_id: comic_id.clone(),
                        page,
                    })
                    .await
                {
                    Ok(resp) => {
                        real_episodes += resp.eps.docs.len() as i32;
                        if page >= resp.eps.pages {
                            break;
                        }
                        page += 1;
                    }
                    Err(e) => bail!(ctx, e.to_string()),
                }
            }

            if real_episodes <= local_episodes {
                tracing::info!(
                    "[{}] 已是最新（真实 {} 章 / 本地 {} 章；epsCount {:?} → {}，仅为字段漂移），\
                     刷新基准后跳过",
                    comic_title,
                    real_episodes,
                    local_episodes,
                    baseline,
                    remote_episodes
                );
                // 基准落库 + 同步内存索引，下次就能走一个请求的快路径
                if let Ok(mut meta) = DownloadTaskMeta::load_by_comic_id(&comic_id) {
                    meta.remote_eps_count = Some(remote_episodes);
                    if let Err(e) = meta.save() {
                        tracing::warn!("[{}] 写回更新基准失败: {}", comic_title, e);
                    }
                }
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world
                        .resource_mut::<DownloadedComicsIndex>()
                        .insert(comic_id, Some(remote_episodes));
                    ctx.world.write_message(RedownloadSkipped {
                        comic_title,
                        error: None,
                    });
                })
                .await;
                return;
            }

            tracing::info!(
                "[{}] 发现新章节（真实 {} 章 / 本地 {} 章），开始下载",
                comic_title,
                real_episodes,
                local_episodes
            );
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(RedownloadConfirmed {
                    comic_id,
                    new_base_path,
                });
            })
            .await;
        });
    }
}

/// 处理重新下载请求（检查更新/补全缺失）
///
/// 入口有二：强制更新的 `RedownloadRequest`（`force = true`），以及普通更新
/// 通过前置检查后的 `RedownloadConfirmed`。两者进入的下载流程完全一致。
fn handle_redownload(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<RedownloadRequest>,
    mut confirmed_messages: MessageReader<RedownloadConfirmed>,
    api_client: Res<ApiClientResource>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    use crate::resources::{DownloadState, DownloadTaskMeta};

    // 强制更新直接入队；普通更新只接受前置检查放行的那部分
    let pending: Vec<(String, Option<String>)> = messages
        .read()
        .filter(|event| event.force)
        .map(|event| (event.comic_id.clone(), event.new_base_path.clone()))
        .chain(
            confirmed_messages
                .read()
                .map(|event| (event.comic_id.clone(), event.new_base_path.clone())),
        )
        .collect();

    for (comic_id, new_base_path) in pending {
        // 检查是否已在下载
        if download_state.downloading_ids.contains(&comic_id) {
            tracing::warn!("漫画 {} 已在下载中，跳过重新下载", comic_id);
            continue;
        }

        // 从数据库查找下载任务元数据
        let Ok(old_meta) = DownloadTaskMeta::load_by_comic_id(&comic_id) else {
            tracing::warn!("找不到漫画 {} 的下载记录", comic_id);
            continue;
        };

        // 如果用户指定了新基础目录（原目录不存在的情况），重新计算路径
        let (save_path, custom_download_path) = if let Some(ref base_path) = new_base_path {
            // 从旧 save_path 提取漫画文件夹名
            let folder_name = std::path::Path::new(&old_meta.save_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| old_meta.comic_title.clone());
            let new_save = std::path::Path::new(base_path)
                .join("image")
                .join(&folder_name);
            tracing::info!(
                "使用用户选择的新目录: {} -> {}",
                old_meta.save_path,
                new_save.display()
            );
            (
                new_save.to_string_lossy().to_string(),
                Some(base_path.clone()),
            )
        } else {
            (
                old_meta.effective_download_path().to_string(),
                old_meta.custom_download_path.clone(),
            )
        };

        let comic_title = old_meta.comic_title.clone();
        let old_categories = old_meta.categories.clone();
        let old_tags = old_meta.tags.clone();

        // 检查是否已达到最大并发数
        let max_concurrent = AppSettings::global().read().max_concurrent_downloads;
        let should_start = download_state.downloading_ids.len() < max_concurrent;

        // 先移除已完成的任务（如果存在）
        download_state.remove_task(&comic_id);

        // 创建任务元数据，保留原有的分类和标签
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if !should_start {
            // 达到最大并发数，创建排队等待的任务
            tracing::info!(
                "已达到最大并发下载数 ({})，重新下载任务 {} 排队等待",
                max_concurrent,
                comic_title
            );
            let queued_meta = DownloadTaskMeta {
                comic_id: comic_id.clone(),
                comic_title: comic_title.clone(),
                save_path: save_path.clone(),
                episode_orders: old_meta.episode_orders.clone(),
                total_episodes: old_meta.total_episodes,
                state: DownloadState::Queued,
                created_at: now,
                updated_at: now,
                categories: old_categories,
                tags: old_tags,
                custom_download_path: custom_download_path.clone(),
                custom_auto_pack_cbz: old_meta.custom_auto_pack_cbz,
                // 保留旧基准，等这轮更新真正跑完再刷新
                remote_eps_count: old_meta.remote_eps_count,
            };
            // 保存到数据库（更新状态从 Completed 变为 Queued）
            if let Err(e) = queued_meta.save() {
                tracing::error!("保存排队任务到数据库失败: {}", e);
            }
            download_state.add_task(queued_meta);
            continue; // 等待其他任务完成后自动启动
        }

        tracing::info!("开始重新下载/检查更新: {} -> {}", comic_title, save_path);

        // 立即添加一个"准备中"状态的临时任务，避免任务从列表消失
        let temp_meta = DownloadTaskMeta {
            comic_id: comic_id.clone(),
            comic_title: comic_title.clone(),
            save_path: save_path.clone(),
            episode_orders: vec![],
            total_episodes: 0,
            state: DownloadState::Downloading {
                current_episode: 0,
                current_page: 0,
            },
            created_at: now,
            updated_at: now,
            categories: old_categories,
            tags: old_tags,
            custom_download_path: custom_download_path.clone(),
            custom_auto_pack_cbz: old_meta.custom_auto_pack_cbz,
            remote_eps_count: old_meta.remote_eps_count,
        };
        download_state.add_task(temp_meta);
        download_state.downloading_ids.insert(comic_id.clone());

        let client = api_client.0.clone();
        let comic_id_clone = comic_id.clone();
        let save_path_clone = save_path.clone();
        let custom_download_path_clone = custom_download_path.clone();
        let custom_auto_pack_cbz = old_meta.custom_auto_pack_cbz;

        // 启动异步任务：获取最新章节列表并开始下载
        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::{GetComicDetailRequest, GetEpisodesRequest};

            // 获取漫画详情
            let detail_request = GetComicDetailRequest {
                comic_id: comic_id_clone.clone(),
            };
            let comic = match client.request(detail_request).await {
                Ok(resp) => resp.comic,
                Err(e) => {
                    tracing::error!("获取漫画详情失败: {}", e);
                    return;
                }
            };

            // 获取所有章节
            let mut all_episodes = Vec::new();
            let mut page = 1;
            loop {
                let episodes_request = GetEpisodesRequest {
                    comic_id: comic_id_clone.clone(),
                    page,
                };
                match client.request(episodes_request).await {
                    Ok(resp) => {
                        let eps = &resp.eps;
                        all_episodes.extend(eps.docs.clone());
                        if page >= eps.pages {
                            break;
                        }
                        page += 1;
                    }
                    Err(e) => {
                        tracing::error!("获取章节列表失败: {}", e);
                        return;
                    }
                }
            }

            // 章节顺序（正序）
            let mut episode_orders: Vec<i32> = all_episodes.iter().map(|e| e.order).collect();
            episode_orders.sort();

            let total_episodes = episode_orders.len() as i32;

            tracing::info!("重新下载: {} 共 {} 章节", comic.title, total_episodes);

            // 创建新的元数据并保存
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let meta = DownloadTaskMeta {
                comic_id: comic_id_clone.clone(),
                comic_title: comic.title.clone(),
                save_path: save_path_clone.clone(),
                episode_orders: episode_orders.clone(),
                total_episodes,
                state: DownloadState::Downloading {
                    current_episode: 1,
                    current_page: 0,
                },
                created_at: now,
                updated_at: now,
                categories: comic.categories.clone(),
                tags: comic.tags.clone(),
                custom_download_path: custom_download_path_clone,
                custom_auto_pack_cbz,
                // 刚取到的详情就是新基准——这轮更新之后再变大才算又有新章节
                remote_eps_count: Some(comic.eps_count),
            };

            if let Err(e) = meta.save() {
                tracing::warn!("保存元数据失败: {}", e);
            }

            // 发送下载请求到主线程，更新任务并获取控制器
            let control = ctx
                .run_on_main_thread(move |ctx| {
                    let mut download_state = ctx.world.resource_mut::<DownloadManagerState>();

                    // 更新现有任务（替换临时任务）
                    if let Some(fsm) = download_state.find_task_mut(&meta.comic_id) {
                        // 更新元数据
                        fsm.meta = meta.clone();
                        fsm.get_control()
                    } else {
                        // 如果任务不存在（理论上不应该发生），添加新任务
                        download_state.add_task(meta.clone());
                        download_state.downloading_ids.insert(meta.comic_id.clone());
                        download_state
                            .find_task(&meta.comic_id)
                            .map(|fsm| fsm.get_control())
                            .unwrap_or_else(|| std::sync::Arc::new(SharedTaskControl::new()))
                    }
                })
                .await;

            // 在当前异步上下文中执行下载（内联下载逻辑）
            execute_download_task(
                &mut ctx,
                client,
                comic_id_clone,
                comic.title.clone(),
                save_path_clone,
                episode_orders,
                total_episodes,
                control,
            )
            .await;
        });
    }
}

// ==================== 搜索处理 ====================

/// 处理搜索请求
fn handle_search_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<SearchComicsRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let keyword = event.keyword.clone();
        let page = event.page;
        let sort = event.sort.clone();
        let categories = event.categories.clone();

        tracing::info!(
            "搜索请求: keyword={}, page={}, sort={}, categories={:?}",
            keyword,
            page,
            sort,
            categories
        );

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::SearchComicsRequest;

            let request = SearchComicsRequest {
                keyword: keyword.clone(),
                page,
                sort,
                categories,
            };

            match client.request(request).await {
                Ok(response) => {
                    let count = response.comics.docs.len();
                    let total_pages = response.comics.pages;
                    tracing::info!(
                        "搜索成功: keyword={}, 结果数={}, 总页数={}",
                        keyword,
                        count,
                        total_pages
                    );
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(SearchResultsLoadedEvent {
                            comics: response.comics.docs,
                            total_pages: response.comics.pages,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    tracing::error!("搜索失败: {}", error);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(SearchFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

// ==================== 热词处理 ====================

/// 处理加载热词请求
fn handle_load_keywords(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadKeywordsRequest>,
    api_client: Res<ApiClientResource>,
    search_state: Res<crate::resources::SearchState>,
) {
    for _event in messages.read() {
        if search_state.hot_keywords_loaded {
            continue;
        }
        let client = api_client.0.clone();

        tracing::info!("加载热门搜索关键词");

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comic::GetKeywordsRequest;

            match client.request(GetKeywordsRequest).await {
                Ok(response) => {
                    let count = response.keywords.len();
                    tracing::info!("热词加载成功，共 {} 个", count);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(KeywordsLoadedEvent {
                            keywords: response.keywords,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    tracing::error!("热词加载失败: {}", error);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(KeywordsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理热词加载响应
fn handle_keywords_response(
    mut loaded_messages: MessageReader<KeywordsLoadedEvent>,
    mut failed_messages: MessageReader<KeywordsLoadFailedEvent>,
    mut search_state: ResMut<SearchState>,
) {
    for event in loaded_messages.read() {
        search_state.hot_keywords = event.keywords.clone();
        search_state.hot_keywords_loaded = true;
        search_state.needs_rebuild = true;
        tracing::info!("热词已更新: {} 个", search_state.hot_keywords.len());
    }

    for event in failed_messages.read() {
        tracing::error!("热词加载失败，不影响搜索功能: {}", event.error);
        // 加载失败不阻塞功能，仅标记已尝试加载
        search_state.hot_keywords_loaded = true;
    }
}

/// 对漫画列表应用屏蔽过滤（只克隆保留项；语义统一走
/// CompiledFilter，含繁简转换）
fn apply_block_filter(comics: &[picacg_api::models::Comic]) -> Vec<picacg_api::models::Comic> {
    let filter = crate::utils::content_filter::CompiledFilter::from_settings();
    if filter.is_noop() {
        return comics.to_vec();
    }
    let before = comics.len();
    let filtered = filter.filter_comics_cloned(comics);
    let after = filtered.len();
    if before != after {
        tracing::debug!(
            "屏蔽过滤: {} -> {} (过滤掉 {} 部)",
            before,
            after,
            before - after
        );
    }
    filtered
}

/// 处理搜索响应
fn handle_search_response(
    mut loaded_messages: MessageReader<SearchResultsLoadedEvent>,
    mut failed_messages: MessageReader<SearchFailedEvent>,
    mut search_state: ResMut<SearchState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        search_state.is_loading = false;
        search_state.has_searched = true;
        let filtered = apply_block_filter(&event.comics);
        search_state.results = filtered;
        search_state.total_pages = event.total_pages;
        search_state.error = None;
        search_state.needs_rebuild = true;

        // 触发加载封面图片
        for comic in &search_state.results {
            image_messages.write(LoadImageRequest {
                url: comic.thumb.url(),
            });
        }
    }

    for event in failed_messages.read() {
        search_state.is_loading = false;
        search_state.has_searched = true;
        search_state.error = Some(event.error.clone());
        search_state.needs_rebuild = true;
    }
}

// ==================== 排行榜处理 ====================

/// 处理加载排行榜请求
fn handle_load_rankings(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadRankingsRequest>,
    api_client: Res<ApiClientResource>,
    mut rankings_state: ResMut<RankingsState>,
) {
    for event in messages.read() {
        rankings_state.is_loading = true;
        rankings_state.error = None;

        let client = api_client.0.clone();
        let time_type = event.time_type;

        tracing::info!("加载 {} 榜数据", time_type.display_name());

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::rank::GetRankingsRequest;

            let request = GetRankingsRequest { time_type };

            match client.request(request).await {
                Ok(response) => {
                    let count = response.comics.len();
                    tracing::info!(
                        "{} 榜加载成功，共 {} 部漫画",
                        time_type.display_name(),
                        count
                    );
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(RankingsLoadedEvent {
                            time_type,
                            comics: response.comics,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    tracing::error!("{} 榜加载失败: {}", time_type.display_name(), error);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(RankingsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理排行榜加载响应
fn handle_rankings_response(
    mut loaded_messages: MessageReader<RankingsLoadedEvent>,
    mut failed_messages: MessageReader<RankingsLoadFailedEvent>,
    mut rankings_state: ResMut<RankingsState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        rankings_state.is_loading = false;
        let filtered = apply_block_filter(&event.comics);
        rankings_state.set_comics(event.time_type, filtered);

        // 触发加载封面图片
        for comic in rankings_state.current_comics() {
            image_messages.write(LoadImageRequest {
                url: comic.thumb.url(),
            });
        }
    }

    for event in failed_messages.read() {
        rankings_state.is_loading = false;
        rankings_state.error = Some(event.error.clone());
    }
}

// ==================== 骑士榜处理 ====================

/// 处理加载骑士榜请求
fn handle_load_knight_rankings(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadKnightRankingsRequest>,
    api_client: Res<ApiClientResource>,
    mut rankings_state: ResMut<RankingsState>,
) {
    for _event in messages.read() {
        if rankings_state.knight_loading {
            continue;
        }
        rankings_state.knight_loading = true;
        rankings_state.knight_error = None;

        let client = api_client.0.clone();

        tracing::info!("加载骑士榜数据");

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::rank::GetKnightRankingsRequest;

            let request = GetKnightRankingsRequest;

            match client.request(request).await {
                Ok(response) => {
                    let count = response.users.len();
                    tracing::info!("骑士榜加载成功，共 {} 位骑士", count);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(KnightRankingsLoadedEvent {
                            users: response.users,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    tracing::error!("骑士榜加载失败: {}", error);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(KnightRankingsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理骑士榜加载响应
fn handle_knight_rankings_response(
    mut loaded_messages: MessageReader<KnightRankingsLoadedEvent>,
    mut failed_messages: MessageReader<KnightRankingsLoadFailedEvent>,
    mut rankings_state: ResMut<RankingsState>,
) {
    for event in loaded_messages.read() {
        rankings_state.knight_loading = false;
        rankings_state.knight_error = None;
        rankings_state.knight_users = event.users.clone();
    }

    for event in failed_messages.read() {
        rankings_state.knight_loading = false;
        rankings_state.knight_error = Some(event.error.clone());
    }
}

// ==================== 启动时自动恢复下载 ====================

/// 加载未完成的下载任务（Startup 阶段）
fn setup_download_manager(
    mut download_state: ResMut<DownloadManagerState>,
    mut downloaded_index: ResMut<DownloadedComicsIndex>,
) {
    download_state.load_incomplete_tasks();
    tracing::info!(
        "加载未完成的下载任务: {} 个",
        download_state.fsm_tasks.len()
    );
    // 封面角标的数据源，随下载任务表一起初始化
    downloaded_index.reload();
}

/// 下载队列管理系统
///
/// 功能：
/// 1. 监听 `UserLoggedInEvent`，登录后激活下载管理
/// 2. 首次登录时：如果启用了 `auto_resume_downloads`，恢复所有暂停的任务
/// 3. 持续运行：管理并发下载数量，有空位时自动启动 Waiting 状态的任务
fn download_queue_manager(
    mut is_logged_in: Local<bool>,
    mut has_auto_resumed: Local<bool>,
    mut user_logged_in_events: MessageReader<UserLoggedInEvent>,
    download_state: Res<DownloadManagerState>,
    mut resume_messages: MessageWriter<ResumeDownloadRequest>,
) {
    // 监听登录事件
    for _ in user_logged_in_events.read() {
        *is_logged_in = true;
        tracing::debug!("收到 UserLoggedInEvent，下载队列管理已激活");
    }

    // 未登录时不处理
    if !*is_logged_in {
        return;
    }

    let settings = AppSettings::global().read();
    let max_concurrent = settings.max_concurrent_downloads;

    // 首次登录后的自动恢复（只执行一次）
    if !*has_auto_resumed {
        *has_auto_resumed = true;

        if settings.auto_resume_downloads {
            // 启用了自动恢复：恢复所有 Paused/Waiting 任务
            let mut resumed_count = 0;
            for fsm in &download_state.fsm_tasks {
                // 直接判 FSM 状态，不做 UI 深拷贝
                if matches!(
                    fsm.meta.state,
                    crate::resources::DownloadState::Paused { .. }
                        | crate::resources::DownloadState::Queued
                ) {
                    resume_messages.write(ResumeDownloadRequest {
                        comic_id: fsm.meta.comic_id.clone(),
                    });
                    resumed_count += 1;
                }
            }
            if resumed_count > 0 {
                tracing::info!("登录后自动恢复下载: {} 个任务", resumed_count);
            }
        }
        return; // 首次恢复后返回，让任务状态先更新
    }

    // 并发下载管理：当有空闲槽位时，自动启动排队中的任务
    let current_downloading = download_state.downloading_ids.len();
    if current_downloading >= max_concurrent {
        return;
    }

    let available_slots = max_concurrent - current_downloading;
    let mut started = 0;

    for fsm in &download_state.fsm_tasks {
        if started >= available_slots {
            break;
        }

        // 跳过已经在下载的
        if download_state.downloading_ids.contains(&fsm.meta.comic_id) {
            continue;
        }

        // 只自动启动排队中的任务（用户手动暂停的不自动恢复）；直接判 FSM
        // 状态免深拷贝
        if matches!(fsm.meta.state, crate::resources::DownloadState::Queued) {
            resume_messages.write(ResumeDownloadRequest {
                comic_id: fsm.meta.comic_id.clone(),
            });
            started += 1;
            tracing::info!(
                "自动启动排队任务: {} (槽位 {}/{})",
                fsm.meta.comic_title,
                current_downloading + started,
                max_concurrent
            );
        }
    }
}

// ==================== 注册处理 ====================

/// 处理注册请求
fn handle_register_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<RegisterRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let email = event.email.clone();
        let password = event.password.clone();
        let name = event.name.clone();
        let birthday = event.birthday.clone();
        let gender = event.gender.clone();
        let question1 = event.question1.clone();
        let question2 = event.question2.clone();
        let question3 = event.question3.clone();
        let answer1 = event.answer1.clone();
        let answer2 = event.answer2.clone();
        let answer3 = event.answer3.clone();
        let client = api_client.0.clone();

        tracing::info!("注册请求: email={}", email);

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::RegisterRequest;

            let request = RegisterRequest {
                email,
                password,
                name,
                birthday,
                gender,
                question1,
                question2,
                question3,
                answer1,
                answer2,
                answer3,
            };

            let result = match client.request(request).await {
                Ok(response) => {
                    tracing::info!("注册成功: {}", response.message);
                    Ok(response.message)
                }
                Err(e) => {
                    tracing::error!("注册失败: {}", e);
                    Err(e.to_string())
                }
            };

            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(RegisterResponseEvent { result });
            })
            .await;
        });
    }
}

// ==================== CBZ 打包处理 ====================

/// 判断是否为图片文件
fn is_image_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            matches!(
                ext_lower.as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp"
            )
        })
        .unwrap_or(false)
}

/// 递归收集图片文件
/// 返回 (相对路径, 绝对路径) 的列表
fn collect_image_files(
    source_dir: &std::path::Path,
) -> std::io::Result<Vec<(String, std::path::PathBuf)>> {
    let mut entries = Vec::new();

    fn walk_dir(
        dir: &std::path::Path,
        base: &std::path::Path,
        entries: &mut Vec<(String, std::path::PathBuf)>,
    ) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, base, entries)?;
                } else if is_image_file(&path) {
                    // 计算相对路径
                    let relative = path
                        .strip_prefix(base)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    // 将 Windows 路径分隔符转换为 /
                    let archive_name = relative.replace('\\', "/");
                    entries.push((archive_name, path));
                }
            }
        }
        Ok(())
    }

    walk_dir(source_dir, source_dir, &mut entries)?;
    Ok(entries)
}

/// 创建 CBZ 文件
/// 使用 Stored 模式（图片本身已压缩，无需再压缩）
fn create_cbz_package(source_path: &str, comic_title: &str) -> Result<String, String> {
    use std::io::Write;

    use zip::{CompressionMethod, write::SimpleFileOptions};

    let source_dir = std::path::Path::new(source_path);
    if !source_dir.exists() {
        return Err(format!("源目录不存在: {}", source_path));
    }

    // 创建 CBZ 输出目录
    // 从 source_path 推导：base/image/漫画名 → base/cbz
    // 这样自定义下载路径的 CBZ 也会保存在同一基础目录下
    let cbz_dir = source_dir
        .parent() // base/image
        .and_then(|p| p.parent()) // base
        .map(|p| p.join("cbz"))
        .unwrap_or_else(get_cbz_output_path);
    if let Err(e) = std::fs::create_dir_all(&cbz_dir) {
        return Err(format!("创建 CBZ 目录失败: {}", e));
    }

    // CBZ 文件路径
    let cbz_filename = format!("{}.cbz", sanitize_filename(comic_title));
    let cbz_path = cbz_dir.join(&cbz_filename);

    // 收集所有图片文件
    let mut entries = collect_image_files(source_dir).map_err(|e| {
        tracing::error!("遍历目录失败: {} - {}", source_path, e);
        e.to_string()
    })?;
    if entries.is_empty() {
        return Err(format!("没有找到图片文件（源目录: {}）", source_path));
    }

    // 按文件名排序（确保章节和页面顺序正确）
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    tracing::info!("开始打包 CBZ: {} ({} 个文件)", cbz_filename, entries.len());

    // 创建 ZIP 文件
    let file = std::fs::File::create(&cbz_path).map_err(|e| format!("创建 CBZ 文件失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);

    // 使用 Stored 模式（不压缩，图片本身已压缩）
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    // 写入文件
    for (archive_name, file_path) in &entries {
        let data = std::fs::read(file_path).map_err(|e| {
            tracing::warn!("读取文件失败: {} - {}", file_path.display(), e);
            format!("读取文件失败: {}", e)
        })?;

        zip.start_file(archive_name, options)
            .map_err(|e| format!("添加文件到 ZIP 失败: {}", e))?;
        zip.write_all(&data)
            .map_err(|e| format!("写入 ZIP 数据失败: {}", e))?;
    }

    zip.finish()
        .map_err(|e| format!("完成 ZIP 文件失败: {}", e))?;

    let cbz_path_str = cbz_path.to_string_lossy().to_string();
    tracing::info!("CBZ 打包完成: {}", cbz_path_str);

    Ok(cbz_path_str)
}

/// 处理 CBZ 打包请求
fn handle_cbz_package_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<CbzPackageRequest>,
    mut packaging: ResMut<CbzPackagingState>,
) {
    for event in messages.read() {
        let comic_id = event.comic_id.clone();
        let comic_title = event.comic_title.clone();
        let source_path = event.source_path.clone();

        tracing::info!("收到 CBZ 打包请求: {}", comic_title);
        packaging.in_flight += 1;

        // 使用 spawn_blocking 在后台线程执行 IO 密集型操作
        runtime.spawn_background_task(move |mut ctx| async move {
            // 在阻塞线程中执行打包
            let result =
                tokio::task::spawn_blocking(move || create_cbz_package(&source_path, &comic_title))
                    .await;

            // 处理结果
            match result {
                Ok(Ok(cbz_path)) => {
                    let comic_id_clone = comic_id.clone();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(CbzPackageCompletedEvent {
                            comic_id: comic_id_clone,
                            cbz_path,
                        });
                    })
                    .await;
                }
                Ok(Err(error)) => {
                    let comic_id_clone = comic_id.clone();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(CbzPackageFailedEvent {
                            comic_id: comic_id_clone,
                            error,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = format!("打包任务执行失败: {}", e);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(CbzPackageFailedEvent { comic_id, error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理 CBZ 打包完成
fn handle_cbz_package_completed(
    mut messages: MessageReader<CbzPackageCompletedEvent>,
    mut packaging: ResMut<CbzPackagingState>,
) {
    for event in messages.read() {
        packaging.finish_one();
        tracing::info!("CBZ 打包完成: {} -> {}", event.comic_id, event.cbz_path);

        // 检查是否需要删除原图文件夹
        let delete_images = AppSettings::global().read().delete_images_after_cbz;
        if delete_images {
            // 从 CBZ 路径推断原图路径
            // CBZ: Downloads/CBZ/漫画标题.cbz
            // Images: Downloads/Images/漫画标题/
            let cbz_path = std::path::Path::new(&event.cbz_path);
            if let Some(cbz_filename) = cbz_path.file_stem() {
                let images_dir = get_images_download_path().join(cbz_filename);
                if images_dir.exists() {
                    match std::fs::remove_dir_all(&images_dir) {
                        Ok(()) => {
                            tracing::info!("已删除原图文件夹: {}", images_dir.display());
                        }
                        Err(e) => {
                            tracing::warn!("删除原图文件夹失败: {} - {}", images_dir.display(), e);
                        }
                    }
                }
            }
        }
    }
}

/// 处理 CBZ 打包失败
fn handle_cbz_package_failed(
    mut messages: MessageReader<CbzPackageFailedEvent>,
    mut packaging: ResMut<CbzPackagingState>,
) {
    for event in messages.read() {
        packaging.finish_one();
        tracing::error!("CBZ 打包失败: {} - {}", event.comic_id, event.error);
    }
}

// ==================== 下载完成后自动退出 ====================

/// CBZ 打包在途计数
///
/// 打包在后台线程写文件，进程若在此时退出会留下半截 .cbz。
/// 「下载完成后退出」据此等打包收尾。
#[derive(Resource, Default)]
pub struct CbzPackagingState {
    /// 已发出但尚未收到完成/失败回执的打包任务数
    pub in_flight: usize,
}

impl CbzPackagingState {
    /// 一个打包任务收尾（成功或失败都算）
    fn finish_one(&mut self) {
        self.in_flight = self.in_flight.saturating_sub(1);
    }
}

/// 「下载全部完成后退出」
///
/// 判定"还有活儿"= 有任务处于下载中/排队中，或 CBZ 还在打包。
/// **已暂停/已失败的任务不算**——它们不会自己往前走，等下去等于永不退出。
///
/// `was_busy` 保证只在"本次运行确实跑过下载"之后才触发：刚启动时队列本来
/// 就空，不能直接退出。退出前保存窗口几何，与用户主动关闭走同一套收尾。
fn exit_after_downloads_complete(
    download_state: Res<DownloadManagerState>,
    packaging: Res<CbzPackagingState>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut was_busy: Local<bool>,
    mut exit_messages: MessageWriter<AppExit>,
) {
    let busy = packaging.in_flight > 0
        || !download_state.downloading_ids.is_empty()
        || download_state.fsm_tasks.iter().any(|fsm| {
            matches!(
                fsm.meta.state,
                DownloadState::Downloading { .. } | DownloadState::Queued
            )
        });

    if busy {
        *was_busy = true;
        return;
    }
    if !*was_busy {
        return;
    }

    // 设置读到最后一刻，避免下载途中改设置后行为与预期不符
    if !AppSettings::global().read().exit_after_downloads {
        *was_busy = false;
        return;
    }

    *was_busy = false;
    tracing::info!("下载队列已清空，按设置自动退出");
    if let Ok(window) = window_query.single() {
        crate::systems::save_window_geometry_to_config(window);
    }
    exit_messages.write(AppExit::Success);
}

/// 自动收集标签到缓存（监听所有漫画状态变化）
fn update_cached_tags(
    mut cached_tags: ResMut<CachedTagsState>,
    comics_state: Res<ComicsListState>,
    search_state: Res<SearchState>,
    rankings_state: Res<RankingsState>,
    favorites_state: Res<FavoritesState>,
    detail_state: Res<ComicDetailState>,
    mut initialized: Local<bool>,
) {
    let any_changed = comics_state.is_changed()
        || search_state.is_changed()
        || rankings_state.is_changed()
        || favorites_state.is_changed()
        || detail_state.is_changed();

    // 首次运行从数据库加载历史标签
    if !*initialized {
        *initialized = true;
        use picacg_db::{get_all_unique_tags_async, get_pool, run_db_operation};
        let pool = get_pool();
        run_db_operation(async move {
            match get_all_unique_tags_async(&pool).await {
                Ok(tags) => {
                    tracing::info!("从数据库加载了 {} 个历史标签", tags.len());
                    // 注：run_db_operation 无法回写到 Resource，后续由 API
                    // 数据补充
                }
                Err(e) => tracing::warn!("加载历史标签失败: {}", e),
            }
        });
        // 即使 DB 加载是异步的，也继续往下收集当前内存中的标签
    }

    if !any_changed && *initialized {
        return;
    }

    let mut tags = std::collections::BTreeSet::new();
    // 保留已有标签
    for tag in &cached_tags.tags {
        tags.insert(tag.clone());
    }

    // 从各页面收集
    for comic in &comics_state.comics {
        tags.extend(comic.tags.iter().cloned());
    }
    for comic in &search_state.results {
        tags.extend(comic.tags.iter().cloned());
    }
    for comic in &rankings_state.h24_comics {
        tags.extend(comic.tags.iter().cloned());
    }
    for comic in &rankings_state.d7_comics {
        tags.extend(comic.tags.iter().cloned());
    }
    for comic in &rankings_state.d30_comics {
        tags.extend(comic.tags.iter().cloned());
    }
    for comic in &favorites_state.comics {
        tags.extend(comic.tags.iter().cloned());
    }
    if let Some(ref comic) = detail_state.comic {
        tags.extend(comic.tags.iter().cloned());
    }

    let new_tags: Vec<String> = tags.into_iter().collect();
    if new_tags.len() != cached_tags.tags.len() {
        cached_tags.tags = new_tags;
    }
}

// ==================== 个人资料处理 ====================

/// 处理加载用户个人资料请求
fn handle_load_user_profile(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadUserProfileRequest>,
    api_client: Res<ApiClientResource>,
    mut profile_state: ResMut<UserProfileState>,
) {
    for _request in messages.read() {
        profile_state.is_loading = true;
        profile_state.error = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::GetUserInfoRequest;

            match client.request(GetUserInfoRequest).await {
                Ok(response) => {
                    let user = response.user;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(UserProfileLoadedEvent { user });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(UserProfileLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

// ==================== 历史记录处理 ====================

/// 处理加载历史记录请求
fn handle_load_history(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadHistoryRequest>,
    mut history_state: ResMut<HistoryState>,
) {
    for _event in messages.read() {
        history_state.is_loading = true;
        history_state.error = None;

        let pool = picacg_db::get_pool();

        runtime.spawn_background_task(|mut ctx| async move {
            let records_result = picacg_db::get_all_histories_async(&pool).await;
            let count_result = picacg_db::get_history_count_async(&pool).await;

            match (records_result, count_result) {
                (Ok(records), Ok(count)) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(HistoryLoadedEvent {
                            records,
                            total_count: count,
                        });
                    })
                    .await;
                }
                (Err(e), _) | (_, Err(e)) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(HistoryLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理保存历史记录请求
fn handle_save_history(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<SaveHistoryRequest>,
) {
    for event in messages.read() {
        let history = picacg_db::DbHistory::with_info(
            event.comic_id.clone(),
            event.comic_title.clone(),
            event.thumb_url.clone(),
            event.last_eps_order as i64,
            event.last_eps_title.clone(),
            event.last_page as i64,
        );

        let pool = picacg_db::get_pool();

        runtime.spawn_background_task(|_ctx| async move {
            if let Err(e) = picacg_db::upsert_history_async(&pool, &history).await {
                tracing::warn!("保存阅读历史失败: {}", e);
            } else {
                tracing::debug!(
                    "保存阅读历史: comic_id={}, eps={}, page={}",
                    history.book_id,
                    history.last_eps,
                    history.last_page
                );
            }
        });
    }
}

/// 处理删除历史记录请求
fn handle_delete_history(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<DeleteHistoryRequest>,
    mut history_state: ResMut<HistoryState>,
) {
    for event in messages.read() {
        let comic_id = event.comic_id.clone();

        // 立即从内存中移除，UI 即时响应
        history_state.records.retain(|r| r.book_id != comic_id);
        history_state.total_count = history_state.records.len() as i64;

        let pool = picacg_db::get_pool();

        runtime.spawn_background_task(|_ctx| async move {
            if let Err(e) = picacg_db::delete_history_async(&pool, &comic_id).await {
                tracing::warn!("删除阅读历史失败: {}", e);
            }
        });
    }
}

/// 处理清空所有历史记录请求
fn handle_clear_all_history(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<ClearAllHistoryRequest>,
    mut history_state: ResMut<HistoryState>,
) {
    for _event in messages.read() {
        // 立即清空内存，UI 即时响应
        history_state.records.clear();
        history_state.total_count = 0;

        let pool = picacg_db::get_pool();

        runtime.spawn_background_task(|_ctx| async move {
            if let Err(e) = picacg_db::clear_all_history_async(&pool).await {
                tracing::warn!("清空阅读历史失败: {}", e);
            } else {
                tracing::info!("已清空所有阅读历史");
            }
        });
    }
}

// ==================== 点赞记录处理 ====================

/// 处理加载点赞记录请求
fn handle_load_like_records(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadLikeRecordsRequest>,
    mut like_records_state: ResMut<crate::resources::LikeRecordsState>,
) {
    for _event in messages.read() {
        like_records_state.is_loading = true;
        like_records_state.error = None;

        let pool = picacg_db::get_pool();

        runtime.spawn_background_task(|mut ctx| async move {
            let records_result = picacg_db::get_all_like_records_async(&pool).await;
            let count_result = picacg_db::get_like_count_async(&pool).await;

            match (records_result, count_result) {
                (Ok(records), Ok(count)) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(LikeRecordsLoadedEvent {
                            records,
                            total_count: count,
                        });
                    })
                    .await;
                }
                (Err(e), _) | (_, Err(e)) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(LikeRecordsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理保存点赞记录请求
fn handle_save_like_record(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<SaveLikeRecordRequest>,
) {
    for event in messages.read() {
        let comic_id = event.comic_id.clone();
        let comic_title = event.comic_title.clone();
        let thumb_url = event.thumb_url.clone();

        let pool = picacg_db::get_pool();

        runtime.spawn_background_task(|_ctx| async move {
            let thumb = if thumb_url.is_empty() {
                None
            } else {
                Some(thumb_url.as_str())
            };
            if let Err(e) =
                picacg_db::insert_like_record_async(&pool, &comic_id, &comic_title, thumb).await
            {
                tracing::warn!("保存点赞记录失败: {}", e);
            } else {
                tracing::debug!("保存点赞记录: comic_id={}", comic_id);
            }
        });
    }
}

/// 处理删除点赞记录请求
fn handle_delete_like_record(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<DeleteLikeRecordRequest>,
    mut like_records_state: ResMut<crate::resources::LikeRecordsState>,
) {
    for event in messages.read() {
        let comic_id = event.comic_id.clone();

        // 立即从内存中移除，UI 即时响应
        like_records_state
            .records
            .retain(|r| r.comic_id != comic_id);
        like_records_state.total_count = like_records_state.records.len() as i64;

        let pool = picacg_db::get_pool();

        runtime.spawn_background_task(|_ctx| async move {
            if let Err(e) = picacg_db::delete_like_record_async(&pool, &comic_id).await {
                tracing::warn!("删除点赞记录失败: {}", e);
            }
        });
    }
}

// ==================== 版本更新检查 ====================

/// GitHub Releases API 地址
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/arxxyr/picacg-rust/releases/latest";

/// 处理检查更新请求
/// 启动后自动检查一次更新（设置开启时）
///
/// 用 `Local<bool>` 保证整个进程只查一次。不等登录——检查更新走的是 GitHub
/// Releases API，与 PicACG 账号无关，没必要挂在登录之后。
fn auto_check_update_on_startup(
    mut has_run: Local<bool>,
    mut check_messages: MessageWriter<CheckUpdateRequest>,
) {
    if *has_run {
        return;
    }
    *has_run = true;

    if !AppSettings::global().read().auto_check_update {
        return;
    }
    tracing::info!("启动自动检查更新");
    check_messages.write(CheckUpdateRequest);
}

fn handle_check_update(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<CheckUpdateRequest>,
    mut update_state: ResMut<crate::resources::UpdateCheckState>,
) {
    for _event in messages.read() {
        if update_state.is_checking {
            continue;
        }

        update_state.is_checking = true;
        update_state.error = None;
        update_state.has_update = None;
        update_state.latest_version = None;
        update_state.release_notes = None;
        update_state.download_url = None;
        update_state.asset_url = None;
        update_state.checksum_url = None;

        let current_version = env!("CARGO_PKG_VERSION").to_string();

        runtime.spawn_background_task(|mut ctx| async move {
            let result = check_github_latest_release(&current_version).await;

            ctx.run_on_main_thread(move |ctx| match result {
                Ok(info) => {
                    ctx.world.write_message(CheckUpdateResponse {
                        latest_version: info.latest_version,
                        has_update: info.has_update,
                        release_notes: info.release_notes,
                        download_url: info.download_url,
                        asset_url: info.asset_url,
                        checksum_url: info.checksum_url,
                    });
                }
                Err(e) => {
                    ctx.world.write_message(CheckUpdateFailedEvent { error: e });
                }
            })
            .await;
        });
    }
}

/// 版本更新信息
struct UpdateInfo {
    latest_version: String,
    has_update: bool,
    release_notes: Option<String>,
    download_url: Option<String>,
    /// 本平台产物的直链（用于自动下载）
    asset_url: Option<String>,
    /// 该产物的 `.sha256` 直链
    checksum_url: Option<String>,
}

/// 请求 GitHub Releases API 获取最新版本
async fn check_github_latest_release(current_version: &str) -> Result<UpdateInfo, String> {
    // 必须走代理：GitHub 在部分地区不可直连，不带代理的话检查更新永远超时。
    // 锁在作用域里取完即放，别把 guard 带过 await
    let proxy_url = AppSettings::global().read().proxy.to_proxy_url();

    let mut builder = reqwest::Client::builder()
        .user_agent("picacg-rust")
        .timeout(std::time::Duration::from_secs(15));
    if let Some(ref url) = proxy_url {
        let proxy = reqwest::Proxy::all(url).map_err(|e| format!("代理配置无效: {e}"))?;
        builder = builder.proxy(proxy);
    }
    let client = builder
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(GITHUB_RELEASES_URL)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 从 tag_name 提取版本号（去除 v 前缀和 +build 后缀）
    let tag_name = body["tag_name"]
        .as_str()
        .ok_or_else(|| "响应中缺少 tag_name 字段".to_string())?;
    let latest_version = tag_name
        .strip_prefix('v')
        .unwrap_or(tag_name)
        .split('+')
        .next()
        .unwrap_or(tag_name)
        .to_string();

    let release_notes = body["body"].as_str().map(|s| s.to_string());
    let download_url = body["html_url"].as_str().map(|s| s.to_string());

    // 挑出本平台的产物直链与它的校验和
    //
    // 产物命名是 `{项目名}-{版本}-{平台}.{扩展名}`（CLAUDE.md §9），
    // 按平台后缀匹配即可。校验和是同名 + `.sha256`。
    let (asset_url, checksum_url) = pick_platform_asset(&body);

    let has_update = compare_versions(&latest_version, current_version);

    tracing::info!(
        "版本检查完成: 当前={}, 最新={}, 有更新={}",
        current_version,
        latest_version,
        has_update
    );

    Ok(UpdateInfo {
        latest_version,
        has_update,
        release_notes,
        download_url,
        asset_url,
        checksum_url,
    })
}

/// 从 release 的 assets 里挑出本平台的产物与其 `.sha256`
///
/// 返回 `(产物直链, 校验和直链)`。任一缺失都返回 None——
/// 没有校验和就不该自动下载执行。
fn pick_platform_asset(body: &serde_json::Value) -> (Option<String>, Option<String>) {
    // 与 CI 打包矩阵的 platform 字段对齐
    let platform = if cfg!(target_os = "macos") {
        "macos-arm64"
    } else if cfg!(target_os = "windows") {
        "windows-x64"
    } else {
        "linux-x64"
    };

    let Some(assets) = body["assets"].as_array() else {
        return (None, None);
    };

    let url_of = |predicate: &dyn Fn(&str) -> bool| -> Option<String> {
        assets.iter().find_map(|a| {
            let name = a["name"].as_str()?;
            predicate(name)
                .then(|| a["browser_download_url"].as_str().map(str::to_string))
                .flatten()
        })
    };

    let asset = url_of(&|n: &str| n.contains(platform) && !n.ends_with(".sha256"));
    let checksum = url_of(&|n: &str| n.contains(platform) && n.ends_with(".sha256"));
    (asset, checksum)
}

/// 简单的语义化版本比较：latest > current 则返回 true
fn compare_versions(latest: &str, current: &str) -> bool {
    // 先切掉 build metadata（`+abc1234`）与预发布后缀（`-rc.1`）再逐段比数字。
    //
    // ⚠️ 本项目的发版格式就是 `v{version}+{commit}`（见 CLAUDE.md §9），
    // 旧实现用 `filter_map(parse::<u64>)` 直接把 `0+abc1234`
    // 这种段**静默丢掉**， `0.5.1+abc` 会被解析成 [0, 5]，与 [0, 5, 0]
    // 比出「无更新」—— patch 位一带后缀就误判。
    let core = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split(['+', '-'])
            .next()
            .unwrap_or("")
            .split('.')
            .map(|s| s.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let latest_parts = core(latest);
    let current_parts = core(current);

    // 逐段比较，缺失的段视为 0
    let max_len = latest_parts.len().max(current_parts.len());
    for i in 0..max_len {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

/// 处理检查更新响应
fn handle_check_update_response(
    mut messages: MessageReader<CheckUpdateResponse>,
    mut update_state: ResMut<crate::resources::UpdateCheckState>,
) {
    for event in messages.read() {
        update_state.is_checking = false;
        update_state.latest_version = Some(event.latest_version.clone());
        update_state.has_update = Some(event.has_update);
        update_state.release_notes.clone_from(&event.release_notes);
        update_state.download_url.clone_from(&event.download_url);
        update_state.asset_url.clone_from(&event.asset_url);
        update_state.checksum_url.clone_from(&event.checksum_url);
        update_state.error = None;
    }
}

/// 处理检查更新失败
fn handle_check_update_failed(
    mut messages: MessageReader<CheckUpdateFailedEvent>,
    mut update_state: ResMut<crate::resources::UpdateCheckState>,
) {
    for event in messages.read() {
        update_state.is_checking = false;
        update_state.error = Some(event.error.clone());
        tracing::warn!("检查更新失败: {}", event.error);
    }
}

// ==================== 忘记/重置密码处理 ====================

/// 处理忘记密码请求（获取安全问题）
fn handle_forgot_password_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<ForgotPasswordRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let email = event.email.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::auth::ForgotPasswordRequest;

            let result = match client
                .request(ForgotPasswordRequest {
                    email: email.clone(),
                })
                .await
            {
                Ok(response) => Ok((response.question1, response.question2, response.question3)),
                Err(e) => Err(e.to_string()),
            };

            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .write_message(ForgotPasswordResponseEvent { result });
            })
            .await;
        });
    }
}

/// 处理重置密码请求（通过安全问题）
fn handle_reset_password_request(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<ResetPasswordRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let email = event.email.clone();
        let question_no = event.question_no;
        let answer = event.answer.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::auth::ResetPasswordRequest;

            let result = match client
                .request(ResetPasswordRequest {
                    email,
                    question_no,
                    answer,
                })
                .await
            {
                Ok(response) => {
                    let msg = response
                        .password
                        .map(|p| format!("密码已重置，新密码: {}", p))
                        .unwrap_or_else(|| "密码已重置".to_string());
                    Ok(msg)
                }
                Err(e) => Err(e.to_string()),
            };

            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .write_message(ResetPasswordResponseEvent { result });
            })
            .await;
        });
    }
}

// ==================== 评论处理 ====================

/// 处理加载评论列表请求
fn handle_load_comments(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadCommentsRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();
        let page = event.page;

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comment::GetCommentsRequest;

            match client.request(GetCommentsRequest { comic_id, page }).await {
                Ok(response) => {
                    let comments = response.comments.docs;
                    let total_pages = response.comments.page.pages;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(CommentsLoadedEvent {
                            comments,
                            total_pages,
                            page,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(CommentsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理发表评论请求
fn handle_post_comment(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<PostCommentRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comic_id = event.comic_id.clone();
        let content = event.content.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::comment::PostCommentRequest as ApiPostComment;

            let result = client.request(ApiPostComment { comic_id, content }).await;

            ctx.run_on_main_thread(move |ctx| match result {
                Ok(_) => {
                    ctx.world.write_message(PostCommentResponseEvent {
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    ctx.world.write_message(PostCommentResponseEvent {
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            })
            .await;
        });
    }
}

/// 处理回复评论请求
fn handle_post_comment_reply(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<PostCommentReplyRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comment_id = event.comment_id.clone();
        let content = event.content.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::comment::PostCommentReplyRequest as ApiReply;

            let result = client
                .request(ApiReply {
                    comment_id,
                    content,
                })
                .await;

            ctx.run_on_main_thread(move |ctx| match result {
                Ok(_) => {
                    ctx.world.write_message(PostCommentReplyResponseEvent {
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    ctx.world.write_message(PostCommentReplyResponseEvent {
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            })
            .await;
        });
    }
}

/// 处理点赞评论请求
fn handle_like_comment(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LikeCommentRequestEvent>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comment_id = event.comment_id.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::comment::LikeCommentRequest as ApiLikeComment;

            let result = client
                .request(ApiLikeComment {
                    comment_id: comment_id.clone(),
                })
                .await;

            ctx.run_on_main_thread(move |ctx| match result {
                Ok(response) => {
                    ctx.world.write_message(LikeCommentResponseEvent {
                        comment_id,
                        action: response.action,
                    });
                }
                Err(e) => {
                    tracing::warn!("点赞评论失败: {}", e);
                }
            })
            .await;
        });
    }
}

/// 处理加载子评论请求
fn handle_load_child_comments(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadChildCommentsRequest>,
    api_client: Res<ApiClientResource>,
) {
    for event in messages.read() {
        let client = api_client.0.clone();
        let comment_id = event.comment_id.clone();
        let page = event.page;

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::comment::GetCommentChildrenRequest;

            match client
                .request(GetCommentChildrenRequest {
                    comment_id: comment_id.clone(),
                    page,
                })
                .await
            {
                Ok(response) => {
                    let comments = response.comments.docs;
                    let total_pages = response.comments.page.pages;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(ChildCommentsLoadedEvent {
                            comment_id,
                            comments,
                            total_pages,
                            page,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    tracing::warn!("加载子评论失败: {}", e);
                }
            }
        });
    }
}

// ==================== 网络诊断 ====================

/// 测速用的固定图片 URL
const SPEED_TEST_IMAGE_URL: &str =
    "https://storage-b.picacomic.com/static/817c4a45-ce32-4ee7-b602-85e39d9ea00b.jpg";

/// 处理网速测试请求：下载固定图片并计时
fn handle_speed_test(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<crate::events::SpeedTestRequest>,
    mut diag_state: ResMut<crate::resources::NetworkDiagState>,
) {
    for _event in messages.read() {
        if diag_state.is_testing_speed {
            continue;
        }

        diag_state.is_testing_speed = true;
        diag_state.error = None;
        diag_state.download_speed = None;

        runtime.spawn_background_task(move |mut ctx| async move {
            let result = async {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

                let start = std::time::Instant::now();
                let response = client
                    .get(SPEED_TEST_IMAGE_URL)
                    .send()
                    .await
                    .map_err(|e| format!("下载失败: {}", e))?;

                if !response.status().is_success() {
                    return Err(format!("服务器返回错误: {}", response.status()));
                }

                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| format!("读取响应失败: {}", e))?;

                let elapsed = start.elapsed();
                let elapsed_ms = elapsed.as_millis() as u64;
                // 避免除零
                let speed_kbps = if elapsed_ms > 0 {
                    (bytes.len() as f64 / 1024.0) / (elapsed_ms as f64 / 1000.0)
                } else {
                    0.0
                };

                Ok((speed_kbps, elapsed_ms))
            }
            .await;

            ctx.run_on_main_thread(move |ctx| match result {
                Ok((speed, elapsed)) => {
                    ctx.world
                        .write_message(crate::events::SpeedTestResultEvent {
                            download_speed: speed,
                            elapsed_ms: elapsed,
                        });
                }
                Err(error) => {
                    ctx.world
                        .write_message(crate::events::NetworkTestFailedEvent { error });
                }
            })
            .await;
        });
    }
}

/// 处理网速测试结果
fn handle_speed_test_result(
    mut messages: MessageReader<crate::events::SpeedTestResultEvent>,
    mut diag_state: ResMut<crate::resources::NetworkDiagState>,
) {
    for event in messages.read() {
        diag_state.is_testing_speed = false;
        diag_state.download_speed = Some(event.download_speed);
        tracing::info!(
            "测速完成: {:.1} KB/s, 耗时 {} ms",
            event.download_speed,
            event.elapsed_ms
        );
    }
}

/// 处理 Ping 测试请求：请求 /categories API 并计时
fn handle_ping_test(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<crate::events::PingTestRequest>,
    api_client: Res<ApiClientResource>,
    mut diag_state: ResMut<crate::resources::NetworkDiagState>,
) {
    for _event in messages.read() {
        if diag_state.is_testing_ping {
            continue;
        }

        diag_state.is_testing_ping = true;
        diag_state.error = None;
        diag_state.latency_ms = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            let result = async {
                use picacg_api::endpoints::category::GetCategoriesRequest;

                let start = std::time::Instant::now();
                client
                    .request(GetCategoriesRequest)
                    .await
                    .map_err(|e| format!("请求失败: {}", e))?;
                let elapsed = start.elapsed();
                let latency_ms = elapsed.as_millis() as u64;

                Ok::<u64, String>(latency_ms)
            }
            .await;

            ctx.run_on_main_thread(move |ctx| match result {
                Ok(latency) => {
                    ctx.world.write_message(crate::events::PingTestResultEvent {
                        latency_ms: latency,
                    });
                }
                Err(error) => {
                    ctx.world
                        .write_message(crate::events::NetworkTestFailedEvent { error });
                }
            })
            .await;
        });
    }
}

/// 处理 Ping 测试结果
fn handle_ping_test_result(
    mut messages: MessageReader<crate::events::PingTestResultEvent>,
    mut diag_state: ResMut<crate::resources::NetworkDiagState>,
) {
    for event in messages.read() {
        diag_state.is_testing_ping = false;
        diag_state.latency_ms = Some(event.latency_ms);
        tracing::info!("Ping 测试完成: {} ms", event.latency_ms);
    }
}

/// 处理网络测试失败事件
fn handle_network_test_failed(
    mut messages: MessageReader<crate::events::NetworkTestFailedEvent>,
    mut diag_state: ResMut<crate::resources::NetworkDiagState>,
) {
    for event in messages.read() {
        diag_state.is_testing_speed = false;
        diag_state.is_testing_ping = false;
        diag_state.error = Some(event.error.clone());
        tracing::warn!("网络诊断失败: {}", event.error);
    }
}

// ==================== 游戏处理 ====================

/// 处理加载游戏列表请求
fn handle_load_games(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadGamesRequest>,
    api_client: Res<ApiClientResource>,
    mut games_state: ResMut<GamesState>,
) {
    for event in messages.read() {
        let page = event.page;

        games_state.is_loading = true;
        games_state.error = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::game::GetGamesRequest;

            let request = GetGamesRequest { page };

            match client.request(request).await {
                Ok(response) => {
                    let games = response.games.docs;
                    let total_pages = response.games.pages;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(GamesLoadedEvent { games, total_pages });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(GamesLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理加载游戏详情请求
fn handle_load_game_detail(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadGameDetailRequest>,
    api_client: Res<ApiClientResource>,
    mut game_detail_state: ResMut<GameDetailState>,
) {
    for event in messages.read() {
        let game_id = event.game_id.clone();

        game_detail_state.is_loading = true;
        game_detail_state.error = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::game::GetGameDetailRequest;

            let request = GetGameDetailRequest { game_id };

            match client.request(request).await {
                Ok(response) => {
                    let game = response.game;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(GameDetailLoadedEvent { game });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(GameDetailLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

// ==================== 锅贴社区处理 ====================

/// 处理加载小程序列表请求
fn handle_load_apps(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadAppsRequest>,
    api_client: Res<ApiClientResource>,
    mut fried_state: ResMut<FriedState>,
) {
    for _event in messages.read() {
        fried_state.is_loading = true;
        fried_state.error = None;

        let client = api_client.0.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::fried::GetAppsRequest;

            let request = GetAppsRequest;

            match client.request(request).await {
                Ok(response) => {
                    let apps = response.apps;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(AppsLoadedEvent { apps });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(AppsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理加载锅贴帖子列表请求
///
/// 锅贴 API 使用独立域名和 PicACG token 认证。
/// 请求流程：
/// 1. 使用 PicACG token 访问锅贴 API
/// 2. 获取帖子列表
fn handle_load_fried_posts(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadFriedPostsRequest>,
    api_client: Res<ApiClientResource>,
    mut fried_state: ResMut<FriedState>,
) {
    for event in messages.read() {
        let page = event.page;
        let limit = if fried_state.limit > 0 {
            fried_state.limit
        } else {
            10
        };
        let offset = page * limit;

        fried_state.is_loading = true;
        fried_state.error = None;

        let client = api_client.0.clone();
        let token = client.get_token().unwrap_or_default();

        runtime.spawn_background_task(move |mut ctx| async move {
            use picacg_api::endpoints::fried::FRIED_API_BASE;

            // 构建锅贴 API 请求（独立域名，使用 PicACG token 认证）
            let url = format!("{}/posts?offset={}", FRIED_API_BASE, offset);

            let http_client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(30))
                .build();

            let http_client = match http_client {
                Ok(c) => c,
                Err(e) => {
                    let error = format!("创建 HTTP 客户端失败: {}", e);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(FriedPostsLoadFailedEvent { error });
                    })
                    .await;
                    return;
                }
            };

            let response = http_client
                .get(&url)
                .header("token", &token)
                .header(
                    "user-agent",
                    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36",
                )
                .header("Referer", format!("{}/?token={}", FRIED_API_BASE, token))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status().as_u16();
                        let error = format!("HTTP 错误: {}", status);
                        ctx.run_on_main_thread(move |ctx| {
                            ctx.world
                                .write_message(FriedPostsLoadFailedEvent { error });
                        })
                        .await;
                        return;
                    }

                    let text = match resp.text().await {
                        Ok(t) => t,
                        Err(e) => {
                            let error = format!("读取响应失败: {}", e);
                            ctx.run_on_main_thread(move |ctx| {
                                ctx.world
                                    .write_message(FriedPostsLoadFailedEvent { error });
                            })
                            .await;
                            return;
                        }
                    };

                    // 安全地截取前 500 个字符用于调试
                    let preview: String = text.chars().take(500).collect();
                    tracing::debug!("锅贴响应(前 500 字符): {}", preview);

                    // 解析响应
                    match serde_json::from_str::<
                        picacg_api::endpoints::fried::FriedPostsResponse,
                    >(&text)
                    {
                        Ok(parsed) => {
                            let posts = parsed.data.posts;
                            let total = parsed.data.total;
                            let limit = parsed.data.limit;
                            ctx.run_on_main_thread(move |ctx| {
                                ctx.world.write_message(FriedPostsLoadedEvent {
                                    posts,
                                    total,
                                    limit,
                                });
                            })
                            .await;
                        }
                        Err(e) => {
                            let error = format!("解析锅贴响应失败: {}", e);
                            tracing::error!("锅贴响应解析失败: {}, 响应体: {}", e, text);
                            ctx.run_on_main_thread(move |ctx| {
                                ctx.world
                                    .write_message(FriedPostsLoadFailedEvent { error });
                            })
                            .await;
                        }
                    }
                }
                Err(e) => {
                    let error = format!("请求锅贴 API 失败: {}", e);
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world
                            .write_message(FriedPostsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

// ==================== 聊天室 API 处理 ====================

/// 处理加载聊天房间列表请求
fn handle_load_chat_rooms(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadChatRoomsRequest>,
    mut chat_state: ResMut<ChatState>,
) {
    for _event in messages.read() {
        if chat_state.is_loading {
            continue;
        }
        chat_state.is_loading = true;
        chat_state.error = None;

        // 从配置读取登录凭据
        let (email, password) = {
            let settings = AppSettings::global().read();
            (
                settings.login.saved_email.clone(),
                settings.login.saved_password.clone(),
            )
        };

        // 复用已有 token（如果存在）
        let existing_token = chat_state.chat_token.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            use picacg_api::endpoints::chat::ChatApiClient;

            let mut chat_client = ChatApiClient::new();

            // 步骤 1：登录（或复用 token）
            let token = if let Some(token) = existing_token {
                chat_client.set_token(token.clone());
                token
            } else {
                match chat_client.signin(&email, &password).await {
                    Ok(token) => token,
                    Err(e) => {
                        let error = e;
                        ctx.run_on_main_thread(move |ctx| {
                            ctx.world.write_message(ChatRoomsLoadFailedEvent { error });
                        })
                        .await;
                        return;
                    }
                }
            };

            // 步骤 2：获取用户资料（可选，失败不影响）
            let profile = match chat_client.get_profile().await {
                Ok(p) => Some(p),
                Err(e) => {
                    tracing::warn!("获取聊天资料失败（不影响功能）: {}", e);
                    None
                }
            };

            // 步骤 3：获取房间列表
            match chat_client.get_rooms().await {
                Ok(rooms) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(ChatRoomsLoadedEvent {
                            rooms,
                            token,
                            profile,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(ChatRoomsLoadFailedEvent { error });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理连接聊天室 WebSocket 请求
fn handle_connect_chat_room(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<ConnectChatRoomRequest>,
    mut chat_room_state: ResMut<ChatRoomState>,
) {
    for event in messages.read() {
        // 关闭旧连接
        if let Some(close_sender) = chat_room_state.ws_close_sender.take() {
            let _ = close_sender.send(());
        }

        chat_room_state.is_connecting = true;
        chat_room_state.is_connected = false;
        chat_room_state.error = None;

        let room_id = event.room_id.clone();
        let token = event.token.clone();
        let ws_url = format!(
            "{}?token={}&room={}",
            picacg_api::endpoints::chat::CHAT_WS_BASE,
            token,
            room_id,
        );

        runtime.spawn_background_task(move |mut ctx| async move {
            match crate::utils::websocket::connect_websocket(&ws_url).await {
                Ok((incoming_rx, outgoing_tx, close_tx)) => {
                    ctx.run_on_main_thread(move |ctx| {
                        let mut state = ctx.world.resource_mut::<ChatRoomState>();
                        state.ws_receiver = Some(std::sync::Mutex::new(incoming_rx));
                        state.ws_sender = Some(outgoing_tx);
                        state.ws_close_sender = Some(close_tx);
                        state.is_connecting = false;
                        state.is_connected = true;
                        tracing::info!("WebSocket 连接已建立");
                    })
                    .await;
                }
                Err(e) => {
                    let error = e;
                    ctx.run_on_main_thread(move |ctx| {
                        let mut state = ctx.world.resource_mut::<ChatRoomState>();
                        state.is_connecting = false;
                        state.is_connected = false;
                        state.error = Some(error);
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理发送聊天消息请求
fn handle_send_chat_message(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<SendChatMessageRequest>,
    chat_state: Res<ChatState>,
) {
    for event in messages.read() {
        let token = match &chat_state.chat_token {
            Some(t) => t.clone(),
            None => {
                tracing::warn!("聊天服务未登录，无法发送消息");
                continue;
            }
        };

        let room_id = event.room_id.clone();
        let message = event.message.clone();

        runtime.spawn_background_task(|mut ctx| async move {
            let mut chat_client = picacg_api::endpoints::chat::ChatApiClient::new();
            chat_client.set_token(token);

            match chat_client.send_message(&room_id, &message, None).await {
                Ok(()) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(SendChatMessageResponse {
                            success: true,
                            error: None,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e;
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(SendChatMessageResponse {
                            success: false,
                            error: Some(error),
                        });
                    })
                    .await;
                }
            }
        });
    }
}

/// 处理断开聊天室 WebSocket
fn handle_disconnect_chat_room(
    mut messages: MessageReader<DisconnectChatRoomRequest>,
    mut chat_room_state: ResMut<ChatRoomState>,
) {
    for _event in messages.read() {
        if let Some(close_sender) = chat_room_state.ws_close_sender.take() {
            let _ = close_sender.send(());
        }
        chat_room_state.is_connected = false;
        chat_room_state.is_connecting = false;
        chat_room_state.ws_receiver = None;
        chat_room_state.ws_sender = None;
    }
}

#[cfg(test)]
mod tests {
    use super::compare_versions;

    #[test]
    fn detects_newer_version() {
        assert!(compare_versions("0.5.0", "0.4.0"));
        assert!(compare_versions("0.4.1", "0.4.0"));
        assert!(compare_versions("1.0.0", "0.9.9"));
    }

    #[test]
    fn rejects_same_or_older() {
        assert!(!compare_versions("0.4.0", "0.4.0"));
        assert!(!compare_versions("0.3.9", "0.4.0"));
    }

    /// 本项目发版格式是 `v{version}+{commit}`——旧实现会把 `0+abc1234` 这种段
    /// 静默丢掉，导致 patch 位带后缀时误判为「无更新」
    #[test]
    fn handles_build_metadata() {
        assert!(compare_versions("v0.5.1+abc1234", "0.5.0"));
        assert!(!compare_versions("v0.5.0+abc1234", "0.5.0"));
        assert!(compare_versions("0.5.0+20260830.abc1234", "0.4.9"));
    }

    /// 预发布后缀同样要切掉再比数字位
    #[test]
    fn handles_prerelease_suffix() {
        assert!(compare_versions("0.5.0-rc.1", "0.4.0"));
        assert!(!compare_versions("0.4.0-beta", "0.4.0"));
    }

    /// 前缀 v 与段数不齐
    #[test]
    fn handles_v_prefix_and_short_versions() {
        assert!(compare_versions("v0.5", "0.4.9"));
        assert!(!compare_versions("0.4", "0.4.0"));
    }
}
