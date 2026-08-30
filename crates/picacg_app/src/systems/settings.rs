//! 设置界面系统
//!
//! 实现应用设置页面

use bevy::{input_focus::InputFocus, prelude::*, time::Timer, ui::RelativeCursorPosition};
use picacg_config::{
    AppSettings, ChannelType, CloseBehavior, FilterSettings, Language, LogLevel, ProxyType,
    ThemeMode, update_log_level,
};

use crate::{
    components::ContentArea,
    systems::{
        downloads::MoveAllDownloadsButton,
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar},
        widgets::{ButtonStyle, ButtonVariant},
    },
    utils::{
        icons::*,
        profiling,
        text_input::{TextInput, TextInputDisplay},
    },
};

/// 设置页面根标记
#[derive(Component, Default, Clone)]
pub struct SettingsRoot;

/// 设置滚动容器标记
#[derive(Component, Default, Clone)]
pub struct SettingsScrollContainer;

/// 下载路径输入框标记（配合 TextInput 使用）
#[derive(Component, Default, Clone)]
pub struct DownloadPathInput;

/// 下载路径目录选择按钮
#[derive(Component, Default, Clone)]
pub struct DownloadPathPickerButton;

/// 目录选择器结果（后台线程 → 主线程，使用 Mutex 包裹 Receiver 以满足 Sync）
#[derive(Resource, Default)]
pub struct DownloadPathPickerResult {
    pub receiver: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Option<String>>>>,
}

/// 下载路径输入状态（焦点归 `InputFocus` 管，此处只存值）
#[derive(Resource)]
pub struct DownloadPathInputState {
    pub value: String,
}

impl Default for DownloadPathInputState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            value: settings.download_path.clone(),
        }
    }
}

/// 清除缓存按钮标记
#[derive(Component, Default, Clone)]
pub struct ClearCacheButton;

/// 设置保存状态提示
#[derive(Resource)]
pub struct SettingsSaveStatus {
    pub visible: bool,
    pub timer: Timer,
    pub message: String,
    pub is_error: bool,
}

impl Default for SettingsSaveStatus {
    fn default() -> Self {
        Self {
            visible: false,
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            message: String::new(),
            is_error: false,
        }
    }
}

/// 底部状态栏文本标记
#[derive(Component, Default, Clone)]
pub struct SettingsStatusText;

/// 底部状态栏容器标记
#[derive(Component, Default, Clone)]
pub struct SettingsStatusBar;

// ==================== 代理设置组件 ====================

/// 代理启用复选框
#[derive(Component, Default, Clone)]
pub struct ProxyEnabledCheckbox;

/// 代理类型按钮
#[derive(Component, Default, Clone)]
pub struct ProxyTypeButton {
    pub proxy_type: ProxyType,
}

/// 代理主机输入框
#[derive(Component, Default, Clone)]
pub struct ProxyHostInput;

/// 代理端口输入框
#[derive(Component, Default, Clone)]
pub struct ProxyPortInput;

/// 代理设置状态
#[derive(Resource)]
pub struct ProxySettingsInputState {
    pub enabled: bool,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: String,
}

impl Default for ProxySettingsInputState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            enabled: settings.proxy.enabled,
            proxy_type: settings.proxy.proxy_type,
            host: settings.proxy.host.clone(),
            port: settings.proxy.port.to_string(),
        }
    }
}

// ==================== 日志等级组件 ====================

/// 日志等级按钮
#[derive(Component, Default, Clone)]
pub struct LogLevelButton {
    pub level: LogLevel,
}

/// 日志等级状态
#[derive(Resource)]
pub struct LogLevelInputState {
    pub level: LogLevel,
}

impl Default for LogLevelInputState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            level: settings.log_level,
        }
    }
}

// ==================== 自动恢复下载设置组件 ====================

/// 自动恢复下载勾选框
#[derive(Component, Default, Clone)]
pub struct AutoResumeDownloadsCheckbox;

/// 下载完成后退出勾选框
#[derive(Component, Default, Clone)]
pub struct ExitAfterDownloadsCheckbox;

/// 下载行为设置状态（两个开关同属"下载队列生命周期"，合在一个资源里）
#[derive(Resource)]
pub struct DownloadBehaviorState {
    /// 启动后自动恢复未完成的下载
    pub auto_resume: bool,
    /// 下载队列全部完成后自动退出程序
    pub exit_after_all_done: bool,
}

// ==================== 最大并发下载数设置组件 ====================

/// 最大并发下载数减少按钮
#[derive(Component, Default, Clone)]
pub struct MaxConcurrentDownloadsDecreaseButton;

/// 最大并发下载数增加按钮
#[derive(Component, Default, Clone)]
pub struct MaxConcurrentDownloadsIncreaseButton;

/// 最大并发下载数显示文本
#[derive(Component, Default, Clone)]
pub struct MaxConcurrentDownloadsText;

/// 最大并发下载数设置状态
#[derive(Resource)]
pub struct MaxConcurrentDownloadsState {
    pub value: usize,
}

// ==================== CBZ 打包设置组件 ====================

/// 自动打包 CBZ 勾选框
#[derive(Component, Default, Clone)]
pub struct AutoPackCbzCheckbox;

/// 打包后删除原图勾选框
#[derive(Component, Default, Clone)]
pub struct DeleteImagesAfterCbzCheckbox;

/// CBZ 打包设置状态
#[derive(Resource)]
pub struct CbzPackageSettingsState {
    /// 是否自动打包 CBZ
    pub auto_pack_cbz: bool,
    /// 打包后是否删除原图
    pub delete_images_after_cbz: bool,
}

// ==================== 内容过滤设置组件 ====================

/// 按分类屏蔽复选框
#[derive(Component, Default, Clone)]
pub struct FilterByCategoryCheckbox;

/// 按标签屏蔽复选框
#[derive(Component, Default, Clone)]
pub struct FilterByTagCheckbox;

/// 按标题屏蔽复选框
#[derive(Component, Default, Clone)]
pub struct FilterByTitleCheckbox;

/// 屏蔽词列表项标记
#[derive(Component, Default, Clone)]
pub struct BlockedKeywordItem;

/// 删除屏蔽词按钮
#[derive(Component, Default, Clone)]
pub struct RemoveKeywordButton {
    pub keyword: String,
}

/// 新增屏蔽词输入框标记
#[derive(Component, Default, Clone)]
pub struct NewKeywordInput;

/// 添加屏蔽词按钮
#[derive(Component, Default, Clone)]
pub struct AddKeywordButton;

/// 下拉建议面板容器
#[derive(Component, Default, Clone)]
pub struct KeywordSuggestionPanel;

/// 建议项按钮
#[derive(Component, Default, Clone)]
pub struct KeywordSuggestionItem {
    pub keyword: String,
    /// 未屏蔽时的静息底色（分类与标签两套配色，恢复时需按项还原）
    pub base_color: Color,
}

/// 展开/折叠下拉按钮
#[derive(Component, Default, Clone)]
pub struct KeywordSuggestionToggle;

/// 屏蔽词列表容器标记
#[derive(Component, Default, Clone)]
pub struct BlockedKeywordsListContainer;

// ==================== 分流设置组件 ====================

/// API 分流按钮
#[derive(Component, Default, Clone)]
pub struct ApiChannelButton {
    pub channel_type: ChannelType,
}

/// 图片分流按钮
#[derive(Component, Default, Clone)]
pub struct ImageChannelButton {
    pub channel_type: ChannelType,
}

/// 自定义 CDN API IP 输入框
#[derive(Component, Default, Clone)]
pub struct CustomCdnApiIpInput;

/// 自定义 CDN 图片 IP 输入框
#[derive(Component, Default, Clone)]
pub struct CustomCdnImgIpInput;

/// 自定义 API IP 输入行容器（条件显示）
#[derive(Component, Default, Clone)]
pub struct CustomCdnApiIpRow;

/// 自定义图片 IP 输入行容器（条件显示）
#[derive(Component, Default, Clone)]
pub struct CustomCdnImgIpRow;

/// 分流设置状态
#[derive(Resource)]
pub struct ChannelSettingsState {
    pub api_channel: ChannelType,
    pub image_channel: ChannelType,
    pub custom_cdn_api_ip: String,
    pub custom_cdn_img_ip: String,
}

impl Default for ChannelSettingsState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            api_channel: settings.channel.api_channel,
            image_channel: settings.channel.image_channel,
            custom_cdn_api_ip: settings.channel.custom_cdn_api_ip.clone(),
            custom_cdn_img_ip: settings.channel.custom_cdn_img_ip.clone(),
        }
    }
}

// ==================== 主题设置组件 ====================

/// 主题模式按钮
#[derive(Component, Default, Clone)]
pub struct ThemeModeButton {
    pub mode: ThemeMode,
}

/// 主题设置状态（同时包含关闭行为，合并以减少系统参数数量）
#[derive(Resource)]
pub struct ThemeModeState {
    pub mode: ThemeMode,
    pub close_behavior: CloseBehavior,
}

impl Default for ThemeModeState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            mode: settings.theme,
            close_behavior: settings.close_behavior,
        }
    }
}

// ==================== 语言设置组件 ====================

/// 语言选择按钮
#[derive(Component, Default, Clone)]
pub struct LanguageButton {
    pub language: Language,
}

/// 语言设置状态
#[derive(Resource)]
pub struct LanguageState {
    pub language: Language,
}

impl Default for LanguageState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            language: settings.language,
        }
    }
}

// ==================== 关闭行为设置组件 ====================

/// 关闭行为按钮
#[derive(Component, Default, Clone)]
pub struct CloseBehaviorButton {
    pub behavior: CloseBehavior,
}

/// 内容过滤设置状态
#[derive(Resource)]
pub struct FilterSettingsState {
    pub blocked_keywords: Vec<String>,
    pub filter_by_category: bool,
    pub filter_by_tag: bool,
    pub filter_by_title: bool,
    pub new_keyword: String,
    /// 是否展开分类建议面板
    pub show_suggestions: bool,
}

impl Default for FilterSettingsState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            blocked_keywords: settings.filter.blocked_keywords.clone(),
            filter_by_category: settings.filter.filter_by_category,
            filter_by_tag: settings.filter.filter_by_tag,
            filter_by_title: settings.filter.filter_by_title,
            new_keyword: String::new(),
            show_suggestions: false,
        }
    }
}

// ==================== 高级设置组件 ====================

/// 界面缩放按钮
#[derive(Component, Default, Clone)]
pub struct UiScaleButton {
    /// 缩放值（0.0 = 自动, 1.0-2.0 = 手动）
    pub scale: f32,
}

/// 界面缩放设置状态
#[derive(Resource)]
pub struct UiScaleState {
    pub scale: f32,
}

/// 自定义字体路径输入框标记
#[derive(Component, Default, Clone)]
pub struct CustomFontPathInput;

/// 自定义字体文件选择按钮
#[derive(Component, Default, Clone)]
pub struct CustomFontPathPickerButton;

/// 自定义字体文件选择器结果
#[derive(Resource, Default)]
pub struct CustomFontPathPickerResult {
    pub receiver: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Option<String>>>>,
}

/// 自定义字体路径输入状态（焦点归 `InputFocus` 管，此处只存值）
#[derive(Resource)]
pub struct CustomFontPathInputState {
    pub value: String,
}

impl Default for CustomFontPathInputState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            value: settings.custom_font_path.clone(),
        }
    }
}

/// SNI 伪装复选框
#[derive(Component, Default, Clone)]
pub struct SniPretendCheckbox;

/// 网络高级设置状态（合并 SNI 伪装与 IPv6 优先以减少系统参数数量）
#[derive(Resource)]
pub struct NetworkAdvancedState {
    /// 是否启用 SNI 伪装
    pub sni_pretend: bool,
    /// 是否优先使用 IPv6
    pub prefer_ipv6: bool,
}

/// IPv6 优先复选框
#[derive(Component, Default, Clone)]
pub struct PreferIpv6Checkbox;

// ==================== 版本更新检查组件 ====================

/// 检查更新按钮
#[derive(Component, Default, Clone)]
pub struct CheckUpdateButton;

/// 更新状态文本
#[derive(Component, Default, Clone)]
pub struct UpdateStatusText;

/// 「前往下载」按钮（仅在检测到新版本时显示）
#[derive(Component, Default, Clone)]
pub struct OpenReleasePageButton;

/// 「启动时自动检查更新」勾选框
#[derive(Component, Default, Clone)]
pub struct AutoCheckUpdateCheckbox;

// ==================== 网络诊断组件 ====================

/// 测速按钮
#[derive(Component, Default, Clone)]
pub struct SpeedTestButton;

/// Ping 测试按钮
#[derive(Component, Default, Clone)]
pub struct PingTestButton;

/// 网络诊断结果文本
#[derive(Component, Default, Clone)]
pub struct NetworkDiagResultText;

