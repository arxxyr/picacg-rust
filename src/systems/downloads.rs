//! 下载管理界面系统
//!
//! 实现下载任务列表和进度显示

#![allow(dead_code)]

use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    components::{
        ContentArea, ContentSizeInfo, ScrollbarContainer, ScrollbarThumb, ScrollbarTrack,
    },
    config::settings::AppSettings,
    events::{DownloadCompletedEvent, RedownloadRequest, ResumeDownloadRequest},
    resources::{ComicDownloadStatus, DownloadManagerState},
    systems::login::{AppColors, FONT_PATH},
};

/// 下载滚动容器组件（本地定义）
#[derive(Component)]
pub struct ScrollContainer;

/// 获取下载保存路径
pub fn get_download_base_path() -> std::path::PathBuf {
    let settings = AppSettings::global().read();
    if !settings.download_path.is_empty() {
        return std::path::PathBuf::from(&settings.download_path);
    }
    drop(settings);

    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Downloads")
}

/// 清理文件名中的非法字符（与 api_plugin.rs 中的 sanitize_filename 保持一致）
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// 下载页面根标记
#[derive(Component)]
pub struct DownloadsRoot;

/// 下载滚动容器标记
#[derive(Component)]
pub struct DownloadsScrollContainer;

/// 下载任务列表容器标记
#[derive(Component)]
pub struct DownloadTaskList;

/// "下载中" 区域容器（包含标题和任务列表）
#[derive(Component)]
pub struct DownloadingSection;

/// "下载中" 标题文本
#[derive(Component)]
pub struct DownloadingTitleText;

/// "开始全部下载" 按钮标记
#[derive(Component)]
pub struct StartAllDownloadsButton;

/// 已完成下载项目（历史记录）
#[derive(Debug, Clone)]
pub struct CompletedDownload {
    pub comic_id: String,
    pub folder_name: String,
    pub episode_count: usize,
    pub path: String,
}

/// 已下载漫画项标记
#[derive(Component)]
pub struct CompletedDownloadItem {
    pub comic_id: String,
    pub path: String,
}

/// 已下载列表容器标记
#[derive(Component)]
pub struct CompletedDownloadList;

/// "已下载" 区域容器
#[derive(Component)]
pub struct CompletedSection;

/// "已下载" 标题文本
#[derive(Component)]
pub struct CompletedTitleText;

/// 重新下载按钮标记
#[derive(Component)]
pub struct RedownloadButton {
    pub comic_id: String,
}

/// 打开已下载漫画文件夹按钮
#[derive(Component)]
pub struct OpenCompletedFolderButton {
    pub path: String,
}

/// 扫描下载目录，获取已完成的下载列表
///
/// 首先从数据库加载已完成的下载任务。
/// 然后扫描文件系统中的旧数据（兼容旧版本）。
///
/// `downloading_ids`: 正在下载中的漫画 ID 列表，用于过滤避免重复显示
fn scan_completed_downloads(
    downloading_ids: &std::collections::HashSet<String>,
) -> Vec<CompletedDownload> {
    use crate::{
        db::database::{Database, run_db_operation},
        resources::DownloadTaskMeta,
    };

    let download_path = get_download_base_path();
    let mut downloads = Vec::new();
    let mut known_comic_ids = std::collections::HashSet::new();

    // 1. 首先从数据库加载已完成的下载任务
    let db_tasks = run_db_operation(async {
        let db = Database::global().read();
        db.get_completed_download_tasks().await
    })
    .unwrap_or_default();

    for db_task in db_tasks {
        // 跳过正在下载中的漫画
        if downloading_ids.contains(&db_task.comic_id) {
            tracing::debug!("跳过正在下载的漫画: {}", db_task.comic_title);
            continue;
        }

        // 从 save_path 提取文件夹名称
        let folder_name = std::path::Path::new(&db_task.save_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&db_task.comic_title)
            .to_string();

        let episode_count = db_task.get_episode_orders().len();

        downloads.push(CompletedDownload {
            comic_id: db_task.comic_id.clone(),
            folder_name,
            episode_count,
            path: db_task.save_path.clone(),
        });

        known_comic_ids.insert(db_task.comic_id);
    }

    // 2. 向后兼容：扫描文件系统中的旧数据
    if download_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&download_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("未知")
                        .to_string();

                    let path_str = path.to_string_lossy().to_string();

                    // 检查是否有旧的元数据文件
                    if let Ok(meta) = DownloadTaskMeta::load(&path_str) {
                        // 跳过已从数据库加载的
                        if known_comic_ids.contains(&meta.comic_id) {
                            continue;
                        }

                        // 跳过正在下载中的漫画
                        if downloading_ids.contains(&meta.comic_id) {
                            tracing::debug!("跳过正在下载的漫画: {}", folder_name);
                            continue;
                        }

                        // 只有已完成状态才显示在已下载列表
                        if meta.state.is_completed() {
                            let episode_count = meta.episode_orders.len();
                            downloads.push(CompletedDownload {
                                comic_id: meta.comic_id.clone(),
                                folder_name,
                                episode_count,
                                path: path_str,
                            });
                            known_comic_ids.insert(meta.comic_id);
                        }
                    } else {
                        // 没有元数据文件，使用旧的逻辑（兼容旧数据）
                        // 统计章节数量（子文件夹数量）
                        let episode_count = if let Ok(sub_entries) = std::fs::read_dir(&path) {
                            sub_entries.flatten().filter(|e| e.path().is_dir()).count()
                        } else {
                            0
                        };

                        // 只有包含章节的才算已下载的漫画
                        // 注意：没有元数据的旧数据无法重新下载（没有 comic_id）
                        if episode_count > 0 {
                            downloads.push(CompletedDownload {
                                comic_id: String::new(), // 旧数据没有 comic_id
                                folder_name,
                                episode_count,
                                path: path_str,
                            });
                        }
                    }
                }
            }
        }
    }

    // 按文件夹名称排序
    downloads.sort_by(|a, b| a.folder_name.cmp(&b.folder_name));
    tracing::info!("扫描到 {} 个已下载漫画", downloads.len());
    downloads
}

