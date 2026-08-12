//! Waifu2x 超分辨率工具系统
//!
//! 通过调用外部 waifu2x-ncnn-vulkan
//! 可执行文件，对图片目录进行批量超分辨率处理。 支持配置缩放倍数、降噪等级、
//! GPU、输出格式等参数。

use std::path::Path;

use bevy::prelude::*;
use picacg_config::{AppSettings, Waifu2xSettings};

use crate::{
    components::*,
    resources::*,
    systems::{
        login::AppColors,
        scrollbar::ScrollArea,
        widgets::{ButtonStyle, ButtonVariant},
    },
    utils::icons::*,
};

// ==================== 组件定义 ====================

/// Waifu2x 页面根节点
#[derive(Component, Default, Clone)]
pub struct Waifu2xRoot;

/// 可执行文件路径显示文本
#[derive(Component, Default, Clone)]
pub struct Waifu2xExePathText;

/// 选择可执行文件按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xSelectExeButton;

/// 输入目录路径显示文本
#[derive(Component, Default, Clone)]
pub struct Waifu2xInputDirText;

/// 选择输入目录按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xSelectInputDirButton;

/// 输出目录路径显示文本
#[derive(Component, Default, Clone)]
pub struct Waifu2xOutputDirText;

/// 选择输出目录按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xSelectOutputDirButton;

/// 缩放倍数选择按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xScaleButton {
    pub scale: i32,
}

/// 降噪等级选择按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xNoiseButton {
    pub level: i32,
}

/// GPU 选择按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xGpuButton {
    pub gpu_id: i32,
}

/// 输出格式选择按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xFormatButton {
    pub format: String,
}

/// 开始处理按钮
#[derive(Component, Default, Clone)]
pub struct Waifu2xStartButton;

/// 进度文本
#[derive(Component, Default, Clone)]
pub struct Waifu2xProgressText;

/// 状态消息文本
#[derive(Component, Default, Clone)]
pub struct Waifu2xStatusText;

/// 当前文件文本
#[derive(Component, Default, Clone)]
pub struct Waifu2xCurrentFileText;

// ==================== 颜色常量 ====================

/// 成功状态文本颜色
const COLOR_SUCCESS: Color = Color::srgb(0.3, 0.9, 0.4);

// ==================== 支持的图片扩展名 ====================

/// waifu2x-ncnn-vulkan 支持的输入图片格式
const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tga"];

/// 判断文件是否为支持的图片格式
fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

// ==================== UI 构建 ====================

/// 创建 Waifu2x 页面 UI
pub fn setup_waifu2x_ui(
    mut commands: Commands,
    content_area_query: Query<Entity, With<ContentArea>>,
    waifu2x_state: Res<Waifu2xState>,
    mut existing_query: Query<&mut Node, With<Waifu2xRoot>>,
) {
    // 如果 Waifu2xRoot 已存在（被 Display::None 隐藏了），直接显示
    if let Ok(mut node) = existing_query.single_mut() {
        node.display = Display::Flex;
        return;
    }

    let content_area = content_area_query.single().ok();

    let root = commands.spawn_scene(waifu2x_page(&waifu2x_state)).id();

    // 挂载到内容区域
    if let Some(content_entity) = content_area {
        commands.entity(content_entity).add_child(root);
    }

    tracing::info!("Waifu2x 页面 UI 已创建");
}

