//! 下载管理界面系统
//!
//! 实现下载任务列表和进度显示

#![allow(dead_code)]

use bevy::prelude::*;

use crate::{
    components::{ContentArea, ContentSizeInfo, ScrollbarThumb, ScrollbarTrack},
    config::settings::AppSettings,
    events::{RedownloadRequest, ResumeDownloadRequest},
    resources::{ComicDownloadStatus, DownloadManagerState},
    systems::login::{AppColors, FONT_PATH},
};

/// 下载滚动容器组件（本地定义）
#[derive(Component)]
pub struct ScrollContainer;

/// 获取下载保存路径
fn get_download_base_path() -> std::path::PathBuf {
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
/// 通过检查 `.download_meta.json` 元数据文件来判断下载是否真正完成。
/// 如果元数据显示 `Completed` 状态，则认为下载完成。
/// 如果没有元数据文件但有章节子文件夹，也认为是已完成（兼容旧数据）。
///
/// `downloading_ids`: 正在下载中的漫画 ID 列表，用于过滤避免重复显示
fn scan_completed_downloads(
    downloading_ids: &std::collections::HashSet<String>,
) -> Vec<CompletedDownload> {
    use crate::resources::DownloadTaskMeta;

    let download_path = get_download_base_path();
    let mut downloads = Vec::new();

    if !download_path.exists() {
        tracing::debug!("下载目录不存在: {:?}", download_path);
        return downloads;
    }

    // 遍历下载目录中的子文件夹
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

                // 检查是否有元数据文件
                if let Ok(meta) = DownloadTaskMeta::load(&path_str) {
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
                    }
                    // 未完成的不显示在已下载列表（会显示在下载中列表）
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
                        flex_direction: FlexDirection::Row,
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
                            if has_active_tasks {
                                // 进行中标题
                                scroll.spawn((
                                    Text::new(format!("📥 下载中 ({})", active_tasks.len())),
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

                                scroll
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
                            }

                            // 已下载列表
                            if !completed_downloads.is_empty() {
                                // 已下载标题
                                scroll.spawn((
                                    Text::new(format!("📚 已下载 ({})", completed_downloads.len())),
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

                                scroll
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
                                            spawn_completed_download_item(list, &font, download);
                                        }
                                    });
                            }

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
                        Text::new("📁 打开文件夹"),
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

    // 判断是否可以暂停/继续
    let can_pause = matches!(
        task.status,
        ComicDownloadStatus::Downloading | ComicDownloadStatus::Waiting
    );
    let can_resume = matches!(
        task.status,
        ComicDownloadStatus::Paused | ComicDownloadStatus::Failed(_)
    );
    let can_delete = !matches!(task.status, ComicDownloadStatus::Downloading);

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
                                "⏸",
                                PauseDownloadButton {
                                    comic_id: task.comic_id.clone(),
                                },
                                Color::srgb(0.8, 0.6, 0.2),
                                can_pause,
                            );

                            // 继续按钮（始终创建，通过 display 控制可见性）
                            spawn_control_button_with_display(
                                btns,
                                font,
                                "▶",
                                ResumeDownloadButton {
                                    comic_id: task.comic_id.clone(),
                                },
                                Color::srgb(0.3, 0.7, 0.3),
                                can_resume,
                            );

                            // 删除按钮（始终创建，通过 display 控制可见性）
                            spawn_control_button_with_display(
                                btns,
                                font,
                                "✕",
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
                                Text::new("🔄"),
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
                            Text::new("📁"),
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
fn spawn_downloads_scrollbar(parent: &mut ChildSpawnerCommands, scroll_container: Entity) {
    parent
        .spawn((
            Node {
                width: Val::Px(12.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
        ))
        .with_children(|scrollbar| {
            scrollbar
                .spawn((
                    ScrollbarTrack { scroll_container },
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.3)),
                    Transform::default(),
                ))
                .with_children(|track| {
                    track.spawn((
                        ScrollbarThumb { scroll_container },
                        Button,
                        Interaction::default(),
                        Node {
                            width: Val::Px(8.0),
                            height: Val::Px(50.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
                        BorderRadius::all(Val::Px(4.0)),
                        Transform::default(),
                    ));
                });
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
    mut delete_btn_query: Query<
        (&DeleteDownloadButton, &mut Node),
        (
            Without<DownloadProgressBar>,
            Without<PauseDownloadButton>,
            Without<ResumeDownloadButton>,
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
        let can_resume = matches!(
            task.status,
            ComicDownloadStatus::Paused | ComicDownloadStatus::Failed(_)
        );
        let can_delete = !matches!(task.status, ComicDownloadStatus::Downloading);

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

/// 重新下载按钮交互
pub fn redownload_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &RedownloadButton),
        Changed<Interaction>,
    >,
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