/// 单个下载任务项标记
#[derive(Component)]
pub struct DownloadTaskItem {
    pub comic_id: String,
}

/// 下载进度条标记
#[derive(Component)]
pub struct DownloadProgressBar {
    pub comic_id: String,
}

/// 下载状态文本标记
#[derive(Component)]
pub struct DownloadStatusText {
    pub comic_id: String,
}

/// 打开下载文件夹按钮标记
#[derive(Component)]
pub struct OpenDownloadFolderButton;

/// 暂停下载按钮标记
#[derive(Component)]
pub struct PauseDownloadButton {
    pub comic_id: String,
}

/// 继续下载按钮标记
#[derive(Component)]
pub struct ResumeDownloadButton {
    pub comic_id: String,
}

/// 删除下载按钮标记
#[derive(Component)]
pub struct DeleteDownloadButton {
    pub comic_id: String,
}

/// 重试下载按钮标记
#[derive(Component)]
pub struct RetryDownloadButton {
    pub comic_id: String,
}

/// 加载未完成的下载任务（进入下载页面时调用）
pub fn load_incomplete_downloads(mut download_state: ResMut<DownloadManagerState>) {
    let download_path = get_download_base_path();
    download_state.load_incomplete_tasks(&download_path);
}

/// 创建下载页面 UI
pub fn setup_downloads_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
    download_state: Res<DownloadManagerState>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    // 查找内容区域
    let content_area = match content_area_query.iter().next() {
        Some(entity) => entity,
        None => {
            tracing::warn!("下载页面：找不到内容区域");
            return;
        }
    };

    // 在内容区域下创建下载页面
    commands.entity(content_area).with_children(|parent| {
        parent
            .spawn((
                DownloadsRoot,
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
                spawn_downloads_header(root, &font);

                // 下载内容（可滚动）
                root.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_shrink: 1.0,
                        flex_basis: Val::Px(0.0),
                        min_height: Val::Px(0.0),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
                ))
                .with_children(|content_wrapper| {
                    // 滚动容器
                    let scroll_container = content_wrapper
                        .spawn((
                            DownloadsScrollContainer,
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
                            // 扫描已下载的漫画（排除正在下载的）
                            let completed_downloads =
                                scan_completed_downloads(&download_state.downloading_ids);

                            // 获取活跃任务列表（未完成的）
                            let active_tasks: Vec<_> = download_state
                                .active_tasks()
                                .into_iter()
                                .map(|fsm| fsm.to_ui_task())
                                .collect();

                            let has_active_tasks = !active_tasks.is_empty();

                            // 始终创建"下载中"区域（初始可能隐藏）
                            scroll
                                .spawn((
                                    DownloadingSection,
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        display: if has_active_tasks {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        },
                                        ..default()
                                    },
                                ))
                                .with_children(|section| {
                                    // 标题行（标题 + 开始全部按钮）
                                    section
                                        .spawn(Node {
                                            width: Val::Percent(100.0),
                                            justify_content: JustifyContent::SpaceBetween,
                                            align_items: AlignItems::Center,
                                            margin: UiRect::bottom(Val::Px(10.0)),
                                            ..default()
                                        })
                                        .with_children(|header| {
                                            // 进行中标题
                                            header.spawn((
                                                DownloadingTitleText,
                                                Text::new(format!(
                                                    "📥 下载中 ({})",
                                                    active_tasks.len()
                                                )),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT),
                                            ));

                                            // 开始全部下载按钮
                                            header
                                                .spawn((
                                                    StartAllDownloadsButton,
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
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                                                    BorderColor::all(Color::srgb(0.3, 0.7, 0.4)),
                                                    BorderRadius::all(Val::Px(4.0)),
                                                ))
                                                .with_children(|btn| {
                                                    btn.spawn((
                                                        Text::new("\u{F040A} 全部开始"), // 󰐊 nf-md-play
                                                        TextFont {
                                                            font: font.clone(),
                                                            font_size: 13.0,
                                                            ..default()
                                                        },
                                                        TextColor(AppColors::TEXT),
                                                    ));
                                                });
                                        });

                                    // 任务列表容器
                                    section
                                        .spawn((
                                            DownloadTaskList,
                                            Node {
                                                width: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(10.0),
                                                margin: UiRect::bottom(Val::Px(20.0)),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|list| {
                                            for task in &active_tasks {
                                                spawn_download_task_item(list, &font, task);
                                            }
                                        });
                                });

                            // 已下载列表（始终创建，初始可能隐藏）
                            let has_completed = !completed_downloads.is_empty();
                            scroll
                                .spawn((
                                    CompletedSection,
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        display: if has_completed {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        },
                                        ..default()
                                    },
                                ))
                                .with_children(|section| {
                                    // 已下载标题
                                    section.spawn((
                                        CompletedTitleText,
                                        Text::new(format!(
                                            "📚 已下载 ({})",
                                            completed_downloads.len()
                                        )),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(AppColors::TEXT),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(10.0)),
                                            ..default()
                                        },
                                    ));

                                    // 列表容器
                                    section
                                        .spawn((
                                            CompletedDownloadList,
                                            Node {
                                                width: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(10.0),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|list| {
                                            for download in &completed_downloads {
                                                spawn_completed_download_item(
                                                    list, &font, download,
                                                );
                                            }
                                        });
                                });

                            // 如果都为空，显示空状态
                            if !has_active_tasks && completed_downloads.is_empty() {
                                spawn_empty_state(scroll, &font);
                            }
                        })
                        .id();

                    // 滚动条
                    spawn_downloads_scrollbar(content_wrapper, scroll_container);
                });
            });
    });

    tracing::info!("下载页面 UI 已创建");
}