/// Waifu2x 页面场景
fn waifu2x_page(state: &Waifu2xState) -> impl Scene + use<> {
    let title = format!("{} Waifu2x 超分辨率", ICON_WAIFU2X);

    bsn! {
        Waifu2xRoot
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
                // ===== 内容区域（可滚动） =====
                ScrollArea
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(25.0)),
                    row_gap: Val::Px(18.0),
                    overflow: Overflow::scroll_y(),
                }
                Children [
                    // --- 可执行文件路径 ---
                    path_row(
                        "waifu2x-ncnn-vulkan 路径",
                        &state.executable_path,
                        "请选择 waifu2x-ncnn-vulkan 可执行文件...",
                        Waifu2xExePathText,
                        Waifu2xSelectExeButton,
                    ),
                    // --- 缩放倍数 ---
                    option_row(
                        "缩放倍数",
                        Waifu2xSettings::SCALES,
                        state.scale,
                        |value| format!("{}x", value),
                        |scale| Waifu2xScaleButton { scale },
                    ),
                    // --- 降噪等级 ---
                    option_row(
                        "降噪等级",
                        Waifu2xSettings::NOISE_LEVELS,
                        state.noise_level,
                        |value| Waifu2xSettings::noise_level_display(value).to_string(),
                        |level| Waifu2xNoiseButton { level },
                    ),
                    // --- GPU ---
                    option_row(
                        "GPU",
                        Waifu2xSettings::GPU_IDS,
                        state.gpu_id,
                        |value| Waifu2xSettings::gpu_id_display(value).to_string(),
                        |gpu_id| Waifu2xGpuButton { gpu_id },
                    ),
                    // --- 输出格式 ---
                    format_row(state),
                    (
                        // 分隔线
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(5.0)),
                        }
                        BackgroundColor(AppColors::BORDER)
                    ),
                    // --- 输入目录 ---
                    path_row(
                        "输入目录",
                        &state.input_dir,
                        "请选择要处理的图片目录...",
                        Waifu2xInputDirText,
                        Waifu2xSelectInputDirButton,
                    ),
                    // --- 输出目录 ---
                    path_row(
                        "输出目录",
                        &state.output_dir,
                        "请选择输出目录（留空则输出到输入目录）...",
                        Waifu2xOutputDirText,
                        Waifu2xSelectOutputDirButton,
                    ),
                    // --- 开始按钮 ---
                    start_button(state),
                    // --- 进度区域 ---
                    progress_area(state),
                    (
                        // 底部间距
                        Node {
                            height: Val::Px(30.0),
                            min_height: Val::Px(30.0),
                        }
                    ),
                ]
            ),
        ]
    }
}

/// 路径选择行场景（标签 + 路径显示框 + 选择按钮）
///
/// `text_marker` 标记路径文本节点，`button_marker` 标记选择按钮，
/// 由调用方决定是可执行文件行、输入目录行还是输出目录行。
fn path_row<T, B>(
    label: &str,
    current_value: &str,
    placeholder: &str,
    text_marker: T,
    button_marker: B,
) -> impl Scene + use<T, B>
where
    T: Component + Default + Clone + Unpin,
    B: Component + Default + Clone + Unpin,
{
    let label = label.to_string();
    let display_text = if current_value.is_empty() {
        placeholder.to_string()
    } else {
        current_value.to_string()
    };
    let text_color = if current_value.is_empty() {
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
                            padding: UiRect::new(
                                Val::Px(10.0),
                                Val::Px(10.0),
                                Val::Px(8.0),
                                Val::Px(8.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            overflow: Overflow::clip(),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                template_value(text_marker)
                                Text({display_text})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor({text_color})
                            )
                        ]
                    ),
                    (
                        // 选择按钮
                        template_value(button_marker)
                        Button
                        template_value(ButtonStyle::primary())
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

/// 选项按钮行场景（缩放/降噪/GPU）
///
/// `format_value` 决定按钮文本，`make_marker` 决定按钮的标记组件。
fn option_row<C, D, M>(
    label: &str,
    values: &[i32],
    current: i32,
    format_value: D,
    make_marker: M,
) -> impl Scene + use<C, D, M>
where
    C: Component + Default + Clone + Unpin,
    D: Fn(i32) -> String,
    M: Fn(i32) -> C,
{
    let label = label.to_string();
    let buttons: Vec<_> = values
        .iter()
        .map(|&value| option_button(make_marker(value), format_value(value), value == current))
        .collect();

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
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 按钮行
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(6.0),
                }
                Children [ {buttons} ]
            ),
        ]
    }
}