/// 创建设置页面 UI
pub fn setup_settings_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
    categories_state: Res<crate::resources::CategoriesState>,
    cached_tags: Res<crate::resources::CachedTagsState>,
    overlay_state: Res<crate::systems::perf_overlay::PerfOverlayState>,
    mut existing_query: Query<&mut Node, With<SettingsRoot>>,
) {
    // 如果 SettingsRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        return;
    }

    let settings = AppSettings::global().read();

    // 查找内容区域
    let content_area = match content_area_query.iter().next() {
        Some(entity) => entity,
        None => {
            tracing::warn!("设置页面：找不到内容区域");
            return;
        }
    };

    // 初始化下载路径输入状态（insert_resource 首次创建，re-enter
    // 时不会执行到此处）
    commands.insert_resource(DownloadPathInputState {
        value: settings.download_path.clone(),
    });
    commands.insert_resource(DownloadPathPickerResult::default());

    // 初始化代理设置状态
    commands.insert_resource(ProxySettingsInputState {
        enabled: settings.proxy.enabled,
        proxy_type: settings.proxy.proxy_type,
        host: settings.proxy.host.clone(),
        port: settings.proxy.port.to_string(),
    });

    // 初始化日志等级状态
    commands.insert_resource(LogLevelInputState {
        level: settings.log_level,
    });

    // 初始化下载行为状态
    commands.insert_resource(DownloadBehaviorState {
        auto_resume: settings.auto_resume_downloads,
        exit_after_all_done: settings.exit_after_downloads,
    });

    // 初始化最大并发下载数状态
    commands.insert_resource(MaxConcurrentDownloadsState {
        value: settings.max_concurrent_downloads,
    });

    // 初始化 CBZ 打包设置状态
    commands.insert_resource(CbzPackageSettingsState {
        auto_pack_cbz: settings.auto_pack_cbz,
        delete_images_after_cbz: settings.delete_images_after_cbz,
    });

    // 初始化内容过滤设置状态
    commands.insert_resource(FilterSettingsState {
        blocked_keywords: settings.filter.blocked_keywords.clone(),
        filter_by_category: settings.filter.filter_by_category,
        filter_by_tag: settings.filter.filter_by_tag,
        filter_by_title: settings.filter.filter_by_title,
        new_keyword: String::new(),
        show_suggestions: false,
    });

    // 初始化分流设置状态
    commands.insert_resource(ChannelSettingsState {
        api_channel: settings.channel.api_channel,
        image_channel: settings.channel.image_channel,
        custom_cdn_api_ip: settings.channel.custom_cdn_api_ip.clone(),
        custom_cdn_img_ip: settings.channel.custom_cdn_img_ip.clone(),
    });

    // 初始化主题 + 关闭行为设置状态
    commands.insert_resource(ThemeModeState {
        mode: settings.theme,
        close_behavior: settings.close_behavior,
    });

    // 初始化语言设置状态
    commands.insert_resource(LanguageState {
        language: settings.language,
    });

    // 初始化高级设置状态
    commands.insert_resource(UiScaleState {
        scale: settings.ui_scale,
    });
    commands.insert_resource(CustomFontPathInputState {
        value: settings.custom_font_path.clone(),
    });
    commands.insert_resource(CustomFontPathPickerResult::default());
    commands.insert_resource(NetworkAdvancedState {
        sni_pretend: settings.use_sni_pretend,
        prefer_ipv6: settings.prefer_ipv6,
    });

    // 初始化保存状态提示
    commands.insert_resource(SettingsSaveStatus::default());

    // 内容过滤分组的建议数据（分类标题 + 缓存标签）
    let category_titles: Vec<String> = categories_state
        .categories
        .iter()
        .map(|c| c.title.clone())
        .collect();
    let tag_titles: Vec<String> = cached_tags.tags.clone();

    // 在内容区域下创建设置页面
    let settings_root = commands
        .spawn_scene(settings_page(
            &settings,
            &category_titles,
            &tag_titles,
            overlay_state.visible,
        ))
        .id();
    commands.entity(content_area).add_child(settings_root);

    tracing::info!("设置页面 UI 已创建");
}

/// 设置页面场景
fn settings_page(
    settings: &AppSettings,
    category_titles: &[String],
    tag_titles: &[String],
    overlay_visible: bool,
) -> impl Scene + use<> {
    // 各分组内容（SceneList 需在 bsn! 宏外构建后传入 settings_section）
    let theme_content = bsn_list![theme_setting(settings.theme)];
    let language_content = bsn_list![language_setting(settings.language)];
    let close_behavior_content = bsn_list![close_behavior_setting(settings.close_behavior)];
    let proxy_content = bsn_list![proxy_setting(settings)];
    let channel_content = bsn_list![channel_setting(settings)];
    let log_level_content = bsn_list![log_level_setting(settings.log_level)];
    let download_content = bsn_list![
        download_path_setting(&settings.download_path),
        max_concurrent_downloads_setting(settings.max_concurrent_downloads),
        auto_resume_downloads_setting(settings.auto_resume_downloads),
        exit_after_downloads_setting(settings.exit_after_downloads),
        auto_pack_cbz_setting(settings.auto_pack_cbz),
        delete_images_after_cbz_setting(settings.delete_images_after_cbz),
    ];
    let filter_content = filter_settings(&settings.filter, category_titles, tag_titles);
    let advanced_content = bsn_list![
        ui_scale_setting(settings.ui_scale),
        custom_font_path_setting(&settings.custom_font_path),
        sni_pretend_setting(settings.use_sni_pretend),
        prefer_ipv6_setting(settings.prefer_ipv6),
    ];
    let cache_content = bsn_list![cache_setting()];
    let network_diag_content = bsn_list![network_diag_section()];
    let profiling_content = bsn_list![profiling_section(
        overlay_visible,
        settings.enable_profiling
    )];
    let about_content = bsn_list![about_section(settings.auto_check_update)];

    bsn! {
        SettingsRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            // 标题栏
            settings_header(),
            (
                // 设置内容（可滚动）- 包装器需要 Relative 定位以支持 Absolute 子元素
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    // 裁剪溢出内容，防止延伸到底部按钮栏
                    overflow: Overflow::clip(),
                }
                Children [
                    (
                        // 滚动容器
                        #SettingsScroll
                        SettingsScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(20.0)),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [
                            // 主题设置分组
                            settings_section("主题设置", theme_content),
                            // 语言设置分组
                            settings_section("语言设置", language_content),
                            // 关闭行为分组
                            settings_section("关闭行为", close_behavior_content),
                            // 代理设置分组
                            settings_section("代理设置", proxy_content),
                            // 分流设置分组
                            settings_section("分流设置", channel_content),
                            // 日志设置分组
                            settings_section("日志设置", log_level_content),
                            // 下载设置分组
                            settings_section("下载设置", download_content),
                            // 内容过滤分组
                            settings_section("内容过滤", filter_content),
                            // 高级设置分组
                            settings_section("高级设置", advanced_content),
                            // 缓存设置分组
                            settings_section("缓存设置", cache_content),
                            // 网络诊断分组
                            settings_section("网络诊断", network_diag_content),
                            // 性能追踪分组
                            settings_section("性能追踪", profiling_content),
                            // 关于分组
                            settings_section("关于", about_content),
                        ]
                    ),
                    // 滚动条
                    scrollbar(#SettingsScroll),
                ]
            ),
            // 底部状态栏（固定在页面底部，显示保存状态提示）
            status_bar(),
        ]
    }
}

/// 创建设置标题栏
fn settings_header() -> impl Scene {
    let title = format!("{ICON_COG} 设置");

    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            padding: UiRect::horizontal(Val::Px(20.0)),
            align_items: AlignItems::Center,
            border: UiRect::bottom(Val::Px(1.0)),
        }
        BackgroundColor(AppColors::HEADER_BG)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                Text({title})
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 创建设置分组（标题 + 任意内容）
fn settings_section<L: SceneList>(title: &str, content: L) -> impl Scene + use<L> {
    let title = title.to_string();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            margin: UiRect::bottom(Val::Px(20.0)),
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(Color::srgb(0.1, 0.1, 0.14))
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 分组标题
                Text({title})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(AppColors::TEXT)
                Node { margin: UiRect::bottom(Val::Px(15.0)) }
            ),
            // 分组内容
            {content},
        ]
    }
}

// ==================== 通用布局片段 ====================

/// 「标签 + 说明 + 按钮组」布局
///
/// 主题 / 语言 / 关闭行为 / 日志等级 / 界面缩放五组设置共用同一套外层结构，
/// 差异只在按钮本身，故按钮列表由调用方构建后传入。
fn option_group<L: SceneList>(label: &str, desc: &str, buttons: L) -> impl Scene + use<L> {
    let label = label.to_string();
    let desc = desc.to_string();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                Text({desc})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 按钮组
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                }
                Children [ {buttons} ]
            ),
        ]
    }
}

/// 单选 / 开关类按钮的静息底色（与 `ButtonStyle::segment` 的解析结果一致，
/// 避免首帧闪烁）
fn selectable_bg(selected: bool) -> Color {
    if selected {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    }
}

/// 单选 / 开关类按钮的边框（选中描主色边）
fn selectable_border(selected: bool) -> BorderColor {
    BorderColor::all(if selected {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    })
}

/// 同步单选 / 开关按钮的选中态：底色交给全局系统，此处只写 `selected` 与边框
fn apply_selected(style: &mut ButtonStyle, border: &mut BorderColor, selected: bool) {
    if style.selected != selected {
        style.selected = selected;
    }
    *border = selectable_border(selected);
}

/// 忙碌态按钮：进行中降为次要色（原先的"置灰"），完成后回到主色
fn set_busy(style: &mut ButtonStyle, busy: bool) {
    let target = if busy {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Primary
    };
    if style.variant != target {
        style.variant = target;
    }
}

/// 写入勾选框内的对勾字符（勾选框的唯一子节点是 Text）
fn set_check_icon<F: bevy::ecs::query::QueryFilter>(
    children: &Children,
    text_query: &mut Query<&mut Text, F>,
    checked: bool,
) {
    for child in children.iter() {
        if let Ok(mut text) = text_query.get_mut(child) {
            **text = if checked {
                ICON_CHECK.to_string()
            } else {
                String::new()
            };
        }
    }
}

/// 「图标 + 文字」选项按钮（主题 / 语言 / 关闭行为共用）
fn icon_option_button<M: Component + Default + Clone + Unpin>(
    marker: M,
    icon: &str,
    label: &str,
    is_selected: bool,
) -> impl Scene + use<M> {
    let icon = icon.to_string();
    let label = label.to_string();
    let style = ButtonStyle::segment(is_selected);
    let bg = selectable_bg(is_selected);
    let border = selectable_border(is_selected);

    bsn! {
        template_value(marker)
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            column_gap: Val::Px(6.0),
            align_items: AlignItems::Center,
        }
        BackgroundColor(bg)
        template_value(border)
        Children [
            (
                // 图标
                Text({icon})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            ),
        ]
    }
}

/// 「左侧标签 + 说明 / 右侧勾选框」行
///
/// 自动恢复下载 / CBZ 打包 / 删除原图 / SNI 伪装 / IPv6 优先五项开关共用。
fn toggle_row<M: Component + Default + Clone + Unpin>(
    marker: M,
    label: &str,
    desc: &str,
    is_enabled: bool,
) -> impl Scene + use<M> {
    let label = label.to_string();
    let desc = desc.to_string();
    let icon = if is_enabled { ICON_CHECK } else { "" };
    let style = ButtonStyle::segment(is_enabled);
    let bg = selectable_bg(is_enabled);
    let border = selectable_border(is_enabled);

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
        }
        Children [
            (
                // 左侧标签和说明
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                }
                Children [
                    (
                        Text({label})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        Text({desc})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            (
                // 右侧勾选框
                template_value(marker)
                Button
                template_value(style)
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor(bg)
                template_value(border)
                Children [
                    (
                        // 勾选标记（使用 Nerd Font 图标）
                        Text({icon})
                        TextFont { font_size: FontSize::Px(16.0) }
                        TextColor(Color::WHITE)
                    )
                ]
            ),
        ]
    }
}

// ==================== 下载设置 ====================