/// 创建下载标题栏
fn spawn_downloads_header(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                padding: UiRect::horizontal(Val::Px(20.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
            BorderColor::all(AppColors::BORDER),
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|header| {
            // 标题
            header.spawn((
                Text::new("📥 下载管理"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 打开文件夹按钮
            header
                .spawn((
                    OpenDownloadFolderButton,
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
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::all(AppColors::BORDER),
                    BorderRadius::all(Val::Px(4.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("\u{F0770} 打开文件夹"), // 󰝰 nf-md-folder_open
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

/// 创建空状态提示
fn spawn_empty_state(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },))
        .with_children(|empty| {
            empty.spawn((
                Text::new("📭"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            empty.spawn((
                Text::new("暂无下载任务"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
            empty.spawn((
                Text::new("在漫画详情页点击下载按钮开始下载"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(AppColors::TEXT_SECONDARY),
            ));
        });
}

/// 创建下载任务项
fn spawn_download_task_item(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    task: &crate::resources::ComicDownloadTask,
) {
    let (status_text, status_color) = match &task.status {
        ComicDownloadStatus::Waiting => ("等待中".to_string(), AppColors::TEXT_SECONDARY),
        ComicDownloadStatus::Downloading => (
            format!(
                "下载中 第{}/{}章 {}/{}",
                task.current_episode, task.total_episodes, task.current_page, task.total_pages
            ),
            AppColors::PRIMARY,
        ),
        ComicDownloadStatus::Paused => ("已暂停".to_string(), Color::srgb(0.8, 0.6, 0.2)),
        ComicDownloadStatus::Completed => ("已完成".to_string(), Color::srgb(0.3, 0.8, 0.3)),
        ComicDownloadStatus::Failed(err) => (format!("失败: {}", err), Color::srgb(0.8, 0.3, 0.3)),
    };

    // 计算进度百分比
    let progress = if task.total_episodes > 0 {
        let episode_progress = (task.current_episode - 1) as f32 / task.total_episodes as f32;
        let page_progress = if task.total_pages > 0 {
            task.current_page as f32 / task.total_pages as f32 / task.total_episodes as f32
        } else {
            0.0
        };
        (episode_progress + page_progress).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // 判断是否可以暂停/继续/重试
    let can_pause = matches!(
        task.status,
        ComicDownloadStatus::Downloading | ComicDownloadStatus::Waiting
    );
    let can_resume = matches!(task.status, ComicDownloadStatus::Paused);
    let can_retry = matches!(task.status, ComicDownloadStatus::Failed(_));
    // 删除按钮始终可见（下载中时作为"取消"功能）
    let can_delete = true;

    parent
        .spawn((
            DownloadTaskItem {
                comic_id: task.comic_id.clone(),
            },
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(1.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
            BorderColor::all(AppColors::BORDER),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|item| {
            // 标题行
            item.spawn((Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },))
                .with_children(|row| {
                    // 漫画标题
                    row.spawn((
                        Text::new(task.comic_title.clone()),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));

                    // 状态标签
                    row.spawn((
                        DownloadStatusText {
                            comic_id: task.comic_id.clone(),
                        },
                        Text::new(status_text),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(status_color),
                    ));
                });

            // 进度条
            item.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                BorderRadius::all(Val::Px(3.0)),
            ))
            .with_children(|track| {
                track.spawn((
                    DownloadProgressBar {
                        comic_id: task.comic_id.clone(),
                    },
                    Node {
                        width: Val::Percent(progress * 100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(match &task.status {
                        ComicDownloadStatus::Completed => Color::srgb(0.3, 0.8, 0.3),
                        ComicDownloadStatus::Failed(_) => Color::srgb(0.8, 0.3, 0.3),
                        ComicDownloadStatus::Paused => Color::srgb(0.8, 0.6, 0.2),
                        _ => AppColors::PRIMARY,
                    }),
                    BorderRadius::all(Val::Px(3.0)),
                ));
            });

            // 路径和控制按钮行
            item.spawn((Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },))
                .with_children(|row| {
                    // 保存路径
                    row.spawn((
                        Text::new(format!("📁 {}", task.save_path)),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));

                    // 控制按钮组
                    row.spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },))
                        .with_children(|btns| {
                            // 暂停按钮（始终创建，通过 display 控制可见性）
                            spawn_control_button_with_display(
                                btns,
                                font,
                                "\u{F03E4}", // 󰏤 nf-md-pause
                                PauseDownloadButton {
                                    comic_id: task.comic_id.clone(),
                                },
                                Color::srgb(0.8, 0.6, 0.2),
                                can_pause,
                            );

                            // 继续按钮（暂停状态显示）
                            spawn_control_button_with_display(
                                btns,
                                font,
                                "\u{F040A}", // 󰐊 nf-md-play
                                ResumeDownloadButton {
                                    comic_id: task.comic_id.clone(),
                                },
                                Color::srgb(0.3, 0.7, 0.3),
                                can_resume,
                            );

                            // 重试按钮（失败状态显示）
                            spawn_control_button_with_display(
                                btns,
                                font,
                                "\u{F0453}", // 󰑓 nf-md-refresh
                                RetryDownloadButton {
                                    comic_id: task.comic_id.clone(),
                                },
                                Color::srgb(0.7, 0.5, 0.2),
                                can_retry,
                            );

                            // 删除按钮（始终可见）
                            spawn_control_button_with_display(
                                btns,
                                font,
                                "\u{F01B4}", // 󰆴 nf-md-delete
                                DeleteDownloadButton {
                                    comic_id: task.comic_id.clone(),
                                },
                                Color::srgb(0.8, 0.3, 0.3),
                                can_delete,
                            );
                        });
                });
        });
}

/// 创建控制按钮（带可见性控制）
fn spawn_control_button_with_display<T: Component>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon: &str,
    marker: T,
    color: Color,
    visible: bool,
) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                display: if visible {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
            BackgroundColor(color.with_alpha(0.2)),
            BorderColor::all(color),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(icon),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(color),
            ));
        });
}

/// 创建已下载漫画项
fn spawn_completed_download_item(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    download: &CompletedDownload,
) {
    let path = download.path.clone();
    let has_comic_id = !download.comic_id.is_empty();

    parent
        .spawn((
            CompletedDownloadItem {
                comic_id: download.comic_id.clone(),
                path: download.path.clone(),
            },
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(1.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
            BorderColor::all(AppColors::BORDER),
            BorderRadius::all(Val::Px(8.0)),
            Transform::default(),
        ))
        .with_children(|item| {
            // 图标
            item.spawn((
                Text::new("📖"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(AppColors::TEXT),
            ));

            // 信息列
            item.spawn((Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(4.0),
                ..default()
            },))
                .with_children(|info| {
                    // 漫画名称
                    info.spawn((
                        Text::new(download.folder_name.clone()),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));

                    // 章节数和路径
                    info.spawn((
                        Text::new(format!("{} 章节 • {}", download.episode_count, path)),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_SECONDARY),
                    ));
                });

            // 按钮组
            item.spawn((Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },))
                .with_children(|btns| {
                    // 重新下载/更新按钮（只有有 comic_id 的才能重新下载）
                    if has_comic_id {
                        btns.spawn((
                            RedownloadButton {
                                comic_id: download.comic_id.clone(),
                            },
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.3).with_alpha(0.2)),
                            BorderColor::all(Color::srgb(0.3, 0.7, 0.4)),
                            BorderRadius::all(Val::Px(4.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F04E6}"), // 󰓦 nf-md-sync
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.3, 0.8, 0.4)),
                            ));
                        });
                    }

                    // 打开文件夹按钮
                    btns.spawn((
                        OpenCompletedFolderButton {
                            path: download.path.clone(),
                        },
                        Button,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.4).with_alpha(0.2)),
                        BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                        BorderRadius::all(Val::Px(4.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("\u{F0770}"), // 󰝰 nf-md-folder_open
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));
                    });
                });
        });
}