/// 单个选项按钮场景（选项行与输出格式行共用，选中态决定配色）
fn option_button<C: Component + Default + Clone + Unpin>(
    marker: C,
    display: String,
    is_selected: bool,
) -> impl Scene + use<C> {
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

    bsn! {
        template_value(marker)
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(14.0), Val::Px(14.0), Val::Px(7.0), Val::Px(7.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template_value(BorderColor::all(border_color))
        BackgroundColor({bg_color})
        Children [
            (
                Text({display})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 输出格式选择行场景
fn format_row(state: &Waifu2xState) -> impl Scene + use<> {
    let buttons: Vec<_> = Waifu2xSettings::OUTPUT_FORMATS
        .iter()
        .map(|&fmt| {
            option_button(
                Waifu2xFormatButton {
                    format: fmt.to_string(),
                },
                fmt.to_ascii_uppercase(),
                fmt == state.output_format,
            )
        })
        .collect();

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
        }
        Children [
            (
                Text("输出格式")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                }
                Children [ {buttons} ]
            ),
        ]
    }
}

/// 开始处理按钮场景
fn start_button(state: &Waifu2xState) -> impl Scene + use<> {
    // 处理中降级为次要配色表达禁用感，三态配色由 ButtonStyle 统一
    let (label, style, bg_color) = if state.is_processing {
        ("处理中...", ButtonStyle::secondary(), AppColors::SECONDARY)
    } else {
        ("开始转换", ButtonStyle::primary(), AppColors::PRIMARY)
    };

    bsn! {
        Waifu2xStartButton
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(24.0), Val::Px(24.0), Val::Px(10.0), Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            align_self: AlignSelf::FlexStart,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
        }
        template_value(BorderColor::all(bg_color))
        BackgroundColor({bg_color})
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
fn progress_area(state: &Waifu2xState) -> impl Scene + use<> {
    // 进度文本
    let progress_text = if state.is_processing {
        format!("进度: {}/{}", state.progress, state.total)
    } else if state.total > 0 {
        format!("已完成: {}/{}", state.progress, state.total)
    } else {
        String::new()
    };

    // 进度条（仅处理中显示）
    let progress_bar: Box<dyn SceneList> = if state.is_processing && state.total > 0 {
        let ratio = state.progress as f32 / state.total as f32;
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
                        width: {Val::Percent(ratio * 100.0)},
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

    // 当前文件名
    let current_file_text = if state.current_file.is_empty() {
        String::new()
    } else {
        format!("正在处理: {}", state.current_file)
    };

    // 状态消息（错误/成功）
    let (status_text, status_color) = if let Some(ref err) = state.error {
        (err.clone(), AppColors::ERROR)
    } else if let Some(ref suc) = state.success {
        (suc.clone(), COLOR_SUCCESS)
    } else {
        (String::new(), AppColors::TEXT)
    };

    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
        }
        Children [
            (
                // 进度文本
                Waifu2xProgressText
                Text({progress_text})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT)
            ),
            {progress_bar},
            (
                // 当前文件名
                Waifu2xCurrentFileText
                Text({current_file_text})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 状态消息（错误/成功）
                Waifu2xStatusText
                Text({status_text})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor({status_color})
            ),
        ]
    }
}

// ==================== 清理系统 ====================

/// 清理 Waifu2x 页面（用 Display::None 隐藏，保留 UI 结构）
pub fn cleanup_waifu2x_ui(mut query: Query<&mut Node, With<Waifu2xRoot>>) {
    for mut node in query.iter_mut() {
        node.display = Display::None;
    }
}

// ==================== 交互系统 ====================

/// 选择可执行文件按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn waifu2x_select_exe_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Waifu2xSelectExeButton>)>,
    mut picker: ResMut<Waifu2xPickerResult>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed && picker.receiver.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            picker.receiver = Some(std::sync::Mutex::new(rx));
            std::thread::spawn(move || {
                // 使用文件选择对话框（选择可执行文件）
                let path = rfd::FileDialog::new()
                    .set_title("选择 waifu2x-ncnn-vulkan 可执行文件")
                    .pick_file()
                    .map(|p| p.to_string_lossy().to_string());
                let _ = tx.send((path, Waifu2xPickerType::Executable));
            });
        }
    }
}

/// 选择输入目录按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn waifu2x_select_input_dir_interaction(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Waifu2xSelectInputDirButton>),
    >,
    mut picker: ResMut<Waifu2xPickerResult>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed && picker.receiver.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            picker.receiver = Some(std::sync::Mutex::new(rx));
            std::thread::spawn(move || {
                let path = rfd::FileDialog::new()
                    .set_title("选择输入图片目录")
                    .pick_folder()
                    .map(|p| p.to_string_lossy().to_string());
                let _ = tx.send((path, Waifu2xPickerType::InputDir));
            });
        }
    }
}

