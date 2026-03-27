//! 图片格式转换工具系统
//!
//! 支持批量将目录中的图片文件转换为指定格式（PNG/JPEG/WebP/BMP）

use bevy::prelude::*;

use super::font_loader::get_font;
use crate::{components::*, resources::*, systems::login::AppColors, utils::icons::*};

// ==================== 组件定义 ====================

/// 图片转换页面根节点
#[derive(Component)]
pub struct ImageConvertRoot;

/// 源目录输入框文本
#[derive(Component)]
pub struct SourceDirText;

/// 选择目录按钮
#[derive(Component)]
pub struct SelectSourceDirButton;

/// 目标格式按钮
#[derive(Component)]
pub struct TargetFormatButton {
    pub format: TargetImageFormat,
}

/// 开始转换按钮
#[derive(Component)]
pub struct StartConvertButton;

/// 进度文本
#[derive(Component)]
pub struct ConvertProgressText;

/// 状态消息文本（错误/成功）
#[derive(Component)]
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

    let font: Handle<Font> = get_font();
    let content_area = content_area_query.single().ok();

    let root = commands
        .spawn((
            ImageConvertRoot,
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
                    Text::new(format!("{} 图片格式转换", ICON_IMAGE_CONVERT)),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));
            });

            // ===== 内容区域 =====
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(25.0)),
                row_gap: Val::Px(20.0),
                ..default()
            })
            .with_children(|content| {
                // --- 源目录选择行 ---
                spawn_source_dir_row(content, &font, &convert_state);

                // --- 目标格式选择行 ---
                spawn_target_format_row(content, &font, &convert_state);

                // --- 开始转换按钮 ---
                spawn_start_button(content, &font, &convert_state);

                // --- 进度显示 ---
                spawn_progress_area(content, &font, &convert_state);
            });
        })
        .id();

    // 挂载到内容区域
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(root);
    }

    tracing::info!("图片转换页面 UI 已创建");
}

