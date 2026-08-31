//! NAS 远程存储系统
//!
//! 支持通过 WebDAV 协议将本地下载目录上传到 NAS。
//! 使用 reqwest 直接发送 WebDAV 请求（PROPFIND, GET, PUT,
//! MKCOL），无需额外依赖。

use bevy::{prelude::*, ui::RelativeCursorPosition};
use picacg_config::AppSettings;

use crate::{
    components::ContentArea,
    events::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::{ScrollArea, scrollbar},
        widgets::ButtonStyle,
    },
    utils::{
        TokioTasksRuntime,
        icons::*,
        text_input::{TextInput, TextInputDisplay},
    },
};

// ==================== 组件定义 ====================

/// NAS 页面根节点
#[derive(Component, Default, Clone)]
pub struct NasRoot;

/// NAS 滚动容器
#[derive(Component, Default, Clone)]
pub struct NasScrollContainer;

/// NAS 输入框统一标记
///
/// 只用于悬停底色；聚焦、边框与 IME 由通用 `text_input` 系统按 `InputFocus`
/// 接管。
#[derive(Component, Default, Clone)]
pub struct NasInputField;

/// 服务器地址输入框
#[derive(Component, Default, Clone)]
pub struct NasServerUrlInput;

/// 用户名输入框
#[derive(Component, Default, Clone)]
pub struct NasUsernameInput;

/// 密码输入框
#[derive(Component, Default, Clone)]
pub struct NasPasswordInput;

/// 远程目录输入框
#[derive(Component, Default, Clone)]
pub struct NasRemotePathInput;

/// 测试连接按钮
#[derive(Component, Default, Clone)]
pub struct NasTestConnectionButton;

/// 上传下载目录按钮
#[derive(Component, Default, Clone)]
pub struct NasUploadButton;

/// 浏览远程文件按钮
#[derive(Component, Default, Clone)]
pub struct NasBrowseButton;

/// 启用 NAS 复选框
#[derive(Component, Default, Clone)]
pub struct NasEnabledCheckbox;

/// 连接状态文本
#[derive(Component, Default, Clone)]
pub struct NasStatusText;

/// 上传进度文本
#[derive(Component, Default, Clone)]
pub struct NasProgressText;

/// 远程文件列表容器
#[derive(Component, Default, Clone)]
pub struct NasFileListContainer;

/// 远程文件条目（预留字段供后续目录导航使用）
#[derive(Component, Default, Clone)]
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

    let root = commands.spawn_scene(nas_page(&settings)).id();

    // 挂载到内容区域
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(root);
    }

    tracing::info!("NAS 页面 UI 已创建");
}

/// NAS 页面场景
fn nas_page(settings: &AppSettings) -> impl Scene + use<> {
    let title = format!("{} NAS 远程存储", ICON_NAS);

    bsn! {
        NasRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // ===== 标题栏 =====
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::bottom(Val::Px(1.0)),
                }
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        Text({title})
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(AppColors::TEXT)
                    )
                ]
            ),
            (
                // ===== 内容区域（可滚动）=====
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    overflow: Overflow::clip(),
                    position_type: PositionType::Relative,
                }
                Children [
                    (
                        // 滚动容器
                        #NasScroll
                        NasScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::scroll_y(),
                            padding: UiRect::all(Val::Px(25.0)),
                            row_gap: Val::Px(20.0),
                        }
                        ScrollArea
                        Children [
                            // --- 启用开关行 ---
                            enabled_row(settings.nas.enabled),
                            // --- 服务器地址 ---
                            input_row(
                                "服务器地址",
                                &settings.nas.server_url,
                                "http://192.168.1.100:5005/webdav",
                                NasServerUrlInput,
                            ),
                            // --- 用户名 ---
                            input_row("用户名", &settings.nas.username, "admin", NasUsernameInput),
                            // --- 密码 ---
                            password_row(&settings.nas.password),
                            // --- 远程目录 ---
                            input_row(
                                "远程目录",
                                &settings.nas.remote_path,
                                "/picacg/",
                                NasRemotePathInput,
                            ),
                            // --- 操作按钮行 ---
                            action_buttons(),
                            // --- 状态显示 ---
                            status_area(),
                            // --- 远程文件列表 ---
                            file_list_area(),
                            (
                                // --- 底部间距 ---
                                Node {
                                    height: Val::Px(30.0),
                                    min_height: Val::Px(30.0),
                                }
                            ),
                        ]
                    ),
                    // 滚动条
                    scrollbar(#NasScroll),
                ]
            ),
        ]
    }
}