/// 创建下载路径设置
fn download_path_setting(current_path: &str) -> impl Scene + use<> {
    // 输入框（TextInput 通用组件）
    let text_input = TextInput::new("（使用默认路径）").with_value(current_path);
    let display_text = if current_path.is_empty() {
        "（使用默认路径）".to_string()
    } else {
        current_path.to_string()
    };
    let display_color = if current_path.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };
    let move_all_label = format!("{ICON_FILE_MOVE} 迁移全部");

    bsn! {
        // 标签行
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                // 标签
                Text("下载保存路径")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 说明
                Text("留空则使用默认路径（程序目录/Downloads）")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 输入框 + 选择目录按钮 行
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        DownloadPathInput
                        template_value(text_input)
                        Button
                        Node {
                            flex_grow: 1.0,
                            height: Val::Px(36.0),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SURFACE_SUNKEN)
                        template_value(BorderColor::all(AppColors::BORDER))
                        RelativeCursorPosition
                        Children [
                            (
                                TextInputDisplay
                                Text({display_text})
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(display_color)
                            )
                        ]
                    ),
                    (
                        // 选择目录按钮
                        DownloadPathPickerButton
                        Button
                        template_value(ButtonStyle::secondary())
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SECONDARY)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text(ICON_FOLDER_OPEN)
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
            (
                // "全部移动已下载漫画"按钮
                MoveAllDownloadsButton
                Button
                template_value(ButtonStyle::card())
                Node {
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(8.0), Val::Px(8.0)),
                    margin: UiRect::top(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(AppColors::SURFACE)
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        Text({move_all_label})
                        TextFont { font_size: FontSize::Px(13.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
        ]
    }
}

/// 创建最大并发下载数设置
fn max_concurrent_downloads_setting(current_value: usize) -> impl Scene + use<> {
    let value_text = format!("{}", current_value);

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
        }
        Children [
            (
                // 左侧标签和说明
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                }
                Children [
                    (
                        Text("最大同时下载数")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        Text("同时下载的漫画数量上限")
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            (
                // 右侧数值调整器
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                }
                Children [
                    (
                        // 减少按钮
                        MaxConcurrentDownloadsDecreaseButton
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        template_value(BorderColor::all(AppColors::BORDER))
                        BackgroundColor(AppColors::SURFACE)
                        Children [
                            (
                                Text(ICON_MINUS)
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 数值显示
                        MaxConcurrentDownloadsText
                        Text({value_text})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                        Node {
                            width: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                        }
                    ),
                    (
                        // 增加按钮
                        MaxConcurrentDownloadsIncreaseButton
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        template_value(BorderColor::all(AppColors::BORDER))
                        BackgroundColor(AppColors::SURFACE)
                        Children [
                            (
                                Text(ICON_PLUS)
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
        ]
    }
}

/// 创建自动恢复下载设置
fn auto_resume_downloads_setting(is_enabled: bool) -> impl Scene {
    toggle_row(
        AutoResumeDownloadsCheckbox,
        "启动后自动开始下载",
        "程序启动时自动恢复未完成的下载任务",
        is_enabled,
    )
}

/// 创建下载完成后退出设置
fn exit_after_downloads_setting(is_enabled: bool) -> impl Scene {
    toggle_row(
        ExitAfterDownloadsCheckbox,
        "下载全部完成后退出",
        "队列清空（含 CBZ 打包）后自动退出程序，适合挂机下载",
        is_enabled,
    )
}

/// 创建自动打包 CBZ 设置
fn auto_pack_cbz_setting(is_enabled: bool) -> impl Scene {
    toggle_row(
        AutoPackCbzCheckbox,
        "下载完成后自动打包 CBZ",
        "将漫画打包为 CBZ 格式，方便导入阅读器",
        is_enabled,
    )
}

/// 创建打包后删除原图设置
fn delete_images_after_cbz_setting(is_enabled: bool) -> impl Scene {
    toggle_row(
        DeleteImagesAfterCbzCheckbox,
        "打包 CBZ 后删除原图",
        "打包成功后自动删除 Images 目录中的原图",
        is_enabled,
    )
}

// ==================== 内容过滤设置 ====================

/// 创建内容过滤设置（若干平级节点，故返回 SceneList）
fn filter_settings(
    filter: &FilterSettings,
    category_titles: &[String],
    tag_titles: &[String],
) -> Vec<Box<dyn SceneList>> {
    let blocked_tags = blocked_keyword_tags(&filter.blocked_keywords);
    let has_suggestions = !category_titles.is_empty() || !tag_titles.is_empty();
    let suggestion_label = format!("选择分类/标签 {ICON_CHEVRON_DOWN}");

    // "选择分类/标签" 展开/折叠按钮（无候选项时不创建）
    let suggestion_toggle: Box<dyn SceneList> = if has_suggestions {
        Box::new(bsn_list![(
            KeywordSuggestionToggle
            Button
            // 展开态由 keyword_suggestion_toggle_interaction 写入 selected
            template_value(ButtonStyle::segment(false))
            Node {
                height: Val::Px(32.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
            }
            BackgroundColor(AppColors::SURFACE_SUNKEN)
            template_value(BorderColor::all(AppColors::BORDER))
            Children [
                (
                    Text({suggestion_label})
                    TextFont { font_size: FontSize::Px(12.0) }
                    TextColor(AppColors::TEXT_SECONDARY)
                )
            ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    let mut items: Vec<Box<dyn SceneList>> = vec![
        // 屏蔽模式复选框行
        Box::new(bsn_list![(
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(8.0),
            }
            Children [
                filter_mode_checkbox(
                    FilterByCategoryCheckbox,
                    "按分类屏蔽",
                    filter.filter_by_category,
                ),
                filter_mode_checkbox(FilterByTagCheckbox, "按标签屏蔽", filter.filter_by_tag),
                filter_mode_checkbox(FilterByTitleCheckbox, "按标题屏蔽", filter.filter_by_title),
            ]
        )]),
        // 屏蔽词列表标签
        Box::new(bsn_list![(
            Text("屏蔽词列表:")
            TextFont { font_size: FontSize::Px(13.0) }
            TextColor(AppColors::TEXT_SECONDARY)
            Node { margin: UiRect::top(Val::Px(12.0)) }
        )]),
        // 屏蔽词列表容器
        Box::new(bsn_list![(
            BlockedKeywordsListContainer
            Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                margin: UiRect::top(Val::Px(6.0)),
            }
            Children [ {blocked_tags} ]
        )]),
        // 新增屏蔽词输入行
        Box::new(bsn_list![(
            Node {
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(10.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
            }
            Children [
                (
                    // 输入框（TextInput 通用组件）
                    NewKeywordInput
                    template_value(TextInput::new("输入新屏蔽词..."))
                    Button
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(32.0),
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                    }
                    BackgroundColor(AppColors::CARD_BG)
                    template_value(BorderColor::all(AppColors::BORDER))
                    RelativeCursorPosition
                    Children [
                        (
                            TextInputDisplay
                            Text("输入新屏蔽词...")
                            TextFont { font_size: FontSize::Px(12.0) }
                            TextColor(AppColors::TEXT_SECONDARY)
                        )
                    ]
                ),
                (
                    // 添加按钮
                    AddKeywordButton
                    Button
                    template_value(ButtonStyle::primary())
                    Node {
                        height: Val::Px(32.0),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                    }
                    BackgroundColor(AppColors::PRIMARY)
                    Children [
                        (
                            Text("添加")
                            TextFont { font_size: FontSize::Px(12.0) }
                            TextColor(AppColors::TEXT)
                        )
                    ]
                ),
                {suggestion_toggle},
            ]
        )]),
    ];

    // 分类/标签建议面板（初始隐藏）
    if has_suggestions {
        items.push(suggestion_panel(filter, category_titles, tag_titles));
    }

    items
}

/// 创建分类/标签建议面板（初始隐藏）
fn suggestion_panel(
    filter: &FilterSettings,
    category_titles: &[String],
    tag_titles: &[String],
) -> Box<dyn SceneList> {
    let mut sections: Vec<Box<dyn SceneList>> = Vec::new();

    // 分类区域
    if !category_titles.is_empty() {
        let items: Vec<_> = category_titles
            .iter()
            .map(|title| suggestion_item(title, filter, Color::srgba(0.2, 0.2, 0.3, 0.7)))
            .collect();
        sections.push(Box::new(bsn_list![
            (
                // 分类标题
                Text("分类")
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 分类标签列表
                Node {
                    width: Val::Percent(100.0),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                }
                Children [ {items} ]
            ),
        ]));
    }

    // 标签区域
    if !tag_titles.is_empty() {
        let items: Vec<_> = tag_titles
            .iter()
            .map(|title| suggestion_item(title, filter, Color::srgba(0.15, 0.25, 0.2, 0.7)))
            .collect();
        sections.push(Box::new(bsn_list![
            (
                // 标签标题
                Text("标签")
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 标签列表
                Node {
                    width: Val::Percent(100.0),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                }
                Children [ {items} ]
            ),
        ]));
    }

    Box::new(bsn_list![(
        KeywordSuggestionPanel
        Node {
            width: Val::Percent(100.0),
            margin: UiRect::top(Val::Px(6.0)),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            // 初始隐藏
            display: Display::None,
        }
        BackgroundColor(Color::srgba(0.12, 0.12, 0.18, 0.9))
        template_value(BorderColor::all(AppColors::BORDER))
        Children [ {sections} ]
    )])
}

/// 建议项已被屏蔽时的禁用底色
const SUGGESTION_BLOCKED_BG: Color = Color::srgba(0.2, 0.2, 0.25, 0.4);
/// 建议项已被屏蔽时的禁用边框
const SUGGESTION_BLOCKED_BORDER: Color = Color::srgba(0.3, 0.3, 0.35, 0.3);

/// 创建建议面板中的单个建议项
fn suggestion_item(
    keyword: &str,
    filter: &FilterSettings,
    base_color: Color,
) -> impl Scene + use<> {
    let already_blocked = filter.blocked_keywords.contains(&keyword.to_string());
    let bg = if already_blocked {
        SUGGESTION_BLOCKED_BG
    } else {
        base_color
    };
    let text_color = if already_blocked {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };
    let border = BorderColor::all(if already_blocked {
        SUGGESTION_BLOCKED_BORDER
    } else {
        AppColors::BORDER
    });
    let label = keyword.to_string();
    let keyword = keyword.to_string();

    bsn! {
        KeywordSuggestionItem { keyword: {keyword}, base_color: {base_color} }
        Button
        Node {
            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(4.0), Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
        }
        BackgroundColor(bg)
        template_value(border)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(text_color)
            )
        ]
    }
}

/// 创建屏蔽词标签列表（可复用：初次构建 + 动态刷新）
fn blocked_keyword_tags(keywords: &[String]) -> Box<dyn SceneList> {
    if keywords.is_empty() {
        Box::new(bsn_list![(
            Text("暂无屏蔽词")
            TextFont { font_size: FontSize::Px(12.0) }
            TextColor(AppColors::TEXT_SECONDARY)
        )])
    } else {
        Box::new(
            keywords
                .iter()
                .map(|keyword| blocked_keyword_tag(keyword.as_str()))
                .collect::<Vec<_>>(),
        )
    }
}

/// 单个屏蔽词标签（文本 + 删除按钮）
fn blocked_keyword_tag(keyword: &str) -> impl Scene + use<> {
    let label = keyword.to_string();
    let keyword = keyword.to_string();

    bsn! {
        BlockedKeywordItem
        Node {
            padding: UiRect::new(Val::Px(8.0), Val::Px(4.0), Val::Px(3.0), Val::Px(3.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
        }
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.5))
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 删除按钮
                RemoveKeywordButton { keyword: {keyword} }
                Button
                template_value(ButtonStyle::danger())
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                }
                BackgroundColor(AppColors::ERROR)
                Children [
                    (
                        Text(ICON_CLOSE)
                        TextFont { font_size: FontSize::Px(10.0) }
                        TextColor(Color::srgb(0.9, 0.5, 0.5))
                    )
                ]
            ),
        ]
    }
}

/// 屏蔽词列表动态刷新系统：添加/删除屏蔽词后立即更新 UI
pub fn refresh_blocked_keywords_ui(
    mut commands: Commands,
    filter_state: Res<FilterSettingsState>,
    list_query: Query<(Entity, Option<&Children>), With<BlockedKeywordsListContainer>>,
    _asset_server: Res<AssetServer>,
    // 同时更新建议面板的禁用状态
    mut suggestion_query: Query<(
        &KeywordSuggestionItem,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
    )>,
    mut text_color_query: Query<&mut TextColor>,
) {
    if !filter_state.is_changed() {
        return;
    }

    // 重建屏蔽词列表
    for (entity, children) in list_query.iter() {
        // 清除旧子节点
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut entity_cmd) = commands.get_entity(child) {
                    entity_cmd.despawn();
                }
            }
        }

        // 重新创建屏蔽词标签
        commands
            .entity(entity)
            .queue_spawn_related_scenes::<Children>(blocked_keyword_tags(
                &filter_state.blocked_keywords,
            ));
    }

    // 更新建议面板中已屏蔽项的禁用外观
    for (item, mut bg, mut border, children) in suggestion_query.iter_mut() {
        let already_blocked = filter_state.blocked_keywords.contains(&item.keyword);
        if already_blocked {
            *bg = BackgroundColor(SUGGESTION_BLOCKED_BG);
            *border = BorderColor::all(SUGGESTION_BLOCKED_BORDER);
        } else {
            // 还原本项自己的底色：分类与标签两套配色不能互相顶替
            *bg = BackgroundColor(item.base_color);
            *border = BorderColor::all(AppColors::BORDER);
        }
        // 更新文本颜色
        for child in children.iter() {
            if let Ok(mut color) = text_color_query.get_mut(child) {
                *color = if already_blocked {
                    TextColor(AppColors::TEXT_SECONDARY)
                } else {
                    TextColor(AppColors::TEXT)
                };
            }
        }
    }
}

/// 创建过滤模式复选框
fn filter_mode_checkbox<M: Component + Default + Clone + Unpin>(
    marker: M,
    label: &str,
    checked: bool,
) -> impl Scene + use<M> {
    let label = label.to_string();
    let icon = if checked { ICON_CHECK } else { "" };
    let style = ButtonStyle::segment(checked);
    let bg = selectable_bg(checked);
    let border = selectable_border(checked);

    bsn! {
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
        }
        Children [
            (
                // 复选框
                template_value(marker)
                Button
                template_value(style)
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor(bg)
                template_value(border)
                Children [
                    (
                        Text({icon})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(Color::WHITE)
                    )
                ]
            ),
            (
                // 标签文本
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            ),
        ]
    }
}

// ==================== 缓存 / 关于 ====================

/// 创建缓存设置
fn cache_setting() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
        }
        Children [
            (
                // 左侧标签和说明
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                }
                Children [
                    (
                        Text("图片缓存")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        Text("清除本地缓存的封面图片")
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            (
                // 清除按钮
                ClearCacheButton
                Button
                template_value(ButtonStyle::danger())
                Node {
                    padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(AppColors::ERROR)
                template_value(BorderColor::all(AppColors::ERROR))
                Children [
                    (
                        Text("清除缓存")
                        TextFont { font_size: FontSize::Px(13.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
        ]
    }
}

/// 创建关于分组
fn about_section(auto_check_update: bool) -> impl Scene + use<> {
    let version_text = format!("当前版本: v{}", env!("CARGO_PKG_VERSION"));
    let bevy_text = format!("框架: Bevy {}", env!("BEVY_VERSION"));
    let check_update_label = format!("{ICON_REFRESH} 检查更新");
    let open_release_label = format!("{ICON_DOWNLOAD} 前往下载");

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                Text("PicACG Rust 客户端")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                Text({version_text})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                Text({bevy_text})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 检查更新按钮行
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    margin: UiRect::top(Val::Px(8.0)),
                }
                Children [
                    (
                        // 检查更新按钮
                        CheckUpdateButton
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        template_value(BorderColor::all(AppColors::PRIMARY))
                        Children [
                            (
                                Text({check_update_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(Color::WHITE)
                            )
                        ]
                    ),
                    (
                        // 前往下载按钮：默认隐藏，检测到新版本时由
                        // refresh_update_status 显示
                        OpenReleasePageButton
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            display: Display::None,
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::PRIMARY))
                        Children [
                            (
                                Text({open_release_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 更新状态文本
                        UpdateStatusText
                        Text(" ")
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            toggle_row(
                AutoCheckUpdateCheckbox,
                "启动时自动检查更新",
                "登录后在后台查一次 GitHub Releases，只查不装",
                auto_check_update,
            ),
        ]
    }
}

// ==================== 代理设置 ====================

/// 创建代理设置
fn proxy_setting(settings: &AppSettings) -> impl Scene + use<> {
    let enabled = settings.proxy.enabled;
    let check_icon = if enabled { ICON_CHECK } else { "" };
    let checkbox_style = ButtonStyle::segment(enabled);
    let checkbox_bg = selectable_bg(enabled);
    let checkbox_border = selectable_border(enabled);

    // 代理类型按钮组
    let type_buttons: Vec<_> = [
        (ProxyType::Http, "HTTP"),
        (ProxyType::Https, "HTTPS"),
        (ProxyType::Socks5, "SOCKS5"),
    ]
    .into_iter()
    .map(|(proxy_type, label)| {
        proxy_type_button(proxy_type, label, settings.proxy.proxy_type == proxy_type)
    })
    .collect();

    // 主机地址输入框
    let host_input = TextInput::new("127.0.0.1").with_value(&settings.proxy.host);
    let host_text = if settings.proxy.host.is_empty() {
        "127.0.0.1".to_string()
    } else {
        settings.proxy.host.clone()
    };
    let host_color = if settings.proxy.host.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };

    // 端口输入框
    let port_str = settings.proxy.port.to_string();
    let port_input = TextInput::new("7890").with_value(port_str.clone());
    let port_text = if port_str.is_empty() {
        "7890".to_string()
    } else {
        port_str.clone()
    };
    let port_color = if port_str.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
        }
        Children [
            (
                // 启用代理复选框
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                }
                Children [
                    (
                        ProxyEnabledCheckbox
                        Button
                        template_value(checkbox_style)
                        Node {
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(checkbox_bg)
                        template_value(checkbox_border)
                        Children [
                            (
                                Text({check_icon})
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        Text("启用代理")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    ),
                ]
            ),
            (
                // 代理类型选择
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                }
                Children [
                    (
                        Text("代理类型")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                        }
                        Children [ {type_buttons} ]
                    ),
                ]
            ),
            (
                // 代理地址和端口
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                }
                Children [
                    (
                        // 主机地址
                        Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                        }
                        Children [
                            (
                                Text("主机地址")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            ),
                            (
                                ProxyHostInput
                                template_value(host_input)
                                Button
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(32.0),
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                }
                                BackgroundColor(AppColors::SURFACE_SUNKEN)
                                template_value(BorderColor::all(AppColors::BORDER))
                                RelativeCursorPosition
                                Children [
                                    (
                                        TextInputDisplay
                                        Text({host_text})
                                        TextFont { font_size: FontSize::Px(13.0) }
                                        TextColor(host_color)
                                    )
                                ]
                            ),
                        ]
                    ),
                    (
                        // 端口
                        Node {
                            width: Val::Px(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                        }
                        Children [
                            (
                                Text("端口")
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            ),
                            (
                                ProxyPortInput
                                template_value(port_input)
                                Button
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(32.0),
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                }
                                BackgroundColor(AppColors::SURFACE_SUNKEN)
                                template_value(BorderColor::all(AppColors::BORDER))
                                RelativeCursorPosition
                                Children [
                                    (
                                        TextInputDisplay
                                        Text({port_text})
                                        TextFont { font_size: FontSize::Px(13.0) }
                                        TextColor(port_color)
                                    )
                                ]
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
}

/// 单个代理类型按钮
fn proxy_type_button(proxy_type: ProxyType, label: &str, is_selected: bool) -> impl Scene + use<> {
    let label = label.to_string();
    let style = ButtonStyle::segment(is_selected);
    let bg = selectable_bg(is_selected);
    let border = selectable_border(is_selected);

    bsn! {
        ProxyTypeButton { proxy_type: {proxy_type} }
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(bg)
        template_value(border)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

// ==================== 分流设置 ====================

/// 创建分流设置
fn channel_setting(settings: &AppSettings) -> impl Scene + use<> {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
        }
        Children [
            // API 分流选择
            channel_row("API 分流", settings.channel.api_channel, true),
            // 自定义 API IP 输入行
            custom_ip_row(
                CustomCdnApiIpRow,
                CustomCdnApiIpInput,
                "自定义 API IP",
                &settings.channel.custom_cdn_api_ip,
                settings.channel.api_channel == ChannelType::CustomCdnIp,
            ),
            // 图片分流选择
            channel_row("图片分流", settings.channel.image_channel, false),
            // 自定义图片 IP 输入行
            custom_ip_row(
                CustomCdnImgIpRow,
                CustomCdnImgIpInput,
                "自定义图片 IP",
                &settings.channel.custom_cdn_img_ip,
                settings.channel.image_channel == ChannelType::CustomCdnIp,
            ),
        ]
    }
}

/// 创建分流按钮行
fn channel_row(label: &str, current: ChannelType, is_api: bool) -> impl Scene + use<> {
    let label = label.to_string();

    // API / 图片两套标记组件不同，分别构建按钮列表
    let buttons: Box<dyn SceneList> = if is_api {
        Box::new(
            ChannelType::all()
                .iter()
                .map(|channel_type| {
                    channel_button(
                        ApiChannelButton {
                            channel_type: *channel_type,
                        },
                        channel_type.display_name(),
                        current == *channel_type,
                    )
                })
                .collect::<Vec<_>>(),
        )
    } else {
        Box::new(
            ChannelType::all()
                .iter()
                .map(|channel_type| {
                    channel_button(
                        ImageChannelButton {
                            channel_type: *channel_type,
                        },
                        channel_type.display_name(),
                        current == *channel_type,
                    )
                })
                .collect::<Vec<_>>(),
        )
    };

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(8.0),
                }
                Children [ {buttons} ]
            ),
        ]
    }
}

/// 单个分流按钮
fn channel_button<M: Component + Default + Clone + Unpin>(
    marker: M,
    label: &str,
    is_selected: bool,
) -> impl Scene + use<M> {
    let label = label.to_string();
    let style = ButtonStyle::segment(is_selected);
    let bg = selectable_bg(is_selected);
    let border = selectable_border(is_selected);

    bsn! {
        template_value(marker)
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(bg)
        template_value(border)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 创建自定义 IP 输入行
///
/// 行容器与输入框的标记组件按 API / 图片区分，故由调用方传入。
fn custom_ip_row<R, I>(
    row_marker: R,
    input_marker: I,
    label: &str,
    value: &str,
    visible: bool,
) -> impl Scene + use<R, I>
where
    R: Component + Default + Clone + Unpin,
    I: Component + Default + Clone + Unpin,
{
    let label = label.to_string();
    let row_display = if visible {
        Display::Flex
    } else {
        Display::None
    };

    let placeholder = "输入 IP 地址，例如 104.21.91.145";
    let text_input = TextInput::new(placeholder).with_value(value);
    let display_text = if value.is_empty() {
        placeholder.to_string()
    } else {
        value.to_string()
    };
    let text_color = if value.is_empty() {
        Color::srgb(0.4, 0.4, 0.5)
    } else {
        AppColors::TEXT
    };

    bsn! {
        template_value(row_marker)
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            display: {row_display},
        }
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                template_value(input_marker)
                template_value(text_input)
                Button
                Node {
                    flex_grow: 1.0,
                    height: Val::Px(32.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(AppColors::SURFACE_SUNKEN)
                template_value(BorderColor::all(AppColors::BORDER))
                RelativeCursorPosition
                Children [
                    (
                        TextInputDisplay
                        Text({display_text})
                        TextFont { font_size: FontSize::Px(13.0) }
                        TextColor(text_color)
                    )
                ]
            ),
        ]
    }
}

// ==================== 主题 / 语言 / 关闭行为 / 日志 ====================

/// 创建主题设置
fn theme_setting(current_mode: ThemeMode) -> impl Scene + use<> {
    let buttons: Vec<_> = ThemeMode::all()
        .iter()
        .map(|mode| {
            // 图标
            let icon = match mode {
                ThemeMode::Dark => "🌙",
                ThemeMode::Light => "☀",
                ThemeMode::Auto => "🔄",
            };
            icon_option_button(
                ThemeModeButton { mode: *mode },
                icon,
                mode.display_name(),
                current_mode == *mode,
            )
        })
        .collect();

    option_group(
        "主题模式",
        "切换应用的颜色主题，修改后需重启应用生效",
        buttons,
    )
}

/// 创建语言设置
fn language_setting(current_language: Language) -> impl Scene + use<> {
    let buttons: Vec<_> = Language::all()
        .iter()
        .map(|lang| {
            // 语言标识图标
            let icon = match lang {
                Language::ZhCN => "简",
                Language::ZhTW => "繁",
                Language::En => "En",
            };
            icon_option_button(
                LanguageButton { language: *lang },
                icon,
                lang.display_name(),
                current_language == *lang,
            )
        })
        .collect();

    option_group(
        "界面语言",
        "切换界面显示语言，修改后需重启应用生效",
        buttons,
    )
}

/// 创建关闭行为设置
fn close_behavior_setting(current_behavior: CloseBehavior) -> impl Scene + use<> {
    let buttons: Vec<_> = CloseBehavior::all()
        .iter()
        .map(|behavior| {
            let icon = match behavior {
                CloseBehavior::Close => "✕",
                CloseBehavior::Minimize => "▼",
                CloseBehavior::Ask => "？",
            };
            icon_option_button(
                CloseBehaviorButton {
                    behavior: *behavior,
                },
                icon,
                behavior.display_name(),
                current_behavior == *behavior,
            )
        })
        .collect();

    option_group("关闭按钮行为", "点击窗口关闭按钮时的行为", buttons)
}

/// 创建日志等级设置
fn log_level_setting(current_level: LogLevel) -> impl Scene + use<> {
    let buttons: Vec<_> = [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ]
    .into_iter()
    .map(|level| log_level_button(level, current_level == level))
    .collect();

    option_group("日志等级", "设置日志输出的详细程度，重启后生效", buttons)
}

/// 单个日志等级按钮
fn log_level_button(level: LogLevel, is_selected: bool) -> impl Scene + use<> {
    let label = level.display_name().to_string();
    let style = ButtonStyle::segment(is_selected);
    let bg = selectable_bg(is_selected);
    let border = selectable_border(is_selected);

    bsn! {
        LogLevelButton { level: {level} }
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(bg)
        template_value(border)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

// ==================== 高级设置 UI 构建 ====================

/// 界面缩放选项
const UI_SCALE_OPTIONS: &[(f32, &str)] = &[
    (0.0, "自动"),
    (1.0, "100%"),
    (1.25, "125%"),
    (1.5, "150%"),
    (1.75, "175%"),
    (2.0, "200%"),
];

/// 创建界面缩放设置
fn ui_scale_setting(current_scale: f32) -> impl Scene + use<> {
    let buttons: Vec<_> = UI_SCALE_OPTIONS
        .iter()
        .map(|&(scale, label)| {
            // 浮点比较：差值小于 0.01 视为相同
            ui_scale_button(scale, label, (current_scale - scale).abs() < 0.01)
        })
        .collect();

    option_group(
        "界面缩放",
        "调整界面缩放比例，修改后需重启应用生效",
        buttons,
    )
}

/// 单个界面缩放按钮
fn ui_scale_button(scale: f32, label: &str, is_selected: bool) -> impl Scene + use<> {
    let label = label.to_string();
    let style = ButtonStyle::segment(is_selected);
    let bg = selectable_bg(is_selected);
    let border = selectable_border(is_selected);

    bsn! {
        UiScaleButton { scale: {scale} }
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            align_items: AlignItems::Center,
        }
        BackgroundColor(bg)
        template_value(border)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 创建自定义字体路径设置
fn custom_font_path_setting(current_path: &str) -> impl Scene + use<> {
    // 输入框（TextInput 通用组件）
    let text_input = TextInput::new("（使用内置字体）").with_value(current_path);
    let display_text = if current_path.is_empty() {
        "（使用内置字体）".to_string()
    } else {
        current_path.to_string()
    };
    let display_color = if current_path.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            margin: UiRect::top(Val::Px(16.0)),
        }
        Children [
            (
                Text("自定义字体")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                Text("指定自定义字体文件路径（.ttf/.otf），留空使用内置字体，修改后需重启生效")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 输入框 + 文件选择按钮 行
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        CustomFontPathInput
                        template_value(text_input)
                        Button
                        Node {
                            flex_grow: 1.0,
                            height: Val::Px(36.0),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SURFACE_SUNKEN)
                        template_value(BorderColor::all(AppColors::BORDER))
                        RelativeCursorPosition
                        Children [
                            (
                                TextInputDisplay
                                Text({display_text})
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(display_color)
                            )
                        ]
                    ),
                    (
                        // 文件选择按钮
                        CustomFontPathPickerButton
                        Button
                        template_value(ButtonStyle::secondary())
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SECONDARY)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text(ICON_FOLDER_OPEN)
                                TextFont { font_size: FontSize::Px(16.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
        ]
    }
}

/// 创建 SNI 伪装设置
fn sni_pretend_setting(is_enabled: bool) -> impl Scene {
    toggle_row(
        SniPretendCheckbox,
        "SNI 伪装",
        "禁用 TLS SNI 扩展以绕过 SNI 封锁，可能导致部分服务不可用",
        is_enabled,
    )
}

/// 创建 IPv6 优先设置
fn prefer_ipv6_setting(is_enabled: bool) -> impl Scene {
    toggle_row(
        PreferIpv6Checkbox,
        "IPv6 优先",
        "绑定本地 IPv6 地址发起连接，需要网络环境支持 IPv6",
        is_enabled,
    )
}

/// 创建底部状态栏（显示自动保存提示）
fn status_bar() -> impl Scene {
    bsn! {
        SettingsStatusBar
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            padding: UiRect::horizontal(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::top(Val::Px(1.0)),
            // 初始隐藏
            display: Display::None,
        }
        BackgroundColor(AppColors::HEADER_BG)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                SettingsStatusText
                Text(" ")
                TextFont { font_size: FontSize::Px(13.0) }
                // 绿色成功提示
                TextColor(Color::srgb(0.4, 0.8, 0.5))
            )
        ]
    }
}

/// 清理设置页面（隐藏而非销毁，保留所有资源状态）
pub fn cleanup_settings_ui(mut query: Query<&mut Node, With<SettingsRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 下载路径动作键处理（Escape/Enter 失焦），点击聚焦与编辑由通用 TextInput
/// 系统处理
pub fn download_path_keyboard_input(
    mut input_focus: ResMut<InputFocus>,
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    input_query: Query<(), With<DownloadPathInput>>,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    if !input_focus.get().is_some_and(|e| input_query.contains(e)) {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if matches!(&event.logical_key, Key::Escape | Key::Enter) {
            input_focus.clear();
        }
    }
}

/// 同步 TextInput.value → DownloadPathInputState
pub fn sync_download_path_value(
    mut input_state: ResMut<DownloadPathInputState>,
    query: Query<&TextInput, (Changed<TextInput>, With<DownloadPathInput>)>,
) {
    for input in query.iter() {
        if input_state.value != input.value {
            input_state.value.clone_from(&input.value);
        }
    }
}

/// 下载路径目录选择按钮交互（异步，不阻塞主线程）
pub fn download_path_picker_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<DownloadPathPickerButton>)>,
    mut picker: ResMut<DownloadPathPickerResult>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 防止重复打开
        if picker.receiver.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            picker.receiver = Some(std::sync::Mutex::new(rx));
            std::thread::spawn(move || {
                let path = rfd::FileDialog::new()
                    .pick_folder()
                    .map(|p| p.to_string_lossy().to_string());
                let _ = tx.send(path);
            });
        }
    }
}

/// 轮询目录选择器的异步结果
pub fn handle_download_path_picker_result(
    mut picker: ResMut<DownloadPathPickerResult>,
    mut input_query: Query<&mut TextInput, With<DownloadPathInput>>,
    mut input_state: ResMut<DownloadPathInputState>,
) {
    let Some(ref receiver) = picker.receiver else {
        return;
    };
    let Ok(receiver) = receiver.lock() else {
        return;
    };
    let Ok(result) = receiver.try_recv() else {
        return;
    };
    drop(receiver);
    // 收到结果，清除 receiver
    picker.receiver = None;
    if let Some(path_str) = result {
        for mut input in input_query.iter_mut() {
            input.set_value(path_str.clone());
        }
        input_state.value = path_str;
    }
}

/// 清除缓存按钮交互
pub fn clear_cache_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClearCacheButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 清除缓存目录
        let cache_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("cache")
            .join("images");

        if cache_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&cache_path) {
                tracing::error!("清除缓存失败: {}", e);
            } else {
                tracing::info!("缓存已清除: {:?}", cache_path);
            }
        } else {
            tracing::info!("缓存目录不存在");
        }
    }
}

/// 将所有设置状态写入 AppSettings 并保存到磁盘
fn save_all_settings(
    input_state: &DownloadPathInputState,
    proxy_state: &ProxySettingsInputState,
    log_state: &LogLevelInputState,
    behavior_state: &DownloadBehaviorState,
    max_concurrent_state: &MaxConcurrentDownloadsState,
    cbz_state: &CbzPackageSettingsState,
    filter_state: &FilterSettingsState,
    channel_state: &ChannelSettingsState,
    theme_state: &ThemeModeState,
    language_state: &LanguageState,
    ui_scale_state: &UiScaleState,
    font_path_state: &CustomFontPathInputState,
    network_advanced_state: &NetworkAdvancedState,
) -> Result<(), String> {
    let mut settings = AppSettings::global().write();
    settings.download_path = input_state.value.clone();
    settings.proxy.enabled = proxy_state.enabled;
    settings.proxy.proxy_type = proxy_state.proxy_type;
    settings.proxy.host = proxy_state.host.clone();
    settings.proxy.port = proxy_state.port.parse().unwrap_or(7890);
    settings.log_level = log_state.level;
    settings.auto_resume_downloads = behavior_state.auto_resume;
    settings.exit_after_downloads = behavior_state.exit_after_all_done;
    settings.max_concurrent_downloads = max_concurrent_state.value;
    settings.auto_pack_cbz = cbz_state.auto_pack_cbz;
    settings.delete_images_after_cbz = cbz_state.delete_images_after_cbz;
    settings.filter = FilterSettings {
        blocked_keywords: filter_state.blocked_keywords.clone(),
        filter_by_category: filter_state.filter_by_category,
        filter_by_tag: filter_state.filter_by_tag,
        filter_by_title: filter_state.filter_by_title,
    };
    settings.channel.api_channel = channel_state.api_channel;
    settings.channel.image_channel = channel_state.image_channel;
    settings.channel.custom_cdn_api_ip = channel_state.custom_cdn_api_ip.clone();
    settings.channel.custom_cdn_img_ip = channel_state.custom_cdn_img_ip.clone();
    settings.theme = theme_state.mode;
    settings.close_behavior = theme_state.close_behavior;
    settings.language = language_state.language;
    settings.ui_scale = ui_scale_state.scale;
    settings.custom_font_path = font_path_state.value.clone();
    settings.use_sni_pretend = network_advanced_state.sni_pretend;
    settings.prefer_ipv6 = network_advanced_state.prefer_ipv6;
    settings.save().map_err(|e| e.to_string())?;
    update_log_level(log_state.level);
    Ok(())
}

/// 自动保存设置：监听所有设置状态变化，有变化时自动保存
pub fn auto_save_settings(
    input_state: Res<DownloadPathInputState>,
    proxy_state: Res<ProxySettingsInputState>,
    log_state: Res<LogLevelInputState>,
    behavior_state: Res<DownloadBehaviorState>,
    max_concurrent_state: Res<MaxConcurrentDownloadsState>,
    cbz_state: Res<CbzPackageSettingsState>,
    filter_state: Res<FilterSettingsState>,
    channel_state: Res<ChannelSettingsState>,
    theme_state: Res<ThemeModeState>,
    language_state: Res<LanguageState>,
    ui_scale_state: Res<UiScaleState>,
    font_path_state: Res<CustomFontPathInputState>,
    network_advanced_state: Res<NetworkAdvancedState>,
    mut save_status: ResMut<SettingsSaveStatus>,
    mut reload_api_messages: MessageWriter<crate::events::ReloadApiClientEvent>,
    mut initialized: Local<bool>,
) {
    let channel_changed = channel_state.is_changed();
    let network_advanced_changed = network_advanced_state.is_changed();
    let any_changed = input_state.is_changed()
        || proxy_state.is_changed()
        || log_state.is_changed()
        || behavior_state.is_changed()
        || max_concurrent_state.is_changed()
        || cbz_state.is_changed()
        || filter_state.is_changed()
        || channel_changed
        || theme_state.is_changed()
        || language_state.is_changed()
        || ui_scale_state.is_changed()
        || font_path_state.is_changed()
        || network_advanced_changed;

    if !any_changed {
        return;
    }

    // 跳过进入设置页面后的第一帧（setup_settings_ui 插入资源会触发 is_changed）
    if !*initialized {
        *initialized = true;
        return;
    }

    match save_all_settings(
        &input_state,
        &proxy_state,
        &log_state,
        &behavior_state,
        &max_concurrent_state,
        &cbz_state,
        &filter_state,
        &channel_state,
        &theme_state,
        &language_state,
        &ui_scale_state,
        &font_path_state,
        &network_advanced_state,
    ) {
        Ok(()) => {
            save_status.visible = true;
            save_status.message = "设置已保存".to_string();
            save_status.is_error = false;
            save_status.timer.reset();
            tracing::debug!("设置已自动保存");

            // 分流/代理/SNI/IPv6 变更时通知重建 API 客户端
            if channel_changed || proxy_state.is_changed() || network_advanced_changed {
                reload_api_messages.write(crate::events::ReloadApiClientEvent);
                tracing::info!("网络设置变更，通知重建 API 客户端");
            }
        }
        Err(e) => {
            save_status.visible = true;
            save_status.message = format!("保存失败: {}", e);
            save_status.is_error = true;
            save_status.timer.reset();
            tracing::error!("自动保存设置失败: {}", e);
        }
    }
}

/// 更新底部状态栏显示（倒计时结束后自动隐藏）
pub fn update_settings_save_status(
    time: Res<Time>,
    mut save_status: ResMut<SettingsSaveStatus>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<SettingsStatusText>>,
    mut bar_query: Query<&mut Node, With<SettingsStatusBar>>,
) {
    if !save_status.visible {
        return;
    }

    save_status.timer.tick(time.delta());

    // 更新文本内容和颜色（比较后写：toast 可见的 2 秒内不再每帧克隆/标脏）
    for (mut text, mut color) in text_query.iter_mut() {
        if **text != save_status.message {
            **text = save_status.message.clone();
        }
        let target = if save_status.is_error {
            TextColor(AppColors::ERROR)
        } else {
            TextColor(Color::srgb(0.4, 0.8, 0.5)) // 绿色成功提示
        };
        if color.0 != target.0 {
            *color = target;
        }
    }

    // 显示状态栏
    for mut node in bar_query.iter_mut() {
        if node.display != Display::Flex {
            node.display = Display::Flex;
        }
    }

    if save_status.timer.just_finished() {
        save_status.visible = false;
        for mut node in bar_query.iter_mut() {
            node.display = Display::None;
        }
    }
}

// ==================== 代理设置交互系统 ====================

/// 代理启用复选框交互
pub fn proxy_enabled_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<ProxyEnabledCheckbox>),
    >,
    mut proxy_state: ResMut<ProxySettingsInputState>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        proxy_state.enabled = !proxy_state.enabled;
        apply_selected(&mut style, &mut border_color, proxy_state.enabled);
        set_check_icon(children, &mut text_query, proxy_state.enabled);
    }
}

