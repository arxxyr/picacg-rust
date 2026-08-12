//! 图片格式转换工具系统
//!
//! 支持批量将目录中的图片文件转换为指定格式（PNG/JPEG/WebP/BMP）

use bevy::prelude::*;

use crate::{
    components::*,
    resources::*,
    systems::{
        login::AppColors,
        widgets::{ButtonStyle, ButtonVariant},
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// 图片转换页面根节点
#[derive(Component, Default, Clone)]
pub struct ImageConvertRoot;

/// 源目录输入框文本
#[derive(Component, Default, Clone)]
pub struct SourceDirText;

/// 选择目录按钮
#[derive(Component, Default, Clone)]
pub struct SelectSourceDirButton;

/// 目标格式按钮
#[derive(Component, Default, Clone)]
pub struct TargetFormatButton {
    pub format: TargetImageFormat,
}

/// 开始转换按钮
#[derive(Component, Default, Clone)]
pub struct StartConvertButton;

/// 进度文本
#[derive(Component, Default, Clone)]
pub struct ConvertProgressText;

/// 状态消息文本（错误/成功）
#[derive(Component, Default, Clone)]
pub struct ConvertStatusText;

// ==================== 颜色常量 ====================

/// 成功状态文本颜色（绿色）
const COLOR_SUCCESS: Color = Color::srgb(0.3, 0.9, 0.4);

// ==================== 支持的图片扩展名 ====================

/// 支持的图片文件扩展名
const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp"];

/// 判断文件路径是否为支持的图片格式
fn is_supported_image(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

// ==================== 系统函数 ====================

/// 创建图片转换页面 UI（如果已存在则只显示）
pub fn setup_image_convert_ui(
    mut commands: Commands,
    content_area_query: Query<Entity, With<ContentArea>>,
    convert_state: Res<ImageConvertState>,
    mut existing_query: Query<&mut Node, With<ImageConvertRoot>>,
) {
    // 如果 ImageConvertRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        return;
    }

    let content_area = content_area_query.single().ok();

    let root = commands
        .spawn_scene(image_convert_page(&convert_state))
        .id();

    // 挂载到内容区域
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(root);
    }

    tracing::info!("图片转换页面 UI 已创建");
}

// ==================== 场景函数 ====================

/// 图片转换页面场景
fn image_convert_page(state: &ImageConvertState) -> impl Scene + use<> {
    let title = format!("{} 图片格式转换", ICON_IMAGE_CONVERT);
    let source_dir = state.source_dir.clone();
    let target_format = state.target_format;
    let is_converting = state.is_converting;
    let progress = state.progress;
    let total = state.total;
    let error = state.error.clone();
    let success = state.success.clone();

    bsn! {
        ImageConvertRoot
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
                // ===== 内容区域 =====
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(25.0)),
                    row_gap: Val::Px(20.0),
                }
                Children [
                    // --- 源目录选择行 ---
                    source_dir_row(source_dir),
                    // --- 目标格式选择行 ---
                    target_format_row(target_format),
                    // --- 开始转换按钮 ---
                    start_button(is_converting),
                    // --- 进度显示 ---
                    progress_area(is_converting, progress, total, error, success),
                ]
            ),
        ]
    }
}

/// 源目录选择行场景
fn source_dir_row(source_dir: String) -> impl Scene {
    // 路径显示框
    let is_empty = source_dir.is_empty();
    let display_text = if is_empty {
        "请选择图片所在目录...".to_string()
    } else {
        source_dir
    };
    let text_color = if is_empty {
        AppColors::TEXT_SECONDARY
    } else {
        AppColors::TEXT
    };
    // 路径显示框内边距（左右 10 / 上下 8）
    let input_padding = UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(8.0), Val::Px(8.0));
    // 选择按钮内边距（左右 12 / 上下 8）
    let button_padding = UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(8.0), Val::Px(8.0));

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                // 标签
                Text("源目录")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 输入行：路径 + 选择按钮
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        // 路径显示框
                        Node {
                            flex_grow: 1.0,
                            padding: {input_padding},
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            overflow: Overflow::clip(),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                SourceDirText
                                Text({display_text})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(text_color)
                            )
                        ]
                    ),
                    (
                        // 选择目录按钮
                        SelectSourceDirButton
                        Button
                        template_value(ButtonStyle::primary())
                        Node {
                            padding: {button_padding},
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                        }
                        template_value(BorderColor::all(AppColors::PRIMARY))
                        BackgroundColor(AppColors::PRIMARY)
                        Children [
                            (
                                Text(ICON_FOLDER_OPEN)
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(Color::WHITE)
                            ),
                            (
                                Text("选择")
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(Color::WHITE)
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
}

/// 目标格式选择行场景
fn target_format_row(target_format: TargetImageFormat) -> impl Scene {
    // 四种格式按钮（当前格式高亮）
    let buttons: Vec<_> = TargetImageFormat::ALL
        .into_iter()
        .map(|format| format_button(format, format == target_format))
        .collect();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                // 标签
                Text("目标格式")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 格式按钮行
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                }
                Children [ {buttons} ]
            ),
        ]
    }
}

