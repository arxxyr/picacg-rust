//! API 插件
//!
//! 处理与后端 API 的异步通信

#![allow(dead_code)]

use bevy::{asset::RenderAssetUsages, prelude::*};
use bevy_tokio_tasks::TokioTasksRuntime;

use crate::{
    api::ApiClient,
    config::settings::AppSettings,
    events::*,
    resources::{DownloadManagerState, DownloadTaskMeta, SharedTaskControl, *},
    systems::login::save_credentials_on_login,
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
            // 注册消息 (Bevy 0.17 使用 add_message)
            .add_message::<LoginRequestEvent>()
            .add_message::<LoginResponseEvent>()
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
                    handle_like_comic,
                    handle_like_response,
                    handle_favorite_comic,
                    handle_favorite_response,
                    handle_load_image,
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
                ),
            )
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
        Self(ApiClient::new().expect("创建 API 客户端失败"))
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
            use crate::api::endpoints::LoginRequest;

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
            use crate::api::endpoints::category::GetCategoriesRequest;

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
        comics_state.is_loading = true;
        comics_state.error = None;

        let client = api_client.0.clone();
        let category = event.category.clone();
        let page = event.page;
        let sort = event.sort.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            use crate::api::endpoints::comic::GetComicsRequest;

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

/// 处理漫画列表加载响应
fn handle_comics_response(
    mut loaded_messages: MessageReader<ComicsLoadedEvent>,
    mut failed_messages: MessageReader<ComicsLoadFailedEvent>,
    mut comics_state: ResMut<ComicsListState>,
    mut image_messages: MessageWriter<LoadImageRequest>,
) {
    for event in loaded_messages.read() {
        comics_state.is_loading = false;
        comics_state.comics = event.comics.clone();
        comics_state.total_pages = event.total_pages;

        // 触发加载图片
        for comic in &event.comics {
            image_messages.write(LoadImageRequest {
                url: comic.thumb.url(),
            });
        }
    }

    for event in failed_messages.read() {
        comics_state.is_loading = false;
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

/// 处理图片加载请求
fn handle_load_image(
    runtime: ResMut<TokioTasksRuntime>,
    mut messages: MessageReader<LoadImageRequest>,
    mut image_cache: ResMut<ImageCache>,
) {
    for event in messages.read() {
        let url = event.url.clone();

        // 跳过已加载或正在加载的图片
        if image_cache.is_loaded(&url) || image_cache.is_loading(&url) {
            continue;
        }

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

            ctx.run_on_main_thread(move |ctx| {
                match result {
                    Ok(image_data) => {
                        // 在主线程创建 Image 资源
                        let image = match image::load_from_memory(&image_data) {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                Image::new(
                                    bevy::render::render_resource::Extent3d {
                                        width,
                                        height,
                                        depth_or_array_layers: 1,
                                    },
                                    bevy::render::render_resource::TextureDimension::D2,
                                    rgba.into_raw(),
                                    bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                                )
                            }
                            Err(e) => {
                                ctx.world.write_message(ImageLoadFailedEvent {
                                    url: url.clone(),
                                    error: e.to_string(),
                                });
                                return;
                            }
                        };

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

    let settings = AppSettings::global().read();
    let proxy_url = settings.proxy.to_proxy_url();
    drop(settings);

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

    let client = builder.build().map_err(|e| e.to_string())?;

    // 生成签名头部
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
        .get(url)
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
            use crate::api::endpoints::auth::PunchInRequest;

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
            use crate::api::endpoints::comic::GetComicDetailRequest;

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

        // 加载章节列表
        episodes_messages.write(LoadEpisodesRequest {
            comic_id: detail_state.comic_id.clone(),
            page: 1,
        });
    }

    for event in failed_messages.read() {
        detail_state.is_loading = false;
        detail_state.error = Some(event.error.clone());
    }
}

// ==================== 章节列表处理 ====================

/// 处理加载章节列表请求
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
        let page = event.page;

        runtime.spawn_background_task(move |mut ctx| async move {
            use crate::api::endpoints::comic::GetEpisodesRequest;

            let request = GetEpisodesRequest { comic_id, page };

            match client.request(request).await {
                Ok(response) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(EpisodesLoadedEvent {
                            episodes: response.eps.docs,
                            total_pages: response.eps.pages,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    let error = e.to_string();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(EpisodesLoadFailedEvent { error });
                    })
                    .await;
                }
            }
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
            use crate::api::endpoints::comic::LikeComicRequest as ApiLikeRequest;

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
            use crate::api::endpoints::comic::FavoriteComicRequest as ApiFavoriteRequest;

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

// ==================== 下载处理 ====================

/// 获取下载保存路径（公开版本，供其他模块调用）
pub fn get_download_base_path_public() -> std::path::PathBuf {
    get_download_base_path()
}

/// 获取下载保存路径
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

/// 清理文件名中的非法字符
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
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

        let save_path = get_download_base_path()
            .join(sanitize_filename(&comic_title))
            .to_string_lossy()
            .to_string();

        // 创建 FSM 任务元数据
        let meta = DownloadTaskMeta::new(
            comic_id.clone(),
            comic_title.clone(),
            episodes_to_download.clone(),
            save_path.clone(),
        );

        // 保存元数据到文件
        if let Err(e) = meta.save() {
            tracing::error!("保存下载元数据失败: {}", e);
        }

        // 添加到下载状态
        download_state.downloading_ids.insert(comic_id.clone());
        let fsm = download_state.add_task(meta);
        let control = fsm.get_control();

        // 启动下载
        if let Err(e) = fsm.start() {
            tracing::error!("启动下载失败: {}", e);
        }

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
        let download_path = std::path::PathBuf::from(&save_path);

        // 创建下载目录
        if let Err(e) = tokio::fs::create_dir_all(&download_path).await {
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.write_message(DownloadFailedEvent {
                    comic_id,
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
                use crate::api::endpoints::comic::GetPicturesRequest;

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

            // 下载每张图片
            let mut success_count = 0;
            let mut skip_count = 0;
            let mut fail_count = 0;

            for (pic_idx, picture) in all_pictures.iter().enumerate() {
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

                let url = picture.media.url();

                // 发送进度更新
                let comic_id_clone = comic_id.clone();
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.write_message(DownloadProgressEvent {
                        comic_id: comic_id_clone,
                        current_episode: ep_idx as i32 + 1,
                        total_episodes,
                        current_page: pic_idx as i32 + 1,
                        total_pages,
                        status: format!("第{}章 {}/{}", episode_order, pic_idx + 1, total_pages),
                    });
                })
                .await;

                // 确定文件扩展名
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

                let file_path = ep_folder.join(format!("{:04}.{}", pic_idx + 1, ext));

                // 如果文件已存在，跳过（支持断点续传）
                if file_path.exists() {
                    skip_count += 1;
                    continue;
                }

                // 下载图片
                match download_image_to_file(&url, &file_path).await {
                    Ok(_) => {
                        success_count += 1;
                    }
                    Err(e) => {
                        fail_count += 1;
                        tracing::error!(
                            "✗ 第{}章 {}/{} 下载失败: {}",
                            episode_order,
                            pic_idx + 1,
                            total_pages,
                            e
                        );
                    }
                }
            }

            tracing::info!(
                "第 {} 章下载完成: 成功={}, 跳过={}, 失败={}",
                episode_order,
                success_count,
                skip_count,
                fail_count
            );
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
    });
}

/// 下载图片到文件（使用代理设置和签名头部）
async fn download_image_to_file(url: &str, file_path: &std::path::Path) -> Result<(), String> {
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

    let settings = AppSettings::global().read();
    let proxy_url = settings.proxy.to_proxy_url();
    drop(settings);

    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .connect_timeout(std::time::Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::limited(10));

    // 添加代理配置
    if let Some(ref proxy_url_str) = proxy_url {
        tracing::debug!("下载使用代理: {}", proxy_url_str);
        let proxy = Proxy::all(proxy_url_str).map_err(|e| format!("代理配置错误: {}", e))?;
        builder = builder.proxy(proxy);
    }

    let client = builder.build().map_err(|e| e.to_string())?;

    // 生成签名头部（参考 Python 的 ToolUtil.GetHeader）
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let nonce = uuid::Uuid::new_v4().simple().to_string();

    // 对于 CDN URL，使用完整 URL 作为 path 进行签名
    let method = "GET";
    let src = format!("{}{}{}{}{}", url, now, nonce, method, API_KEY);

    let mut mac =
        HmacSha256::new_from_slice(SECRET_KEY.as_bytes()).expect("HMAC can take key of any size");
    mac.update(src.to_lowercase().as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // 发送带签名头部的请求
    let response = client
        .get(url)
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
) {
    for event in messages.read() {
        download_state.downloading_ids.remove(&event.comic_id);

        // 更新 FSM 状态
        if let Some(fsm) = download_state.find_task_mut(&event.comic_id) {
            if let Err(e) = fsm.complete() {
                tracing::warn!("更新下载完成状态失败: {}", e);
            }
        }

        tracing::info!("下载完成: {} -> {}", event.comic_id, event.save_path);
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
        if let Some(fsm) = download_state.find_task_mut(&event.comic_id) {
            if let Err(e) = fsm.fail(event.error.clone()) {
                tracing::warn!("更新下载失败状态失败: {}", e);
            }
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
        if let Some(fsm) = download_state.find_task_mut(&event.comic_id) {
            if let Err(e) = fsm.pause() {
                tracing::warn!("更新下载暂停状态失败: {}", e);
            }
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
                fsm.meta.save_path.clone(),
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

        // 尝试加载元数据获取保存路径
        let download_base_path = crate::config::settings::AppSettings::global()
            .read()
            .download_path
            .clone();
        let download_base_path = if download_base_path.is_empty() {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("Downloads")
        } else {
            std::path::PathBuf::from(&download_base_path)
        };

        // 查找对应的保存路径（遍历下载目录）
        let mut save_path = None;
        let mut comic_title = String::new();

        if download_base_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&download_base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(meta) = DownloadTaskMeta::load(path.to_str().unwrap_or_default())
                        {
                            if meta.comic_id == comic_id {
                                save_path = Some(path.to_string_lossy().to_string());
                                comic_title = meta.comic_title.clone();
                                break;
                            }
                        }
                    }
                }
            }
        }

        let Some(save_path) = save_path else {
            tracing::warn!("找不到漫画 {} 的下载目录", comic_id);
            continue;
        };

        tracing::info!("开始重新下载/检查更新: {} -> {}", comic_title, save_path);

        // 先移除已完成的任务（如果存在）
        download_state.remove_task(&comic_id);

        let client = api_client.0.clone();
        let comic_id_clone = comic_id.clone();
        let save_path_clone = save_path.clone();

        // 启动异步任务：获取最新章节列表并开始下载
        runtime.spawn_background_task(|mut ctx| async move {
            use crate::api::endpoints::comic::{GetComicDetailRequest, GetEpisodesRequest};

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
            };

            if let Err(e) = meta.save() {
                tracing::warn!("保存元数据失败: {}", e);
            }

            // 发送下载请求到主线程
            ctx.run_on_main_thread(move |ctx| {
                // 添加任务到状态
                let mut download_state = ctx.world.resource_mut::<DownloadManagerState>();

                // 添加新任务
                download_state.add_task(meta.clone());

                // 获取控制器
                let control = download_state
                    .find_task(&comic_id_clone)
                    .map(|fsm| fsm.get_control())
                    .unwrap_or_else(|| std::sync::Arc::new(SharedTaskControl::new()));

                // 标记为下载中
                download_state
                    .downloading_ids
                    .insert(comic_id_clone.clone());

                // 获取 runtime 和 api_client
                let runtime = ctx.world.resource::<TokioTasksRuntime>();
                let api_client = ctx.world.resource::<ApiClientResource>();

                // 启动下载任务
                spawn_download_task(
                    runtime,
                    api_client.0.clone(),
                    comic_id_clone,
                    save_path_clone,
                    episode_orders,
                    total_episodes,
                    control,
                );
            })
            .await;
        });
    }
}
