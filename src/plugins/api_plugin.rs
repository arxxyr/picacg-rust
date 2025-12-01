//! API 插件
//!
//! 处理与后端 API 的异步通信

use bevy::{asset::RenderAssetUsages, prelude::*};
use bevy_tokio_tasks::TokioTasksRuntime;

use crate::{
    api::ApiClient, config::settings::AppSettings, events::*, resources::*,
    systems::login::save_credentials_on_login,
};

/// API 插件
pub struct ApiPlugin;

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册 API 客户端资源
            .insert_resource(ApiClientResource::new())
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
            // 注册系统
            .add_systems(
                Update,
                (
                    handle_login_request,
                    handle_login_response,
                    handle_load_categories,
                    handle_categories_response,
                    handle_load_comics,
                    handle_comics_response,
                    handle_load_image,
                    handle_image_response,
                    handle_punch_in_request,
                    handle_punch_in_response,
                ),
            )
            // 启动时自动登录系统
            .add_systems(Startup, auto_login_on_startup);
    }
}

/// API 客户端资源包装
#[derive(Resource)]
pub struct ApiClientResource(pub ApiClient);

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
            // 下载图片
            let result = download_image(&url).await;

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

/// 下载图片数据
async fn download_image(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

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

/// 启动时自动登录系统
fn auto_login_on_startup(mut login_messages: MessageWriter<LoginRequestEvent>) {
    let settings = AppSettings::global().read();

    // 检查是否启用自动登录且有保存的凭据
    if settings.login.auto_login
        && !settings.login.saved_email.is_empty()
        && !settings.login.saved_password.is_empty()
    {
        let email = settings.login.saved_email.clone();
        let password = settings.login.saved_password.clone();
        drop(settings); // 释放锁

        tracing::info!("自动登录中...");
        login_messages.write(LoginRequestEvent { email, password });
    }
}