/// 选择输出目录按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn waifu2x_select_output_dir_interaction(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Waifu2xSelectOutputDirButton>),
    >,
    mut picker: ResMut<Waifu2xPickerResult>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed && picker.receiver.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            picker.receiver = Some(std::sync::Mutex::new(rx));
            std::thread::spawn(move || {
                let path = rfd::FileDialog::new()
                    .set_title("选择输出目录")
                    .pick_folder()
                    .map(|p| p.to_string_lossy().to_string());
                let _ = tx.send((path, Waifu2xPickerType::OutputDir));
            });
        }
    }
}

/// 轮询目录/文件选择器的异步结果
pub fn handle_waifu2x_picker_result(
    mut picker: ResMut<Waifu2xPickerResult>,
    mut state: ResMut<Waifu2xState>,
    mut exe_text_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<Waifu2xExePathText>,
            Without<Waifu2xInputDirText>,
            Without<Waifu2xOutputDirText>,
        ),
    >,
    mut input_text_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<Waifu2xInputDirText>,
            Without<Waifu2xExePathText>,
            Without<Waifu2xOutputDirText>,
        ),
    >,
    mut output_text_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<Waifu2xOutputDirText>,
            Without<Waifu2xExePathText>,
            Without<Waifu2xInputDirText>,
        ),
    >,
) {
    let Some(ref receiver) = picker.receiver else {
        return;
    };
    let Ok(receiver) = receiver.lock() else {
        return;
    };
    let Ok((result, picker_type)) = receiver.try_recv() else {
        return;
    };
    drop(receiver);
    picker.receiver = None;

    let Some(path_str) = result else {
        return;
    };

    match picker_type {
        Waifu2xPickerType::Executable => {
            state.executable_path = path_str.clone();
            // 保存到配置
            save_waifu2x_settings(&state);
            for (mut text, mut text_color) in exe_text_query.iter_mut() {
                **text = path_str.clone();
                text_color.0 = AppColors::TEXT;
            }
            tracing::info!("已选择 waifu2x 可执行文件: {}", path_str);
        }
        Waifu2xPickerType::InputDir => {
            state.input_dir = path_str.clone();
            // 清除旧状态
            state.error = None;
            state.success = None;
            state.progress = 0;
            state.total = 0;
            state.current_file.clear();
            for (mut text, mut text_color) in input_text_query.iter_mut() {
                **text = path_str.clone();
                text_color.0 = AppColors::TEXT;
            }
            tracing::info!("已选择输入目录: {}", path_str);
        }
        Waifu2xPickerType::OutputDir => {
            state.output_dir = path_str.clone();
            for (mut text, mut text_color) in output_text_query.iter_mut() {
                **text = path_str.clone();
                text_color.0 = AppColors::TEXT;
            }
            tracing::info!("已选择输出目录: {}", path_str);
        }
    }
}

/// 缩放倍数按钮交互（选中态由 `refresh_waifu2x_option_buttons` 统一刷新）
pub fn waifu2x_scale_interaction(
    interaction_query: Query<(&Interaction, &Waifu2xScaleButton), Changed<Interaction>>,
    mut state: ResMut<Waifu2xState>,
) {
    for (interaction, scale_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed && !state.is_processing {
            state.scale = scale_btn.scale;
            save_waifu2x_settings(&state);
        }
    }
}

/// 降噪等级按钮交互（选中态由 `refresh_waifu2x_option_buttons` 统一刷新）
pub fn waifu2x_noise_interaction(
    interaction_query: Query<(&Interaction, &Waifu2xNoiseButton), Changed<Interaction>>,
    mut state: ResMut<Waifu2xState>,
) {
    for (interaction, noise_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed && !state.is_processing {
            state.noise_level = noise_btn.level;
            save_waifu2x_settings(&state);
        }
    }
}