/// 代理类型按钮交互（单选组：选中态写入 `ButtonStyle.selected`）
pub fn proxy_type_button_interaction(
    interaction_query: Query<(&Interaction, &ProxyTypeButton), Changed<Interaction>>,
    mut proxy_state: ResMut<ProxySettingsInputState>,
    mut all_buttons_query: Query<(&ProxyTypeButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    let Some(picked) = interaction_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, btn)| btn.proxy_type)
    else {
        return;
    };

    proxy_state.proxy_type = picked;

    for (btn, mut style, mut border) in &mut all_buttons_query {
        apply_selected(&mut style, &mut border, btn.proxy_type == picked);
    }
}

/// 代理输入动作键处理（Escape/Enter 失焦），点击聚焦与编辑由通用 TextInput
/// 系统处理
pub fn proxy_input_keyboard(
    mut input_focus: ResMut<InputFocus>,
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    input_query: Query<(), Or<(With<ProxyHostInput>, With<ProxyPortInput>)>>,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    if !input_focus.get().is_some_and(|e| input_query.contains(e)) {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if matches!(&event.logical_key, Key::Escape | Key::Enter) {
            input_focus.clear();
        }
    }
}

/// 同步 TextInput.value → ProxySettingsInputState
pub fn sync_proxy_input_values(
    mut proxy_state: ResMut<ProxySettingsInputState>,
    host_query: Query<
        &TextInput,
        (
            Changed<TextInput>,
            With<ProxyHostInput>,
            Without<ProxyPortInput>,
        ),
    >,
    port_query: Query<
        &TextInput,
        (
            Changed<TextInput>,
            With<ProxyPortInput>,
            Without<ProxyHostInput>,
        ),
    >,
) {
    for input in host_query.iter() {
        if proxy_state.host != input.value {
            proxy_state.host.clone_from(&input.value);
        }
    }
    for input in port_query.iter() {
        if proxy_state.port != input.value {
            proxy_state.port.clone_from(&input.value);
        }
    }
}