/// 单个目标格式按钮场景
fn format_button(format: TargetImageFormat, is_selected: bool) -> impl Scene {
    // 单选组统一走 segment：未选 surface_sunken，选中钉 primary
    let style = ButtonStyle::segment(is_selected);
    let bg_color = if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::SURFACE_SUNKEN
    };
    let border_color = if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    };
    let label = format.display_name();
    // 按钮内边距（左右 16 / 上下 8）
    let button_padding = UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0));

    bsn! {
        TargetFormatButton { format: {format} }
        Button
        template_value(style)
        Node {
            padding: {button_padding},
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template_value(BorderColor::all(border_color))
        BackgroundColor(bg_color)
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 开始转换按钮场景
fn start_button(is_converting: bool) -> impl Scene {
    // 转换中降级为次要配色表达禁用感，三态配色由 ButtonStyle 统一
    let (label, style, bg_color) = if is_converting {
        ("转换中...", ButtonStyle::secondary(), AppColors::SECONDARY)
    } else {
        ("开始转换", ButtonStyle::primary(), AppColors::PRIMARY)
    };
    // 按钮内边距（左右 24 / 上下 10）
    let button_padding = UiRect::new(Val::Px(24.0), Val::Px(24.0), Val::Px(10.0), Val::Px(10.0));

    bsn! {
        StartConvertButton
        Button
        template_value(style)
        Node {
            padding: {button_padding},
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            align_self: AlignSelf::FlexStart,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
        }
        template_value(BorderColor::all(bg_color))
        BackgroundColor(bg_color)
        Children [
            (
                Text(ICON_PLAY)
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::WHITE)
            ),
            (
                Text({label})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::WHITE)
            ),
        ]
    }
}