/// 启用开关行场景
fn enabled_row(enabled: bool) -> impl Scene {
    // 复选框按钮：二态选中项走 Segment（未选 surface_sunken，选中钉 primary）
    let check_icon = if enabled { ICON_CHECK } else { " " };
    let style = ButtonStyle::segment(enabled);
    let bg = if enabled {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
        }
        Children [
            (
                Text("启用 NAS 远程存储")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                NasEnabledCheckbox
                Button
                template_value(style)
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor({bg})
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        Text({check_icon})
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(Color::WHITE)
                    )
                ]
            ),
        ]
    }
}

/// 通用输入行场景
///
/// `Unpin` 是 `template_value` 的 `Template` blanket impl 要求
/// （`Clone + Default + Unpin`），泛型参数不会自动带上。
fn input_row<C: Component + Default + Clone + Unpin>(
    label: &str,
    value: &str,
    placeholder: &str,
    marker: C,
) -> impl Scene + use<C> {
    let label = label.to_string();
    let text_input = TextInput::new(placeholder).with_value(value);

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

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
        }
        Children [
            (
                // 标签
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                template_value(marker)
                NasInputField
                template_value(text_input)
                Button
                RelativeCursorPosition
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(36.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    overflow: Overflow::clip(),
                }
                BackgroundColor(AppColors::SURFACE_SUNKEN)
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // TextInputDisplay 必须挂在子 Text 节点上：
                        // text_input_cursor_blink 是「遍历 TextInput 的 Children
                        // 找 TextInputDisplay」，挂在容器上则永远匹配不到，
                        // 键入内容不会显示。
                        TextInputDisplay
                        Text({display_text})
                        TextFont { font_size: FontSize::Px(13.0) }
                        TextColor({text_color})
                    )
                ]
            ),
        ]
    }
}

/// 密码输入行场景
fn password_row(value: &str) -> impl Scene + use<> {
    let placeholder = "密码";
    let display_text = if value.is_empty() {
        placeholder.to_string()
    } else {
        // 掩码按字符数而非字节数（中文密码按字节会多出 2 倍星号）
        "*".repeat(value.chars().count())
    };
    let text_color = if value.is_empty() {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };
    let text_input = TextInput::new(placeholder)
        .with_value(value)
        .with_password();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
        }
        Children [
            (
                Text("密码")
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                NasPasswordInput
                NasInputField
                template_value(text_input)
                Button
                RelativeCursorPosition
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(36.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    overflow: Overflow::clip(),
                }
                BackgroundColor(AppColors::SURFACE_SUNKEN)
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // 同 input_row：标记必须落在子 Text 上
                        TextInputDisplay
                        Text({display_text})
                        TextFont { font_size: FontSize::Px(13.0) }
                        TextColor({text_color})
                    )
                ]
            ),
        ]
    }
}

/// 操作按钮行场景
fn action_buttons() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            margin: UiRect::top(Val::Px(5.0)),
        }
        Children [
            // 测试连接按钮
            action_button("测试连接", NasTestConnectionButton),
            // 上传下载目录按钮
            action_button("上传下载目录", NasUploadButton),
            // 浏览远程文件按钮
            action_button("浏览远程文件", NasBrowseButton),
        ]
    }
}

/// 通用操作按钮场景
fn action_button<C: Component + Default + Clone + Unpin>(
    label: &str,
    marker: C,
) -> impl Scene + use<C> {
    let label = label.to_string();

    bsn! {
        template_value(marker)
        Button
        template_value(ButtonStyle::primary())
        Node {
            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        BackgroundColor(AppColors::PRIMARY)
        template_value(BorderColor::all(AppColors::PRIMARY))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(Color::WHITE)
            )
        ]
    }
}