// ==================== 主题设置交互系统 ====================

/// 主题模式按钮交互（单选组：选中态写入 `ButtonStyle.selected`）
pub fn theme_mode_button_interaction(
    interaction_query: Query<(&Interaction, &ThemeModeButton), Changed<Interaction>>,
    mut theme_state: ResMut<ThemeModeState>,
    mut save_status: ResMut<SettingsSaveStatus>,
    mut all_buttons: Query<(&ThemeModeButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    // 只在选到不同模式时才动作
    let Some(mode) = interaction_query
        .iter()
        .find(|(interaction, btn)| {
            **interaction == Interaction::Pressed && theme_state.mode != btn.mode
        })
        .map(|(_, btn)| btn.mode)
    else {
        return;
    };

    tracing::info!("主题模式已选择: {:?}", mode);
    theme_state.mode = mode;

    // 显示重启提示
    save_status.visible = true;
    save_status.message = format!("主题已切换为「{}」，重启应用后生效", mode.display_name());
    save_status.is_error = false;
    save_status.timer.reset();

    for (btn, mut style, mut border) in &mut all_buttons {
        apply_selected(&mut style, &mut border, btn.mode == mode);
    }
}

// ==================== 语言设置交互系统 ====================

/// 语言选择按钮交互（单选组：选中态写入 `ButtonStyle.selected`）
pub fn language_button_interaction(
    interaction_query: Query<(&Interaction, &LanguageButton), Changed<Interaction>>,
    mut language_state: ResMut<LanguageState>,
    mut save_status: ResMut<SettingsSaveStatus>,
    mut all_buttons: Query<(&LanguageButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    let Some(lang) = interaction_query
        .iter()
        .find(|(interaction, btn)| {
            **interaction == Interaction::Pressed && language_state.language != btn.language
        })
        .map(|(_, btn)| btn.language)
    else {
        return;
    };

    tracing::info!("界面语言已选择: {:?}", lang);
    language_state.language = lang;

    // 显示重启提示
    save_status.visible = true;
    save_status.message = format!("语言已切换为「{}」，重启应用后生效", lang.display_name());
    save_status.is_error = false;
    save_status.timer.reset();

    for (btn, mut style, mut border) in &mut all_buttons {
        apply_selected(&mut style, &mut border, btn.language == lang);
    }
}

// ==================== 关闭行为设置交互系统 ====================

/// 关闭行为按钮交互（单选组：选中态写入 `ButtonStyle.selected`）
pub fn close_behavior_button_interaction(
    interaction_query: Query<(&Interaction, &CloseBehaviorButton), Changed<Interaction>>,
    mut theme_state: ResMut<ThemeModeState>,
    mut all_buttons: Query<(&CloseBehaviorButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    let Some(behavior) = interaction_query
        .iter()
        .find(|(interaction, btn)| {
            **interaction == Interaction::Pressed && theme_state.close_behavior != btn.behavior
        })
        .map(|(_, btn)| btn.behavior)
    else {
        return;
    };

    tracing::info!("关闭行为已选择: {:?}", behavior);
    theme_state.close_behavior = behavior;

    for (btn, mut style, mut border) in &mut all_buttons {
        apply_selected(&mut style, &mut border, btn.behavior == behavior);
    }
}

// ==================== 日志等级交互系统 ====================

/// 日志等级按钮交互（单选组：选中态写入 `ButtonStyle.selected`）
pub fn log_level_button_interaction(
    interaction_query: Query<(&Interaction, &LogLevelButton), Changed<Interaction>>,
    mut log_state: ResMut<LogLevelInputState>,
    mut all_buttons: Query<(&LogLevelButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    let Some(level) = interaction_query
        .iter()
        .find(|(interaction, btn)| {
            **interaction == Interaction::Pressed && log_state.level != btn.level
        })
        .map(|(_, btn)| btn.level)
    else {
        return;
    };

    tracing::info!("日志等级已选择: {:?}", level);
    log_state.level = level;

    for (btn, mut style, mut border) in &mut all_buttons {
        apply_selected(&mut style, &mut border, btn.level == level);
    }
}

// ==================== 自动恢复下载交互系统 ====================

/// 自动恢复下载勾选框交互
pub fn auto_resume_downloads_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<AutoResumeDownloadsCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut behavior_state: ResMut<DownloadBehaviorState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        behavior_state.auto_resume = !behavior_state.auto_resume;
        let is_enabled = behavior_state.auto_resume;

        tracing::info!("自动恢复下载: {}", if is_enabled { "启用" } else { "禁用" });

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

/// 下载完成后退出勾选框交互
pub fn exit_after_downloads_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<ExitAfterDownloadsCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut behavior_state: ResMut<DownloadBehaviorState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        behavior_state.exit_after_all_done = !behavior_state.exit_after_all_done;
        let is_enabled = behavior_state.exit_after_all_done;

        tracing::info!(
            "下载全部完成后退出: {}",
            if is_enabled { "启用" } else { "禁用" }
        );

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

// ==================== 最大并发下载数交互系统 ====================

/// 最大并发下载数减少按钮交互
pub fn max_concurrent_downloads_decrease_interaction(
    interaction_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<MaxConcurrentDownloadsDecreaseButton>,
        ),
    >,
    mut state: ResMut<MaxConcurrentDownloadsState>,
    mut text_query: Query<&mut Text, With<MaxConcurrentDownloadsText>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 最小值为 1
        if state.value > 1 {
            state.value -= 1;
            tracing::info!("最大并发下载数: {}", state.value);
            // 更新显示文本
            for mut text in text_query.iter_mut() {
                **text = format!("{}", state.value);
            }
        }
    }
}

/// 最大并发下载数增加按钮交互
pub fn max_concurrent_downloads_increase_interaction(
    interaction_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<MaxConcurrentDownloadsIncreaseButton>,
        ),
    >,
    mut state: ResMut<MaxConcurrentDownloadsState>,
    mut text_query: Query<&mut Text, With<MaxConcurrentDownloadsText>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 最大值为 10
        if state.value < 10 {
            state.value += 1;
            tracing::info!("最大并发下载数: {}", state.value);
            // 更新显示文本
            for mut text in text_query.iter_mut() {
                **text = format!("{}", state.value);
            }
        }
    }
}

