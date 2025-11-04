/// UI 消息枚举
#[derive(Debug, Clone)]
pub enum Message {
    // ==================== 登录界面消息 ====================
    /// 用户名输入
    EmailChanged(String),
    /// 密码输入
    PasswordChanged(String),
    /// 登录按钮点击
    LoginPressed,
    /// 登录成功
    LoginSuccess(String), // token
    /// 登录失败
    LoginFailed(String), // error message

    // ==================== 主界面消息 ====================
    /// 切换到主页
    NavigateToHome,
    /// 切换到分类
    NavigateToCategories,
    /// 切换到搜索
    NavigateToSearch,
    /// 切换到收藏
    NavigateToFavorites,
    /// 切换到下载
    NavigateToDownloads,
    /// 切换到设置
    NavigateToSettings,
    /// 切换到代理设置（登录前）
    NavigateToProxySettings,
    /// 返回登录页面
    BackToLogin,

    // ==================== 分类列表消息 ====================
    /// 加载分类列表
    LoadCategories,
    /// 分类列表加载成功
    CategoriesLoaded(Vec<crate::api::models::Category>),
    /// 分类列表加载失败
    CategoriesLoadFailed(String),
    /// 分类项点击
    CategoryClicked(String), // category_title

    // ==================== 漫画列表消息 ====================
    /// 加载漫画列表
    LoadComics(String), // category
    /// 漫画列表加载成功
    ComicsLoaded(Vec<crate::api::models::Comic>, i32), // comics, total_pages
    /// 漫画列表加载失败
    ComicsLoadFailed(String),
    /// 漫画项点击
    ComicClicked(String), // comic_id
    /// 上一页
    PrevPage,
    /// 下一页
    NextPage,

    // ==================== 漫画详情消息 ====================
    /// 加载漫画详情
    LoadComicDetail(String), // comic_id
    /// 漫画详情加载成功
    ComicDetailLoaded(crate::api::models::Comic),
    /// 漫画详情加载失败
    ComicDetailLoadFailed(String),

    // ==================== 下载管理消息 ====================
    /// 开始下载
    StartDownload {
        url: String,
        save_path: std::path::PathBuf,
    },
    /// 下载进度更新
    DownloadProgress(crate::download::DownloadProgress),
    /// 下载完成
    DownloadCompleted(u64), // task_id
    /// 下载失败
    DownloadFailed { task_id: u64, error: String },
    /// 取消下载
    CancelDownload(u64), // task_id

    // ==================== 代理设置消息 ====================
    /// 切换代理启用状态
    ProxyEnabledToggled(bool),
    /// 代理类型变更
    ProxyTypeChanged(crate::config::settings::ProxyType),
    /// 代理主机地址变更
    ProxyHostChanged(String),
    /// 代理端口变更
    ProxyPortChanged(String),
    /// 代理认证启用切换
    ProxyAuthToggled(bool),
    /// 代理用户名变更
    ProxyUsernameChanged(String),
    /// 代理密码变更
    ProxyPasswordChanged(String),
    /// 保存代理设置
    SaveProxySettings,
    /// 测试代理连接
    TestProxyConnection,
    /// 代理测试结果
    ProxyTestResult(Result<(), String>),

    // ==================== 图片加载消息 ====================
    /// 加载图片
    LoadImage(String), // url
    /// 图片加载成功
    ImageLoaded {
        url: String,
        handle: iced::widget::image::Handle,
    },
    /// 图片加载失败
    ImageLoadFailed { url: String, error: String },

    // ==================== 通用消息 ====================
    /// 无操作（用于忽略某些事件）
    Noop,
    /// 错误提示
    ShowError(String),
    /// 成功提示
    ShowSuccess(String),
}