/// GPU 选择按钮交互（选中态由 `refresh_waifu2x_option_buttons` 统一刷新）
pub fn waifu2x_gpu_interaction(
    interaction_query: Query<(&Interaction, &Waifu2xGpuButton), Changed<Interaction>>,
    mut state: ResMut<Waifu2xState>,
) {
    for (interaction, gpu_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed && !state.is_processing {
            state.gpu_id = gpu_btn.gpu_id;
            save_waifu2x_settings(&state);
        }
    }
}

/// 输出格式按钮交互（选中态由 `refresh_waifu2x_option_buttons` 统一刷新）
pub fn waifu2x_format_interaction(
    interaction_query: Query<(&Interaction, &Waifu2xFormatButton), Changed<Interaction>>,
    mut state: ResMut<Waifu2xState>,
) {
    for (interaction, fmt_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed && !state.is_processing {
            state.output_format = fmt_btn.format.clone();
            save_waifu2x_settings(&state);
        }
    }
}

/// 刷新所有选项按钮的选中状态（当状态变化时）
pub fn refresh_waifu2x_option_buttons(
    state: Res<Waifu2xState>,
    mut scale_query: Query<
        (&Waifu2xScaleButton, &mut ButtonStyle, &mut BorderColor),
        Without<Waifu2xNoiseButton>,
    >,
    mut noise_query: Query<
        (&Waifu2xNoiseButton, &mut ButtonStyle, &mut BorderColor),
        Without<Waifu2xScaleButton>,
    >,
    mut gpu_query: Query<
        (&Waifu2xGpuButton, &mut ButtonStyle, &mut BorderColor),
        (Without<Waifu2xScaleButton>, Without<Waifu2xNoiseButton>),
    >,
    mut format_query: Query<
        (&Waifu2xFormatButton, &mut ButtonStyle, &mut BorderColor),
        (
            Without<Waifu2xScaleButton>,
            Without<Waifu2xNoiseButton>,
            Without<Waifu2xGpuButton>,
        ),
    >,
) {
    if !state.is_changed() {
        return;
    }

    for (btn, mut style, mut border) in scale_query.iter_mut() {
        update_option_button_style(btn.scale == state.scale, &mut style, &mut border);
    }

    for (btn, mut style, mut border) in noise_query.iter_mut() {
        update_option_button_style(btn.level == state.noise_level, &mut style, &mut border);
    }

    for (btn, mut style, mut border) in gpu_query.iter_mut() {
        update_option_button_style(btn.gpu_id == state.gpu_id, &mut style, &mut border);
    }

    for (btn, mut style, mut border) in format_query.iter_mut() {
        update_option_button_style(btn.format == state.output_format, &mut style, &mut border);
    }
}

/// 更新选项按钮的选中态（背景色由 `apply_button_interaction` 接管）
fn update_option_button_style(
    is_selected: bool,
    style: &mut ButtonStyle,
    border_color: &mut BorderColor,
) {
    if style.selected != is_selected {
        style.selected = is_selected;
    }
    *border_color = BorderColor::all(if is_selected {
        AppColors::PRIMARY
    } else {
        AppColors::BORDER
    });
}

/// 按「是否可用」切换开始按钮的变体（处理中降级为次要配色表达禁用感）
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