/// 创建下载页面滚动条
///
/// 布局结构（与其他页面保持一致）：
/// ScrollbarContainer (Absolute, right=0)
///   ├── ScrollbarTrack (Absolute, fills 100%, ZIndex=0)
///   └── ScrollbarThumb (Absolute, ZIndex=1)
///
/// 滑块和轨道作为兄弟节点，避免父子节点交互事件冲突
fn spawn_downloads_scrollbar(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
    const SCROLLBAR_WIDTH: f32 = 12.0;

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
            Transform::default(),
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
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.3)),
                ZIndex(0),
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
                    height: Val::Px(30.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
                BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                ZIndex(1),
            ));
        });
}

/// 清理下载页面
pub fn cleanup_downloads_ui(mut commands: Commands, query: Query<Entity, With<DownloadsRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 打开下载文件夹按钮交互
pub fn open_download_folder_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<OpenDownloadFolderButton>),
    >,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));

                // 打开下载文件夹
                let download_path = get_download_base_path();

                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg(&download_path)
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg(&download_path)
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&download_path)
                        .spawn();
                }

                tracing::info!("打开下载文件夹: {:?}", download_path);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}

/// 更新下载任务列表 UI（监听状态变化并实时更新进度条、
/// 状态文本和按钮可见性）（FSM 架构）
pub fn refresh_downloads_ui(
    download_state: Res<DownloadManagerState>,
    mut progress_bar_query: Query<(&DownloadProgressBar, &mut Node)>,
    mut status_text_query: Query<(&DownloadStatusText, &mut Text, &mut TextColor)>,
    mut pause_btn_query: Query<(&PauseDownloadButton, &mut Node), Without<DownloadProgressBar>>,
    mut resume_btn_query: Query<
        (&ResumeDownloadButton, &mut Node),
        (Without<DownloadProgressBar>, Without<PauseDownloadButton>),
    >,
    mut retry_btn_query: Query<
        (&RetryDownloadButton, &mut Node),
        (
            Without<DownloadProgressBar>,
            Without<PauseDownloadButton>,
            Without<ResumeDownloadButton>,
        ),
    >,
    mut delete_btn_query: Query<
        (&DeleteDownloadButton, &mut Node),
        (
            Without<DownloadProgressBar>,
            Without<PauseDownloadButton>,
            Without<ResumeDownloadButton>,
            Without<RetryDownloadButton>,
        ),
    >,
) {
    // 只在状态变化时更新
    if !download_state.is_changed() {
        return;
    }

    // 从 FSM 任务列表获取 UI 任务
    let tasks = download_state.tasks();

    // 更新每个任务的进度条、状态文本和按钮可见性
    for task in &tasks {
        // 计算进度百分比
        let progress = if task.total_episodes > 0 {
            let episode_progress = (task.current_episode - 1) as f32 / task.total_episodes as f32;
            let page_progress = if task.total_pages > 0 {
                task.current_page as f32 / task.total_pages as f32 / task.total_episodes as f32
            } else {
                0.0
            };
            (episode_progress + page_progress).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // 更新进度条宽度
        for (bar, mut node) in progress_bar_query.iter_mut() {
            if bar.comic_id == task.comic_id {
                node.width = Val::Percent(progress * 100.0);
            }
        }

        // 更新状态文本
        let (status_text, status_color) = match &task.status {
            ComicDownloadStatus::Waiting => ("等待中".to_string(), AppColors::TEXT_SECONDARY),
            ComicDownloadStatus::Downloading => (
                format!(
                    "下载中 第{}/{}章 {}/{}",
                    task.current_episode, task.total_episodes, task.current_page, task.total_pages
                ),
                AppColors::PRIMARY,
            ),
            ComicDownloadStatus::Paused => ("已暂停".to_string(), Color::srgb(0.8, 0.6, 0.2)),
            ComicDownloadStatus::Completed => ("已完成".to_string(), Color::srgb(0.3, 0.8, 0.3)),
            ComicDownloadStatus::Failed(err) => {
                (format!("失败: {}", err), Color::srgb(0.8, 0.3, 0.3))
            }
        };

        for (text_marker, mut text, mut color) in status_text_query.iter_mut() {
            if text_marker.comic_id == task.comic_id {
                **text = status_text.clone();
                *color = TextColor(status_color);
            }
        }

        // 计算按钮可见性
        let can_pause = matches!(
            task.status,
            ComicDownloadStatus::Downloading | ComicDownloadStatus::Waiting
        );
        let can_resume = matches!(task.status, ComicDownloadStatus::Paused);
        let can_retry = matches!(task.status, ComicDownloadStatus::Failed(_));
        // 删除按钮始终可见
        let can_delete = true;

        // 更新暂停按钮可见性
        for (btn, mut node) in pause_btn_query.iter_mut() {
            if btn.comic_id == task.comic_id {
                node.display = if can_pause {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }

        // 更新继续按钮可见性
        for (btn, mut node) in resume_btn_query.iter_mut() {
            if btn.comic_id == task.comic_id {
                node.display = if can_resume {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }

        // 更新重试按钮可见性
        for (btn, mut node) in retry_btn_query.iter_mut() {
            if btn.comic_id == task.comic_id {
                node.display = if can_retry {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }

        // 更新删除按钮可见性
        for (btn, mut node) in delete_btn_query.iter_mut() {
            if btn.comic_id == task.comic_id {
                node.display = if can_delete {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }

    tracing::debug!(
        "下载状态更新: {} 个任务, {} 个正在下载",
        download_state.fsm_tasks.len(),
        download_state.downloading_ids.len()
    );
}

/// 动态添加新任务的 UI（检测没有 UI 的任务并创建）
pub fn add_new_task_ui(
    mut commands: Commands,
    download_state: Res<DownloadManagerState>,
    task_list_query: Query<Entity, With<DownloadTaskList>>,
    mut section_query: Query<&mut Node, With<DownloadingSection>>,
    mut title_query: Query<&mut Text, With<DownloadingTitleText>>,
    existing_items_query: Query<&DownloadTaskItem>,
    asset_server: Res<AssetServer>,
) {
    // 只在状态变化时检查
    if !download_state.is_changed() {
        return;
    }

    // 获取所有活跃任务
    let active_tasks: Vec<_> = download_state
        .active_tasks()
        .into_iter()
        .map(|fsm| fsm.to_ui_task())
        .collect();

    if active_tasks.is_empty() {
        return;
    }

    // 获取已存在 UI 的任务 ID
    let existing_ids: std::collections::HashSet<_> = existing_items_query
        .iter()
        .map(|item| item.comic_id.clone())
        .collect();

    // 找出没有 UI 的新任务
    let new_tasks: Vec<_> = active_tasks
        .iter()
        .filter(|task| !existing_ids.contains(&task.comic_id))
        .collect();

    if new_tasks.is_empty() {
        return;
    }

    // 显示"下载中"区域（如果之前是隐藏的）
    if let Ok(mut section_node) = section_query.single_mut() {
        if section_node.display == Display::None {
            section_node.display = Display::Flex;
            tracing::info!("显示下载中区域");
        }
    }

    // 更新标题文本
    if let Ok(mut title_text) = title_query.single_mut() {
        **title_text = format!("📥 下载中 ({})", active_tasks.len());
    }

    // 获取任务列表容器
    let Ok(list_entity) = task_list_query.single() else {
        tracing::warn!(
            "没有找到下载任务列表容器，需要 {} 个新任务 UI",
            new_tasks.len()
        );
        return;
    };

    let font: Handle<Font> = asset_server.load(crate::systems::login::FONT_PATH);

    // 为每个新任务添加 UI
    for task in new_tasks {
        tracing::info!("动态添加下载任务 UI: {}", task.comic_title);

        let task_entity = commands
            .spawn((
                DownloadTaskItem {
                    comic_id: task.comic_id.clone(),
                },
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
                BorderColor::all(AppColors::BORDER),
                BorderRadius::all(Val::Px(4.0)),
            ))
            .with_children(|item| {
                // 标题和按钮行
                item.spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // 标题
                    row.spawn((
                        Text::new(&task.comic_title),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT),
                    ));

                    // 按钮容器
                    row.spawn(Node {
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|btns| {
                        // 暂停按钮
                        btns.spawn((
                            PauseDownloadButton {
                                comic_id: task.comic_id.clone(),
                            },
                            Button,
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                display: Display::Flex,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.5, 0.2)),
                            BorderRadius::all(Val::Px(3.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F03E4}"), // 󰏤 nf-md-pause
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });

                        // 继续按钮（暂停状态）
                        btns.spawn((
                            ResumeDownloadButton {
                                comic_id: task.comic_id.clone(),
                            },
                            Button,
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
                            BorderRadius::all(Val::Px(3.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F040A}"), // 󰐊 nf-md-play
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });

                        // 重试按钮（失败状态）
                        btns.spawn((
                            RetryDownloadButton {
                                comic_id: task.comic_id.clone(),
                            },
                            Button,
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.7, 0.5, 0.2)),
                            BorderRadius::all(Val::Px(3.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F0453}"), // 󰑓 nf-md-refresh
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });

                        // 删除按钮（始终可见）
                        btns.spawn((
                            DeleteDownloadButton {
                                comic_id: task.comic_id.clone(),
                            },
                            Button,
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                display: Display::Flex, // 始终可见
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                            BorderRadius::all(Val::Px(3.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F01B4}"), // 󰆴 nf-md-delete
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                    });
                });

                // 进度条容器
                item.spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgb(0.2, 0.2, 0.25)))
                .insert(BorderRadius::all(Val::Px(2.0)))
                .with_children(|bar_bg| {
                    bar_bg.spawn((
                        DownloadProgressBar {
                            comic_id: task.comic_id.clone(),
                        },
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(AppColors::PRIMARY),
                        BorderRadius::all(Val::Px(2.0)),
                    ));
                });

                // 状态文本
                item.spawn((
                    DownloadStatusText {
                        comic_id: task.comic_id.clone(),
                    },
                    Text::new("准备中..."),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));
            })
            .id();

        // 添加到任务列表容器
        commands.entity(list_entity).add_child(task_entity);
    }
}

/// 更新下载页面标题数字（基于实际 UI 元素数量）
pub fn update_download_titles(
    task_items_query: Query<&DownloadTaskItem>,
    completed_items_query: Query<&CompletedDownloadItem>,
    mut downloading_title_query: Query<
        &mut Text,
        (With<DownloadingTitleText>, Without<CompletedTitleText>),
    >,
    mut completed_title_query: Query<
        &mut Text,
        (With<CompletedTitleText>, Without<DownloadingTitleText>),
    >,
    mut downloading_section_query: Query<
        &mut Node,
        (With<DownloadingSection>, Without<CompletedSection>),
    >,
    mut completed_section_query: Query<
        &mut Node,
        (With<CompletedSection>, Without<DownloadingSection>),
    >,
) {
    let task_count = task_items_query.iter().count();
    let completed_count = completed_items_query.iter().count();

    // 更新下载中标题和区域显示状态
    if let Ok(mut title) = downloading_title_query.single_mut() {
        let new_text = format!("📥 下载中 ({})", task_count);
        if **title != new_text {
            **title = new_text;
        }
    }
    if let Ok(mut section) = downloading_section_query.single_mut() {
        let should_display = task_count > 0;
        let new_display = if should_display {
            Display::Flex
        } else {
            Display::None
        };
        if section.display != new_display {
            section.display = new_display;
        }
    }

    // 更新已下载标题和区域显示状态
    if let Ok(mut title) = completed_title_query.single_mut() {
        let new_text = format!("📚 已下载 ({})", completed_count);
        if **title != new_text {
            **title = new_text;
        }
    }
    if let Ok(mut section) = completed_section_query.single_mut() {
        let should_display = completed_count > 0;
        let new_display = if should_display {
            Display::Flex
        } else {
            Display::None
        };
        if section.display != new_display {
            section.display = new_display;
        }
    }
}

/// 处理下载页面滚动
pub fn handle_downloads_scroll(
    mut scroll_query: Query<
        (&mut ScrollPosition, Option<&ContentSizeInfo>),
        With<DownloadsScrollContainer>,
    >,
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        let scroll_delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        };

        for (mut scroll_pos, content_info) in scroll_query.iter_mut() {
            let max_scroll = content_info
                .map(|info| (info.content_height - info.viewport_height).max(0.0))
                .unwrap_or(0.0);
            scroll_pos.y = (scroll_pos.y - scroll_delta).clamp(0.0, max_scroll);
        }
    }
}

/// 更新下载页面内容尺寸
pub fn update_downloads_content_size(
    mut scroll_query: Query<
        (&ComputedNode, &mut ContentSizeInfo, &Children),
        With<DownloadsScrollContainer>,
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

/// 已下载项点击交互（点击打开文件夹）
pub fn completed_download_item_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &CompletedDownloadItem),
        Changed<Interaction>,
    >,
) {
    for (interaction, mut bg_color, item) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.08, 0.08, 0.12));

                // 打开文件夹
                let path = std::path::Path::new(&item.path);
                if path.exists() {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("explorer").arg(path).spawn();
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open").arg(path).spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                    }
                    tracing::info!("打开已下载漫画文件夹: {:?}", path);
                } else {
                    tracing::warn!("文件夹不存在: {:?}", path);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.14));
            }
        }
    }
}

