//! NAS 远程存储系统
//!
//! 支持通过 WebDAV 协议将本地下载目录上传到 NAS。
//! 使用 reqwest 直接发送 WebDAV 请求（PROPFIND, GET, PUT,
//! MKCOL），无需额外依赖。

use bevy::prelude::*;
use picacg_config::AppSettings;

use super::font_loader::get_font;
use crate::{
    components::{ContentArea, ContentSizeInfo},
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        ui_common::{Scrollable, spawn_scrollbar},
    },
    utils::{
        TokioTasksRuntime,
        icons::*,
        text_input::{TextInput, TextInputDisplay},
    },
};

// ==================== 组件定义 ====================

/// NAS 页面根节点
#[derive(Component)]
pub struct NasRoot;

/// NAS 滚动容器
#[derive(Component)]
pub struct NasScrollContainer;

/// 服务器地址输入框
#[derive(Component)]
pub struct NasServerUrlInput;

/// 用户名输入框
#[derive(Component)]
pub struct NasUsernameInput;

/// 密码输入框
#[derive(Component)]
pub struct NasPasswordInput;

/// 远程目录输入框
#[derive(Component)]
pub struct NasRemotePathInput;

/// 测试连接按钮
#[derive(Component)]
pub struct NasTestConnectionButton;

/// 上传下载目录按钮
#[derive(Component)]
pub struct NasUploadButton;

/// 浏览远程文件按钮
#[derive(Component)]
pub struct NasBrowseButton;

/// 启用 NAS 复选框
#[derive(Component)]
pub struct NasEnabledCheckbox;

/// 连接状态文本
#[derive(Component)]
pub struct NasStatusText;

/// 上传进度文本
#[derive(Component)]
pub struct NasProgressText;

/// 远程文件列表容器
#[derive(Component)]
pub struct NasFileListContainer;

/// 远程文件条目（预留字段供后续目录导航使用）
#[derive(Component)]
#[allow(dead_code)]
pub struct NasFileEntryItem {
    pub path: String,
    pub is_dir: bool,
}

// ==================== 输入状态资源 ====================

/// NAS 输入框状态（用于同步 TextInput 值到资源）
#[derive(Resource)]
pub struct NasInputState {
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub remote_path: String,
    pub enabled: bool,
}

impl Default for NasInputState {
    fn default() -> Self {
        let settings = AppSettings::global().read();
        Self {
            server_url: settings.nas.server_url.clone(),
            username: settings.nas.username.clone(),
            password: settings.nas.password.clone(),
            remote_path: settings.nas.remote_path.clone(),
            enabled: settings.nas.enabled,
        }
    }
}

// ==================== 系统函数 ====================

/// 创建 NAS 页面 UI
pub fn setup_nas_ui(
    mut commands: Commands,
    content_area_query: Query<Entity, With<ContentArea>>,
    mut existing_query: Query<&mut Node, With<NasRoot>>,
) {
    // 如果 NasRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        return;
    }

    let font: Handle<Font> = get_font();
    let settings = AppSettings::global().read();
    let content_area = content_area_query.single().ok();

    // 初始化输入状态（init_resource 不会覆盖已存在的资源）
    commands.insert_resource(NasInputState {
        server_url: settings.nas.server_url.clone(),
        username: settings.nas.username.clone(),
        password: settings.nas.password.clone(),
        remote_path: settings.nas.remote_path.clone(),
        enabled: settings.nas.enabled,
    });

    let scroll_id = std::cell::Cell::new(Entity::PLACEHOLDER);

    let root = commands
        .spawn((
            NasRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(AppColors::BACKGROUND),
        ))
        .with_children(|root| {
            // ===== 标题栏 =====
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|header| {
                header.spawn((
                    Text::new(format!("{} NAS 远程存储", ICON_NAS)),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // ===== 内容区域（可滚动）=====
            root.spawn((Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                overflow: Overflow::clip(),
                position_type: PositionType::Relative,
                ..default()
            },))
                .with_children(|wrapper| {
                    // 滚动容器
                    let sc_id = wrapper
                        .spawn((
                            NasScrollContainer,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::scroll_y(),
                                padding: UiRect::all(Val::Px(25.0)),
                                row_gap: Val::Px(20.0),
                                ..default()
                            },
                            Scrollable,
                            ScrollPosition::default(),
                            ContentSizeInfo::default(),
                        ))
                        .with_children(|content| {
                            // --- 启用开关行 ---
                            spawn_enabled_row(content, &font, &settings);

                            // --- 服务器地址 ---
                            spawn_input_row(
                                content,
                                &font,
                                "服务器地址",
                                &settings.nas.server_url,
                                "http://192.168.1.100:5005/webdav",
                                NasServerUrlInput,
                            );

                            // --- 用户名 ---
                            spawn_input_row(
                                content,
                                &font,
                                "用户名",
                                &settings.nas.username,
                                "admin",
                                NasUsernameInput,
                            );

                            // --- 密码 ---
                            spawn_password_row(content, &font, &settings);

                            // --- 远程目录 ---
                            spawn_input_row(
                                content,
                                &font,
                                "远程目录",
                                &settings.nas.remote_path,
                                "/picacg/",
                                NasRemotePathInput,
                            );

                            // --- 操作按钮行 ---
                            spawn_action_buttons(content, &font);

                            // --- 状态显示 ---
                            spawn_status_area(content, &font);

                            // --- 远程文件列表 ---
                            spawn_file_list_area(content, &font);

                            // --- 底部间距 ---
                            content.spawn(Node {
                                height: Val::Px(30.0),
                                min_height: Val::Px(30.0),
                                ..default()
                            });
                        })
                        .id();
                    scroll_id.set(sc_id);

                    // 滚动条
                    spawn_scrollbar(wrapper, sc_id);
                });
        })
        .id();

    // 挂载到内容区域
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(root);
    }

    tracing::info!("NAS 页面 UI 已创建");
}

