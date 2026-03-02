//! API 插件
//!
//! 处理与后端 API 的异步通信

#![allow(dead_code)]

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
            // CBZ 打包相关消息
            .add_message::<CbzPackageRequest>()
            .add_message::<CbzPackageCompletedEvent>()
            .add_message::<CbzPackageFailedEvent>()
            // 搜索相关消息
            .add_message::<SearchComicsRequestEvent>()
            .add_message::<SearchResultsLoadedEvent>()
            .add_message::<SearchFailedEvent>()
            // 排行榜相关消息
            .add_message::<LoadRankingsRequest>()
            .add_message::<RankingsLoadedEvent>()
            .add_message::<RankingsLoadFailedEvent>()
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
                    handle_redownload,
                    download_queue_manager,
                ),
            )
            // 注册系统 - 搜索
            .add_systems(Update, (handle_search_request, handle_search_response))
            // 注册系统 - 排行榜
            .add_systems(Update, (handle_load_rankings, handle_rankings_response))
            // 注册系统 - 收藏列表
            .add_systems(Update, (handle_load_favorites, handle_favorites_response))
            // 注册系统 - 首页推荐
            .add_systems(
                Update,
                (handle_load_recommendations, handle_recommendations_response),
            )
            // 注册系统 - 用户注册
            .add_systems(Update, handle_register_request)
            // 注册系统 - CBZ 打包
            .add_systems(
                Update,
                (
                    handle_cbz_package_request,
                    handle_cbz_package_completed,
                    handle_cbz_package_failed,
                ),
            )
            // API 客户端重载（通道/代理变更）
            .add_systems(Update, handle_reload_api_client)
            // 启动时自动登录系统
            .add_systems(Startup, auto_login_on_startup)
            // 检查自动登录计时器（在 Update 中运行）
            .add_systems(Update, check_auto_login_timer);
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
        // 读取代理和分流设置
        let (proxy_url, api_channel, custom_cdn_api_ip) = {
            let settings = AppSettings::global().read();
            (
                settings.proxy.to_proxy_url(),
                settings.channel.api_channel,
                settings.channel.custom_cdn_api_ip.clone(),
            )
        };
        Self(
            ApiClient::with_config(proxy_url, api_channel, &custom_cdn_api_ip)
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
        let filtered = apply_block_filter(event.comics.clone());

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

        // 触发加载图片
        for comic in &event.comics {
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

            // 在后台线程解码图片（CPU 密集型操作）
            let decoded_result = match result {
                Ok(image_data) => match image::load_from_memory(&image_data) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        Ok((width, height, rgba.into_raw()))
                    }
                    Err(e) => Err(e.to_string()),
                },
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
                            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
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

    use hmac::{Hmac, Mac};
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
fn handle_punch_in_response(mut messages: MessageReader<PunchInResponseEvent>) {
    for event in messages.read() {
        match &event.result {
            Ok(status) => {
                tracing::info!("打卡成功: {}", status);
            }
            Err(error) => {
                tracing::warn!("打卡失败: {}", error);
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
        let (proxy_url, api_channel, custom_cdn_api_ip) = {
            let settings = AppSettings::global().read();
            (
                settings.proxy.to_proxy_url(),
                settings.channel.api_channel,
                settings.channel.custom_cdn_api_ip.clone(),
            )
        };

        if let Err(e) = api_client
            .0
            .reload_config(proxy_url, api_channel, &custom_cdn_api_ip)
        {
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
                            total_pages: response.pages.pages,
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

            let request = ApiLikeRequest { comic_id };

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(LikeComicResponse {
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
fn handle_like_response(
    mut messages: MessageReader<LikeComicResponse>,
    mut detail_state: ResMut<ComicDetailState>,
) {
    for event in messages.read() {
        detail_state.is_liked = event.action == "like";
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
        let filtered = apply_block_filter(event.comics.clone());
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
        let filtered = apply_block_filter(event.comics.clone());
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

/// 获取下载保存路径（公开版本，供其他模块调用）
pub fn get_download_base_path_public() -> std::path::PathBuf {
    get_download_base_path()
}

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
/// 新结构：Downloads/Images/<漫画标题>
fn get_images_download_path() -> std::path::PathBuf {
    get_download_base_path().join("Images")
}

/// 获取 CBZ 文件保存目录
/// 新结构：Downloads/CBZ/<漫画标题>.cbz
fn get_cbz_output_path() -> std::path::PathBuf {
    get_download_base_path().join("CBZ")
}

/// 清理文件名中的非法字符
///
/// 替换 Windows 文件系统禁止的字符以及可能导致兼容性问题的全角标点
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // ASCII 非法文件名字符
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            // 全角标点（可能导致 ZIP 兼容性问题）
            '\u{FF1A}' // ： 全角冒号
            | '\u{FF0F}' // ／ 全角斜杠
            | '\u{FF3C}' // ＼ 全角反斜杠
            | '\u{FF1C}' // ＜ 全角小于号
            | '\u{FF1E}' // ＞ 全角大于号
            | '\u{FF5C}' // ｜ 全角竖线
            | '\u{FF02}' // ＂ 全角双引号
            | '\u{FF0A}' // ＊ 全角星号
            | '\u{FF1F}' // ？ 全角问号
            => '_',
            _ => c,
        })
        .collect()
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
        let mut episodes_to_download: Vec<i32> = if event.episodes.is_empty() {
            // 下载所有章节
            detail_state.episodes.iter().map(|e| e.order).collect()
        } else {
            event.episodes.clone()
        };
        // 从第一章开始下载（正序）
        episodes_to_download.sort();

        if episodes_to_download.is_empty() {
            tracing::warn!("没有章节可下载");
            continue;
        }

        let save_path = get_images_download_path()
            .join(sanitize_filename(&comic_title))
            .to_string_lossy()
            .to_string();

        // 从漫画详情获取分类和标签
        let (categories, tags) = detail_state
            .comic
            .as_ref()
            .map(|c| (c.categories.clone(), c.tags.clone()))
            .unwrap_or_default();

        // 创建 FSM 任务元数据
        let meta = DownloadTaskMeta::new(
            comic_id.clone(),
            comic_title.clone(),
            episodes_to_download.clone(),
            save_path.clone(),
            categories,
            tags,
        );

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
            save_path,
            episodes_to_download,
            total_episodes,
            control,
        );
    }
}

/// 启动后台下载任务
fn spawn_download_task(
    runtime: &TokioTasksRuntime,
    client: ApiClient,
    comic_id: String,
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
async fn execute_download_task(
    ctx: &mut crate::utils::TaskContext,
    client: ApiClient,
    comic_id: String,
    save_path: String,
    episodes_to_download: Vec<i32>,
    total_episodes: i32,
    control: std::sync::Arc<SharedTaskControl>,
) {
    let download_path = std::path::PathBuf::from(&save_path);

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
                total_episodes,
                current_page: 0,
                total_pages: 0,
                status: format!("正在获取第 {} 章图片列表...", episode_order),
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
                    let error = format!("获取第 {} 章图片列表失败: {}", episode_order, e);
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
        tracing::info!("第 {} 章共 {} 张图片", episode_order, total_pages);

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
                "第 {} 章本地已完整（{} 张），跳过下载",
                episode_order,
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
                    total_episodes,
                    current_page: total_pages,
                    total_pages,
                    status: format!("第{}章 本地已完整，跳过", episode_order),
                });
            })
            .await;
            continue; // 跳过该章节，继续下一章
        } else {
            tracing::info!(
                "第 {} 章缺少 {} 张图片，开始下载",
                episode_order,
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
                total_episodes,
                current_page: 0,
                total_pages,
                status: format!(
                    "第{}章 并发下载中（{} 个线程，待下载 {} 张）",
                    episode_order, download_workers, pending_count
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
                            "⚠ 第{}章 {}/{} 首次下载失败（稍后重试）: {}",
                            episode_order,
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
                        total_episodes,
                        current_page,
                        total_pages,
                        status: format!(
                            "第{}章 下载中 {}/{}",
                            episode_order, current_page, total_pages
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
                "第 {} 章有 {} 张图片下载失败，{}秒后进行第 {} 次重试...",
                episode_order,
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
                "第{}章 并发重试 {}/{}（剩余 {} 张）",
                episode_order,
                retry_count,
                MAX_RETRIES,
                failed_images.len()
            );
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(DownloadProgressEvent {
                    comic_id: comic_id_clone,
                    current_episode: ep_idx as i32 + 1,
                    total_episodes,
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
                        "✓ 第{}章 {}/{} 文件已存在",
                        episode_order,
                        pic_idx + 1,
                        total_pages
                    );
                    continue;
                }

                let semaphore = semaphore.clone();
                let control = control.clone();
                let retry_success_count = retry_success_count.clone();
                let still_failed = still_failed.clone();

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
                                "✓ 第{}章 {}/{} 重试成功",
                                episode_order,
                                pic_idx + 1,
                                total_pages
                            );
                        }
                        Err(e) => {
                            still_failed.lock().push((pic_idx, url, file_path));
                            tracing::warn!(
                                "⚠ 第{}章 {}/{} 重试失败: {}",
                                episode_order,
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
                "✗ 第 {} 章有 {} 张图片下载失败（已跳过）",
                episode_order,
                final_fail_count
            );
        }

        tracing::info!(
            "第 {} 章下载完成: 成功={}, 跳过={}, 失败={}",
            episode_order,
            success_count,
            skip_count,
            final_fail_count
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

    use hmac::{Hmac, Mac};
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
        tracing::debug!("下载使用代理: {}", proxy_url_str);
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
    mut cbz_messages: MessageWriter<CbzPackageRequest>,
) {
    for event in messages.read() {
        download_state.downloading_ids.remove(&event.comic_id);

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

        // 启动后台下载任务（使用元数据中保存的章节信息，会自动跳过已存在的文件）
        spawn_download_task(
            runtime.as_ref(),
            client,
            comic_id,
            save_path,
            episode_orders,
            total_episodes,
            control,
        );
    }
}

/// 处理重新下载请求（检查更新/补全缺失）
fn handle_redownload(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<RedownloadRequest>,
    api_client: Res<ApiClientResource>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    use crate::resources::{DownloadState, DownloadTaskMeta};

    for event in messages.read() {
        let comic_id = event.comic_id.clone();

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

        let save_path = old_meta.effective_download_path().to_string();
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
                custom_download_path: old_meta.custom_download_path.clone(),
                custom_auto_pack_cbz: old_meta.custom_auto_pack_cbz,
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
            custom_download_path: old_meta.custom_download_path.clone(),
            custom_auto_pack_cbz: old_meta.custom_auto_pack_cbz,
        };
        download_state.add_task(temp_meta);
        download_state.downloading_ids.insert(comic_id.clone());

        let client = api_client.0.clone();
        let comic_id_clone = comic_id.clone();
        let save_path_clone = save_path.clone();

        // 启动异步任务：获取最新章节列表并开始下载
        runtime.spawn_background_task(|mut ctx| async move {
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
                custom_download_path: None,
                custom_auto_pack_cbz: None,
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
                            keyword,
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

/// 判断漫画是否应该被屏蔽
fn is_comic_blocked(
    comic: &picacg_api::models::Comic,
    filter: &picacg_config::FilterSettings,
) -> bool {
    if filter.blocked_keywords.is_empty() {
        return false;
    }
    for keyword in &filter.blocked_keywords {
        let kw = keyword.to_lowercase();
        if filter.filter_by_category && comic.categories.iter().any(|c| c.to_lowercase() == kw) {
            return true;
        }
        if filter.filter_by_tag && comic.tags.iter().any(|t| t.to_lowercase() == kw) {
            return true;
        }
        if filter.filter_by_title && comic.title.to_lowercase().contains(&kw) {
            return true;
        }
    }
    false
}

/// 对漫画列表应用屏蔽过滤
fn apply_block_filter(comics: Vec<picacg_api::models::Comic>) -> Vec<picacg_api::models::Comic> {
    let filter = AppSettings::global().read().filter.clone();
    if filter.blocked_keywords.is_empty() {
        return comics;
    }
    let before = comics.len();
    let filtered: Vec<_> = comics
        .into_iter()
        .filter(|c| !is_comic_blocked(c, &filter))
        .collect();
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
        let filtered = apply_block_filter(event.comics.clone());
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
        let filtered = apply_block_filter(event.comics.clone());
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

// ==================== 启动时自动恢复下载 ====================

/// 加载未完成的下载任务（Startup 阶段）
fn setup_download_manager(mut download_state: ResMut<DownloadManagerState>) {
    download_state.load_incomplete_tasks();
    tracing::info!(
        "加载未完成的下载任务: {} 个",
        download_state.fsm_tasks.len()
    );
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
                let task = fsm.to_ui_task();
                if matches!(
                    task.status,
                    crate::resources::ComicDownloadStatus::Paused
                        | crate::resources::ComicDownloadStatus::Waiting
                ) {
                    resume_messages.write(ResumeDownloadRequest {
                        comic_id: task.comic_id.clone(),
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

        let task = fsm.to_ui_task();
        // 只自动启动 Waiting 状态的任务（用户手动暂停的不自动恢复）
        if matches!(task.status, crate::resources::ComicDownloadStatus::Waiting) {
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
    let cbz_dir = get_cbz_output_path();
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
) {
    for event in messages.read() {
        let comic_id = event.comic_id.clone();
        let comic_title = event.comic_title.clone();
        let source_path = event.source_path.clone();

        tracing::info!("收到 CBZ 打包请求: {}", comic_title);

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
fn handle_cbz_package_completed(mut messages: MessageReader<CbzPackageCompletedEvent>) {
    for event in messages.read() {
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
fn handle_cbz_package_failed(mut messages: MessageReader<CbzPackageFailedEvent>) {
    for event in messages.read() {
        tracing::error!("CBZ 打包失败: {} - {}", event.comic_id, event.error);
    }
}
