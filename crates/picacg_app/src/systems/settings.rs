//! 设置界面系统
//!
//! 实现应用设置页面

use bevy::{
    prelude::*,
    time::Timer,
    ui::{FocusPolicy, RelativeCursorPosition},
    window::PrimaryWindow,
};
use picacg_config::{
    AppSettings, ChannelType, FilterSettings, LogLevel, ProxyType, update_log_level,
};

use super::font_loader::get_font;
use crate::{
    components::{
        ContentArea, ContentSizeInfo, ScrollbarContainer, ScrollbarThumb, ScrollbarTrack,
    },
    systems::{login::AppColors, scrollbar::scrollbar_config::*},
    utils::{
        icons::*,
        text_input::{TextInput, TextInputDisplay},
    },
};

/// 设置滚动容器组件（本地定义）
#[derive(Component)]
pub struct ScrollContainer;

/// 设置页面根标记
#[derive(Component)]
pub struct SettingsRoot;

/// 设置滚动容器标记
#[derive(Component)]
pub struct SettingsScrollContainer;

/// 下载路径输入框标记（配合 TextInput 使用）
#[derive(Component)]
pub struct DownloadPathInput;

/// 下载路径目录选择按钮
#[derive(Component)]
pub struct DownloadPathPickerButton;

/// 目录选择器结果（后台线程 → 主线程，使用 Mutex 包裹 Receiver 以满足 Sync）
#[derive(Resource)]
pub struct DownloadPathPickerResult {
    pub receiver: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Option<String>>>>,
}

impl Default for DownloadPathPickerResult {
    fn default() -> Self {
        Self { receiver: None }
    }
}

/// 下载路径输入状态
#[derive(Resource)]
pub struct DownloadPathInputState {
    pub value: String,
    pub is_focused: bool,
}

impl Default for DownloadPathInputState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            value: settings.download_path.clone(),
            is_focused: false,
        }
    }
}

/// 清除缓存按钮标记
#[derive(Component)]
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
#[derive(Component)]
pub struct SettingsStatusText;

/// 底部状态栏容器标记
#[derive(Component)]
pub struct SettingsStatusBar;

// ==================== 代理设置组件 ====================

/// 代理启用复选框
#[derive(Component)]
pub struct ProxyEnabledCheckbox;

/// 代理类型按钮
#[derive(Component)]
pub struct ProxyTypeButton {
    pub proxy_type: ProxyType,
}

/// 代理主机输入框
#[derive(Component)]
pub struct ProxyHostInput;

/// 代理端口输入框
#[derive(Component)]
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
#[derive(Component)]
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
#[derive(Component)]
pub struct AutoResumeDownloadsCheckbox;

/// 自动恢复下载设置状态
#[derive(Resource)]
pub struct AutoResumeDownloadsState {
    pub enabled: bool,
}

// ==================== 最大并发下载数设置组件 ====================

/// 最大并发下载数减少按钮
#[derive(Component)]
pub struct MaxConcurrentDownloadsDecreaseButton;

/// 最大并发下载数增加按钮
#[derive(Component)]
pub struct MaxConcurrentDownloadsIncreaseButton;

/// 最大并发下载数显示文本
#[derive(Component)]
pub struct MaxConcurrentDownloadsText;

/// 最大并发下载数设置状态
#[derive(Resource)]
pub struct MaxConcurrentDownloadsState {
    pub value: usize,
}

// ==================== CBZ 打包设置组件 ====================

/// 自动打包 CBZ 勾选框
#[derive(Component)]
pub struct AutoPackCbzCheckbox;

/// 打包后删除原图勾选框
#[derive(Component)]
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
#[derive(Component)]
pub struct FilterByCategoryCheckbox;

/// 按标签屏蔽复选框
#[derive(Component)]
pub struct FilterByTagCheckbox;

/// 按标题屏蔽复选框
#[derive(Component)]
pub struct FilterByTitleCheckbox;

/// 屏蔽词列表项标记
#[derive(Component)]
pub struct BlockedKeywordItem;

/// 删除屏蔽词按钮
#[derive(Component)]
pub struct RemoveKeywordButton {
    pub keyword: String,
}

/// 新增屏蔽词输入框标记
#[derive(Component)]
pub struct NewKeywordInput;

/// 添加屏蔽词按钮
#[derive(Component)]
pub struct AddKeywordButton;

/// 下拉建议面板容器
#[derive(Component)]
pub struct KeywordSuggestionPanel;

/// 建议项按钮
#[derive(Component)]
pub struct KeywordSuggestionItem {
    pub keyword: String,
}

/// 展开/折叠下拉按钮
#[derive(Component)]
pub struct KeywordSuggestionToggle;

/// 屏蔽词列表容器标记
#[derive(Component)]
pub struct BlockedKeywordsListContainer;

// ==================== 分流设置组件 ====================

/// API 分流按钮
#[derive(Component)]
pub struct ApiChannelButton {
    pub channel_type: ChannelType,
}

/// 图片分流按钮
#[derive(Component)]
pub struct ImageChannelButton {
    pub channel_type: ChannelType,
}

/// 自定义 CDN API IP 输入框
#[derive(Component)]
pub struct CustomCdnApiIpInput;

/// 自定义 CDN 图片 IP 输入框
#[derive(Component)]
pub struct CustomCdnImgIpInput;

/// 自定义 API IP 输入行容器（条件显示）
#[derive(Component)]
pub struct CustomCdnApiIpRow;

/// 自定义图片 IP 输入行容器（条件显示）
#[derive(Component)]
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

/// 创建设置页面 UI
pub fn setup_settings_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
    categories_state: Res<crate::resources::CategoriesState>,
    cached_tags: Res<crate::resources::CachedTagsState>,
) {
    let font: Handle<Font> = get_font();
    let settings = AppSettings::global().read();

    // 查找内容区域
    let content_area = match content_area_query.iter().next() {
        Some(entity) => entity,
        None => {
            tracing::warn!("设置页面：找不到内容区域");
            return;
        }
    };

    // 初始化下载路径输入状态
    commands.insert_resource(DownloadPathInputState {
        value: settings.download_path.clone(),
        is_focused: false,
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

    // 初始化自动恢复下载状态
    commands.insert_resource(AutoResumeDownloadsState {
        enabled: settings.auto_resume_downloads,
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

    // 初始化保存状态提示
    commands.insert_resource(SettingsSaveStatus::default());

    // 在内容区域下创建设置页面
    commands.entity(content_area).with_children(|parent| {
        parent
            .spawn((
                SettingsRoot,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(AppColors::BACKGROUND),
            ))
            .with_children(|root| {
                // 标题栏
                spawn_settings_header(root, &font);

                // 设置内容（可滚动）- 包装器需要 Relative 定位以支持 Absolute 子元素
                root.spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(), // 裁剪溢出内容，防止延伸到底部按钮栏
                    ..default()
                },))
                    .with_children(|content_wrapper| {
                        // 滚动容器
                        let scroll_container = content_wrapper
                            .spawn((
                                SettingsScrollContainer,
                                ScrollContainer,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Px(20.0)),
                                    overflow: Overflow::scroll_y(),
                                    ..default()
                                },
                                ScrollPosition::default(),
                                ContentSizeInfo::default(),
                            ))
                            .with_children(|scroll| {
                                // 代理设置分组
                                spawn_settings_section(scroll, &font, "代理设置", |section| {
                                    spawn_proxy_setting(section, &font, &settings);
                                });

                                // 分流设置分组
                                spawn_settings_section(scroll, &font, "分流设置", |section| {
                                    spawn_channel_setting(section, &font, &settings);
                                });

                                // 日志设置分组
                                spawn_settings_section(scroll, &font, "日志设置", |section| {
                                    spawn_log_level_setting(section, &font, settings.log_level);
                                });

                                // 下载设置分组
                                spawn_settings_section(scroll, &font, "下载设置", |section| {
                                    spawn_download_path_setting(
                                        section,
                                        &font,
                                        &settings.download_path,
                                    );
                                    spawn_max_concurrent_downloads_setting(
                                        section,
                                        &font,
                                        settings.max_concurrent_downloads,
                                    );
                                    spawn_auto_resume_downloads_setting(
                                        section,
                                        &font,
                                        settings.auto_resume_downloads,
                                    );
                                    spawn_auto_pack_cbz_setting(
                                        section,
                                        &font,
                                        settings.auto_pack_cbz,
                                    );
                                    spawn_delete_images_after_cbz_setting(
                                        section,
                                        &font,
                                        settings.delete_images_after_cbz,
                                    );
                                });

                                // 内容过滤分组
                                let category_titles: Vec<String> = categories_state
                                    .categories
                                    .iter()
                                    .map(|c| c.title.clone())
                                    .collect();
                                let tag_titles: Vec<String> = cached_tags.tags.clone();
                                spawn_settings_section(scroll, &font, "内容过滤", |section| {
                                    spawn_filter_settings(
                                        section,
                                        &font,
                                        &settings.filter,
                                        &category_titles,
                                        &tag_titles,
                                    );
                                });

                                // 缓存设置分组
                                spawn_settings_section(scroll, &font, "缓存设置", |section| {
                                    spawn_cache_setting(section, &font);
                                });

                                // 关于分组
                                spawn_settings_section(scroll, &font, "关于", |section| {
                                    spawn_about_section(section, &font);
                                });

                                // 底部间距（确保最后的内容可以完全滚动到可见区域）
                                scroll.spawn((Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(120.0),
                                    min_height: Val::Px(120.0),
                                    ..default()
                                },));
                            })
                            .id();

                        // 滚动条
                        spawn_settings_scrollbar(content_wrapper, scroll_container);
                    });

                // 底部状态栏（固定在页面底部，显示保存状态提示）
                spawn_status_bar(root, &font);
            });
    });

    tracing::info!("设置页面 UI 已创建");
}