// ==================== CBZ 打包设置交互系统 ====================

/// 自动打包 CBZ 勾选框交互
pub fn auto_pack_cbz_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<AutoPackCbzCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut cbz_state: ResMut<CbzPackageSettingsState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        cbz_state.auto_pack_cbz = !cbz_state.auto_pack_cbz;
        let is_enabled = cbz_state.auto_pack_cbz;

        tracing::info!("自动打包 CBZ: {}", if is_enabled { "启用" } else { "禁用" });

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

/// 打包后删除原图勾选框交互
pub fn delete_images_after_cbz_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<DeleteImagesAfterCbzCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut cbz_state: ResMut<CbzPackageSettingsState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        cbz_state.delete_images_after_cbz = !cbz_state.delete_images_after_cbz;
        let is_enabled = cbz_state.delete_images_after_cbz;

        tracing::info!(
            "打包后删除原图: {}",
            if is_enabled { "启用" } else { "禁用" }
        );

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

// ==================== 内容过滤交互系统 ====================

/// 按分类屏蔽复选框交互
pub fn filter_by_category_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<FilterByCategoryCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        filter_state.filter_by_category = !filter_state.filter_by_category;
        let checked = filter_state.filter_by_category;
        apply_selected(&mut style, &mut border_color, checked);
        set_check_icon(children, &mut text_query, checked);
    }
}

/// 按标签屏蔽复选框交互
pub fn filter_by_tag_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<FilterByTagCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        filter_state.filter_by_tag = !filter_state.filter_by_tag;
        let checked = filter_state.filter_by_tag;
        apply_selected(&mut style, &mut border_color, checked);
        set_check_icon(children, &mut text_query, checked);
    }
}

/// 按标题屏蔽复选框交互
pub fn filter_by_title_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<FilterByTitleCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        filter_state.filter_by_title = !filter_state.filter_by_title;
        let checked = filter_state.filter_by_title;
        apply_selected(&mut style, &mut border_color, checked);
        set_check_icon(children, &mut text_query, checked);
    }
}

/// 删除屏蔽词按钮交互
pub fn remove_keyword_interaction(
    interaction_query: Query<(&Interaction, &RemoveKeywordButton), Changed<Interaction>>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    let Some(keyword) = interaction_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, btn)| btn.keyword.clone())
    else {
        return;
    };

    filter_state.blocked_keywords.retain(|k| k != &keyword);
    tracing::info!("删除屏蔽词: {}", keyword);
}

/// 新增屏蔽词动作键处理（Enter 添加屏蔽词，Escape 失焦）
///
/// 点击聚焦、IME 开关与编辑由通用 TextInput 系统处理。
pub fn new_keyword_keyboard_input(
    mut input_focus: ResMut<InputFocus>,
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut input_query: Query<&mut TextInput, With<NewKeywordInput>>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut input) = input_query.get_mut(focused) else {
        return;
    };

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Escape => input_focus.clear(),
            Key::Enter => {
                // 回车添加屏蔽词（从 TextInput.value 读取）
                let keyword = input.value.trim().to_string();
                if !keyword.is_empty() && !filter_state.blocked_keywords.contains(&keyword) {
                    tracing::info!("添加屏蔽词: {}", keyword);
                    filter_state.blocked_keywords.push(keyword);
                    input.set_value("");
                }
            }
            _ => {}
        }
    }
}

/// 同步 TextInput.value → FilterSettingsState.new_keyword
pub fn sync_keyword_input_value(
    mut filter_state: ResMut<FilterSettingsState>,
    query: Query<&TextInput, (Changed<TextInput>, With<NewKeywordInput>)>,
) {
    for input in query.iter() {
        if filter_state.new_keyword != input.value {
            filter_state.new_keyword.clone_from(&input.value);
        }
    }
}

/// 添加屏蔽词按钮交互
pub fn add_keyword_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<AddKeywordButton>)>,
    mut filter_state: ResMut<FilterSettingsState>,
    mut input_query: Query<&mut TextInput, With<NewKeywordInput>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 从 TextInput.value 读取关键词
        for mut input in input_query.iter_mut() {
            let keyword = input.value.trim().to_string();
            if !keyword.is_empty() && !filter_state.blocked_keywords.contains(&keyword) {
                tracing::info!("添加屏蔽词: {}", keyword);
                filter_state.blocked_keywords.push(keyword);
                input.set_value("");
            }
        }
    }
}