/// 进度显示区域场景
fn progress_area(
    is_converting: bool,
    progress: u32,
    total: u32,
    error: Option<String>,
    success: Option<String>,
) -> impl Scene {
    // 进度文本
    let progress_text = if is_converting {
        format!("进度: {}/{}", progress, total)
    } else if total > 0 {
        format!("已完成: {}/{}", progress, total)
    } else {
        String::new()
    };

    // 进度条（仅转换中显示）
    let progress_bar: Box<dyn SceneList> = if is_converting && total > 0 {
        let ratio = progress as f32 / total as f32;
        let fill_width = Val::Percent(ratio * 100.0);
        Box::new(bsn_list![(
            Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(500.0),
                height: Val::Px(6.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
            }
            Children [
                (
                    Node {
                        width: {fill_width},
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                    }
                    BackgroundColor(AppColors::PRIMARY)
                )
            ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 状态消息（错误/成功）
    let (status_text, status_color) = if let Some(err) = error {
        (err, AppColors::ERROR)
    } else if let Some(suc) = success {
        (suc, COLOR_SUCCESS)
    } else {
        (String::new(), AppColors::TEXT)
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
        }
        Children [
            (
                // 进度文本
                ConvertProgressText
                Text({progress_text})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            {progress_bar},
            (
                // 状态消息（错误/成功）
                ConvertStatusText
                Text({status_text})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(status_color)
            ),
        ]
    }
}

/// 清理图片转换页面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_image_convert_ui(mut query: Query<&mut Node, With<ImageConvertRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 选择源目录按钮交互（使用 rfd 异步对话框，不阻塞主线程）
///
/// 配色由 `apply_button_interaction` 统一处理。
pub fn select_source_dir_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SelectSourceDirButton>)>,
    mut picker: ResMut<ImageConvertPickerResult>,
) {
    for interaction in interaction_query.iter() {
        // 防止重复打开对话框
        if *interaction == Interaction::Pressed && picker.receiver.is_none() {
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
pub fn handle_source_dir_picker_result(
    mut picker: ResMut<ImageConvertPickerResult>,
    mut convert_state: ResMut<ImageConvertState>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<SourceDirText>>,
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
        convert_state.source_dir = path_str.clone();
        // 清除旧的状态信息
        convert_state.error = None;
        convert_state.success = None;
        convert_state.progress = 0;
        convert_state.total = 0;

        // 更新显示文本
        for (mut text, mut text_color) in text_query.iter_mut() {
            **text = path_str.clone();
            text_color.0 = AppColors::TEXT;
        }
        tracing::info!("已选择源目录: {}", path_str);
    }
}

/// 目标格式按钮交互（选中态由 `refresh_format_buttons` 统一刷新）
pub fn target_format_button_interaction(
    interaction_query: Query<(&Interaction, &TargetFormatButton), Changed<Interaction>>,
    mut convert_state: ResMut<ImageConvertState>,
) {
    for (interaction, fmt_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed && !convert_state.is_converting {
            convert_state.target_format = fmt_btn.format;
        }
    }
}

/// 刷新所有格式按钮的选中状态（当 target_format 变化时）
///
/// 背景色由 `apply_button_interaction` 接管，这里只写选中态与边框。
pub fn refresh_format_buttons(
    convert_state: Res<ImageConvertState>,
    mut button_query: Query<(&TargetFormatButton, &mut ButtonStyle, &mut BorderColor)>,
) {
    if !convert_state.is_changed() {
        return;
    }

    for (fmt_btn, mut style, mut border_color) in button_query.iter_mut() {
        let is_selected = fmt_btn.format == convert_state.target_format;
        if style.selected != is_selected {
            style.selected = is_selected;
        }
        *border_color = BorderColor::all(if is_selected {
            AppColors::PRIMARY
        } else {
            AppColors::BORDER
        });
    }
}

/// 按「是否可用」切换开始按钮的变体（转换中降级为次要配色表达禁用感）
fn set_start_button_enabled(style: &mut ButtonStyle, enabled: bool) {
    let variant = if enabled {
        ButtonVariant::Primary
    } else {
        ButtonVariant::Secondary
    };
    if style.variant != variant {
        style.variant = variant;
    }
}

/// 开始转换按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn start_convert_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle),
        (Changed<Interaction>, With<StartConvertButton>),
    >,
    mut convert_state: ResMut<ImageConvertState>,
    mut progress_result: ResMut<ImageConvertProgressResult>,
) {
    for (interaction, mut style) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if convert_state.is_converting {
            return;
        }
        if convert_state.source_dir.is_empty() {
            convert_state.error = Some("请先选择源目录".to_string());
            return;
        }

        let source_dir = convert_state.source_dir.clone();
        let target_format = convert_state.target_format;

        // 清除旧状态
        convert_state.error = None;
        convert_state.success = None;
        convert_state.progress = 0;
        convert_state.total = 0;
        convert_state.is_converting = true;

        set_start_button_enabled(&mut style, false);

        tracing::info!(
            "开始图片格式转换: dir={}, format={}",
            source_dir,
            target_format.display_name()
        );

        // 创建进度通道
        let (tx, rx) = std::sync::mpsc::channel();
        progress_result.receiver = Some(std::sync::Mutex::new(rx));

        // 在后台线程执行转换（IO 密集型操作）
        std::thread::spawn(move || {
            convert_images_in_dir(&source_dir, target_format, tx);
        });
    }
}

/// 轮询转换进度并更新 UI
pub fn refresh_convert_progress(
    mut progress_result: ResMut<ImageConvertProgressResult>,
    mut convert_state: ResMut<ImageConvertState>,
    mut progress_text_query: Query<(&mut Text, &mut TextColor), With<ConvertProgressText>>,
    mut status_text_query: Query<
        (&mut Text, &mut TextColor),
        (With<ConvertStatusText>, Without<ConvertProgressText>),
    >,
    mut start_button_query: Query<&mut ButtonStyle, With<StartConvertButton>>,
) {
    let Some(ref receiver) = progress_result.receiver else {
        return;
    };
    let Ok(receiver) = receiver.lock() else {
        return;
    };

    // 消费所有可用的消息（非阻塞）
    let mut last_msg = None;
    while let Ok(msg) = receiver.try_recv() {
        last_msg = Some(msg);
    }
    drop(receiver);

    let Some(msg) = last_msg else {
        return;
    };

    match msg {
        ImageConvertProgressMsg::Progress { done, total } => {
            convert_state.progress = done;
            convert_state.total = total;

            for (mut text, _) in progress_text_query.iter_mut() {
                **text = format!("进度: {}/{}", done, total);
            }
        }
        ImageConvertProgressMsg::Completed { done, total } => {
            convert_state.progress = done;
            convert_state.total = total;
            convert_state.is_converting = false;
            convert_state.success = Some(format!("转换完成！成功转换 {} 张图片", done));

            // 清除 receiver
            progress_result.receiver = None;
            for mut style in start_button_query.iter_mut() {
                set_start_button_enabled(&mut style, true);
            }

            for (mut text, _) in progress_text_query.iter_mut() {
                **text = format!("已完成: {}/{}", done, total);
            }
            for (mut text, mut text_color) in status_text_query.iter_mut() {
                **text = format!("转换完成！成功转换 {} 张图片", done);
                text_color.0 = COLOR_SUCCESS;
            }

            tracing::info!("图片转换完成: {}/{}", done, total);
        }
        ImageConvertProgressMsg::Error(err) => {
            convert_state.is_converting = false;
            convert_state.error = Some(err.clone());

            // 清除 receiver
            progress_result.receiver = None;
            for mut style in start_button_query.iter_mut() {
                set_start_button_enabled(&mut style, true);
            }

            for (mut text, mut text_color) in status_text_query.iter_mut() {
                **text = err.clone();
                text_color.0 = AppColors::ERROR;
            }
        }
    }
}

// ==================== 后台转换逻辑 ====================

/// 在指定目录中执行图片格式转换（后台线程运行）
fn convert_images_in_dir(
    source_dir: &str,
    target_format: TargetImageFormat,
    tx: std::sync::mpsc::Sender<ImageConvertProgressMsg>,
) {
    let dir_path = std::path::Path::new(source_dir);

    // 验证目录存在
    if !dir_path.is_dir() {
        let _ = tx.send(ImageConvertProgressMsg::Error(format!(
            "目录不存在: {}",
            source_dir
        )));
        return;
    }

    // 收集所有支持的图片文件
    let image_files: Vec<std::path::PathBuf> = match std::fs::read_dir(dir_path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && is_supported_image(path))
            .collect(),
        Err(e) => {
            let _ = tx.send(ImageConvertProgressMsg::Error(format!(
                "无法读取目录: {}",
                e
            )));
            return;
        }
    };

    let total = image_files.len() as u32;
    if total == 0 {
        let _ = tx.send(ImageConvertProgressMsg::Error(
            "目录中没有找到支持的图片文件（jpg/png/webp/bmp）".to_string(),
        ));
        return;
    }

    // 发送初始进度
    let _ = tx.send(ImageConvertProgressMsg::Progress { done: 0, total });

    let target_ext = target_format.extension();
    let mut done: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for file_path in &image_files {
        // 跳过已经是目标格式的文件
        let current_ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if current_ext == target_ext
            || (target_ext == "jpg" && current_ext == "jpeg")
            || (target_ext == "jpeg" && current_ext == "jpg")
        {
            done += 1;
            let _ = tx.send(ImageConvertProgressMsg::Progress { done, total });
            continue;
        }

        // 生成输出路径（同目录，替换扩展名）
        let output_path = file_path.with_extension(target_ext);

        // 打开并转换图片
        match image::open(file_path) {
            Ok(img) => {
                let save_result = match target_format {
                    TargetImageFormat::Png => {
                        img.save_with_format(&output_path, image::ImageFormat::Png)
                    }
                    TargetImageFormat::Jpeg => {
                        img.save_with_format(&output_path, image::ImageFormat::Jpeg)
                    }
                    TargetImageFormat::Webp => {
                        img.save_with_format(&output_path, image::ImageFormat::WebP)
                    }
                    TargetImageFormat::Bmp => {
                        img.save_with_format(&output_path, image::ImageFormat::Bmp)
                    }
                };

                match save_result {
                    Ok(()) => {
                        done += 1;
                    }
                    Err(e) => {
                        let file_name = file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("未知文件");
                        errors.push(format!("{}: {}", file_name, e));
                        done += 1;
                    }
                }
            }
            Err(e) => {
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件");
                errors.push(format!("{}: {}", file_name, e));
                done += 1;
            }
        }

        // 发送进度更新
        let _ = tx.send(ImageConvertProgressMsg::Progress { done, total });
    }

    // 发送完成消息
    if errors.is_empty() {
        let _ = tx.send(ImageConvertProgressMsg::Completed { done, total });
    } else {
        // 有部分文件出错，但整体完成
        let error_summary = format!(
            "转换完成，但有 {} 个文件出错:\n{}",
            errors.len(),
            errors
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
        // 先发送完成状态
        let _ = tx.send(ImageConvertProgressMsg::Completed { done, total });
        // 然后发送错误详情（如果接收端还在）
        tracing::warn!("{}", error_summary);
    }
}