/// 创建设置标题栏
fn spawn_settings_header(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                padding: UiRect::horizontal(Val::Px(20.0)),
                align_items: AlignItems::Center,
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|header| {
            header.spawn((
                Text::new(format!("{ICON_COG} 设置")),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 创建设置分组
fn spawn_settings_section<F>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    title: &str,
    content_builder: F,
) where
    F: FnOnce(&mut ChildSpawnerCommands),
{
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(20.0)),
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|section| {
            // 分组标题
            section.spawn((
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));

            // 分组内容
            content_builder(section);
        });
}

/// 创建下载路径设置
fn spawn_download_path_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_path: &str,
) {
    // 标签行
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|row| {
            // 标签
            row.spawn((
                Text::new("下载保存路径"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 说明
            row.spawn((
                Text::new("留空则使用默认路径（程序目录/Downloads）"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));

            // 输入框 + 选择目录按钮 行
            row.spawn((Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },))
                .with_children(|input_row| {
                    // 输入框（TextInput 通用组件）
                    let display_text = if current_path.is_empty() {
                        "（使用默认路径）".to_string()
                    } else {
                        current_path.to_string()
                    };
                    input_row
                        .spawn((
                            DownloadPathInput,
                            TextInput::new("（使用默认路径）").with_value(current_path),
                            Button,
                            Interaction::default(),
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(36.0),
                                padding: UiRect::horizontal(Val::Px(12.0)),
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                            BorderColor::all(AppColors::BORDER),
                            RelativeCursorPosition::default(),
                        ))
                        .with_children(|input| {
                            input.spawn((
                                TextInputDisplay,
                                Text::new(display_text),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(if current_path.is_empty() {
                                    AppColors::TEXT_SECONDARY
                                } else {
                                    AppColors::TEXT
                                }),
                            ));
                        });

                    // 选择目录按钮
                    input_row
                        .spawn((
                            DownloadPathPickerButton,
                            Button,
                            Interaction::default(),
                            Node {
                                width: Val::Px(36.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(AppColors::SECONDARY),
                            BorderColor::all(AppColors::BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(ICON_FOLDER_OPEN),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                });
        });
}

/// 创建最大并发下载数设置
fn spawn_max_concurrent_downloads_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_value: usize,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },))
        .with_children(|row| {
            // 左侧标签和说明
            row.spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|left| {
                    left.spawn((
                        Text::new("最大同时下载数"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                    left.spawn((
                        Text::new("同时下载的漫画数量上限"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

            // 右侧数值调整器
            row.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },))
                .with_children(|controls| {
                    // 减少按钮
                    controls
                        .spawn((
                            MaxConcurrentDownloadsDecreaseButton,
                            Button,
                            Interaction::default(),
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(AppColors::BORDER),
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(ICON_MINUS),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });

                    // 数值显示
                    controls.spawn((
                        MaxConcurrentDownloadsText,
                        Text::new(format!("{}", current_value)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                        Node {
                            width: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ));

                    // 增加按钮
                    controls
                        .spawn((
                            MaxConcurrentDownloadsIncreaseButton,
                            Button,
                            Interaction::default(),
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(AppColors::BORDER),
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(ICON_PLUS),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                });
        });
}

/// 创建自动恢复下载设置
fn spawn_auto_resume_downloads_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    is_enabled: bool,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },))
        .with_children(|row| {
            // 左侧标签和说明
            row.spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|left| {
                    left.spawn((
                        Text::new("启动后自动开始下载"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                    left.spawn((
                        Text::new("程序启动时自动恢复未完成的下载任务"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

            // 右侧勾选框
            row.spawn((
                AutoResumeDownloadsCheckbox,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(if is_enabled {
                    AppColors::PRIMARY
                } else {
                    Color::srgb(0.12, 0.12, 0.16)
                }),
                BorderColor::all(if is_enabled {
                    AppColors::PRIMARY
                } else {
                    AppColors::BORDER
                }),
            ))
            .with_children(|checkbox| {
                // 勾选标记（使用 Nerd Font 图标）
                checkbox.spawn((
                    Text::new(if is_enabled { ICON_CHECK } else { "" }), // 󰄬 nf-md-check
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 创建自动打包 CBZ 设置
fn spawn_auto_pack_cbz_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    is_enabled: bool,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },))
        .with_children(|row| {
            // 左侧标签和说明
            row.spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|left| {
                    left.spawn((
                        Text::new("下载完成后自动打包 CBZ"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                    left.spawn((
                        Text::new("将漫画打包为 CBZ 格式，方便导入阅读器"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

            // 右侧勾选框
            row.spawn((
                AutoPackCbzCheckbox,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(if is_enabled {
                    AppColors::PRIMARY
                } else {
                    Color::srgb(0.12, 0.12, 0.16)
                }),
                BorderColor::all(if is_enabled {
                    AppColors::PRIMARY
                } else {
                    AppColors::BORDER
                }),
            ))
            .with_children(|checkbox| {
                checkbox.spawn((
                    Text::new(if is_enabled { ICON_CHECK } else { "" }), // 󰄬 nf-md-check
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 创建打包后删除原图设置
fn spawn_delete_images_after_cbz_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    is_enabled: bool,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },))
        .with_children(|row| {
            // 左侧标签和说明
            row.spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|left| {
                    left.spawn((
                        Text::new("打包 CBZ 后删除原图"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                    left.spawn((
                        Text::new("打包成功后自动删除 Images 目录中的原图"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

            // 右侧勾选框
            row.spawn((
                DeleteImagesAfterCbzCheckbox,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(if is_enabled {
                    AppColors::PRIMARY
                } else {
                    Color::srgb(0.12, 0.12, 0.16)
                }),
                BorderColor::all(if is_enabled {
                    AppColors::PRIMARY
                } else {
                    AppColors::BORDER
                }),
            ))
            .with_children(|checkbox| {
                checkbox.spawn((
                    Text::new(if is_enabled { ICON_CHECK } else { "" }), // 󰄬 nf-md-check
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 创建内容过滤设置
fn spawn_filter_settings(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    filter: &FilterSettings,
    category_titles: &[String],
    tag_titles: &[String],
) {
    // 屏蔽模式复选框行
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|row| {
            spawn_filter_mode_checkbox(
                row,
                font,
                "按分类屏蔽",
                filter.filter_by_category,
                FilterCheckboxType::Category,
            );
            spawn_filter_mode_checkbox(
                row,
                font,
                "按标签屏蔽",
                filter.filter_by_tag,
                FilterCheckboxType::Tag,
            );
            spawn_filter_mode_checkbox(
                row,
                font,
                "按标题屏蔽",
                filter.filter_by_title,
                FilterCheckboxType::Title,
            );
        });

    // 屏蔽词列表标签
    parent.spawn((
        Text::new("屏蔽词列表:"),
        TextFont {
            font: font.clone(),
            font_size: 13.0,
            ..default()
        },
        TextColor(AppColors::TEXT_SECONDARY),
        Node {
            margin: UiRect::top(Val::Px(12.0)),
            ..default()
        },
    ));

    // 屏蔽词列表容器
    parent
        .spawn((
            BlockedKeywordsListContainer,
            Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            },
        ))
        .with_children(|list| {
            spawn_blocked_keyword_tags(list, font, &filter.blocked_keywords);
        });

    // 新增屏蔽词输入行
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            margin: UiRect::top(Val::Px(10.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|row| {
            // 输入框（TextInput 通用组件）
            row.spawn((
                NewKeywordInput,
                TextInput::new("输入新屏蔽词..."),
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(32.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(AppColors::CARD_BG),
                BorderColor::all(AppColors::BORDER),
                RelativeCursorPosition::default(),
            ))
            .with_children(|input| {
                input.spawn((
                    TextInputDisplay,
                    Text::new("输入新屏蔽词..."),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });

            // 添加按钮
            row.spawn((
                AddKeywordButton,
                Button,
                Interaction::default(),
                Node {
                    height: Val::Px(32.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(AppColors::PRIMARY),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("添加"),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // "选择分类/标签" 展开/折叠按钮
            if !category_titles.is_empty() || !tag_titles.is_empty() {
                row.spawn((
                    KeywordSuggestionToggle,
                    Button,
                    Interaction::default(),
                    Node {
                        height: Val::Px(32.0),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::all(AppColors::BORDER),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("选择分类/标签 {ICON_CHEVRON_DOWN}")),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });
            }
        });

    // 分类/标签建议面板（初始隐藏）
    if !category_titles.is_empty() || !tag_titles.is_empty() {
        parent
            .spawn((
                KeywordSuggestionPanel,
                Node {
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(6.0)),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    display: Display::None, // 初始隐藏
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.12, 0.18, 0.9)),
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|panel| {
                // 分类区域
                if !category_titles.is_empty() {
                    // 分类标题
                    panel.spawn((
                        Text::new("分类"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                    // 分类标签列表
                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for title in category_titles {
                                spawn_suggestion_item(
                                    row,
                                    font,
                                    title,
                                    filter,
                                    Color::srgba(0.2, 0.2, 0.3, 0.7),
                                );
                            }
                        });
                }

                // 标签区域
                if !tag_titles.is_empty() {
                    // 标签标题
                    panel.spawn((
                        Text::new("标签"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                    // 标签列表
                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for title in tag_titles {
                                spawn_suggestion_item(
                                    row,
                                    font,
                                    title,
                                    filter,
                                    Color::srgba(0.15, 0.25, 0.2, 0.7),
                                );
                            }
                        });
                }
            });
    }
}

/// 创建建议面板中的单个建议项
fn spawn_suggestion_item(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    keyword: &str,
    filter: &FilterSettings,
    base_color: Color,
) {
    let already_blocked = filter.blocked_keywords.contains(&keyword.to_string());
    let bg = if already_blocked {
        Color::srgba(0.2, 0.2, 0.25, 0.4)
    } else {
        base_color
    };
    let text_color = if already_blocked {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };

    parent
        .spawn((
            KeywordSuggestionItem {
                keyword: keyword.to_string(),
            },
            Button,
            Interaction::default(),
            Node {
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(4.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(if already_blocked {
                Color::srgba(0.3, 0.3, 0.35, 0.3)
            } else {
                AppColors::BORDER
            }),
        ))
        .with_children(|item| {
            item.spawn((
                Text::new(keyword),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

/// 创建屏蔽词标签列表（可复用）
fn spawn_blocked_keyword_tags(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    keywords: &[String],
) {
    if keywords.is_empty() {
        parent.spawn((
            Text::new("暂无屏蔽词"),
            TextFont {
                font: font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(AppColors::TEXT_SECONDARY),
        ));
    } else {
        for keyword in keywords {
            parent
                .spawn((
                    BlockedKeywordItem,
                    Node {
                        padding: UiRect::new(
                            Val::Px(8.0),
                            Val::Px(4.0),
                            Val::Px(3.0),
                            Val::Px(3.0),
                        ),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.5)),
                    BorderColor::all(AppColors::BORDER),
                ))
                .with_children(|tag| {
                    tag.spawn((
                        Text::new(keyword),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                    // 删除按钮
                    tag.spawn((
                        RemoveKeywordButton {
                            keyword: keyword.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.5)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_CLOSE),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.5, 0.5)),
                        ));
                    });
                });
        }
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

    let font: Handle<Font> = get_font();

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
        commands.entity(entity).with_children(|list| {
            spawn_blocked_keyword_tags(list, &font, &filter_state.blocked_keywords);
        });
    }

    // 更新建议面板中已屏蔽项的禁用外观
    for (item, mut bg, mut border, children) in suggestion_query.iter_mut() {
        let already_blocked = filter_state.blocked_keywords.contains(&item.keyword);
        if already_blocked {
            *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.4));
            *border = BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.3));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.7));
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

/// 过滤复选框类型（内部使用）
enum FilterCheckboxType {
    Category,
    Tag,
    Title,
}

/// 创建过滤模式复选框
fn spawn_filter_mode_checkbox(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    checked: bool,
    checkbox_type: FilterCheckboxType,
) {
    let bg = if checked {
        AppColors::PRIMARY
    } else {
        Color::srgb(0.12, 0.12, 0.16)
    };
    let border = if checked {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };

    parent
        .spawn((Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        },))
        .with_children(|row| {
            // 复选框
            let mut checkbox_entity = row.spawn((
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border),
            ));

            // 添加对应的组件标记
            match checkbox_type {
                FilterCheckboxType::Category => {
                    checkbox_entity.insert(FilterByCategoryCheckbox);
                }
                FilterCheckboxType::Tag => {
                    checkbox_entity.insert(FilterByTagCheckbox);
                }
                FilterCheckboxType::Title => {
                    checkbox_entity.insert(FilterByTitleCheckbox);
                }
            }

            checkbox_entity.with_children(|cb| {
                cb.spawn((
                    Text::new(if checked { ICON_CHECK } else { "" }),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 标签文本
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
        });
}

/// 创建缓存设置
fn spawn_cache_setting(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|row| {
            // 左侧标签和说明
            row.spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|left| {
                    left.spawn((
                        Text::new("图片缓存"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                    left.spawn((
                        Text::new("清除本地缓存的封面图片"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

            // 清除按钮
            row.spawn((
                ClearCacheButton,
                Button,
                Node {
                    padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                BorderColor::all(Color::srgb(0.8, 0.3, 0.3)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("清除缓存"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });
        });
}

/// 创建关于分组
fn spawn_about_section(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|col| {
            col.spawn((
                Text::new("PicACG Rust 客户端"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));
            col.spawn((
                Text::new(format!("版本: {}", env!("CARGO_PKG_VERSION"))),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            col.spawn((
                Text::new(format!("框架: Bevy {}", env!("BEVY_VERSION"))),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 创建代理设置
fn spawn_proxy_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    settings: &picacg_config::AppSettings,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        },))
        .with_children(|col| {
            // 启用代理复选框
            col.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },))
                .with_children(|row| {
                    row.spawn((
                        ProxyEnabledCheckbox,
                        Button,
                        Node {
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(if settings.proxy.enabled {
                            AppColors::PRIMARY
                        } else {
                            Color::srgb(0.12, 0.12, 0.16)
                        }),
                        BorderColor::all(if settings.proxy.enabled {
                            AppColors::PRIMARY
                        } else {
                            AppColors::BORDER
                        }),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(if settings.proxy.enabled {
                                ICON_CHECK
                            } else {
                                ""
                            }),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                    row.spawn((
                        Text::new("启用代理"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));
                });

            // 代理类型选择
            col.spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },))
                .with_children(|type_col| {
                    type_col.spawn((
                        Text::new("代理类型"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));

                    type_col
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },))
                        .with_children(|btn_row| {
                            for (proxy_type, label) in [
                                (ProxyType::Http, "HTTP"),
                                (ProxyType::Https, "HTTPS"),
                                (ProxyType::Socks5, "SOCKS5"),
                            ] {
                                let is_selected = settings.proxy.proxy_type == proxy_type;
                                btn_row
                                    .spawn((
                                        ProxyTypeButton { proxy_type },
                                        Button,
                                        Node {
                                            padding: UiRect::new(
                                                Val::Px(12.0),
                                                Val::Px(12.0),
                                                Val::Px(6.0),
                                                Val::Px(6.0),
                                            ),
                                            border: UiRect::all(Val::Px(1.0)),
                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor(if is_selected {
                                            AppColors::PRIMARY
                                        } else {
                                            Color::srgb(0.12, 0.12, 0.16)
                                        }),
                                        BorderColor::all(if is_selected {
                                            AppColors::PRIMARY
                                        } else {
                                            AppColors::BORDER
                                        }),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(label),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 13.0,
                                                ..default()
                                            },
                                            TextColor(AppColors::TEXT),
                                        ));
                                    });
                            }
                        });
                });

            // 代理地址和端口
            col.spawn((Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },))
                .with_children(|row| {
                    // 主机地址
                    row.spawn((Node {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },))
                        .with_children(|host_col| {
                            host_col.spawn((
                                Text::new("主机地址"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                            host_col
                                .spawn((
                                    ProxyHostInput,
                                    TextInput::new("127.0.0.1").with_value(&settings.proxy.host),
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(32.0),
                                        padding: UiRect::horizontal(Val::Px(10.0)),
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                                    BorderColor::all(AppColors::BORDER),
                                    RelativeCursorPosition::default(),
                                ))
                                .with_children(|input| {
                                    input.spawn((
                                        TextInputDisplay,
                                        Text::new(if settings.proxy.host.is_empty() {
                                            "127.0.0.1".to_string()
                                        } else {
                                            settings.proxy.host.clone()
                                        }),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(if settings.proxy.host.is_empty() {
                                            AppColors::TEXT_SECONDARY
                                        } else {
                                            AppColors::TEXT
                                        }),
                                    ));
                                });
                        });

                    // 端口
                    row.spawn((Node {
                        width: Val::Px(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },))
                        .with_children(|port_col| {
                            port_col.spawn((
                                Text::new("端口"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                            port_col
                                .spawn((
                                    ProxyPortInput,
                                    TextInput::new("7890")
                                        .with_value(settings.proxy.port.to_string()),
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(32.0),
                                        padding: UiRect::horizontal(Val::Px(10.0)),
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                                    BorderColor::all(AppColors::BORDER),
                                    RelativeCursorPosition::default(),
                                ))
                                .with_children(|input| {
                                    let port_str = settings.proxy.port.to_string();
                                    input.spawn((
                                        TextInputDisplay,
                                        Text::new(if port_str.is_empty() {
                                            "7890".to_string()
                                        } else {
                                            port_str.clone()
                                        }),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(if port_str.is_empty() {
                                            AppColors::TEXT_SECONDARY
                                        } else {
                                            AppColors::TEXT
                                        }),
                                    ));
                                });
                        });
                });
        });
}

/// 创建分流设置
fn spawn_channel_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    settings: &picacg_config::AppSettings,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        },))
        .with_children(|col| {
            // API 分流选择
            spawn_channel_row(col, font, "API 分流", settings.channel.api_channel, true);

            // 自定义 API IP 输入行
            spawn_custom_ip_row(
                col,
                font,
                "自定义 API IP",
                &settings.channel.custom_cdn_api_ip,
                true,
                settings.channel.api_channel == ChannelType::CustomCdnIp,
            );

            // 图片分流选择
            spawn_channel_row(col, font, "图片分流", settings.channel.image_channel, false);

            // 自定义图片 IP 输入行
            spawn_custom_ip_row(
                col,
                font,
                "自定义图片 IP",
                &settings.channel.custom_cdn_img_ip,
                false,
                settings.channel.image_channel == ChannelType::CustomCdnIp,
            );
        });
}

/// 创建分流按钮行
fn spawn_channel_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    current: ChannelType,
    is_api: bool,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|type_col| {
            type_col.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            type_col
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(8.0),
                    ..default()
                },))
                .with_children(|btn_row| {
                    for channel_type in ChannelType::all() {
                        let is_selected = current == *channel_type;
                        let mut btn = btn_row.spawn((
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::new(
                                    Val::Px(10.0),
                                    Val::Px(10.0),
                                    Val::Px(6.0),
                                    Val::Px(6.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(if is_selected {
                                AppColors::PRIMARY
                            } else {
                                Color::srgb(0.12, 0.12, 0.16)
                            }),
                            BorderColor::all(if is_selected {
                                AppColors::PRIMARY
                            } else {
                                AppColors::BORDER
                            }),
                        ));
                        if is_api {
                            btn.insert(ApiChannelButton {
                                channel_type: *channel_type,
                            });
                        } else {
                            btn.insert(ImageChannelButton {
                                channel_type: *channel_type,
                            });
                        }
                        btn.with_children(|btn| {
                            btn.spawn((
                                Text::new(channel_type.display_name()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                    }
                });
        });
}

/// 创建自定义 IP 输入行
fn spawn_custom_ip_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    value: &str,
    is_api: bool,
    visible: bool,
) {
    let mut row = parent.spawn((Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(10.0),
        display: if visible {
            Display::Flex
        } else {
            Display::None
        },
        ..default()
    },));
    if is_api {
        row.insert(CustomCdnApiIpRow);
    } else {
        row.insert(CustomCdnImgIpRow);
    }
    row.with_children(|row| {
        row.spawn((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(AppColors::TEXT),
        ));

        let placeholder = "输入 IP 地址，例如 104.21.91.145";
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

        let mut input = row.spawn((
            TextInput::new(placeholder).with_value(value),
            Button,
            Interaction::default(),
            Node {
                flex_grow: 1.0,
                height: Val::Px(32.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
            BorderColor::all(AppColors::BORDER),
            RelativeCursorPosition::default(),
        ));
        if is_api {
            input.insert(CustomCdnApiIpInput);
        } else {
            input.insert(CustomCdnImgIpInput);
        }
        input.with_children(|input| {
            input.spawn((
                TextInputDisplay,
                Text::new(display_text),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
    });
}

/// 创建日志等级设置
fn spawn_log_level_setting(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    current_level: LogLevel,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|col| {
            col.spawn((
                Text::new("日志等级"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            col.spawn((
                Text::new("设置日志输出的详细程度，重启后生效"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));

            // 日志等级按钮组
            col.spawn((Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(8.0),
                ..default()
            },))
                .with_children(|btn_row| {
                    for level in [
                        LogLevel::Trace,
                        LogLevel::Debug,
                        LogLevel::Info,
                        LogLevel::Warn,
                        LogLevel::Error,
                    ] {
                        let is_selected = current_level == level;
                        btn_row
                            .spawn((
                                LogLevelButton { level },
                                Button,
                                Interaction::default(),
                                Node {
                                    padding: UiRect::new(
                                        Val::Px(12.0),
                                        Val::Px(12.0),
                                        Val::Px(6.0),
                                        Val::Px(6.0),
                                    ),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_selected {
                                    AppColors::PRIMARY
                                } else {
                                    Color::srgb(0.12, 0.12, 0.16)
                                }),
                                BorderColor::all(if is_selected {
                                    AppColors::PRIMARY
                                } else {
                                    AppColors::BORDER
                                }),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(level.display_name()),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(AppColors::TEXT),
                                ));
                            });
                    }
                });
        });
}

/// 创建底部状态栏（显示自动保存提示）
fn spawn_status_bar(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            SettingsStatusBar,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::horizontal(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::top(Val::Px(1.0)),
                display: Display::None, // 初始隐藏
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
            BorderColor::all(AppColors::BORDER),
        ))
        .with_children(|bar| {
            bar.spawn((
                SettingsStatusText,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.8, 0.5)), // 绿色成功提示
            ));
        });
}

/// 创建设置页面滚动条
///
/// 布局结构（与 categories.rs 一致）：
/// ScrollbarContainer (Absolute, right=0)
///   ├── ScrollbarTrack (Button, fills 100%, ZIndex=0)
///   └── ScrollbarThumb (Button, Absolute, ZIndex=1)
///
/// 滑块和轨道作为兄弟节点，避免父子节点交互事件冲突
fn spawn_settings_scrollbar(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
    parent
        .spawn((
            ScrollbarContainer { scroll_container },
            Node {
                width: Val::Px(SCROLLBAR_WIDTH),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(10),
        ))
        .with_children(|scrollbar| {
            // 滚动条轨道（与滑块同级，ZIndex 较低）
            scrollbar.spawn((
                ScrollbarTrack { scroll_container },
                Button,
                Interaction::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(TRACK_COLOR),
                ZIndex(0),
                RelativeCursorPosition::default(),
            ));

            // 滚动条滑块（与轨道同级，ZIndex 较高以覆盖轨道）
            // 使用 FocusPolicy::Block 阻止事件穿透到轨道
            scrollbar.spawn((
                ScrollbarThumb { scroll_container },
                Button,
                Interaction::default(),
                FocusPolicy::Block,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(THUMB_MIN_HEIGHT),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    border_radius: BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                    ..default()
                },
                BackgroundColor(THUMB_COLOR),
                ZIndex(1),
            ));
        });
}

/// 清理设置页面
pub fn cleanup_settings_ui(mut commands: Commands, query: Query<Entity, With<SettingsRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DownloadPathInputState>();
    commands.remove_resource::<ProxySettingsInputState>();
    commands.remove_resource::<LogLevelInputState>();
    commands.remove_resource::<AutoResumeDownloadsState>();
    commands.remove_resource::<MaxConcurrentDownloadsState>();
    commands.remove_resource::<CbzPackageSettingsState>();
    commands.remove_resource::<ChannelSettingsState>();
    commands.remove_resource::<SettingsSaveStatus>();
}

/// 下载路径输入框交互
pub fn download_path_input_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (Changed<Interaction>, With<DownloadPathInput>),
    >,
    mut input_state: ResMut<DownloadPathInputState>,
) {
    for (interaction, mut bg_color, mut border_color, mut input) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                input.focused = true;
                input_state.is_focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                if !input.focused {
                    *border_color = BorderColor::all(Color::srgb(0.4, 0.4, 0.5));
                }
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                if !input.focused {
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 下载路径动作键处理（Escape/Enter 失焦），编辑由通用 TextInput 处理
pub fn download_path_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut input_query: Query<&mut TextInput, With<DownloadPathInput>>,
    mut input_state: ResMut<DownloadPathInputState>,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    let has_focus = input_query.iter().any(|i| i.focused);
    if !has_focus {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if matches!(&event.logical_key, Key::Escape | Key::Enter) {
            for mut input in input_query.iter_mut() {
                input.focused = false;
                input_state.is_focused = false;
            }
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
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DownloadPathPickerButton>),
    >,
    mut picker: ResMut<DownloadPathPickerResult>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
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
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::SECONDARY_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::SECONDARY);
            }
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
            input.focused = false;
        }
        input_state.value = path_str;
        input_state.is_focused = false;
    }
}

/// 清除缓存按钮交互
pub fn clear_cache_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ClearCacheButton>),
    >,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.1, 0.1));

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
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.7, 0.25, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2));
            }
        }
    }
}

/// 将所有设置状态写入 AppSettings 并保存到磁盘
fn save_all_settings(
    input_state: &DownloadPathInputState,
    proxy_state: &ProxySettingsInputState,
    log_state: &LogLevelInputState,
    auto_resume_state: &AutoResumeDownloadsState,
    max_concurrent_state: &MaxConcurrentDownloadsState,
    cbz_state: &CbzPackageSettingsState,
    filter_state: &FilterSettingsState,
    channel_state: &ChannelSettingsState,
) -> Result<(), String> {
    let mut settings = AppSettings::global().write();
    settings.download_path = input_state.value.clone();
    settings.proxy.enabled = proxy_state.enabled;
    settings.proxy.proxy_type = proxy_state.proxy_type;
    settings.proxy.host = proxy_state.host.clone();
    settings.proxy.port = proxy_state.port.parse().unwrap_or(7890);
    settings.log_level = log_state.level;
    settings.auto_resume_downloads = auto_resume_state.enabled;
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
    settings.save().map_err(|e| e.to_string())?;
    update_log_level(log_state.level);
    Ok(())
}

/// 自动保存设置：监听所有设置状态变化，有变化时自动保存
pub fn auto_save_settings(
    input_state: Res<DownloadPathInputState>,
    proxy_state: Res<ProxySettingsInputState>,
    log_state: Res<LogLevelInputState>,
    auto_resume_state: Res<AutoResumeDownloadsState>,
    max_concurrent_state: Res<MaxConcurrentDownloadsState>,
    cbz_state: Res<CbzPackageSettingsState>,
    filter_state: Res<FilterSettingsState>,
    channel_state: Res<ChannelSettingsState>,
    mut save_status: ResMut<SettingsSaveStatus>,
    mut reload_api_messages: MessageWriter<crate::events::ReloadApiClientEvent>,
    mut initialized: Local<bool>,
) {
    let channel_changed = channel_state.is_changed();
    let any_changed = input_state.is_changed()
        || proxy_state.is_changed()
        || log_state.is_changed()
        || auto_resume_state.is_changed()
        || max_concurrent_state.is_changed()
        || cbz_state.is_changed()
        || filter_state.is_changed()
        || channel_changed;

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
        &auto_resume_state,
        &max_concurrent_state,
        &cbz_state,
        &filter_state,
        &channel_state,
    ) {
        Ok(()) => {
            save_status.visible = true;
            save_status.message = "设置已保存".to_string();
            save_status.is_error = false;
            save_status.timer.reset();
            tracing::debug!("设置已自动保存");

            // 分流或代理变更时通知重建 API 客户端
            if channel_changed || proxy_state.is_changed() {
                reload_api_messages.write(crate::events::ReloadApiClientEvent);
                tracing::info!("分流/代理设置变更，通知重建 API 客户端");
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

    // 更新文本内容和颜色
    for (mut text, mut color) in text_query.iter_mut() {
        **text = save_status.message.clone();
        *color = if save_status.is_error {
            TextColor(Color::srgb(0.9, 0.3, 0.3)) // 红色错误提示
        } else {
            TextColor(Color::srgb(0.4, 0.8, 0.5)) // 绿色成功提示
        };
    }

    // 显示状态栏
    for mut node in bar_query.iter_mut() {
        node.display = Display::Flex;
    }

    if save_status.timer.just_finished() {
        save_status.visible = false;
        for mut node in bar_query.iter_mut() {
            node.display = Display::None;
        }
    }
}

/// 处理设置页面滚动
pub fn handle_settings_scroll(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut scroll_query: Query<
        (&mut ScrollPosition, &ComputedNode, Option<&ContentSizeInfo>),
        With<SettingsScrollContainer>,
    >,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll_position, computed_node, content_size_info) in &mut scroll_query {
            let scroll_delta = match event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
                bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
            };

            // 获取内容和视口高度
            let (content_height, viewport_height) = if let Some(info) = content_size_info {
                (info.content_height, info.viewport_height)
            } else {
                let size = computed_node.size();
                (size.y, size.y)
            };

            let max_scroll = (content_height - viewport_height).max(0.0);

            // 更新滚动位置
            let old_scroll = scroll_position.y;
            scroll_position.y = (scroll_position.y - scroll_delta).clamp(0.0, max_scroll);

            // 详细日志：每次滚动时输出（trace 级别）
            tracing::trace!(
                "[Settings] 滚动: delta={:.1}, old={:.1}, new={:.1}, max={:.1}, content={:.1}, viewport={:.1}",
                scroll_delta,
                old_scroll,
                scroll_position.y,
                max_scroll,
                content_height,
                viewport_height
            );
        }
    }
}

/// 限制设置页面滚动范围（防止越界）
pub fn clamp_settings_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<SettingsScrollContainer>,
    >,
) {
    for (mut scroll_position, content_size_info) in &mut scroll_query {
        if scroll_position.y < 0.0 {
            scroll_position.y = 0.0;
        }

        if let Some(content_info) = content_size_info {
            let max_scroll = (content_info.content_height - content_info.viewport_height).max(0.0);
            if scroll_position.y > max_scroll {
                scroll_position.y = max_scroll;
            }
        }
    }
}

/// 更新设置页面内容尺寸
pub fn update_settings_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &Node, &mut ContentSizeInfo, &Children),
        With<SettingsScrollContainer>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    for (scroll_computed, node, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        let mut content_height = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(child) {
                content_height += child_computed.size().y / scale_factor;
            }
        }

        // 加上容器的上下 padding（ComputedNode::size 包含 padding，
        // 但子元素高度之和不含容器 padding，需要补偿）
        let padding_top = match node.padding.top {
            Val::Px(px) => px,
            _ => 0.0,
        };
        let padding_bottom = match node.padding.bottom {
            Val::Px(px) => px,
            _ => 0.0,
        };
        content_height += padding_top + padding_bottom;

        // 加上 row_gap（子元素间距）
        let child_count = children.len();
        if child_count > 1 {
            let gap = match node.row_gap {
                Val::Px(px) => px,
                _ => 0.0,
            };
            content_height += gap * (child_count - 1) as f32;
        }

        content_info.viewport_height = viewport_height;
        content_info.content_height = content_height;
    }
}

// ==================== 代理设置交互系统 ====================

/// 代理启用复选框交互
pub fn proxy_enabled_checkbox_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<ProxyEnabledCheckbox>),
    >,
    mut proxy_state: ResMut<ProxySettingsInputState>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut bg_color, mut border_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                proxy_state.enabled = !proxy_state.enabled;

                // 更新显示
                if proxy_state.enabled {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }

                // 更新勾选符号
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = if proxy_state.enabled {
                            ICON_CHECK.to_string()
                        } else {
                            String::new()
                        };
                    }
                }
            }
            Interaction::Hovered => {
                if !proxy_state.enabled {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if !proxy_state.enabled {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 代理类型按钮交互
pub fn proxy_type_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ProxyTypeButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut proxy_state: ResMut<ProxySettingsInputState>,
    mut all_buttons_query: Query<
        (&ProxyTypeButton, &mut BackgroundColor, &mut BorderColor),
        Without<Interaction>,
    >,
) {
    for (interaction, btn, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                proxy_state.proxy_type = btn.proxy_type;

                // 更新当前按钮
                *bg_color = BackgroundColor(AppColors::PRIMARY);
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 更新其他按钮
                for (other_btn, mut other_bg, mut other_border) in all_buttons_query.iter_mut() {
                    if other_btn.proxy_type != btn.proxy_type {
                        *other_bg = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                        *other_border = BorderColor::all(AppColors::BORDER);
                    }
                }
            }
            Interaction::Hovered => {
                if proxy_state.proxy_type != btn.proxy_type {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if proxy_state.proxy_type != btn.proxy_type {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 代理主机输入框交互（设置 TextInput.focused）
pub fn proxy_host_input_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (Changed<Interaction>, With<ProxyHostInput>),
    >,
    mut port_query: Query<
        (&mut TextInput, &mut BorderColor),
        (With<ProxyPortInput>, Without<ProxyHostInput>),
    >,
) {
    for (interaction, mut bg_color, mut border_color, mut input) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                input.focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
                // 失焦端口输入框
                for (mut port_input, mut port_border) in port_query.iter_mut() {
                    if port_input.focused {
                        port_input.focused = false;
                        *port_border = BorderColor::all(AppColors::BORDER);
                    }
                }
            }
            Interaction::Hovered => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                }
            }
            Interaction::None => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 代理端口输入框交互（设置 TextInput.focused）
pub fn proxy_port_input_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (Changed<Interaction>, With<ProxyPortInput>),
    >,
    mut host_query: Query<
        (&mut TextInput, &mut BorderColor),
        (With<ProxyHostInput>, Without<ProxyPortInput>),
    >,
) {
    for (interaction, mut bg_color, mut border_color, mut input) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                input.focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
                // 失焦主机输入框
                for (mut host_input, mut host_border) in host_query.iter_mut() {
                    if host_input.focused {
                        host_input.focused = false;
                        *host_border = BorderColor::all(AppColors::BORDER);
                    }
                }
            }
            Interaction::Hovered => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                }
            }
            Interaction::None => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 代理输入动作键处理（Escape/Enter 失焦），编辑由通用 TextInput 处理
pub fn proxy_input_keyboard(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut host_query: Query<
        (&mut TextInput, &mut BorderColor),
        (With<ProxyHostInput>, Without<ProxyPortInput>),
    >,
    mut port_query: Query<
        (&mut TextInput, &mut BorderColor),
        (With<ProxyPortInput>, Without<ProxyHostInput>),
    >,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    let has_focus =
        host_query.iter().any(|(i, _)| i.focused) || port_query.iter().any(|(i, _)| i.focused);
    if !has_focus {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if matches!(&event.logical_key, Key::Escape | Key::Enter) {
            for (mut input, mut border) in host_query.iter_mut() {
                if input.focused {
                    input.focused = false;
                    *border = BorderColor::all(AppColors::BORDER);
                }
            }
            for (mut input, mut border) in port_query.iter_mut() {
                if input.focused {
                    input.focused = false;
                    *border = BorderColor::all(AppColors::BORDER);
                }
            }
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

// ==================== 日志等级交互系统 ====================

/// 日志等级按钮交互
pub fn log_level_button_interaction(
    mut buttons_query: Query<(
        &Interaction,
        &LogLevelButton,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut log_state: ResMut<LogLevelInputState>,
) {
    // 先检查是否有按钮被按下，收集新选择的等级
    let mut new_level: Option<LogLevel> = None;
    for (interaction, btn, _, _) in buttons_query.iter() {
        if *interaction == Interaction::Pressed && log_state.level != btn.level {
            new_level = Some(btn.level);
            break;
        }
    }

    // 如果有新选择，更新状态
    if let Some(level) = new_level {
        tracing::info!("日志等级已选择: {:?}", level);
        log_state.level = level;
    }

    // 更新所有按钮的外观
    for (interaction, btn, mut bg_color, mut border_color) in buttons_query.iter_mut() {
        let is_selected = log_state.level == btn.level;

        if is_selected {
            *bg_color = BackgroundColor(AppColors::PRIMARY);
            *border_color = BorderColor::all(AppColors::PRIMARY);
        } else {
            match *interaction {
                Interaction::Hovered => {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
                _ => {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

// ==================== 自动恢复下载交互系统 ====================

/// 自动恢复下载勾选框交互
pub fn auto_resume_downloads_checkbox_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<AutoResumeDownloadsCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut auto_resume_state: ResMut<AutoResumeDownloadsState>,
) {
    for (interaction, mut bg_color, mut border_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // 切换状态
                auto_resume_state.enabled = !auto_resume_state.enabled;
                let is_enabled = auto_resume_state.enabled;

                tracing::info!("自动恢复下载: {}", if is_enabled { "启用" } else { "禁用" });

                // 更新外观
                if is_enabled {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }

                // 更新勾选标记
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = if is_enabled {
                            ICON_CHECK.to_string()
                        } else {
                            String::new()
                        };
                    }
                }
            }
            Interaction::Hovered => {
                if !auto_resume_state.enabled {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if !auto_resume_state.enabled {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

// ==================== 最大并发下载数交互系统 ====================

/// 最大并发下载数减少按钮交互
pub fn max_concurrent_downloads_decrease_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<MaxConcurrentDownloadsDecreaseButton>,
        ),
    >,
    mut state: ResMut<MaxConcurrentDownloadsState>,
    mut text_query: Query<&mut Text, With<MaxConcurrentDownloadsText>>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));
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
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.24));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}

/// 最大并发下载数增加按钮交互
pub fn max_concurrent_downloads_increase_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<MaxConcurrentDownloadsIncreaseButton>,
        ),
    >,
    mut state: ResMut<MaxConcurrentDownloadsState>,
    mut text_query: Query<&mut Text, With<MaxConcurrentDownloadsText>>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));
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
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.24));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}

// ==================== CBZ 打包设置交互系统 ====================

/// 自动打包 CBZ 勾选框交互
pub fn auto_pack_cbz_checkbox_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<AutoPackCbzCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut cbz_state: ResMut<CbzPackageSettingsState>,
) {
    for (interaction, mut bg_color, mut border_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // 切换状态
                cbz_state.auto_pack_cbz = !cbz_state.auto_pack_cbz;
                let is_enabled = cbz_state.auto_pack_cbz;

                tracing::info!("自动打包 CBZ: {}", if is_enabled { "启用" } else { "禁用" });

                // 更新外观
                if is_enabled {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }

                // 更新勾选标记
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = if is_enabled {
                            ICON_CHECK.to_string()
                        } else {
                            String::new()
                        };
                    }
                }
            }
            Interaction::Hovered => {
                if !cbz_state.auto_pack_cbz {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if !cbz_state.auto_pack_cbz {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 打包后删除原图勾选框交互
pub fn delete_images_after_cbz_checkbox_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<DeleteImagesAfterCbzCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut cbz_state: ResMut<CbzPackageSettingsState>,
) {
    for (interaction, mut bg_color, mut border_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // 切换状态
                cbz_state.delete_images_after_cbz = !cbz_state.delete_images_after_cbz;
                let is_enabled = cbz_state.delete_images_after_cbz;

                tracing::info!(
                    "打包后删除原图: {}",
                    if is_enabled { "启用" } else { "禁用" }
                );

                // 更新外观
                if is_enabled {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }

                // 更新勾选标记
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = if is_enabled {
                            ICON_CHECK.to_string()
                        } else {
                            String::new()
                        };
                    }
                }
            }
            Interaction::Hovered => {
                if !cbz_state.delete_images_after_cbz {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if !cbz_state.delete_images_after_cbz {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

// ==================== 内容过滤交互系统 ====================

/// 通用过滤模式复选框交互逻辑
fn toggle_filter_checkbox(
    bg_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
    children: &Children,
    text_query: &mut Query<&mut Text>,
    is_enabled: bool,
) {
    if is_enabled {
        *bg_color = BackgroundColor(AppColors::PRIMARY);
        *border_color = BorderColor::all(AppColors::PRIMARY);
    } else {
        *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
        *border_color = BorderColor::all(AppColors::BORDER);
    }
    for child in children.iter() {
        if let Ok(mut text) = text_query.get_mut(child) {
            **text = if is_enabled {
                ICON_CHECK.to_string()
            } else {
                String::new()
            };
        }
    }
}

/// 按分类屏蔽复选框交互
pub fn filter_by_category_checkbox_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<FilterByCategoryCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    for (interaction, mut bg_color, mut border_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                filter_state.filter_by_category = !filter_state.filter_by_category;
                toggle_filter_checkbox(
                    &mut bg_color,
                    &mut border_color,
                    children,
                    &mut text_query,
                    filter_state.filter_by_category,
                );
            }
            Interaction::Hovered => {
                if !filter_state.filter_by_category {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if !filter_state.filter_by_category {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 按标签屏蔽复选框交互
pub fn filter_by_tag_checkbox_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<FilterByTagCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    for (interaction, mut bg_color, mut border_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                filter_state.filter_by_tag = !filter_state.filter_by_tag;
                toggle_filter_checkbox(
                    &mut bg_color,
                    &mut border_color,
                    children,
                    &mut text_query,
                    filter_state.filter_by_tag,
                );
            }
            Interaction::Hovered => {
                if !filter_state.filter_by_tag {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if !filter_state.filter_by_tag {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 按标题屏蔽复选框交互
pub fn filter_by_title_checkbox_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<FilterByTitleCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    for (interaction, mut bg_color, mut border_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                filter_state.filter_by_title = !filter_state.filter_by_title;
                toggle_filter_checkbox(
                    &mut bg_color,
                    &mut border_color,
                    children,
                    &mut text_query,
                    filter_state.filter_by_title,
                );
            }
            Interaction::Hovered => {
                if !filter_state.filter_by_title {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if !filter_state.filter_by_title {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 删除屏蔽词按钮交互
pub fn remove_keyword_interaction(
    mut interaction_query: Query<
        (&Interaction, &RemoveKeywordButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut filter_state: ResMut<FilterSettingsState>,
) {
    let mut keyword_to_remove: Option<String> = None;
    for (interaction, btn, _) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            keyword_to_remove = Some(btn.keyword.clone());
            break;
        }
    }

    if let Some(keyword) = keyword_to_remove {
        filter_state.blocked_keywords.retain(|k| k != &keyword);
        tracing::info!("删除屏蔽词: {}", keyword);
    }

    // 更新悬停样式
    for (interaction, _, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.7, 0.2, 0.2, 0.7));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.5));
            }
            _ => {}
        }
    }
}

/// 新增屏蔽词输入框交互（设置 TextInput.focused，含 IME 启用）
pub fn new_keyword_input_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
            &ComputedNode,
        ),
        (Changed<Interaction>, With<NewKeywordInput>),
    >,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for (interaction, mut bg_color, mut border_color, mut input, computed) in
        interaction_query.iter_mut()
    {
        match *interaction {
            Interaction::Pressed => {
                input.focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 启用 IME 并设置候选框位置
                if let Ok(mut window) = window_query.single_mut() {
                    window.ime_enabled = true;
                    if let Some(cursor_pos) = window.cursor_position() {
                        let scale_factor = window.scale_factor();
                        let input_height = computed.size().y / scale_factor;
                        window.ime_position = bevy::math::Vec2::new(
                            cursor_pos.x,
                            cursor_pos.y + input_height / 2.0 + 5.0,
                        );
                    }
                }
            }
            Interaction::Hovered => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                }
            }
            Interaction::None => {
                if !input.focused {
                    *bg_color = BackgroundColor(AppColors::CARD_BG);
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 新增屏蔽词动作键处理（Enter 添加屏蔽词，Escape 失焦），编辑由通用 TextInput
/// 处理
pub fn new_keyword_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut input_query: Query<&mut TextInput, With<NewKeywordInput>>,
    mut filter_state: ResMut<FilterSettingsState>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let has_focus = input_query.iter().any(|i| i.focused);
    if !has_focus {
        return;
    }

    use bevy::input::{ButtonState, keyboard::Key};

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Escape => {
                for mut input in input_query.iter_mut() {
                    input.focused = false;
                }
                // 禁用 IME
                if let Ok(mut window) = window_query.single_mut() {
                    window.ime_enabled = false;
                }
            }
            Key::Enter => {
                // 回车添加屏蔽词（从 TextInput.value 读取）
                for mut input in input_query.iter_mut() {
                    let keyword = input.value.trim().to_string();
                    if !keyword.is_empty() && !filter_state.blocked_keywords.contains(&keyword) {
                        tracing::info!("添加屏蔽词: {}", keyword);
                        filter_state.blocked_keywords.push(keyword);
                        input.set_value("");
                    }
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
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AddKeywordButton>),
    >,
    mut filter_state: ResMut<FilterSettingsState>,
    mut input_query: Query<&mut TextInput, With<NewKeywordInput>>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.8));
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
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.9));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
        }
    }
}

/// 点击输入框外部取消焦点
pub fn unfocus_keyword_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut input_query: Query<(&Interaction, &mut BorderColor, &mut TextInput), With<NewKeywordInput>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (interaction, mut border, mut input) in input_query.iter_mut() {
            if *interaction == Interaction::None && input.focused {
                input.focused = false;
                *border = BorderColor::all(AppColors::BORDER);

                // 禁用 IME
                if let Ok(mut window) = window_query.single_mut() {
                    window.ime_enabled = false;
                }
            }
        }
    }
}

/// 建议面板展开/折叠交互
pub fn keyword_suggestion_toggle_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<KeywordSuggestionToggle>),
    >,
    mut filter_state: ResMut<FilterSettingsState>,
    mut panel_query: Query<&mut Node, With<KeywordSuggestionPanel>>,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                filter_state.show_suggestions = !filter_state.show_suggestions;
                let display = if filter_state.show_suggestions {
                    Display::Flex
                } else {
                    Display::None
                };
                for mut node in panel_query.iter_mut() {
                    node.display = display;
                }
                // 按下状态高亮
                if filter_state.show_suggestions {
                    *bg_color = BackgroundColor(AppColors::PRIMARY.with_alpha(0.3));
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
            Interaction::Hovered => {
                if !filter_state.show_suggestions {
                    *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.24));
                }
            }
            Interaction::None => {
                if !filter_state.show_suggestions {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
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

    // 更新悬停样式
    for (interaction, item, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let already_blocked = filter_state.blocked_keywords.contains(&item.keyword);
        match *interaction {
            Interaction::Hovered if !already_blocked => {
                *bg_color = BackgroundColor(Color::srgba(0.25, 0.25, 0.35, 0.8));
            }
            Interaction::None => {
                if already_blocked {
                    *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.4));
                    *border_color = BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.3));
                } else {
                    *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.7));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
            _ => {}
        }
    }
}

// ==================== 分流设置交互系统 ====================

/// API 分流按钮交互
pub fn api_channel_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ApiChannelButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut channel_state: ResMut<ChannelSettingsState>,
    mut all_buttons_query: Query<
        (&ApiChannelButton, &mut BackgroundColor, &mut BorderColor),
        Without<Interaction>,
    >,
    mut api_ip_row_query: Query<&mut Node, With<CustomCdnApiIpRow>>,
) {
    for (interaction, btn, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                channel_state.api_channel = btn.channel_type;

                // 更新当前按钮样式
                *bg_color = BackgroundColor(AppColors::PRIMARY);
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 更新其他按钮样式
                for (other_btn, mut other_bg, mut other_border) in all_buttons_query.iter_mut() {
                    if other_btn.channel_type != btn.channel_type {
                        *other_bg = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                        *other_border = BorderColor::all(AppColors::BORDER);
                    }
                }

                // 切换自定义 IP 输入行显示
                for mut node in api_ip_row_query.iter_mut() {
                    node.display = if btn.channel_type == ChannelType::CustomCdnIp {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
            }
            Interaction::Hovered => {
                if channel_state.api_channel != btn.channel_type {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if channel_state.api_channel != btn.channel_type {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 图片分流按钮交互
pub fn image_channel_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ImageChannelButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut channel_state: ResMut<ChannelSettingsState>,
    mut all_buttons_query: Query<
        (&ImageChannelButton, &mut BackgroundColor, &mut BorderColor),
        Without<Interaction>,
    >,
    mut img_ip_row_query: Query<&mut Node, With<CustomCdnImgIpRow>>,
) {
    for (interaction, btn, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                channel_state.image_channel = btn.channel_type;

                // 更新当前按钮样式
                *bg_color = BackgroundColor(AppColors::PRIMARY);
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 更新其他按钮样式
                for (other_btn, mut other_bg, mut other_border) in all_buttons_query.iter_mut() {
                    if other_btn.channel_type != btn.channel_type {
                        *other_bg = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                        *other_border = BorderColor::all(AppColors::BORDER);
                    }
                }

                // 切换自定义 IP 输入行显示
                for mut node in img_ip_row_query.iter_mut() {
                    node.display = if btn.channel_type == ChannelType::CustomCdnIp {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
            }
            Interaction::Hovered => {
                if channel_state.image_channel != btn.channel_type {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if channel_state.image_channel != btn.channel_type {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}

/// 自定义 CDN IP 输入框交互（设置 TextInput.focused）
pub fn custom_cdn_ip_input_interaction(
    mut api_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (
            Changed<Interaction>,
            With<CustomCdnApiIpInput>,
            Without<CustomCdnImgIpInput>,
        ),
    >,
    mut img_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (
            Changed<Interaction>,
            With<CustomCdnImgIpInput>,
            Without<CustomCdnApiIpInput>,
        ),
    >,
) {
    for (interaction, mut bg_color, mut border_color, mut input) in api_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                input.focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
                // 失焦图片 IP 输入框
                for (_, _, mut img_border, mut img_input) in img_query.iter_mut() {
                    if img_input.focused {
                        img_input.focused = false;
                        *img_border = BorderColor::all(AppColors::BORDER);
                    }
                }
            }
            Interaction::Hovered => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                }
            }
            Interaction::None => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }

    for (interaction, mut bg_color, mut border_color, mut input) in img_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                input.focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
                // 失焦 API IP 输入框
                for (_, _, mut api_border, mut api_input) in api_query.iter_mut() {
                    if api_input.focused {
                        api_input.focused = false;
                        *api_border = BorderColor::all(AppColors::BORDER);
                    }
                }
            }
            Interaction::Hovered => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                }
            }
            Interaction::None => {
                if !input.focused {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 自定义 CDN IP 动作键处理（Escape/Enter 失焦），编辑由通用 TextInput 处理
pub fn custom_cdn_ip_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut api_query: Query<
        (&mut TextInput, &mut BorderColor),
        (With<CustomCdnApiIpInput>, Without<CustomCdnImgIpInput>),
    >,
    mut img_query: Query<
        (&mut TextInput, &mut BorderColor),
        (With<CustomCdnImgIpInput>, Without<CustomCdnApiIpInput>),
    >,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    let has_focus =
        api_query.iter().any(|(i, _)| i.focused) || img_query.iter().any(|(i, _)| i.focused);
    if !has_focus {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if matches!(&event.logical_key, Key::Escape | Key::Enter) {
            for (mut input, mut border) in api_query.iter_mut() {
                if input.focused {
                    input.focused = false;
                    *border = BorderColor::all(AppColors::BORDER);
                }
            }
            for (mut input, mut border) in img_query.iter_mut() {
                if input.focused {
                    input.focused = false;
                    *border = BorderColor::all(AppColors::BORDER);
                }
            }
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