/// 创建启用开关行
fn spawn_enabled_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    settings: &AppSettings,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new("启用 NAS 远程存储"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 复选框按钮
            let check_icon = if settings.nas.enabled {
                ICON_CHECK
            } else {
                " "
            };
            let bg = if settings.nas.enabled {
                AppColors::PRIMARY
            } else {
                Color::srgb(0.2, 0.2, 0.25)
            };

            row.spawn((
                NasEnabledCheckbox,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(check_icon),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 创建通用输入行
fn spawn_input_row<C: Component>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    value: &str,
    placeholder: &str,
    marker: C,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|col| {
            // 标签
            col.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));

            // 输入框
            let display_text = if value.is_empty() {
                placeholder.to_string()
            } else {
                value.to_string()
            };
            let text_color = if value.is_empty() {
                AppColors::TEXT_SECONDARY
            } else {
                AppColors::TEXT
            };

            col.spawn((
                marker,
                TextInput::new(placeholder).with_value(value),
                TextInputDisplay,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(36.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|input_box| {
                input_box.spawn((
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

/// 创建密码输入行
fn spawn_password_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    settings: &AppSettings,
) {
    let value = &settings.nas.password;
    let placeholder = "密码";
    let display_text = if value.is_empty() {
        placeholder.to_string()
    } else {
        "*".repeat(value.len())
    };
    let text_color = if value.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Text::new("密码"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));

            col.spawn((
                NasPasswordInput,
                TextInput::new(placeholder)
                    .with_value(value)
                    .with_password(),
                TextInputDisplay,
                Button,
                Interaction::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(36.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|input_box| {
                input_box.spawn((
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

/// 创建操作按钮行
fn spawn_action_buttons(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            margin: UiRect::top(Val::Px(5.0)),
            ..default()
        })
        .with_children(|row| {
            // 测试连接按钮
            spawn_action_button(row, font, "测试连接", NasTestConnectionButton);
            // 上传下载目录按钮
            spawn_action_button(row, font, "上传下载目录", NasUploadButton);
            // 浏览远程文件按钮
            spawn_action_button(row, font, "浏览远程文件", NasBrowseButton);
        });
}

/// 创建通用操作按钮
fn spawn_action_button<C: Component>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    marker: C,
) {
    parent
        .spawn((
            marker,
            Button,
            Interaction::default(),
            Node {
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(AppColors::PRIMARY),
            BorderColor::all(AppColors::PRIMARY),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 创建状态显示区域
fn spawn_status_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|col| {
            // 连接状态
            col.spawn((
                NasStatusText,
                Text::new("状态: 未连接"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));

            // 上传进度
            col.spawn((
                NasProgressText,
                Text::new(" "),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
        });
}

/// 创建远程文件列表区域
fn spawn_file_list_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Text::new("远程文件"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 文件列表容器
            col.spawn((
                NasFileListContainer,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    min_height: Val::Px(50.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|list| {
                list.spawn((
                    Text::new("点击「浏览远程文件」查看 NAS 上的文件"),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });
        });
}

/// 清理 NAS 页面（隐藏而非销毁，保留资源状态）
pub fn cleanup_nas_ui(mut query: Query<&mut Node, With<NasRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

// ==================== 交互系统 ====================

/// NAS 输入框交互（聚焦/失焦）
pub fn nas_input_interaction(
    mut url_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (Changed<Interaction>, With<NasServerUrlInput>),
    >,
    mut user_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (
            Changed<Interaction>,
            With<NasUsernameInput>,
            Without<NasServerUrlInput>,
        ),
    >,
    mut pass_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (
            Changed<Interaction>,
            With<NasPasswordInput>,
            Without<NasServerUrlInput>,
            Without<NasUsernameInput>,
        ),
    >,
    mut path_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextInput,
        ),
        (
            Changed<Interaction>,
            With<NasRemotePathInput>,
            Without<NasServerUrlInput>,
            Without<NasUsernameInput>,
            Without<NasPasswordInput>,
        ),
    >,
) {
    for (interaction, mut bg, mut border, mut input) in url_query.iter_mut() {
        handle_input_interaction(*interaction, &mut bg, &mut border, &mut input);
    }
    for (interaction, mut bg, mut border, mut input) in user_query.iter_mut() {
        handle_input_interaction(*interaction, &mut bg, &mut border, &mut input);
    }
    for (interaction, mut bg, mut border, mut input) in pass_query.iter_mut() {
        handle_input_interaction(*interaction, &mut bg, &mut border, &mut input);
    }
    for (interaction, mut bg, mut border, mut input) in path_query.iter_mut() {
        handle_input_interaction(*interaction, &mut bg, &mut border, &mut input);
    }
}

/// 通用输入框交互处理
fn handle_input_interaction(
    interaction: Interaction,
    bg: &mut BackgroundColor,
    border: &mut BorderColor,
    input: &mut TextInput,
) {
    match interaction {
        Interaction::Pressed => {
            input.focused = true;
            *bg = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            *border = BorderColor::all(AppColors::PRIMARY);
        }
        Interaction::Hovered => {
            *bg = BackgroundColor(Color::srgb(0.14, 0.14, 0.18));
            if !input.focused {
                *border = BorderColor::all(Color::srgb(0.4, 0.4, 0.5));
            }
        }
        Interaction::None => {
            *bg = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
            if !input.focused {
                *border = BorderColor::all(AppColors::BORDER);
            }
        }
    }
}

/// 同步输入框值到 NasInputState 资源
pub fn sync_nas_input_values(
    url_query: Query<&TextInput, With<NasServerUrlInput>>,
    user_query: Query<&TextInput, (With<NasUsernameInput>, Without<NasServerUrlInput>)>,
    pass_query: Query<
        &TextInput,
        (
            With<NasPasswordInput>,
            Without<NasServerUrlInput>,
            Without<NasUsernameInput>,
        ),
    >,
    path_query: Query<
        &TextInput,
        (
            With<NasRemotePathInput>,
            Without<NasServerUrlInput>,
            Without<NasUsernameInput>,
            Without<NasPasswordInput>,
        ),
    >,
    mut input_state: ResMut<NasInputState>,
) {
    if let Ok(input) = url_query.single()
        && input_state.server_url != input.value
    {
        input_state.server_url = input.value.clone();
    }
    if let Ok(input) = user_query.single()
        && input_state.username != input.value
    {
        input_state.username = input.value.clone();
    }
    if let Ok(input) = pass_query.single()
        && input_state.password != input.value
    {
        input_state.password = input.value.clone();
    }
    if let Ok(input) = path_query.single()
        && input_state.remote_path != input.value
    {
        input_state.remote_path = input.value.clone();
    }
}

/// NAS 启用复选框交互
pub fn nas_enabled_checkbox_interaction(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<NasEnabledCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut input_state: ResMut<NasInputState>,
) {
    for (interaction, mut bg, children) in query.iter_mut() {
        if *interaction == Interaction::Pressed {
            input_state.enabled = !input_state.enabled;
            let icon = if input_state.enabled { ICON_CHECK } else { " " };
            let color = if input_state.enabled {
                AppColors::PRIMARY
            } else {
                Color::srgb(0.2, 0.2, 0.25)
            };
            *bg = BackgroundColor(color);

            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = icon.to_string();
                }
            }
        }
    }
}

/// 自动保存 NAS 设置（输入值变化时自动保存到配置文件）
pub fn auto_save_nas_settings(input_state: Res<NasInputState>, mut initialized: Local<bool>) {
    if !input_state.is_changed() {
        return;
    }
    // 跳过初始化帧
    if !*initialized {
        *initialized = true;
        return;
    }

    let mut settings = AppSettings::global().write();
    settings.nas.server_url = input_state.server_url.clone();
    settings.nas.username = input_state.username.clone();
    settings.nas.password = input_state.password.clone();
    settings.nas.remote_path = input_state.remote_path.clone();
    settings.nas.enabled = input_state.enabled;
    if let Err(e) = settings.save() {
        tracing::error!("保存 NAS 设置失败: {}", e);
    } else {
        tracing::debug!("NAS 设置已自动保存");
    }
}

/// 测试连接按钮交互
pub fn nas_test_connection_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<NasTestConnectionButton>)>,
    mut test_messages: MessageWriter<NasTestConnectionRequest>,
    nas_state: Res<NasState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed && !nas_state.is_testing {
            test_messages.write(NasTestConnectionRequest);
        }
    }
}

/// 上传按钮交互
pub fn nas_upload_button_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<NasUploadButton>)>,
    mut upload_messages: MessageWriter<NasUploadRequest>,
    nas_state: Res<NasState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed && !nas_state.is_uploading {
            upload_messages.write(NasUploadRequest);
        }
    }
}

/// 浏览按钮交互
pub fn nas_browse_button_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<NasBrowseButton>)>,
    mut browse_messages: MessageWriter<NasBrowseRequest>,
    nas_state: Res<NasState>,
    input_state: Res<NasInputState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed && !nas_state.is_browsing {
            let path = if input_state.remote_path.is_empty() {
                "/".to_string()
            } else {
                input_state.remote_path.clone()
            };
            browse_messages.write(NasBrowseRequest { path });
        }
    }
}

/// 处理测试连接响应
pub fn handle_nas_test_response(
    mut nas_state: ResMut<NasState>,
    mut events: MessageReader<NasTestConnectionResponse>,
) {
    for event in events.read() {
        nas_state.is_testing = false;
        nas_state.is_connected = event.success;
        nas_state.test_message = Some(event.message.clone());
        nas_state.test_success = event.success;
        nas_state.needs_rebuild = true;
    }
}

/// 处理上传进度
pub fn handle_nas_upload_progress(
    mut nas_state: ResMut<NasState>,
    mut events: MessageReader<NasUploadProgressEvent>,
) {
    for event in events.read() {
        // 更新或新增上传任务
        if let Some(task) = nas_state
            .upload_tasks
            .iter_mut()
            .find(|t| t.comic_title == event.comic_title)
        {
            task.uploaded_files = event.uploaded_files;
            task.total_files = event.total_files;
            task.status = NasUploadStatus::Uploading;
        } else {
            nas_state.upload_tasks.push(NasUploadTask {
                comic_title: event.comic_title.clone(),
                local_path: String::new(),
                remote_path: String::new(),
                status: NasUploadStatus::Uploading,
                uploaded_files: event.uploaded_files,
                total_files: event.total_files,
            });
        }
        nas_state.needs_rebuild = true;
    }
}

/// 处理上传完成
pub fn handle_nas_upload_completed(
    mut nas_state: ResMut<NasState>,
    mut events: MessageReader<NasUploadCompletedEvent>,
) {
    for event in events.read() {
        nas_state.is_uploading = false;
        nas_state.success = Some(event.message.clone());
        nas_state.error = None;
        // 标记所有任务为完成
        for task in &mut nas_state.upload_tasks {
            task.status = NasUploadStatus::Completed;
        }
        nas_state.needs_rebuild = true;
    }
}

/// 处理上传失败
pub fn handle_nas_upload_failed(
    mut nas_state: ResMut<NasState>,
    mut events: MessageReader<NasUploadFailedEvent>,
) {
    for event in events.read() {
        nas_state.is_uploading = false;
        nas_state.error = Some(event.error.clone());
        nas_state.success = None;
        nas_state.needs_rebuild = true;
    }
}

/// 处理浏览响应
pub fn handle_nas_browse_response(
    mut nas_state: ResMut<NasState>,
    mut events: MessageReader<NasBrowseResponse>,
) {
    for event in events.read() {
        nas_state.is_browsing = false;
        nas_state.remote_entries = event.entries.clone();
        nas_state.browse_path = event.path.clone();
        nas_state.needs_rebuild = true;
    }
}

/// 处理浏览失败
pub fn handle_nas_browse_failed(
    mut nas_state: ResMut<NasState>,
    mut events: MessageReader<NasBrowseFailedEvent>,
) {
    for event in events.read() {
        nas_state.is_browsing = false;
        nas_state.error = Some(event.error.clone());
        nas_state.needs_rebuild = true;
    }
}

/// 更新 NAS 状态文本
pub fn refresh_nas_status_ui(
    nas_state: Res<NasState>,
    mut status_text_query: Query<&mut Text, With<NasStatusText>>,
    mut progress_text_query: Query<
        (&mut Text, &mut Node),
        (With<NasProgressText>, Without<NasStatusText>),
    >,
    mut file_list_query: Query<
        (Entity, &mut Node, Option<&Children>),
        (
            With<NasFileListContainer>,
            Without<NasProgressText>,
            Without<NasStatusText>,
        ),
    >,
    mut commands: Commands,
) {
    if !nas_state.is_changed() || !nas_state.needs_rebuild {
        return;
    }

    let font: Handle<Font> = get_font();

    // 更新连接状态文本
    if let Ok(mut text) = status_text_query.single_mut() {
        if nas_state.is_testing {
            **text = "状态: 正在测试连接...".to_string();
        } else if let Some(ref msg) = nas_state.test_message {
            if nas_state.test_success {
                **text = format!("状态: {} {}", ICON_CHECK, msg);
            } else {
                **text = format!("状态: {} {}", ICON_CLOSE, msg);
            }
        } else if nas_state.is_connected {
            **text = format!("状态: {} 已连接", ICON_CHECK);
        } else {
            **text = "状态: 未连接".to_string();
        }

        // 显示错误/成功信息
        if let Some(ref error) = nas_state.error {
            **text = format!("状态: {} {}", ICON_CLOSE, error);
        }
        if let Some(ref success) = nas_state.success {
            **text = format!("状态: {} {}", ICON_CHECK, success);
        }
    }

    // 更新上传进度
    if let Ok((mut text, mut node)) = progress_text_query.single_mut() {
        if nas_state.is_uploading && !nas_state.upload_tasks.is_empty() {
            let total_uploaded: u32 = nas_state
                .upload_tasks
                .iter()
                .map(|t| t.uploaded_files)
                .sum();
            let total_files: u32 = nas_state.upload_tasks.iter().map(|t| t.total_files).sum();
            **text = format!("上传进度: {}/{} 文件", total_uploaded, total_files);
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }

    // 更新远程文件列表
    if let Ok((entity, mut _node, children)) = file_list_query.single_mut() {
        // 清除旧的子元素
        if let Some(children) = children {
            let child_entities: Vec<Entity> = children.iter().collect();
            for child in child_entities {
                commands.entity(child).despawn();
            }
        }

        if nas_state.is_browsing {
            commands.entity(entity).with_children(|list| {
                list.spawn((
                    Text::new("正在加载..."),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });
        } else if nas_state.remote_entries.is_empty() {
            commands.entity(entity).with_children(|list| {
                list.spawn((
                    Text::new("点击「浏览远程文件」查看 NAS 上的文件"),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            });
        } else {
            let entries = nas_state.remote_entries.clone();
            commands.entity(entity).with_children(|list| {
                // 显示当前路径
                if !nas_state.browse_path.is_empty() {
                    list.spawn((
                        Text::new(format!("路径: {}", nas_state.browse_path)),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::PRIMARY),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                }

                for entry in &entries {
                    let icon = if entry.is_dir { ICON_FOLDER } else { ICON_BOOK };
                    let size_str = if entry.is_dir {
                        String::new()
                    } else {
                        format_file_size(entry.size)
                    };
                    let display = if size_str.is_empty() {
                        format!("{} {}", icon, entry.name)
                    } else {
                        format!("{} {} ({})", icon, entry.name, size_str)
                    };

                    list.spawn((
                        NasFileEntryItem {
                            path: entry.path.clone(),
                            is_dir: entry.is_dir,
                        },
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::new(
                                Val::Px(8.0),
                                Val::Px(8.0),
                                Val::Px(4.0),
                                Val::Px(4.0),
                            ),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|item| {
                        item.spawn((
                            Text::new(display),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(if entry.is_dir {
                                AppColors::PRIMARY
                            } else {
                                AppColors::TEXT
                            }),
                        ));
                    });
                }
            });
        }
    }
}

/// 格式化文件大小
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// NAS 滚动处理
pub fn handle_nas_scroll(
    _scroll_query: Query<(&mut ScrollPosition, Option<&ContentSizeInfo>), With<NasScrollContainer>>,
    mut _mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
) {
    // Bevy 内置 overflow: scroll_y() 自动处理滚动
}

/// NAS 内容尺寸更新
pub fn update_nas_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<NasScrollContainer>,
    >,
    children_query: Query<&ComputedNode>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
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

// ==================== WebDAV 异步操作 ====================

/// 处理测试连接请求（异步，通过 TokioTasksRuntime）
pub fn handle_nas_test_connection(
    mut events: MessageReader<NasTestConnectionRequest>,
    mut nas_state: ResMut<NasState>,
    input_state: Res<NasInputState>,
    runtime: Res<TokioTasksRuntime>,
) {
    for _ in events.read() {
        if nas_state.is_testing {
            continue;
        }
        nas_state.is_testing = true;
        nas_state.test_message = Some("正在测试连接...".to_string());
        nas_state.needs_rebuild = true;

        let server_url = input_state.server_url.clone();
        let username = input_state.username.clone();
        let password = input_state.password.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            let result = webdav_propfind(&server_url, &username, &password, "/", 0).await;

            let (success, message) = match result {
                Ok(_) => (true, "连接成功".to_string()),
                Err(e) => (false, format!("连接失败: {}", e)),
            };

            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .write_message(NasTestConnectionResponse { success, message });
            })
            .await;
        });
    }
}

/// 处理上传请求（扫描本地下载目录，逐个文件上传到 NAS）
pub fn handle_nas_upload_request(
    mut events: MessageReader<NasUploadRequest>,
    mut nas_state: ResMut<NasState>,
    input_state: Res<NasInputState>,
    download_state: Res<DownloadManagerState>,
    runtime: Res<TokioTasksRuntime>,
) {
    for _ in events.read() {
        if nas_state.is_uploading {
            continue;
        }
        nas_state.is_uploading = true;
        nas_state.upload_tasks.clear();
        nas_state.error = None;
        nas_state.success = None;
        nas_state.needs_rebuild = true;

        let server_url = input_state.server_url.clone();
        let username = input_state.username.clone();
        let password = input_state.password.clone();
        let remote_path = if input_state.remote_path.is_empty() {
            "/picacg/".to_string()
        } else {
            let mut p = input_state.remote_path.clone();
            if !p.ends_with('/') {
                p.push('/');
            }
            p
        };

        // 收集已完成的下载任务的保存路径
        let completed_tasks: Vec<(String, String)> = download_state
            .completed_tasks()
            .iter()
            .map(|t| (t.meta.comic_title.clone(), t.meta.save_path.clone()))
            .filter(|(_, path)| !path.is_empty())
            .collect();

        if completed_tasks.is_empty() {
            nas_state.is_uploading = false;
            nas_state.error = Some("没有已完成的下载任务".to_string());
            nas_state.needs_rebuild = true;
            continue;
        }

        runtime.spawn_background_task(move |mut ctx| async move {
            let mut total_uploaded = 0u32;
            let mut total_errors = 0u32;

            for (comic_title, save_path) in &completed_tasks {
                // 扫描本地目录中的文件
                let local_path = std::path::Path::new(save_path);
                if !local_path.exists() {
                    tracing::warn!("下载目录不存在: {}", save_path);
                    total_errors += 1;
                    continue;
                }

                let files = match scan_local_files(local_path).await {
                    Ok(files) => files,
                    Err(e) => {
                        tracing::error!("扫描目录失败: {} - {}", save_path, e);
                        total_errors += 1;
                        continue;
                    }
                };

                let total_files = files.len() as u32;
                let comic_remote_path =
                    format!("{}{}/", remote_path, sanitize_remote_name(comic_title));

                // 创建远程目录
                if let Err(e) =
                    webdav_mkcol(&server_url, &username, &password, &comic_remote_path).await
                {
                    tracing::warn!("创建远程目录失败（可能已存在）: {}", e);
                }

                let mut uploaded_count = 0u32;
                for file_path in &files {
                    let file_name = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let remote_file_path = format!("{}{}", comic_remote_path, file_name);

                    match webdav_put_file(
                        &server_url,
                        &username,
                        &password,
                        &remote_file_path,
                        file_path,
                    )
                    .await
                    {
                        Ok(()) => {
                            uploaded_count += 1;
                            total_uploaded += 1;
                        }
                        Err(e) => {
                            tracing::error!("上传文件失败: {} - {}", file_name, e);
                            total_errors += 1;
                        }
                    }

                    // 发送进度更新
                    let title = comic_title.clone();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(NasUploadProgressEvent {
                            comic_title: title,
                            uploaded_files: uploaded_count,
                            total_files,
                        });
                    })
                    .await;
                }
            }

            let message = format!(
                "上传完成: {} 个文件成功, {} 个错误",
                total_uploaded, total_errors
            );

            if total_errors > 0 && total_uploaded == 0 {
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world
                        .write_message(NasUploadFailedEvent { error: message });
                })
                .await;
            } else {
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.write_message(NasUploadCompletedEvent { message });
                })
                .await;
            }
        });
    }
}

/// 处理浏览请求
pub fn handle_nas_browse_request(
    mut events: MessageReader<NasBrowseRequest>,
    mut nas_state: ResMut<NasState>,
    input_state: Res<NasInputState>,
    runtime: Res<TokioTasksRuntime>,
) {
    for event in events.read() {
        if nas_state.is_browsing {
            continue;
        }
        nas_state.is_browsing = true;
        nas_state.needs_rebuild = true;

        let server_url = input_state.server_url.clone();
        let username = input_state.username.clone();
        let password = input_state.password.clone();
        let path = event.path.clone();

        runtime.spawn_background_task(move |mut ctx| async move {
            match webdav_propfind(&server_url, &username, &password, &path, 1).await {
                Ok(entries) => {
                    let remote_entries: Vec<NasRemoteEntry> = entries
                        .into_iter()
                        .map(|(name, href, is_dir, size)| NasRemoteEntry {
                            name,
                            path: href,
                            is_dir,
                            size,
                        })
                        .collect();

                    let browse_path = path.clone();
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(NasBrowseResponse {
                            entries: remote_entries,
                            path: browse_path,
                        });
                    })
                    .await;
                }
                Err(e) => {
                    ctx.run_on_main_thread(move |ctx| {
                        ctx.world.write_message(NasBrowseFailedEvent {
                            error: format!("浏览失败: {}", e),
                        });
                    })
                    .await;
                }
            }
        });
    }
}

// ==================== WebDAV HTTP 操作 ====================

/// 发送 PROPFIND 请求（列出目录内容）
///
/// 返回 (名称, href, 是否目录, 文件大小) 的列表
async fn webdav_propfind(
    server_url: &str,
    username: &str,
    password: &str,
    path: &str,
    depth: u8,
) -> Result<Vec<(String, String, bool, u64)>, String> {
    let url = build_webdav_url(server_url, path);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let propfind_body = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:displayname/>
    <D:resourcetype/>
    <D:getcontentlength/>
  </D:prop>
</D:propfind>"#;

    let response = client
        .request(
            reqwest::Method::from_bytes(b"PROPFIND")
                .map_err(|e| format!("无效的 HTTP 方法: {}", e))?,
            &url,
        )
        .header("Depth", depth.to_string())
        .header("Content-Type", "application/xml")
        .basic_auth(username, Some(password))
        .body(propfind_body.to_string())
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() && response.status().as_u16() != 207 {
        return Err(format!(
            "服务器返回错误: {} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ));
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 简易 XML 解析 WebDAV PROPFIND 响应
    parse_propfind_response(&body)
}

/// 发送 MKCOL 请求（创建远程目录）
async fn webdav_mkcol(
    server_url: &str,
    username: &str,
    password: &str,
    path: &str,
) -> Result<(), String> {
    let url = build_webdav_url(server_url, path);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .request(
            reqwest::Method::from_bytes(b"MKCOL")
                .map_err(|e| format!("无效的 HTTP 方法: {}", e))?,
            &url,
        )
        .basic_auth(username, Some(password))
        .send()
        .await
        .map_err(|e| format!("创建目录请求失败: {}", e))?;

    let status = response.status().as_u16();
    // 201 = 创建成功, 405 = 已存在, 301 = 重定向（部分 NAS 行为）
    if status == 201 || status == 405 || status == 301 {
        Ok(())
    } else {
        Err(format!("创建目录失败: HTTP {}", status))
    }
}

/// 发送 PUT 请求（上传文件）
async fn webdav_put_file(
    server_url: &str,
    username: &str,
    password: &str,
    remote_path: &str,
    local_path: &std::path::Path,
) -> Result<(), String> {
    let url = build_webdav_url(server_url, remote_path);

    let file_content = tokio::fs::read(local_path)
        .await
        .map_err(|e| format!("读取本地文件失败: {}", e))?;

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .put(&url)
        .basic_auth(username, Some(password))
        .body(file_content)
        .send()
        .await
        .map_err(|e| format!("上传文件请求失败: {}", e))?;

    let status = response.status().as_u16();
    // 201 = 创建, 204 = 覆盖成功
    if status == 201 || status == 204 || status == 200 {
        Ok(())
    } else {
        Err(format!("上传文件失败: HTTP {}", status))
    }
}

/// 构建 WebDAV 完整 URL
fn build_webdav_url(server_url: &str, path: &str) -> String {
    let base = server_url.trim_end_matches('/');
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    format!("{}{}", base, path)
}

/// 清理远程路径名称（移除特殊字符）
fn sanitize_remote_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// 扫描本地目录中的文件（递归）
async fn scan_local_files(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files = Vec::new();

    let mut entries = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| format!("读取目录失败: {}", e))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("读取目录条目失败: {}", e))?
    {
        let path = entry.path();
        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|e| format!("读取文件元数据失败: {}", e))?;

        if metadata.is_file() {
            files.push(path);
        } else if metadata.is_dir() {
            // 递归扫描子目录
            match Box::pin(scan_local_files(&path)).await {
                Ok(sub_files) => files.extend(sub_files),
                Err(e) => tracing::warn!("扫描子目录失败: {} - {}", path.display(), e),
            }
        }
    }

    Ok(files)
}

/// 简易解析 WebDAV PROPFIND XML 响应
///
/// 返回 (displayname, href, is_collection, content_length) 列表。
/// 不引入 XML 解析库，使用字符串查找提取关键信息。
fn parse_propfind_response(xml: &str) -> Result<Vec<(String, String, bool, u64)>, String> {
    let mut entries = Vec::new();

    // 找到所有 response 块
    let lower_xml = xml.to_lowercase();
    let mut search_start = 0;

    loop {
        // 查找下一个 response 开始标签
        let start_pos = find_any_tag(&lower_xml, search_start, &["<d:response", "<response"]);
        if start_pos.is_none() {
            break;
        }
        let start = start_pos.unwrap();

        // 查找对应的结束标签
        let end_pos = find_any_tag(&lower_xml, start + 1, &["</d:response>", "</response>"]);
        if end_pos.is_none() {
            break;
        }
        let end = end_pos.unwrap();

        let block = &xml[start..end + 20.min(xml.len() - end)];

        // 提取 href
        let href = extract_tag_content(block, "href").unwrap_or_default();

        // 提取 displayname
        let displayname = extract_tag_content(block, "displayname").unwrap_or_default();

        // 判断是否为目录（<D:collection/> 或 <d:collection/>）
        let block_lower = block.to_lowercase();
        let is_dir = block_lower.contains("collection");

        // 提取文件大小
        let size_str = extract_tag_content(block, "getcontentlength").unwrap_or_default();
        let size: u64 = size_str.parse().unwrap_or(0);

        // 使用 href 的最后一段作为名称（如果 displayname 为空）
        let name = if displayname.is_empty() {
            href.trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or(&href)
                .to_string()
        } else {
            displayname
        };

        // URL 解码名称
        let name = urlencoding::decode(&name)
            .map(|s| s.into_owned())
            .unwrap_or(name);

        if !name.is_empty() {
            entries.push((name, href, is_dir, size));
        }

        search_start = end + 1;
    }

    Ok(entries)
}

/// 在字符串中查找任意一个标签变体
fn find_any_tag(s: &str, from: usize, tags: &[&str]) -> Option<usize> {
    tags.iter()
        .filter_map(|tag| s[from..].find(tag).map(|pos| from + pos))
        .min()
}

/// 从 XML 块中提取标签内容（忽略命名空间前缀）
fn extract_tag_content(block: &str, tag_name: &str) -> Option<String> {
    let lower = block.to_lowercase();
    let tag_lower = tag_name.to_lowercase();

    // 尝试匹配 <D:tag>, <d:tag>, <tag> 等变体
    let patterns = [
        format!("<d:{}>", tag_lower),
        format!("<{}>", tag_lower),
        format!("<d:{} ", tag_lower), // 带属性
        format!("<{} ", tag_lower),
    ];

    for pattern in &patterns {
        if let Some(start_idx) = lower.find(pattern.as_str()) {
            // 找到标签开始后的 > 位置
            let content_start = block[start_idx..].find('>')? + start_idx + 1;
            // 找到对应的结束标签
            let end_patterns = [format!("</d:{}>", tag_lower), format!("</{}>", tag_lower)];
            for end_pattern in &end_patterns {
                if let Some(end_idx) = lower[content_start..].find(end_pattern.as_str()) {
                    let content = &block[content_start..content_start + end_idx];
                    return Some(content.trim().to_string());
                }
            }
        }
    }

    None
}
