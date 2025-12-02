//! 设置界面系统
//!
//! 实现应用设置页面

use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    components::{
        ContentArea, ContentSizeInfo, ScrollbarContainer, ScrollbarThumb, ScrollbarTrack,
    },
    config::settings::{AppSettings, LogLevel, ProxyType},
    systems::{
        login::{AppColors, FONT_PATH},
        scrollbar::scrollbar_config::*,
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

/// 下载路径输入框标记
#[derive(Component)]
pub struct DownloadPathInput;

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

/// 保存设置按钮标记
#[derive(Component)]
pub struct SaveSettingsButton;

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
    pub host_focused: bool,
    pub port_focused: bool,
}

impl Default for ProxySettingsInputState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            enabled: settings.proxy.enabled,
            proxy_type: settings.proxy.proxy_type,
            host: settings.proxy.host.clone(),
            port: settings.proxy.port.to_string(),
            host_focused: false,
            port_focused: false,
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

/// 创建设置页面 UI
pub fn setup_settings_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);
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

    // 初始化代理设置状态
    commands.insert_resource(ProxySettingsInputState {
        enabled: settings.proxy.enabled,
        proxy_type: settings.proxy.proxy_type,
        host: settings.proxy.host.clone(),
        port: settings.proxy.port.to_string(),
        host_focused: false,
        port_focused: false,
    });

    // 初始化日志等级状态
    commands.insert_resource(LogLevelInputState {
        level: settings.log_level,
    });

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
                root.spawn(Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    ..default()
                })
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
                            });

                            // 缓存设置分组
                            spawn_settings_section(scroll, &font, "缓存设置", |section| {
                                spawn_cache_setting(section, &font);
                            });

                            // 关于分组
                            spawn_settings_section(scroll, &font, "关于", |section| {
                                spawn_about_section(section, &font);
                            });

                            // 底部保存按钮
                            spawn_save_button(scroll, &font);

                            // 底部间距（确保最后的内容可以完全滚动到可见区域）
                            // 需要足够的高度来容纳滚动，特别是在高 DPI 屏幕上
                            scroll.spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(100.0),
                                min_height: Val::Px(100.0),
                                ..default()
                            });
                        })
                        .id();

                    // 滚动条
                    spawn_settings_scrollbar(content_wrapper, scroll_container);
                });
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
                Text::new("⚙️ 设置"),
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
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
            BorderColor::all(AppColors::BORDER),
            BorderRadius::all(Val::Px(8.0)),
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
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
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

            // 输入框
            row.spawn((
                DownloadPathInput,
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(36.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                BorderColor::all(AppColors::BORDER),
                BorderRadius::all(Val::Px(4.0)),
            ))
            .with_children(|input| {
                let display_text = if current_path.is_empty() {
                    "（使用默认路径）".to_string()
                } else {
                    current_path.to_string()
                };
                input.spawn((
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
        });
}

/// 创建缓存设置
fn spawn_cache_setting(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
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
                    ..default()
                },
                BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                BorderColor::all(Color::srgb(0.8, 0.3, 0.3)),
                BorderRadius::all(Val::Px(4.0)),
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
                Text::new("版本: 0.2.0"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            col.spawn((
                Text::new("框架: Bevy 0.17.3"),
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
    settings: &crate::config::settings::AppSettings,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            Transform::default(),
        ))
        .with_children(|col| {
            // 启用代理复选框
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                Transform::default(),
            ))
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
                    BorderRadius::all(Val::Px(4.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(if settings.proxy.enabled { "✓" } else { "" }),
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
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                Transform::default(),
            ))
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
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },
                        Transform::default(),
                    ))
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
                                    BorderRadius::all(Val::Px(4.0)),
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
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                Transform::default(),
            ))
            .with_children(|row| {
                // 主机地址
                row.spawn((
                    Node {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    Transform::default(),
                ))
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
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(32.0),
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                            BorderColor::all(AppColors::BORDER),
                            BorderRadius::all(Val::Px(4.0)),
                        ))
                        .with_children(|input| {
                            input.spawn((
                                Text::new(&settings.proxy.host),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                });

                // 端口
                row.spawn((
                    Node {
                        width: Val::Px(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    Transform::default(),
                ))
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
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(32.0),
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                            BorderColor::all(AppColors::BORDER),
                            BorderRadius::all(Val::Px(4.0)),
                        ))
                        .with_children(|input| {
                            input.spawn((
                                Text::new(settings.proxy.port.to_string()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                });
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
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            Transform::default(),
        ))
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
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                Transform::default(),
            ))
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
                            Node {
                                padding: UiRect::new(
                                    Val::Px(12.0),
                                    Val::Px(12.0),
                                    Val::Px(6.0),
                                    Val::Px(6.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
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
                            BorderRadius::all(Val::Px(4.0)),
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

/// 创建保存按钮
fn spawn_save_button(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|row| {
            row.spawn((
                SaveSettingsButton,
                Button,
                Node {
                    padding: UiRect::new(
                        Val::Px(24.0),
                        Val::Px(24.0),
                        Val::Px(10.0),
                        Val::Px(10.0),
                    ),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(AppColors::PRIMARY),
                BorderColor::all(AppColors::PRIMARY),
                BorderRadius::all(Val::Px(4.0)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("保存设置"),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });
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
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
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
                // 添加 Transform 以获得 GlobalTransform（滚动条点击需要）
                Transform::default(),
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
                    ..default()
                },
                BackgroundColor(THUMB_COLOR),
                BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
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
}

/// 下载路径输入框交互
pub fn download_path_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<DownloadPathInput>),
    >,
    mut input_state: ResMut<DownloadPathInputState>,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                input_state.is_focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                if !input_state.is_focused {
                    *border_color = BorderColor::all(Color::srgb(0.4, 0.4, 0.5));
                }
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                if !input_state.is_focused {
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 下载路径键盘输入
pub fn download_path_keyboard_input(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut input_state: ResMut<DownloadPathInputState>,
    mut text_query: Query<&mut Text, With<DownloadPathInput>>,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    if !input_state.is_focused {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Backspace => {
                input_state.value.pop();
            }
            Key::Escape | Key::Enter => {
                input_state.is_focused = false;
            }
            Key::Character(input) => {
                for c in input.chars() {
                    if !c.is_control() {
                        input_state.value.push(c);
                    }
                }
            }
            _ => {}
        }
    }

    // 更新显示文本
    for children in text_query.iter_mut() {
        // 由于 Text 结构的访问方式，这里需要重新查询
        let _ = children;
    }
}

/// 更新下载路径输入框显示
pub fn update_download_path_display(
    input_state: Res<DownloadPathInputState>,
    input_query: Query<&Children, With<DownloadPathInput>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    if !input_state.is_changed() {
        return;
    }

    for children in input_query.iter() {
        for child in children.iter() {
            if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                if input_state.value.is_empty() {
                    **text = "（使用默认路径）".to_string();
                    *color = TextColor(AppColors::TEXT_SECONDARY);
                } else {
                    **text = input_state.value.clone();
                    *color = TextColor(AppColors::TEXT);
                }
            }
        }
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

/// 保存设置按钮交互
pub fn save_settings_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SaveSettingsButton>),
    >,
    input_state: Res<DownloadPathInputState>,
    proxy_state: Res<ProxySettingsInputState>,
    log_state: Res<LogLevelInputState>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);

                // 保存设置
                let mut settings = AppSettings::global().write();
                settings.download_path = input_state.value.clone();

                // 保存代理设置
                settings.proxy.enabled = proxy_state.enabled;
                settings.proxy.proxy_type = proxy_state.proxy_type;
                settings.proxy.host = proxy_state.host.clone();
                settings.proxy.port = proxy_state.port.parse().unwrap_or(7890);

                // 保存日志等级
                settings.log_level = log_state.level;

                if let Err(e) = settings.save() {
                    tracing::error!("保存设置失败: {}", e);
                } else {
                    tracing::info!("设置已保存");
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
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
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<SettingsScrollContainer>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0);

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        let mut content_height = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(child) {
                content_height += child_computed.size().y / scale_factor;
            }
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
                            "✓".to_string()
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

/// 代理主机输入框交互
pub fn proxy_host_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ProxyHostInput>),
    >,
    mut proxy_state: ResMut<ProxySettingsInputState>,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                proxy_state.host_focused = true;
                proxy_state.port_focused = false;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::Hovered => {
                if !proxy_state.host_focused {
                    *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                }
            }
            Interaction::None => {
                if !proxy_state.host_focused {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 代理端口输入框交互
pub fn proxy_port_input_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ProxyPortInput>),
    >,
    mut proxy_state: ResMut<ProxySettingsInputState>,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                proxy_state.port_focused = true;
                proxy_state.host_focused = false;
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(AppColors::PRIMARY);
            }
            Interaction::Hovered => {
                if !proxy_state.port_focused {
                    *bg_color = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
                }
            }
            Interaction::None => {
                if !proxy_state.port_focused {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                    *border_color = BorderColor::all(AppColors::BORDER);
                }
            }
        }
    }
}

/// 代理输入键盘处理
pub fn proxy_input_keyboard(
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut proxy_state: ResMut<ProxySettingsInputState>,
    host_query: Query<&Children, With<ProxyHostInput>>,
    port_query: Query<&Children, With<ProxyPortInput>>,
    mut text_query: Query<&mut Text>,
) {
    use bevy::input::{ButtonState, keyboard::Key};

    if !proxy_state.host_focused && !proxy_state.port_focused {
        return;
    }

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Backspace => {
                if proxy_state.host_focused {
                    proxy_state.host.pop();
                } else if proxy_state.port_focused {
                    proxy_state.port.pop();
                }
            }
            Key::Escape | Key::Enter => {
                proxy_state.host_focused = false;
                proxy_state.port_focused = false;
            }
            Key::Character(input) => {
                for c in input.chars() {
                    if !c.is_control() {
                        if proxy_state.host_focused {
                            proxy_state.host.push(c);
                        } else if proxy_state.port_focused && c.is_ascii_digit() {
                            proxy_state.port.push(c);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // 更新主机显示
    for children in host_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = proxy_state.host.clone();
            }
        }
    }

    // 更新端口显示
    for children in port_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = proxy_state.port.clone();
            }
        }
    }
}

// ==================== 日志等级交互系统 ====================

/// 日志等级按钮交互
pub fn log_level_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &LogLevelButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut log_state: ResMut<LogLevelInputState>,
    mut all_buttons_query: Query<
        (&LogLevelButton, &mut BackgroundColor, &mut BorderColor),
        Without<Interaction>,
    >,
) {
    for (interaction, btn, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                log_state.level = btn.level;

                // 更新当前按钮
                *bg_color = BackgroundColor(AppColors::PRIMARY);
                *border_color = BorderColor::all(AppColors::PRIMARY);

                // 更新其他按钮
                for (other_btn, mut other_bg, mut other_border) in all_buttons_query.iter_mut() {
                    if other_btn.level != btn.level {
                        *other_bg = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                        *other_border = BorderColor::all(AppColors::BORDER);
                    }
                }
            }
            Interaction::Hovered => {
                if log_state.level != btn.level {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                }
            }
            Interaction::None => {
                if log_state.level != btn.level {
                    *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
                }
            }
        }
    }
}