/// 暂停下载按钮交互（FSM 架构）
pub fn pause_download_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &PauseDownloadButton),
        Changed<Interaction>,
    >,
    download_state: Res<DownloadManagerState>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let pause_color = Color::srgb(0.8, 0.6, 0.2);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(pause_color.with_alpha(0.4));

                // 设置暂停标志（通过 FSM 控制器）
                if let Some(fsm) = download_state.find_task(&btn.comic_id) {
                    fsm.request_pause();
                    tracing::info!("设置暂停标志: {}", btn.comic_id);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(pause_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(pause_color.with_alpha(0.2));
            }
        }
    }
}

/// 继续下载按钮交互
pub fn resume_download_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ResumeDownloadButton),
        Changed<Interaction>,
    >,
    mut resume_messages: MessageWriter<ResumeDownloadRequest>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let resume_color = Color::srgb(0.3, 0.7, 0.3);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(resume_color.with_alpha(0.4));

                // 发送恢复下载请求
                resume_messages.write(ResumeDownloadRequest {
                    comic_id: btn.comic_id.clone(),
                });
                tracing::info!("请求继续下载: {}", btn.comic_id);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(resume_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(resume_color.with_alpha(0.2));
            }
        }
    }
}

/// 删除下载按钮交互（FSM 架构）
pub fn delete_download_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &DeleteDownloadButton),
        Changed<Interaction>,
    >,
    mut download_state: ResMut<DownloadManagerState>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let delete_color = Color::srgb(0.8, 0.3, 0.3);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(delete_color.with_alpha(0.4));

                // 删除元数据文件
                if let Some(fsm) = download_state.find_task(&btn.comic_id) {
                    if let Err(e) = fsm.meta.delete() {
                        tracing::warn!("删除元数据文件失败: {}", e);
                    }
                }

                // 从任务列表中删除
                download_state.remove_task(&btn.comic_id);
                tracing::info!("删除下载任务: {}", btn.comic_id);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(delete_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(delete_color.with_alpha(0.2));
            }
        }
    }
}