/// 建议面板展开/折叠交互
pub fn keyword_suggestion_toggle_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor),
        (Changed<Interaction>, With<KeywordSuggestionToggle>),
    >,
    mut filter_state: ResMut<FilterSettingsState>,
    mut panel_query: Query<&mut Node, With<KeywordSuggestionPanel>>,
) {
    for (interaction, mut style, mut border_color) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        filter_state.show_suggestions = !filter_state.show_suggestions;
        let expanded = filter_state.show_suggestions;

        let display = if expanded {
            Display::Flex
        } else {
            Display::None
        };
        for mut node in panel_query.iter_mut() {
            node.display = display;
        }

        // 展开态钉在主色
        apply_selected(&mut style, &mut border_color, expanded);
    }
}

/// 建议项点击添加屏蔽词
pub fn keyword_suggestion_item_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &KeywordSuggestionItem,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    let mut keyword_to_add: Option<String> = None;
    for (interaction, item, _, _) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && !filter_state.blocked_keywords.contains(&item.keyword)
        {
            keyword_to_add = Some(item.keyword.clone());
            break;
        }
    }

    if let Some(keyword) = keyword_to_add {
        tracing::info!("从建议面板添加屏蔽词: {}", keyword);
        filter_state.blocked_keywords.push(keyword);
    }

    // 更新悬停样式（分类/标签两套底色 + 已屏蔽的禁用态，`ButtonStyle`
    // 表达不了，故保留手写）
    for (interaction, item, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let already_blocked = filter_state.blocked_keywords.contains(&item.keyword);
        match *interaction {
            Interaction::Hovered if !already_blocked => {
                *bg_color = BackgroundColor(item.base_color.lighter(0.05));
            }
            Interaction::None => {
                if already_blocked {
                    *bg_color = BackgroundColor(SUGGESTION_BLOCKED_BG);
                    *border_color = BorderColor::all(SUGGESTION_BLOCKED_BORDER);
                } else {
                    // 还原本项自己的底色，而非固定用分类色
                    *bg_color = BackgroundColor(item.base_color);
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
            _ => {}
        }
    }
}

// ==================== 分流设置交互系统 ====================

/// API 分流按钮交互（单选组：选中态写入 `ButtonStyle.selected`）
pub fn api_channel_button_interaction(
    interaction_query: Query<(&Interaction, &ApiChannelButton), Changed<Interaction>>,
    mut channel_state: ResMut<ChannelSettingsState>,
    mut all_buttons_query: Query<(&ApiChannelButton, &mut ButtonStyle, &mut BorderColor)>,
    mut api_ip_row_query: Query<&mut Node, With<CustomCdnApiIpRow>>,
) {
    let Some(picked) = interaction_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, btn)| btn.channel_type)
    else {
        return;
    };

    channel_state.api_channel = picked;

    for (btn, mut style, mut border) in &mut all_buttons_query {
        apply_selected(&mut style, &mut border, btn.channel_type == picked);
    }

    // 切换自定义 IP 输入行显示
    let display = if picked == ChannelType::CustomCdnIp {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in api_ip_row_query.iter_mut() {
        node.display = display;
    }
}

/// 图片分流按钮交互（单选组：选中态写入 `ButtonStyle.selected`）
pub fn image_channel_button_interaction(
    interaction_query: Query<(&Interaction, &ImageChannelButton), Changed<Interaction>>,
    mut channel_state: ResMut<ChannelSettingsState>,
    mut all_buttons_query: Query<(&ImageChannelButton, &mut ButtonStyle, &mut BorderColor)>,
    mut img_ip_row_query: Query<&mut Node, With<CustomCdnImgIpRow>>,
) {
    let Some(picked) = interaction_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, btn)| btn.channel_type)
    else {
        return;
    };

    channel_state.image_channel = picked;

    for (btn, mut style, mut border) in &mut all_buttons_query {
        apply_selected(&mut style, &mut border, btn.channel_type == picked);
    }

    // 切换自定义 IP 输入行显示
    let display = if picked == ChannelType::CustomCdnIp {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in img_ip_row_query.iter_mut() {
        node.display = display;
    }
}

/// 自定义 CDN IP 动作键处理（Escape/Enter 失焦），点击聚焦与编辑由通用
/// TextInput 系统处理
pub fn custom_cdn_ip_keyboard_input(
    mut input_focus: ResMut<InputFocus>,
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    input_query: Query<(), Or<(With<CustomCdnApiIpInput>, With<CustomCdnImgIpInput>)>>,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    if !input_focus.get().is_some_and(|e| input_query.contains(e)) {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if matches!(&event.logical_key, Key::Escape | Key::Enter) {
            input_focus.clear();
        }
    }
}

/// 同步 TextInput.value → ChannelSettingsState
pub fn sync_cdn_ip_input_values(
    mut channel_state: ResMut<ChannelSettingsState>,
    api_query: Query<
        &TextInput,
        (
            Changed<TextInput>,
            With<CustomCdnApiIpInput>,
            Without<CustomCdnImgIpInput>,
        ),
    >,
    img_query: Query<
        &TextInput,
        (
            Changed<TextInput>,
            With<CustomCdnImgIpInput>,
            Without<CustomCdnApiIpInput>,
        ),
    >,
) {
    for input in api_query.iter() {
        if channel_state.custom_cdn_api_ip != input.value {
            channel_state.custom_cdn_api_ip.clone_from(&input.value);
        }
    }
    for input in img_query.iter() {
        if channel_state.custom_cdn_img_ip != input.value {
            channel_state.custom_cdn_img_ip.clone_from(&input.value);
        }
    }
}

// ==================== 版本更新检查系统 ====================

/// 检查更新按钮点击交互（预留，待接入更新检查流程）
#[allow(dead_code)]
pub fn check_update_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CheckUpdateButton>)>,
    mut check_update_messages: MessageWriter<crate::events::CheckUpdateRequest>,
    update_state: Res<crate::resources::UpdateCheckState>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if !update_state.is_checking {
            check_update_messages.write(crate::events::CheckUpdateRequest);
            tracing::info!("用户点击检查更新");
        }
    }
}

/// 刷新版本更新状态文本（预留，待接入更新检查流程）
#[allow(dead_code)]
pub fn refresh_update_status(
    update_state: Res<crate::resources::UpdateCheckState>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<UpdateStatusText>>,
    mut btn_query: Query<&mut ButtonStyle, With<CheckUpdateButton>>,
    mut release_btn_query: Query<&mut Node, With<OpenReleasePageButton>>,
) {
    if !update_state.is_changed() {
        return;
    }

    // 只有确实检测到新版本、且拿到了下载地址时才给入口
    let show_release_btn =
        update_state.has_update == Some(true) && update_state.download_url.is_some();
    let display = if show_release_btn {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in release_btn_query.iter_mut() {
        if node.display != display {
            node.display = display;
        }
    }

    // 检查中降为次要色
    for mut style in btn_query.iter_mut() {
        set_busy(&mut style, update_state.is_checking);
    }

    // 更新状态文本
    for (mut text, mut color) in text_query.iter_mut() {
        if update_state.is_checking {
            **text = "正在检查更新...".to_string();
            *color = TextColor(AppColors::TEXT_SECONDARY);
        } else if let Some(ref error) = update_state.error {
            **text = format!("检查失败: {}", error);
            *color = TextColor(AppColors::ERROR);
        } else if let Some(has_update) = update_state.has_update {
            if has_update {
                let version = update_state.latest_version.as_deref().unwrap_or("未知");
                **text = format!("发现新版本 v{}！", version);
                *color = TextColor(Color::srgb(0.3, 0.8, 0.4));
            } else {
                **text = "已是最新版本".to_string();
                *color = TextColor(Color::srgb(0.4, 0.8, 0.5));
            }
        }
    }
}

/// 「前往下载」按钮交互：用系统浏览器打开 Release 页面
///
/// 只跳转、不自替换——真正的原地升级在三平台各有坑（Windows 不能覆盖运行中的
/// exe、macOS 未签名替换会触发 Gatekeeper），另案调研。
pub fn open_release_page_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<OpenReleasePageButton>)>,
    update_state: Res<crate::resources::UpdateCheckState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(url) = update_state.download_url.as_deref() else {
            tracing::warn!("没有可用的下载地址");
            continue;
        };
        tracing::info!("打开 Release 页面: {}", url);
        if let Err(e) = open::that(url) {
            tracing::error!("打开下载页面失败: {} - {}", url, e);
        }
    }
}

/// 「启动时自动检查更新」勾选框交互
pub fn auto_check_update_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<AutoCheckUpdateCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 与「系统耗时追踪」同理：这一项不参与 auto_save_settings
        // 的状态资源体系， 直接落全局配置
        let is_enabled = {
            let settings = AppSettings::global();
            let mut settings = settings.write();
            settings.auto_check_update = !settings.auto_check_update;
            settings.auto_check_update
        };
        if let Err(e) = AppSettings::global().read().save() {
            tracing::error!("保存自动检查更新开关失败: {}", e);
        }
        tracing::info!(
            "启动时自动检查更新: {}",
            if is_enabled { "启用" } else { "禁用" }
        );

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

// ==================== 网络诊断 ====================

/// 创建网络诊断区域
fn network_diag_section() -> impl Scene {
    let speed_label = format!("{ICON_DOWNLOAD} 测速");
    let ping_label = format!("{ICON_REFRESH} Ping");

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
        }
        Children [
            (
                // 说明文本
                Text("测试当前网络到服务器的连通性和速度")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 按钮行
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        // 测速按钮
                        SpeedTestButton
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        template_value(BorderColor::all(AppColors::PRIMARY))
                        Children [
                            (
                                Text({speed_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(Color::WHITE)
                            )
                        ]
                    ),
                    (
                        // Ping 按钮
                        PingTestButton
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        template_value(BorderColor::all(AppColors::PRIMARY))
                        Children [
                            (
                                Text({ping_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(Color::WHITE)
                            )
                        ]
                    ),
                ]
            ),
            (
                // 结果文本
                NetworkDiagResultText
                Text("点击按钮开始测试")
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
        ]
    }
}

/// 测速按钮交互
pub fn speed_test_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SpeedTestButton>)>,
    diag_state: Res<crate::resources::NetworkDiagState>,
    mut speed_test_messages: MessageWriter<crate::events::SpeedTestRequest>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if diag_state.is_testing_speed {
            return;
        }
        speed_test_messages.write(crate::events::SpeedTestRequest);
    }
}

/// Ping 测试按钮交互
pub fn ping_test_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PingTestButton>)>,
    diag_state: Res<crate::resources::NetworkDiagState>,
    mut ping_test_messages: MessageWriter<crate::events::PingTestRequest>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if diag_state.is_testing_ping {
            return;
        }
        ping_test_messages.write(crate::events::PingTestRequest);
    }
}

/// 更新网络诊断结果显示
pub fn update_network_diag_result(
    diag_state: Res<crate::resources::NetworkDiagState>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<NetworkDiagResultText>>,
    mut speed_btn_query: Query<&mut ButtonStyle, (With<SpeedTestButton>, Without<PingTestButton>)>,
    mut ping_btn_query: Query<&mut ButtonStyle, (With<PingTestButton>, Without<SpeedTestButton>)>,
) {
    if !diag_state.is_changed() {
        return;
    }

    // 测试中降为次要色
    for mut style in speed_btn_query.iter_mut() {
        set_busy(&mut style, diag_state.is_testing_speed);
    }
    for mut style in ping_btn_query.iter_mut() {
        set_busy(&mut style, diag_state.is_testing_ping);
    }

    // 构建结果文本
    let mut parts: Vec<String> = Vec::new();

    if diag_state.is_testing_speed {
        parts.push("正在测速...".to_string());
    } else if let Some(speed) = diag_state.download_speed {
        if speed >= 1024.0 {
            parts.push(format!("下载速度: {:.2} MB/s", speed / 1024.0));
        } else {
            parts.push(format!("下载速度: {:.1} KB/s", speed));
        }
    }

    if diag_state.is_testing_ping {
        parts.push("正在 Ping...".to_string());
    } else if let Some(latency) = diag_state.latency_ms {
        parts.push(format!("延迟: {} ms", latency));
    }

    if let Some(ref error) = diag_state.error {
        parts.push(format!("错误: {}", error));
    }

    let display_text = if parts.is_empty() {
        "点击按钮开始测试".to_string()
    } else {
        parts.join("  |  ")
    };

    // 决定文本颜色
    let color = if diag_state.error.is_some() {
        AppColors::ERROR
    } else if diag_state.is_testing_speed || diag_state.is_testing_ping {
        AppColors::TEXT_SECONDARY
    } else {
        Color::srgb(0.4, 0.8, 0.5)
    };

    for (mut text, mut text_color) in text_query.iter_mut() {
        **text = display_text.clone();
        *text_color = TextColor(color);
    }
}

// ==================== 高级设置交互系统 ====================