/// 创建源目录选择行
fn spawn_source_dir_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    state: &ImageConvertState,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|section| {
            // 标签
            section.spawn((
                Text::new("源目录"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 输入行：路径 + 选择按钮
            section
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // 路径显示框
                    let display_text = if state.source_dir.is_empty() {
                        "请选择图片所在目录...".to_string()
                    } else {
                        state.source_dir.clone()
                    };
                    let text_color = if state.source_dir.is_empty() {
                        AppColors::TEXT_SECONDARY
                    } else {
                        AppColors::TEXT
                    };

                    row.spawn((
                        Node {
                            flex_grow: 1.0,
                            padding: UiRect::new(
                                Val::Px(10.0),
                                Val::Px(10.0),
                                Val::Px(8.0),
                                Val::Px(8.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(AppColors::SURFACE),
                        BorderColor::all(AppColors::BORDER),
                    ))
                    .with_children(|input_box| {
                        input_box.spawn((
                            SourceDirText,
                            Text::new(display_text),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(text_color),
                        ));
                    });

                    // 选择目录按钮
                    row.spawn((
                        SelectSourceDirButton,
                        Button,
                        Interaction::default(),
                        Node {
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(12.0),
                                Val::Px(8.0),
                                Val::Px(8.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                        BorderColor::all(AppColors::PRIMARY),
                        BackgroundColor(AppColors::PRIMARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(ICON_FOLDER_OPEN),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        btn.spawn((
                            Text::new("选择"),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        });
}

/// 创建目标格式选择行
fn spawn_target_format_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    state: &ImageConvertState,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|section| {
            // 标签
            section.spawn((
                Text::new("目标格式"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 格式按钮行
            section
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    for format in TargetImageFormat::ALL {
                        let is_selected = format == state.target_format;
                        let bg_color = if is_selected {
                            AppColors::PRIMARY
                        } else {
                            AppColors::SECONDARY
                        };
                        let text_color = if is_selected {
                            Color::WHITE
                        } else {
                            AppColors::TEXT
                        };

                        row.spawn((
                            TargetFormatButton { format },
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::new(
                                    Val::Px(16.0),
                                    Val::Px(16.0),
                                    Val::Px(8.0),
                                    Val::Px(8.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(if is_selected {
                                AppColors::PRIMARY
                            } else {
                                AppColors::BORDER
                            }),
                            BackgroundColor(bg_color),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(format.display_name()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(text_color),
                            ));
                        });
                    }
                });
        });
}

/// 创建开始转换按钮
fn spawn_start_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    state: &ImageConvertState,
) {
    let (label, bg_color) = if state.is_converting {
        ("转换中...", AppColors::TEXT_SECONDARY)
    } else {
        ("开始转换", AppColors::PRIMARY)
    };

    parent
        .spawn((
            StartConvertButton,
            Button,
            Interaction::default(),
            Node {
                padding: UiRect::new(Val::Px(24.0), Val::Px(24.0), Val::Px(10.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                align_self: AlignSelf::FlexStart,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
            BorderColor::all(bg_color),
            BackgroundColor(bg_color),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(ICON_PLAY),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 创建进度显示区域
fn spawn_progress_area(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    state: &ImageConvertState,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|area| {
            // 进度文本
            let progress_text = if state.is_converting {
                format!("进度: {}/{}", state.progress, state.total)
            } else if state.total > 0 {
                format!("已完成: {}/{}", state.progress, state.total)
            } else {
                String::new()
            };

            area.spawn((
                ConvertProgressText,
                Text::new(progress_text),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 进度条（仅转换中显示）
            if state.is_converting && state.total > 0 {
                let ratio = state.progress as f32 / state.total as f32;
                area.spawn(Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(500.0),
                    height: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                })
                .with_child((
                    Node {
                        width: Val::Percent(ratio * 100.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(AppColors::PRIMARY),
                ));
            }

            // 状态消息（错误/成功）
            let (status_text, status_color) = if let Some(ref err) = state.error {
                (err.clone(), AppColors::ERROR)
            } else if let Some(ref suc) = state.success {
                (suc.clone(), COLOR_SUCCESS)
            } else {
                (String::new(), AppColors::TEXT)
            };

            area.spawn((
                ConvertStatusText,
                Text::new(status_text),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(status_color),
            ));
        });
}

/// 清理图片转换页面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_image_convert_ui(mut query: Query<&mut Node, With<ImageConvertRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

/// 选择源目录按钮交互（使用 rfd 异步对话框，不阻塞主线程）
pub fn select_source_dir_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SelectSourceDirButton>),
    >,
    mut picker: ResMut<ImageConvertPickerResult>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(AppColors::PRIMARY_PRESSED);
                // 防止重复打开对话框
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
                *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(AppColors::PRIMARY);
            }
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

/// 目标格式按钮交互
pub fn target_format_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &TargetFormatButton,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut convert_state: ResMut<ImageConvertState>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, fmt_btn, mut bg_color, mut border_color, children) in
        interaction_query.iter_mut()
    {
        let is_selected = fmt_btn.format == convert_state.target_format;

        match *interaction {
            Interaction::Pressed => {
                if !convert_state.is_converting {
                    convert_state.target_format = fmt_btn.format;
                    // 选中状态会在下面的 None 分支中通过 refresh 更新
                }
            }
            Interaction::Hovered => {
                if !is_selected {
                    *bg_color = BackgroundColor(AppColors::SECONDARY_HOVER);
                }
            }
            Interaction::None => {
                // 根据是否选中设置颜色
                let new_is_selected = fmt_btn.format == convert_state.target_format;
                if new_is_selected {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                    *border_color = BorderColor::all(AppColors::PRIMARY);
                    // 更新子文本颜色
                    for child in children.iter() {
                        if let Ok(mut tc) = text_query.get_mut(child) {
                            tc.0 = Color::WHITE;
                        }
                    }
                } else {
                    *bg_color = BackgroundColor(AppColors::SECONDARY);
                    *border_color = BorderColor::all(AppColors::BORDER);
                    for child in children.iter() {
                        if let Ok(mut tc) = text_query.get_mut(child) {
                            tc.0 = AppColors::TEXT;
                        }
                    }
                }
            }
        }
    }
}

/// 刷新所有格式按钮的选中状态（当 target_format 变化时）
pub fn refresh_format_buttons(
    convert_state: Res<ImageConvertState>,
    mut button_query: Query<(
        &TargetFormatButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
    )>,
    mut text_query: Query<&mut TextColor>,
) {
    if !convert_state.is_changed() {
        return;
    }

    for (fmt_btn, mut bg_color, mut border_color, children) in button_query.iter_mut() {
        let is_selected = fmt_btn.format == convert_state.target_format;
        if is_selected {
            *bg_color = BackgroundColor(AppColors::PRIMARY);
            *border_color = BorderColor::all(AppColors::PRIMARY);
            for child in children.iter() {
                if let Ok(mut tc) = text_query.get_mut(child) {
                    tc.0 = Color::WHITE;
                }
            }
        } else {
            *bg_color = BackgroundColor(AppColors::SECONDARY);
            *border_color = BorderColor::all(AppColors::BORDER);
            for child in children.iter() {
                if let Ok(mut tc) = text_query.get_mut(child) {
                    tc.0 = AppColors::TEXT;
                }
            }
        }
    }
}

/// 开始转换按钮交互
pub fn start_convert_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartConvertButton>),
    >,
    mut convert_state: ResMut<ImageConvertState>,
    mut progress_result: ResMut<ImageConvertProgressResult>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
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

                *bg_color = BackgroundColor(AppColors::TEXT_SECONDARY);

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
            Interaction::Hovered => {
                if !convert_state.is_converting {
                    *bg_color = BackgroundColor(AppColors::PRIMARY_HOVER);
                }
            }
            Interaction::None => {
                if convert_state.is_converting {
                    *bg_color = BackgroundColor(AppColors::TEXT_SECONDARY);
                } else {
                    *bg_color = BackgroundColor(AppColors::PRIMARY);
                }
            }
        }
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