/// 重试下载按钮交互（失败状态下的重试）
pub fn retry_download_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &RetryDownloadButton),
        Changed<Interaction>,
    >,
    mut resume_messages: MessageWriter<ResumeDownloadRequest>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let retry_color = Color::srgb(0.7, 0.5, 0.2);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(retry_color.with_alpha(0.4));

                // 发送恢复下载请求（重试本质上就是恢复）
                resume_messages.write(ResumeDownloadRequest {
                    comic_id: btn.comic_id.clone(),
                });
                tracing::info!("请求重试下载: {}", btn.comic_id);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(retry_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(retry_color.with_alpha(0.2));
            }
        }
    }
}

/// 重新下载按钮交互
pub fn redownload_button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &RedownloadButton),
        Changed<Interaction>,
    >,
    completed_item_query: Query<(Entity, &CompletedDownloadItem)>,
    mut redownload_messages: MessageWriter<RedownloadRequest>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let redownload_color = Color::srgb(0.3, 0.7, 0.4);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(redownload_color.with_alpha(0.4));

                // 发送重新下载请求
                redownload_messages.write(RedownloadRequest {
                    comic_id: btn.comic_id.clone(),
                });
                tracing::info!("请求重新下载/检查更新: {}", btn.comic_id);

                // 从已下载列表中移除该项目（避免重复）
                for (entity, item) in completed_item_query.iter() {
                    if item.comic_id == btn.comic_id {
                        commands.entity(entity).despawn();
                        tracing::debug!("已从已下载列表移除: {}", btn.comic_id);
                        break;
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(redownload_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(redownload_color.with_alpha(0.2));
            }
        }
    }
}