/// 状态显示区域场景
fn status_area() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
        }
        Children [
            (
                // 连接状态
                NasStatusText
                Text("状态: 未连接")
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 上传进度
                NasProgressText
                Text(" ")
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
                Node { display: Display::None }
            ),
        ]
    }
}

/// 远程文件列表区域场景
fn file_list_area() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
        }
        Children [
            (
                Text("远程文件")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 文件列表容器
                NasFileListContainer
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    min_height: Val::Px(50.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                }
                BackgroundColor(Color::srgb(0.1, 0.1, 0.14))
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    file_list_hint("点击「浏览远程文件」查看 NAS 上的文件"),
                ]
            ),
        ]
    }
}

/// 文件列表占位提示场景（加载中 / 未浏览）
fn file_list_hint(hint: &str) -> impl Scene + use<> {
    let hint = hint.to_string();

    bsn! {
        Text({hint})
        TextFont { font_size: FontSize::Px(12.0) }
        TextColor(AppColors::TEXT_SECONDARY)
    }
}

/// 当前浏览路径场景
fn browse_path_label(path: &str) -> impl Scene + use<> {
    let label = format!("路径: {}", path);

    bsn! {
        Text({label})
        TextFont { font_size: FontSize::Px(12.0) }
        TextColor(AppColors::PRIMARY)
        Node { margin: UiRect::bottom(Val::Px(8.0)) }
    }
}

/// 单个远程文件条目场景
fn file_entry_item(entry: &NasRemoteEntry) -> impl Scene + use<> {
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
    let text_color = if entry.is_dir {
        AppColors::PRIMARY
    } else {
        AppColors::TEXT
    };
    let path = entry.path.clone();
    let is_dir = entry.is_dir;

    bsn! {
        NasFileEntryItem { path: {path}, is_dir: {is_dir} }
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::NONE)
        Children [
            (
                Text({display})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor({text_color})
            )
        ]
    }
}

/// 清理 NAS 页面（隐藏而非销毁，保留资源状态）
pub fn cleanup_nas_ui(mut query: Query<&mut Node, With<NasRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

// ==================== 交互系统 ====================

/// NAS 输入框悬停底色
///
/// 聚焦、边框与 IME 由通用 `text_input` 系统按 `InputFocus` 统一处理，
/// 此处只保留鼠标悬停的底色反馈。
pub fn nas_input_interaction(
    mut inputs: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<NasInputField>),
    >,
) {
    for (interaction, mut bg) in inputs.iter_mut() {
        let target = match *interaction {
            Interaction::Pressed | Interaction::Hovered => AppColors::SURFACE,
            Interaction::None => AppColors::SURFACE_SUNKEN,
        };
        if bg.0 != target {
            bg.0 = target;
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
        (&Interaction, &mut ButtonStyle, &Children),
        (Changed<Interaction>, With<NasEnabledCheckbox>),
    >,
    mut text_query: Query<&mut Text>,
    mut input_state: ResMut<NasInputState>,
) {
    for (interaction, mut style, children) in query.iter_mut() {
        if *interaction == Interaction::Pressed {
            input_state.enabled = !input_state.enabled;
            let icon = if input_state.enabled { ICON_CHECK } else { " " };
            // 配色交给全局 apply_button_interaction，此处只翻选中态
            if style.selected != input_state.enabled {
                style.selected = input_state.enabled;
            }

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
        } else {
            nas_state.upload_tasks.push(NasUploadTask {
                comic_title: event.comic_title.clone(),
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
    mut nas_state: ResMut<NasState>,
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
    nas_state.needs_rebuild = false;

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
            commands
                .spawn_scene(file_list_hint("正在加载..."))
                .insert(ChildOf(entity));
        } else if nas_state.remote_entries.is_empty() {
            commands
                .spawn_scene(file_list_hint("点击「浏览远程文件」查看 NAS 上的文件"))
                .insert(ChildOf(entity));
        } else {
            // 显示当前路径
            if !nas_state.browse_path.is_empty() {
                commands
                    .spawn_scene(browse_path_label(&nas_state.browse_path))
                    .insert(ChildOf(entity));
            }

            for entry in &nas_state.remote_entries {
                commands
                    .spawn_scene(file_entry_item(entry))
                    .insert(ChildOf(entity));
            }
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
