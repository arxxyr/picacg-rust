use iced::{
    Element, Settings, Task, Theme,
    font::{Font, Weight},
    widget::{text, text_input},
};

use crate::{
    api::{
        ApiClient,
        endpoints::{LoginRequest, category::GetCategoriesRequest, comic::GetComicsRequest},
    },
    config::settings::AppSettings,
    ui::{
        message::Message,
        state::{AppState, Route},
        views,
    },
};

/// 中文字体常量
const SARASA_TERM_FONT: Font = Font {
    family: iced::font::Family::Name("Sarasa Term SC Nerd"),
    weight: Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

/// PicACG 应用
pub struct PicACGApp {
    /// 应用状态
    state: AppState,
    /// API 客户端
    api_client: ApiClient,
}

impl Default for PicACGApp {
    fn default() -> Self {
        Self {
            state: AppState::new(),
            api_client: ApiClient::new().expect("Failed to create API client"),
        }
    }
}

impl PicACGApp {
    pub fn new() -> (Self, Task<Message>) {
        // 初始化时设置焦点到用户名输入框
        let focus_task = text_input::focus(text_input::Id::new(views::login::USERNAME_INPUT_ID));
        (Self::default(), focus_task)
    }

    pub fn title(&self) -> String {
        String::from("PicACG - Rust 版本")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ==================== 登录消息 ====================
            Message::EmailChanged(email) => {
                self.state.login_state.email = email;
                Task::none()
            }

            Message::PasswordChanged(password) => {
                self.state.login_state.password = password;
                Task::none()
            }

            Message::LoginPressed => {
                // 验证输入
                if self.state.login_state.email.is_empty()
                    || self.state.login_state.password.is_empty()
                {
                    self.state.login_state.error = Some("请输入用户名和密码".to_string());
                    return Task::none();
                }

                // 设置加载状态
                self.state.login_state.is_loading = true;
                self.state.login_state.error = None;

                // 执行登录
                let email = self.state.login_state.email.clone();
                let password = self.state.login_state.password.clone();
                let client = self.api_client.clone();

                Task::perform(
                    async move {
                        let request = LoginRequest { email, password };
                        match client.request(request).await {
                            Ok(response) => Message::LoginSuccess(response.token),
                            Err(e) => Message::LoginFailed(e.to_string()),
                        }
                    },
                    |msg| msg,
                )
            }

            Message::LoginSuccess(token) => {
                // 设置 token
                self.api_client.set_token(token.clone());
                self.state.set_token(token);

                // 重置登录状态
                self.state.login_state.is_loading = false;
                self.state.login_state.error = None;

                // 导航到主页
                self.state.navigate_to(Route::Home);

                Task::none()
            }

            Message::LoginFailed(error) => {
                self.state.login_state.is_loading = false;
                self.state.login_state.error = Some(format!("登录失败: {}", error));
                Task::none()
            }

            // ==================== 导航消息 ====================
            Message::NavigateToHome => {
                self.state.navigate_to(Route::Home);
                Task::none()
            }

            Message::NavigateToCategories => {
                self.state.navigate_to(Route::Categories);
                // 如果分类列表为空，自动加载
                if self.state.categories_state.categories.is_empty()
                    && !self.state.categories_state.is_loading
                {
                    self.update(Message::LoadCategories)
                } else {
                    Task::none()
                }
            }

            Message::NavigateToSearch => {
                self.state.navigate_to(Route::Search);
                Task::none()
            }

            Message::NavigateToFavorites => {
                self.state.navigate_to(Route::Favorites);
                Task::none()
            }

            Message::NavigateToDownloads => {
                self.state.navigate_to(Route::Downloads);
                Task::none()
            }

            Message::NavigateToSettings => {
                self.state.navigate_to(Route::Settings);
                Task::none()
            }

            Message::NavigateToProxySettings => {
                self.state.navigate_to(Route::ProxySettings);
                Task::none()
            }

            Message::BackToLogin => {
                self.state.navigate_to(Route::Login);
                Task::none()
            }

            // ==================== 分类列表消息 ====================
            Message::LoadCategories => {
                self.state.categories_state.is_loading = true;
                self.state.categories_state.error = None;

                let client = self.api_client.clone();
                Task::perform(
                    async move {
                        match client.request(GetCategoriesRequest).await {
                            Ok(response) => Message::CategoriesLoaded(response.categories),
                            Err(e) => Message::CategoriesLoadFailed(e.to_string()),
                        }
                    },
                    |msg| msg,
                )
            }

            Message::CategoriesLoaded(categories) => {
                self.state.categories_state.is_loading = false;

                // 批量触发图片加载
                let tasks: Vec<_> = categories
                    .iter()
                    .map(|cat| {
                        let url = cat.thumb.url();
                        Task::done(Message::LoadImage(url))
                    })
                    .collect();

                self.state.categories_state.categories = categories;

                // 执行所有图片加载任务
                Task::batch(tasks)
            }

            Message::CategoriesLoadFailed(error) => {
                self.state.categories_state.is_loading = false;
                self.state.categories_state.error = Some(format!("加载分类失败: {}", error));
                Task::none()
            }

            Message::CategoryClicked(title) => {
                // 跳转到漫画列表页面
                self.state.navigate_to(Route::ComicsList(title.clone()));

                // 设置当前分类并触发漫画列表加载
                self.state.comics_list_state.category = title.clone();
                self.state.comics_list_state.page = 1; // 重置页码
                self.update(Message::LoadComics(title))
            }

            // ==================== 漫画列表消息 ====================
            Message::LoadComics(category) => {
                self.state.comics_list_state.is_loading = true;
                self.state.comics_list_state.error = None;

                let client = self.api_client.clone();
                let page = self.state.comics_list_state.page;
                let sort = self.state.comics_list_state.sort.clone();

                Task::perform(
                    async move {
                        let request = GetComicsRequest {
                            category,
                            page,
                            sort,
                        };
                        match client.request(request).await {
                            Ok(response) => {
                                Message::ComicsLoaded(response.comics.docs, response.comics.pages)
                            }
                            Err(e) => Message::ComicsLoadFailed(e.to_string()),
                        }
                    },
                    |msg| msg,
                )
            }

            Message::ComicsLoaded(comics, total_pages) => {
                self.state.comics_list_state.is_loading = false;
                self.state.comics_list_state.total_pages = total_pages;

                // 批量触发图片加载
                let tasks: Vec<_> = comics
                    .iter()
                    .map(|comic| {
                        let url = comic.thumb.url();
                        Task::done(Message::LoadImage(url))
                    })
                    .collect();

                self.state.comics_list_state.comics = comics;

                // 执行所有图片加载任务
                Task::batch(tasks)
            }

            Message::ComicsLoadFailed(error) => {
                self.state.comics_list_state.is_loading = false;
                self.state.comics_list_state.error = Some(format!("加载漫画列表失败: {}", error));
                Task::none()
            }

            Message::ComicClicked(comic_id) => {
                // 跳转到漫画详情页面并触发加载
                self.state.navigate_to(Route::ComicDetail(comic_id.clone()));

                // 创建新的详情状态
                self.state.comic_detail_state =
                    Some(crate::ui::state::ComicDetailState::new(comic_id.clone()));

                // 触发加载
                self.update(Message::LoadComicDetail(comic_id))
            }

            Message::PrevPage => {
                if self.state.comics_list_state.page > 1 {
                    self.state.comics_list_state.page -= 1;
                    let category = self.state.comics_list_state.category.clone();
                    self.update(Message::LoadComics(category))
                } else {
                    Task::none()
                }
            }

            Message::NextPage => {
                if self.state.comics_list_state.page < self.state.comics_list_state.total_pages {
                    self.state.comics_list_state.page += 1;
                    let category = self.state.comics_list_state.category.clone();
                    self.update(Message::LoadComics(category))
                } else {
                    Task::none()
                }
            }

            // ==================== 漫画详情消息 ====================
            Message::LoadComicDetail(comic_id) => {
                if let Some(ref mut detail_state) = self.state.comic_detail_state {
                    detail_state.is_loading = true;
                    detail_state.error = None;

                    let client = self.api_client.clone();
                    Task::perform(
                        async move {
                            let request =
                                crate::api::endpoints::comic::GetComicDetailRequest { comic_id };
                            match client.request(request).await {
                                Ok(response) => Message::ComicDetailLoaded(response.comic),
                                Err(e) => Message::ComicDetailLoadFailed(e.to_string()),
                            }
                        },
                        |msg| msg,
                    )
                } else {
                    Task::none()
                }
            }

            Message::ComicDetailLoaded(comic) => {
                if let Some(ref mut detail_state) = self.state.comic_detail_state {
                    detail_state.is_loading = false;

                    // 加载封面图片
                    let img_url = comic.thumb.url();
                    let load_image_task = Task::done(Message::LoadImage(img_url));

                    detail_state.comic = Some(comic);

                    load_image_task
                } else {
                    Task::none()
                }
            }

            Message::ComicDetailLoadFailed(error) => {
                if let Some(ref mut detail_state) = self.state.comic_detail_state {
                    detail_state.is_loading = false;
                    detail_state.error = Some(format!("加载漫画详情失败: {}", error));
                }
                Task::none()
            }

            // ==================== 图片加载消息 ====================
            Message::LoadImage(url) => {
                let cache = self.state.image_cache.clone();
                let client = self.api_client.clone();
                let url_clone = url.clone();

                Task::perform(
                    async move {
                        // 检查缓存
                        if cache.is_loaded(&url_clone).await {
                            return Message::Noop;
                        }

                        // 检查是否正在加载
                        if cache.is_loading(&url_clone).await {
                            return Message::Noop;
                        }

                        // 标记为加载中
                        cache
                            .set(url_clone.clone(), crate::ui::ImageState::Loading)
                            .await;

                        // 下载图片
                        match crate::ui::image_loader::download_image(client, url_clone.clone())
                            .await
                        {
                            Ok(handle) => Message::ImageLoaded {
                                url: url_clone,
                                handle,
                            },
                            Err(e) => Message::ImageLoadFailed {
                                url: url_clone,
                                error: e,
                            },
                        }
                    },
                    |msg| msg,
                )
            }

            Message::ImageLoaded { url, handle } => {
                // 存储到分类状态中
                self.state
                    .categories_state
                    .thumbnails
                    .insert(url.clone(), handle.clone());

                // 如果是详情页的封面图片，也存储到详情状态中
                if let Some(ref mut detail_state) = self.state.comic_detail_state {
                    if let Some(ref comic) = detail_state.comic {
                        if url == comic.thumb.url() {
                            detail_state.cover_image = Some(handle.clone());
                        }
                    }
                }

                let cache = self.state.image_cache.clone();
                Task::perform(
                    async move {
                        cache.set(url, crate::ui::ImageState::Loaded(handle)).await;
                        Message::Noop
                    },
                    |msg| msg,
                )
            }

            Message::ImageLoadFailed { url, error } => {
                let cache = self.state.image_cache.clone();
                Task::perform(
                    async move {
                        cache.set(url, crate::ui::ImageState::Failed(error)).await;
                        Message::Noop
                    },
                    |msg| msg,
                )
            }

            // ==================== 其他消息 ====================
            Message::Noop => Task::none(),

            Message::ShowError(error) => {
                self.state.set_error(error);
                Task::none()
            }

            Message::ShowSuccess(success) => {
                self.state.set_success(success);
                Task::none()
            }

            // 其他消息暂时返回 none（待实现）
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        // 根据路由渲染不同视图
        match &self.state.route {
            Route::Login => views::login_view(&self.state.login_state),
            Route::ProxySettings => {
                // 代理设置页面（登录前可访问）
                views::proxy_settings_view(&self.state.proxy_settings_state)
            }
            Route::Home => {
                let content = views::home_view();
                views::main_layout_view(&self.state, content)
            }
            Route::Categories => {
                let content = views::categories_view(&self.state.categories_state);
                views::main_layout_view(&self.state, content)
            }
            Route::ComicsList(_) => {
                let content = views::comics_list_view(&self.state.comics_list_state);
                views::main_layout_view(&self.state, content)
            }
            Route::Search => {
                let content = text("搜索页面（待实现）").into();
                views::main_layout_view(&self.state, content)
            }
            Route::Favorites => {
                let content = text("收藏页面（待实现）").into();
                views::main_layout_view(&self.state, content)
            }
            Route::Downloads => {
                let content = text("下载页面（待实现）").into();
                views::main_layout_view(&self.state, content)
            }
            Route::Settings => {
                let content = views::proxy_settings_view(&self.state.proxy_settings_state);

                views::main_layout_view(&self.state, content)
            }
            Route::ComicDetail(_) => {
                let content = if let Some(ref detail_state) = self.state.comic_detail_state {
                    views::comic_detail_view(detail_state, &self.state.image_cache)
                } else {
                    text("加载中...").into()
                };
                views::main_layout_view(&self.state, content)
            }
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/// 运行应用
pub fn run() -> iced::Result {
    // 设置环境变量强制使用Vulkan
    unsafe {
        std::env::set_var("WGPU_BACKEND", "vulkan");
    }

    iced::application("PicACG", PicACGApp::update, PicACGApp::view)
        .font(include_bytes!(
            "../../resources/fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf"
        ))
        .default_font(SARASA_TERM_FONT)
        .theme(PicACGApp::theme)
        .window_size((1024.0, 768.0))
        .antialiasing(true)
        .run()
}