/// 打开已下载漫画文件夹按钮交互
pub fn open_completed_folder_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &OpenCompletedFolderButton,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let folder_color = Color::srgb(0.5, 0.5, 0.6);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(folder_color.with_alpha(0.4));

                // 打开文件夹
                let path = std::path::Path::new(&btn.path);
                if path.exists() {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("explorer").arg(path).spawn();
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open").arg(path).spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                    }
                    tracing::info!("打开已下载漫画文件夹: {:?}", path);
                } else {
                    tracing::warn!("文件夹不存在: {:?}", path);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(folder_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(folder_color.with_alpha(0.2));
            }
        }
    }
}

/// 处理下载完成后的 UI 更新
/// 将任务从下载中列表移动到已下载列表
pub fn handle_download_completed_ui(
    mut commands: Commands,
    mut messages: MessageReader<DownloadCompletedEvent>,
    download_state: Res<DownloadManagerState>,
    task_item_query: Query<(Entity, &DownloadTaskItem)>,
    completed_item_query: Query<(Entity, &CompletedDownloadItem)>,
    completed_list_query: Query<Entity, With<CompletedDownloadList>>,
    asset_server: Res<AssetServer>,
) {
    for event in messages.read() {
        let comic_id = &event.comic_id;
        let save_path = &event.save_path;

        tracing::info!("UI: 下载完成，移动任务到已下载列表: {}", comic_id);

        // 1. 找到并移除下载中的任务 UI
        for (entity, task_item) in task_item_query.iter() {
            if task_item.comic_id == *comic_id {
                commands.entity(entity).despawn();
                tracing::debug!("已移除下载任务 UI: {}", comic_id);
                break;
            }
        }

        // 2. 检查是否已存在于已下载列表（避免重复）
        let mut already_exists = false;
        for (entity, item) in completed_item_query.iter() {
            if item.comic_id == *comic_id {
                // 已存在，移除旧的（稍后会添加新的）
                commands.entity(entity).despawn();
                tracing::debug!("移除已存在的已下载项: {}", comic_id);
                already_exists = true;
                break;
            }
        }
        let _ = already_exists; // 标记使用

        // 3. 从 FSM 状态中获取漫画标题
        let comic_title = download_state
            .find_task(comic_id)
            .map(|fsm| fsm.meta.comic_title.clone())
            .unwrap_or_else(|| {
                // 如果找不到，从保存路径提取文件夹名
                std::path::Path::new(save_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知漫画")
                    .to_string()
            });

        let episode_count = download_state
            .find_task(comic_id)
            .map(|fsm| fsm.meta.episode_orders.len())
            .unwrap_or(0);

        // 4. 添加到已下载列表
        if let Ok(list_entity) = completed_list_query.single() {
            let font: Handle<Font> = asset_server.load(FONT_PATH);
            let download = CompletedDownload {
                comic_id: comic_id.clone(),
                folder_name: comic_title,
                path: save_path.clone(),
                episode_count,
            };

            commands.entity(list_entity).with_children(|parent| {
                spawn_completed_download_item(parent, &font, &download);
            });

            tracing::info!("已添加到已下载列表: {}", comic_id);
        } else {
            tracing::warn!("未找到已下载列表容器，可能不在下载页面");
        }
    }
}

/// "开始全部下载" 按钮交互
/// 恢复所有暂停状态的下载任务
pub fn start_all_downloads_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartAllDownloadsButton>),
    >,
    download_state: Res<DownloadManagerState>,
    mut resume_messages: MessageWriter<ResumeDownloadRequest>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        let btn_color = Color::srgb(0.2, 0.5, 0.3);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(btn_color.lighter(0.1));

                // 恢复所有暂停状态的任务
                let mut resumed_count = 0;
                for fsm in &download_state.fsm_tasks {
                    let task = fsm.to_ui_task();
                    if matches!(
                        task.status,
                        crate::resources::ComicDownloadStatus::Paused
                            | crate::resources::ComicDownloadStatus::Waiting
                    ) {
                        resume_messages.write(ResumeDownloadRequest {
                            comic_id: task.comic_id.clone(),
                        });
                        resumed_count += 1;
                    }
                }

                if resumed_count > 0 {
                    tracing::info!("开始全部下载: 恢复 {} 个任务", resumed_count);
                } else {
                    tracing::info!("没有需要恢复的下载任务");
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(btn_color.lighter(0.05));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(btn_color);
            }
        }
    }
}