/// 界面缩放按钮交互
pub fn ui_scale_button_interaction(
    interaction_query: Query<(&Interaction, &UiScaleButton), Changed<Interaction>>,
    mut ui_scale_state: ResMut<UiScaleState>,
    mut save_status: ResMut<SettingsSaveStatus>,
    mut all_buttons: Query<(&UiScaleButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    // 浮点比较：差值小于 0.01 视为同一档
    let Some(scale) = interaction_query
        .iter()
        .find(|(interaction, btn)| {
            **interaction == Interaction::Pressed && (ui_scale_state.scale - btn.scale).abs() > 0.01
        })
        .map(|(_, btn)| btn.scale)
    else {
        return;
    };

    tracing::info!("界面缩放已选择: {}", scale);
    ui_scale_state.scale = scale;

    let label = UI_SCALE_OPTIONS
        .iter()
        .find(|(s, _)| (*s - scale).abs() < 0.01)
        .map(|(_, l)| *l)
        .unwrap_or("未知");
    save_status.visible = true;
    save_status.message = format!("界面缩放已切换为「{}」，重启应用后生效", label);
    save_status.is_error = false;
    save_status.timer.reset();

    for (btn, mut style, mut border) in &mut all_buttons {
        apply_selected(&mut style, &mut border, (btn.scale - scale).abs() < 0.01);
    }
}

/// 自定义字体路径双向同步
///
/// 输入框被编辑 → 写回状态；状态被外部改写（文件选择器）→ 推回输入框。
pub fn sync_custom_font_path_value(
    mut input_query: Query<&mut TextInput, With<CustomFontPathInput>>,
    mut font_path_state: ResMut<CustomFontPathInputState>,
) {
    for mut text_input in input_query.iter_mut() {
        if text_input.value == font_path_state.value {
            continue;
        }
        // 编辑优先：输入框本帧有改动时以输入框为准
        if text_input.is_changed() {
            font_path_state.value.clone_from(&text_input.value);
        } else if font_path_state.is_changed() {
            text_input.set_value(font_path_state.value.clone());
        }
    }
}

/// 自定义字体文件选择按钮交互
pub fn custom_font_path_picker_interaction(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<CustomFontPathPickerButton>),
    >,
    mut picker: ResMut<CustomFontPathPickerResult>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 防止重复打开
        if picker.receiver.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            picker.receiver = Some(std::sync::Mutex::new(rx));
            std::thread::spawn(move || {
                let path = rfd::FileDialog::new()
                    .add_filter("字体文件", &["ttf", "otf", "ttc", "otc"])
                    .pick_file()
                    .map(|p| p.to_string_lossy().to_string());
                let _ = tx.send(path);
            });
        }
    }
}

/// 轮询自定义字体文件选择器的异步结果
pub fn handle_custom_font_path_picker_result(
    mut picker: ResMut<CustomFontPathPickerResult>,
    mut input_query: Query<&mut TextInput, With<CustomFontPathInput>>,
    mut font_path_state: ResMut<CustomFontPathInputState>,
    mut save_status: ResMut<SettingsSaveStatus>,
) {
    let Some(ref receiver) = picker.receiver else {
        return;
    };
    let Ok(receiver) = receiver.lock() else {
        return;
    };
    let Ok(result) = receiver.try_recv() else {
        return;
    };
    drop(receiver);
    // 收到结果，清除 receiver
    picker.receiver = None;
    if let Some(path_str) = result {
        for mut input in input_query.iter_mut() {
            input.set_value(path_str.clone());
        }
        font_path_state.value = path_str;

        // 提示重启生效
        save_status.visible = true;
        save_status.message = "自定义字体路径已设置，重启应用后生效".to_string();
        save_status.is_error = false;
        save_status.timer.reset();
    }
}

/// SNI 伪装复选框交互
pub fn sni_pretend_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<SniPretendCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut network_state: ResMut<NetworkAdvancedState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        network_state.sni_pretend = !network_state.sni_pretend;
        let is_enabled = network_state.sni_pretend;

        tracing::info!("SNI 伪装: {}", if is_enabled { "启用" } else { "禁用" });

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

/// IPv6 优先复选框交互
pub fn prefer_ipv6_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<PreferIpv6Checkbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut network_state: ResMut<NetworkAdvancedState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        network_state.prefer_ipv6 = !network_state.prefer_ipv6;
        let is_enabled = network_state.prefer_ipv6;

        tracing::info!("IPv6 优先: {}", if is_enabled { "启用" } else { "禁用" });

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

// ==================== 性能追踪设置 ====================

/// 「性能叠加层」开关（立即生效）
#[derive(Component, Default, Clone)]
pub struct PerfOverlayCheckbox;

/// 「系统耗时追踪」开关（重启后生效）
#[derive(Component, Default, Clone)]
pub struct ProfilingCheckbox;

/// 「刷新耗时榜」按钮
#[derive(Component, Default, Clone)]
pub struct RefreshTimingsButton;

/// 耗时榜列表容器（局部刷新的靶子）
#[derive(Component, Default, Clone)]
pub struct TimingsListContainer;

/// 耗时榜状态提示文本
#[derive(Component, Default, Clone)]
pub struct TimingsHintText;

/// 「打开日志目录」按钮
#[derive(Component, Default, Clone)]
pub struct OpenProfilingLogButton;

/// 榜单在设置页里显示几行
const TIMINGS_ROWS: usize = 12;

/// 性能追踪分组
///
/// 榜单渲染在页面里而不是只打日志——打包成 .app 之后没有终端可看。
fn profiling_section(overlay_visible: bool, profiling_enabled: bool) -> impl Scene + use<> {
    let refresh_label = format!("{ICON_REFRESH} 刷新耗时榜");
    let open_log_label = format!("{ICON_FOLDER_OPEN} 日志目录");
    let hint = profiling_hint_text(profiling_enabled);
    let log_path_label = format!("榜单落盘: {}", profiling::report_log_path().display());

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        Children [
            (
                Text("叠加层显示 FPS / 帧时间 / 实体数；耗时榜按累计耗时列出最慢的系统")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            toggle_row(
                PerfOverlayCheckbox,
                "性能叠加层",
                "右上角实时显示 FPS / 帧时间 / 实体数（快捷键 F3）",
                overlay_visible,
            ),
            toggle_row(
                ProfilingCheckbox,
                "系统耗时追踪",
                "统计每个 ECS 系统的耗时，用于定位卡顿来源；重启后生效",
                profiling_enabled,
            ),
            (
                // 刷新按钮行
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    margin: UiRect::top(Val::Px(16.0)),
                }
                Children [
                    (
                        RefreshTimingsButton
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::PRIMARY)
                        template_value(BorderColor::all(AppColors::PRIMARY))
                        Children [
                            (
                                Text({refresh_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(Color::WHITE)
                            )
                        ]
                    ),
                    (
                        OpenProfilingLogButton
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text({open_log_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        TimingsHintText
                        Text({hint})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                ]
            ),
            (
                // 落盘路径（方便直接把文件发出来）
                Text({log_path_label})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(AppColors::TEXT_MUTED)
                Node { margin: UiRect::top(Val::Px(6.0)) }
            ),
            (
                // 榜单列表（由 refresh_timings_list 填充）
                TimingsListContainer
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    margin: UiRect::top(Val::Px(10.0)),
                }
            ),
        ]
    }
}

/// 状态提示文案
fn profiling_hint_text(profiling_enabled: bool) -> String {
    if !profiling::is_compiled_in() {
        "本次构建未编入耗时 span，需 --features profiling 重新构建".to_string()
    } else if profiling::is_enabled() {
        "统计中，点击查看上次刷新之后的累计".to_string()
    } else if profiling_enabled {
        "已打开，重启后开始统计".to_string()
    } else {
        "未启用".to_string()
    }
}

/// 榜单表头
fn timings_header_row() -> impl Scene {
    timings_row_scene("总耗时ms", "峰值ms", "次数", "系统", AppColors::TEXT)
}

/// 榜单一行
fn timings_row_scene(
    total: &str,
    peak: &str,
    calls: &str,
    name: &str,
    color: Color,
) -> impl Scene + use<> {
    let (total, peak, calls, name) = (
        total.to_string(),
        peak.to_string(),
        calls.to_string(),
        name.to_string(),
    );

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            overflow: Overflow::clip(),
        }
        Children [
            (
                Text({total})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(color)
                Node { width: Val::Px(70.0) }
            ),
            (
                Text({peak})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(color)
                Node { width: Val::Px(64.0) }
            ),
            (
                Text({calls})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(color)
                Node { width: Val::Px(48.0) }
            ),
            (
                Text({name})
                TextFont { font_size: FontSize::Px(11.0) }
                TextColor(color)
                Node { flex_grow: 1.0, overflow: Overflow::clip() }
            ),
        ]
    }
}

/// 「性能叠加层」开关交互（立即生效，与 F3 同源）
pub fn perf_overlay_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<PerfOverlayCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut overlay_state: ResMut<crate::systems::perf_overlay::PerfOverlayState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        overlay_state.visible = !overlay_state.visible;
        let is_enabled = overlay_state.visible;
        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);
    }
}

/// 「系统耗时追踪」开关交互（写配置，重启后生效）
pub fn profiling_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle, &mut BorderColor, &Children),
        (Changed<Interaction>, With<ProfilingCheckbox>),
    >,
    mut text_query: Query<&mut Text, Without<TimingsHintText>>,
    mut hint_query: Query<&mut Text, With<TimingsHintText>>,
    mut behavior_state: ResMut<DownloadBehaviorState>,
) {
    for (interaction, mut style, mut border_color, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 直接改全局配置：这一项不参与 auto_save_settings 的状态资源体系
        //（它不影响任何运行时行为，只在下次启动被读一次）
        let is_enabled = {
            let settings = AppSettings::global();
            let mut settings = settings.write();
            settings.enable_profiling = !settings.enable_profiling;
            settings.enable_profiling
        };
        if let Err(e) = AppSettings::global().read().save() {
            tracing::error!("保存性能追踪开关失败: {}", e);
        }
        // 借下载行为资源的变更去触发底部「已保存」状态栏，免得用户以为没生效
        behavior_state.set_changed();

        tracing::info!(
            "系统耗时追踪: {}（重启后生效）",
            if is_enabled { "启用" } else { "禁用" }
        );

        apply_selected(&mut style, &mut border_color, is_enabled);
        set_check_icon(children, &mut text_query, is_enabled);

        let hint = profiling_hint_text(is_enabled);
        for mut text in hint_query.iter_mut() {
            **text = hint.clone();
        }
    }
}

/// 「刷新耗时榜」按钮交互：取一次榜单并重建列表
pub fn refresh_timings_button_interaction(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RefreshTimingsButton>)>,
    list_query: Query<(Entity, Option<&Children>), With<TimingsListContainer>>,
    mut hint_query: Query<&mut Text, With<TimingsHintText>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok((container, children)) = list_query.single() else {
            continue;
        };

        // 清空旧行
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(child) {
                    entity_commands.despawn();
                }
            }
        }

        if !profiling::is_enabled() {
            for mut text in hint_query.iter_mut() {
                **text = profiling_hint_text(AppSettings::global().read().enable_profiling);
            }
            continue;
        }

        let rows = profiling::take_report(TIMINGS_ROWS);
        if rows.is_empty() {
            for mut text in hint_query.iter_mut() {
                **text = "本区间无数据（刚刷新过？让它跑一会儿再点）".to_string();
            }
            continue;
        }

        profiling::append_report_to_log("设置页手动刷新（页面 Settings）", &rows);

        let header = commands.spawn_scene(timings_header_row()).id();
        commands.entity(container).add_child(header);
        for row in &rows {
            let entity = commands
                .spawn_scene(timings_row_scene(
                    &format!("{:.2}", row.total_ms),
                    &format!("{:.3}", row.max_ms),
                    &row.calls.to_string(),
                    &short_system_name(&row.name),
                    AppColors::TEXT_SECONDARY,
                ))
                .id();
            commands.entity(container).add_child(entity);
        }

        for mut text in hint_query.iter_mut() {
            **text = format!("上次刷新之后的累计（Top {}），已追加到日志", rows.len());
        }
    }
}

/// 「打开日志目录」按钮交互
pub fn open_profiling_log_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<OpenProfilingLogButton>)>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let dir = AppSettings::log_dir();
        // 目录可能还没建（一次都没打过榜），先建再开，免得资源管理器报错
        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::error!("创建日志目录失败: {} - {}", dir.display(), e);
            continue;
        }
        if let Err(e) = open::that(&dir) {
            tracing::error!("打开日志目录失败: {} - {}", dir.display(), e);
        }
    }
}

/// 系统名去掉模块前缀，只留最后两段——全路径太长会把整行挤没
fn short_system_name(full: &str) -> String {
    let segments: Vec<&str> = full.split("::").collect();
    match segments.as_slice() {
        [.., module, name] => format!("{module}::{name}"),
        _ => full.to_string(),
    }
}