/// 开始处理按钮交互（配色由 `apply_button_interaction` 统一处理）
pub fn waifu2x_start_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonStyle),
        (Changed<Interaction>, With<Waifu2xStartButton>),
    >,
    mut state: ResMut<Waifu2xState>,
    mut progress_result: ResMut<Waifu2xProgressResult>,
) {
    for (interaction, mut style) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if state.is_processing {
            return;
        }

        // 验证可执行文件路径
        if state.executable_path.is_empty() {
            state.error = Some("请先选择 waifu2x-ncnn-vulkan 可执行文件".to_string());
            return;
        }
        if !Path::new(&state.executable_path).exists() {
            state.error = Some(format!("可执行文件不存在: {}", state.executable_path));
            return;
        }

        // 验证输入目录
        if state.input_dir.is_empty() {
            state.error = Some("请先选择输入目录".to_string());
            return;
        }
        if !Path::new(&state.input_dir).is_dir() {
            state.error = Some(format!("输入目录不存在: {}", state.input_dir));
            return;
        }

        // 准备参数
        let exe_path = state.executable_path.clone();
        let input_dir = state.input_dir.clone();
        let output_dir = if state.output_dir.is_empty() {
            state.input_dir.clone()
        } else {
            state.output_dir.clone()
        };
        let scale = state.scale;
        let noise_level = state.noise_level;
        let gpu_id = state.gpu_id;
        let output_format = state.output_format.clone();

        // 清除旧状态
        state.error = None;
        state.success = None;
        state.progress = 0;
        state.total = 0;
        state.current_file.clear();
        state.is_processing = true;

        set_start_button_enabled(&mut style, false);

        tracing::info!(
            "开始 Waifu2x 处理: input={}, output={}, scale={}, noise={}, gpu={}, format={}",
            input_dir,
            output_dir,
            scale,
            noise_level,
            gpu_id,
            output_format
        );

        // 创建进度通道
        let (tx, rx) = std::sync::mpsc::channel();
        progress_result.receiver = Some(std::sync::Mutex::new(rx));

        // 在后台线程执行处理
        std::thread::spawn(move || {
            run_waifu2x_batch(
                &exe_path,
                &input_dir,
                &output_dir,
                scale,
                noise_level,
                gpu_id,
                &output_format,
                tx,
            );
        });
    }
}

/// 轮询 Waifu2x 处理进度并更新 UI
pub fn refresh_waifu2x_progress(
    mut progress_result: ResMut<Waifu2xProgressResult>,
    mut state: ResMut<Waifu2xState>,
    mut progress_text_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<Waifu2xProgressText>,
            Without<Waifu2xStatusText>,
            Without<Waifu2xCurrentFileText>,
        ),
    >,
    mut status_text_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<Waifu2xStatusText>,
            Without<Waifu2xProgressText>,
            Without<Waifu2xCurrentFileText>,
        ),
    >,
    mut current_file_query: Query<
        &mut Text,
        (
            With<Waifu2xCurrentFileText>,
            Without<Waifu2xProgressText>,
            Without<Waifu2xStatusText>,
        ),
    >,
    mut start_button_query: Query<&mut ButtonStyle, With<Waifu2xStartButton>>,
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
        Waifu2xProgressMsg::Progress {
            done,
            total,
            current_file,
        } => {
            state.progress = done;
            state.total = total;
            state.current_file = current_file.clone();

            for (mut text, _) in progress_text_query.iter_mut() {
                **text = format!("进度: {}/{}", done, total);
            }
            for mut text in current_file_query.iter_mut() {
                if current_file.is_empty() {
                    **text = String::new();
                } else {
                    **text = format!("正在处理: {}", current_file);
                }
            }
        }
        Waifu2xProgressMsg::Completed { done, total } => {
            state.progress = done;
            state.total = total;
            state.is_processing = false;
            state.current_file.clear();
            state.success = Some(format!("处理完成！成功处理 {} 张图片", done));

            progress_result.receiver = None;
            for mut style in start_button_query.iter_mut() {
                set_start_button_enabled(&mut style, true);
            }

            for (mut text, _) in progress_text_query.iter_mut() {
                **text = format!("已完成: {}/{}", done, total);
            }
            for (mut text, mut text_color) in status_text_query.iter_mut() {
                **text = format!("处理完成！成功处理 {} 张图片", done);
                text_color.0 = COLOR_SUCCESS;
            }
            for mut text in current_file_query.iter_mut() {
                **text = String::new();
            }

            tracing::info!("Waifu2x 处理完成: {}/{}", done, total);
        }
        Waifu2xProgressMsg::Error(err) => {
            state.is_processing = false;
            state.current_file.clear();
            state.error = Some(err.clone());

            progress_result.receiver = None;
            for mut style in start_button_query.iter_mut() {
                set_start_button_enabled(&mut style, true);
            }

            for (mut text, mut text_color) in status_text_query.iter_mut() {
                **text = err.clone();
                text_color.0 = AppColors::ERROR;
            }
            for mut text in current_file_query.iter_mut() {
                **text = String::new();
            }
        }
    }
}

// ==================== 辅助函数 ====================

/// 将当前 Waifu2x 设置保存到配置文件
fn save_waifu2x_settings(state: &Waifu2xState) {
    let mut settings = AppSettings::global().write();
    settings.waifu2x.executable_path = state.executable_path.clone();
    settings.waifu2x.scale = state.scale;
    settings.waifu2x.noise_level = state.noise_level;
    settings.waifu2x.gpu_id = state.gpu_id;
    settings.waifu2x.output_format = state.output_format.clone();
    if let Err(e) = settings.save() {
        tracing::error!("保存 Waifu2x 设置失败: {}", e);
    }
}

// ==================== 后台处理逻辑 ====================

/// 批量调用 waifu2x-ncnn-vulkan 处理图片（后台线程运行）
fn run_waifu2x_batch(
    exe_path: &str,
    input_dir: &str,
    output_dir: &str,
    scale: i32,
    noise_level: i32,
    gpu_id: i32,
    output_format: &str,
    tx: std::sync::mpsc::Sender<Waifu2xProgressMsg>,
) {
    let input_path = Path::new(input_dir);
    let output_path = Path::new(output_dir);

    // 确保输出目录存在
    if !output_path.exists()
        && let Err(e) = std::fs::create_dir_all(output_path)
    {
        let _ = tx.send(Waifu2xProgressMsg::Error(format!(
            "无法创建输出目录: {}",
            e
        )));
        return;
    }

    // 收集所有支持的图片文件
    let image_files: Vec<std::path::PathBuf> = match std::fs::read_dir(input_path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && is_supported_image(path))
            .collect(),
        Err(e) => {
            let _ = tx.send(Waifu2xProgressMsg::Error(format!(
                "无法读取输入目录: {}",
                e
            )));
            return;
        }
    };

    let total = image_files.len() as u32;
    if total == 0 {
        let _ = tx.send(Waifu2xProgressMsg::Error(
            "输入目录中没有找到支持的图片文件（jpg/png/webp/bmp/tga）".to_string(),
        ));
        return;
    }

    // 发送初始进度
    let _ = tx.send(Waifu2xProgressMsg::Progress {
        done: 0,
        total,
        current_file: String::new(),
    });

    let mut done: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for file_path in &image_files {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知文件")
            .to_string();

        // 发送当前文件名
        let _ = tx.send(Waifu2xProgressMsg::Progress {
            done,
            total,
            current_file: file_name.clone(),
        });

        // 构建输出文件路径：输出目录 / 原文件名（替换扩展名）
        let output_file = output_path.join(
            file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                + "."
                + output_format,
        );

        // 调用 waifu2x-ncnn-vulkan
        let result = std::process::Command::new(exe_path)
            .args([
                "-i",
                &file_path.to_string_lossy(),
                "-o",
                &output_file.to_string_lossy(),
                "-s",
                &scale.to_string(),
                "-n",
                &noise_level.to_string(),
                "-g",
                &gpu_id.to_string(),
                "-f",
                output_format,
            ])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    done += 1;
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let err_msg = if stderr.is_empty() {
                        format!("{}: 进程退出码 {}", file_name, output.status)
                    } else {
                        format!("{}: {}", file_name, stderr.trim())
                    };
                    errors.push(err_msg);
                    done += 1;
                }
            }
            Err(e) => {
                errors.push(format!("{}: {}", file_name, e));
                // 如果是可执行文件找不到等致命错误，直接终止
                if e.kind() == std::io::ErrorKind::NotFound {
                    let _ = tx.send(Waifu2xProgressMsg::Error(format!(
                        "找不到 waifu2x-ncnn-vulkan 可执行文件: {}",
                        exe_path
                    )));
                    return;
                }
                done += 1;
            }
        }

        // 发送进度更新
        let _ = tx.send(Waifu2xProgressMsg::Progress {
            done,
            total,
            current_file: String::new(),
        });
    }

    // 发送完成消息
    if errors.is_empty() {
        let _ = tx.send(Waifu2xProgressMsg::Completed { done, total });
    } else {
        let error_summary = format!(
            "处理完成，但有 {} 个文件出错:\n{}",
            errors.len(),
            errors
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
        let _ = tx.send(Waifu2xProgressMsg::Completed { done, total });
        tracing::warn!("{}", error_summary);
    }
}
