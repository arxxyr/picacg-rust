//! 下载管理界面系统
//!
//! 实现下载任务列表和进度显示

#![allow(dead_code)]

use bevy::{prelude::*, ui::FocusPolicy};
use picacg_config::AppSettings;

use super::font_loader::get_font;
use crate::{
    components::{
        ContentArea, ContentSizeInfo, ScrollbarContainer, ScrollbarThumb, ScrollbarTrack,
    },
    events::{
        DownloadCompletedEvent, NavigateToComicDetailEvent, NavigateToComicsListEvent,
        RedownloadRequest, ResumeDownloadRequest, SearchComicsRequestEvent,
    },
    resources::{AppRoute, ComicDownloadStatus, DownloadManagerState, SearchState},
    systems::{login::AppColors, navigation::NavigationHistory},
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
    /// 分类列表
    pub categories: Vec<String>,
    /// 标签列表
    pub tags: Vec<String>,
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

/// "等待中" 区域容器（排队等待下载的任务）
#[derive(Component)]
pub struct WaitingSection;

/// "等待中" 标题文本
#[derive(Component)]
pub struct WaitingTitleText;

/// "等待中" 任务列表容器
#[derive(Component)]
pub struct WaitingTaskList;

/// "已停止" 区域容器（暂停和失败的任务）
#[derive(Component)]
pub struct StoppedSection;

/// "已停止" 标题文本
#[derive(Component)]
pub struct StoppedTitleText;

/// "已停止" 任务列表容器
#[derive(Component)]
pub struct StoppedTaskList;

/// 下载区域折叠状态资源
#[derive(Resource)]
pub struct DownloadSectionCollapseState {
    /// 下载中区域是否折叠
    pub downloading_collapsed: bool,
    /// 等待中区域是否折叠
    pub waiting_collapsed: bool,
    /// 已停止区域是否折叠
    pub stopped_collapsed: bool,
    /// 已下载区域是否折叠
    pub completed_collapsed: bool,
}

impl Default for DownloadSectionCollapseState {
    fn default() -> Self {
        Self {
            downloading_collapsed: false, // 下载中默认展开
            waiting_collapsed: true,      // 等待中默认折叠
            stopped_collapsed: true,      // 已停止默认折叠
            completed_collapsed: true,    // 已下载默认折叠
        }
    }
}

/// 区域类型（用于折叠交互）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Downloading,
    Waiting,
    Stopped,
    Completed,
}

/// 可折叠的区域标题按钮
#[derive(Component)]
pub struct CollapsibleSectionHeader {
    pub section_type: SectionType,
}

/// 折叠图标标记
#[derive(Component)]
pub struct CollapseIcon {
    pub section_type: SectionType,
}

/// 区域内容容器标记（用于折叠/展开）
#[derive(Component)]
pub struct SectionContent {
    pub section_type: SectionType,
}

/// 浮动标题容器（固定在滚动区域顶部）
#[derive(Component)]
pub struct FloatingHeader;

/// 浮动标题文本
#[derive(Component)]
pub struct FloatingHeaderText;

/// 浮动标题折叠图标
#[derive(Component)]
pub struct FloatingHeaderIcon;

/// 浮动标题按钮（可点击折叠）
#[derive(Component)]
pub struct FloatingHeaderButton {
    pub section_type: Option<SectionType>,
}

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

/// 扫描已完成的下载列表
///
/// 从数据库加载已完成的下载任务。
///
/// `active_task_ids`: 所有活跃任务（包括下载中、等待中、已停止）的漫画 ID
/// 列表，用于过滤避免重复显示
fn scan_completed_downloads(
    active_task_ids: &std::collections::HashSet<String>,
) -> Vec<CompletedDownload> {
    use picacg_db::{get_completed_download_tasks_async, get_pool, run_db_operation};

    let mut downloads = Vec::new();
    let pool = get_pool();

    // 从数据库加载已完成的下载任务
    let db_tasks = run_db_operation(async move { get_completed_download_tasks_async(&pool).await })
        .unwrap_or_default();

    for db_task in db_tasks {
        // 跳过活跃任务（下载中、等待中、已停止的）
        if active_task_ids.contains(&db_task.comic_id) {
            tracing::debug!("跳过活跃任务: {}", db_task.comic_title);
            continue;
        }

        // 从 save_path 提取文件夹名称
        let folder_name = std::path::Path::new(&db_task.save_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&db_task.comic_title)
            .to_string();

        let episode_count = db_task.get_episode_orders().len();

        let categories = db_task.get_categories();
        let tags = db_task.get_tags();
        downloads.push(CompletedDownload {
            comic_id: db_task.comic_id,
            folder_name,
            episode_count,
            path: db_task.save_path,
            categories,
            tags,
        });
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

/// 下载任务分类标签容器标记
#[derive(Component)]
pub struct DownloadTaskTagsContainer {
    pub comic_id: String,
}

/// 标签颜色类型
enum TagColor {
    /// 分类（蓝色）
    Category,
    /// 标签（绿色）
    Tag,
}

/// 下载列表中可点击的标题（跳转到漫画详情）
#[derive(Component)]
pub struct DownloadTitleButton {
    pub comic_id: String,
}

/// 下载列表中可点击的分类标签（跳转到分类列表）
#[derive(Component)]
pub struct DownloadCategoryTag {
    pub category: String,
}

/// 下载列表中可点击的标签（跳转到搜索）
#[derive(Component)]
pub struct DownloadTagButton {
    pub tag: String,
}

/// 创建标签徽章（可点击）
fn spawn_tag_badge(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font: &Handle<Font>,
    color_type: TagColor,
) {
    let (bg_color, text_color) = match color_type {
        TagColor::Category => (Color::srgba(0.2, 0.4, 0.8, 0.3), Color::srgb(0.6, 0.8, 1.0)),
        TagColor::Tag => (Color::srgba(0.2, 0.6, 0.4, 0.3), Color::srgb(0.5, 0.9, 0.7)),
    };

    // 截断过长的文本
    let display_text = if text.chars().count() > 8 {
        format!("{}...", text.chars().take(6).collect::<String>())
    } else {
        text.to_string()
    };

    let text_owned = text.to_string();

    let mut entity_commands = parent.spawn((
        Button,
        Interaction::default(),
        Node {
            padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(2.0), Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(bg_color),
    ));

    // 根据类型添加不同的组件
    match color_type {
        TagColor::Category => {
            entity_commands.insert(DownloadCategoryTag {
                category: text_owned,
            });
        }
        TagColor::Tag => {
            entity_commands.insert(DownloadTagButton { tag: text_owned });
        }
    }

    entity_commands.with_children(|badge| {
        badge.spawn((
            Text::new(display_text),
            TextFont {
                font: font.clone(),
                font_size: 10.0,
                ..default()
            },
            TextColor(text_color),
        ));
    });
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

/// 打开 CBZ 文件夹按钮标记
#[derive(Component)]
pub struct OpenCbzFolderButton;

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

/// 删除已下载漫画按钮标记
#[derive(Component)]
pub struct DeleteCompletedDownloadButton {
    pub comic_id: String,
    pub path: String,
}

/// 删除确认面板标记
#[derive(Component)]
pub struct DeleteConfirmPanel {
    pub comic_id: String,
    pub path: String,
}

/// "同时删除磁盘文件" 勾选框标记
#[derive(Component)]
pub struct DeleteFilesCheckbox {
    pub comic_id: String,
    pub checked: bool,
}

/// 确认删除按钮标记
#[derive(Component)]
pub struct ConfirmDeleteButton {
    pub comic_id: String,
    pub path: String,
}

/// 取消删除按钮标记
#[derive(Component)]
pub struct CancelDeleteButton {
    pub comic_id: String,
}

/// 下载任务独立设置按钮标记
#[derive(Component)]
pub struct DownloadTaskSettingsButton {
    pub comic_id: String,
}

/// 下载任务独立设置面板标记
#[derive(Component)]
pub struct DownloadTaskSettingsPanel {
    pub comic_id: String,
}

/// 下载任务独立路径选择按钮
#[derive(Component)]
pub struct TaskPathSelectButton {
    pub comic_id: String,
}

/// 下载任务独立 CBZ 开关
#[derive(Component)]
pub struct TaskCbzToggle {
    pub comic_id: String,
}

/// 加载未完成的下载任务（进入下载页面时调用）
pub fn load_incomplete_downloads(mut download_state: ResMut<DownloadManagerState>) {
    download_state.load_incomplete_tasks();
}

/// 创建下载页面 UI
pub fn setup_downloads_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
    download_state: Res<DownloadManagerState>,
    collapse_state: Res<DownloadSectionCollapseState>,
) {
    let font: Handle<Font> = get_font();

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
                Transform::default(), // 必须添加，否则子实体的 GlobalTransform 会报警告
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
                            // 获取所有任务并按状态分类
                            let all_tasks: Vec<_> = download_state
                                .active_tasks()
                                .into_iter()
                                .map(|fsm| fsm.to_ui_task())
                                .collect();

                            // 收集所有活跃任务的 ID（用于过滤已完成列表）
                            let active_task_ids: std::collections::HashSet<String> =
                                all_tasks.iter().map(|t| t.comic_id.clone()).collect();

                            // 扫描已下载的漫画（排除所有活跃任务）
                            let completed_downloads = scan_completed_downloads(&active_task_ids);

                            // 分类任务
                            let downloading_tasks: Vec<_> = all_tasks
                                .iter()
                                .filter(|t| matches!(t.status, ComicDownloadStatus::Downloading))
                                .collect();
                            let waiting_tasks: Vec<_> = all_tasks
                                .iter()
                                .filter(|t| matches!(t.status, ComicDownloadStatus::Waiting))
                                .collect();
                            let stopped_tasks: Vec<_> = all_tasks
                                .iter()
                                .filter(|t| {
                                    matches!(
                                        t.status,
                                        ComicDownloadStatus::Paused
                                            | ComicDownloadStatus::Failed(_)
                                    )
                                })
                                .collect();

                            // 获取折叠状态
                            let downloading_collapsed = collapse_state.downloading_collapsed;
                            let waiting_collapsed = collapse_state.waiting_collapsed;
                            let stopped_collapsed = collapse_state.stopped_collapsed;
                            let completed_collapsed = collapse_state.completed_collapsed;

                            // 1. "下载中" 区域 - 始终显示标题，内容可折叠
                            scroll
                                .spawn((
                                    DownloadingSection,
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::bottom(Val::Px(10.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|section| {
                                    // 可折叠标题
                                    section
                                        .spawn((
                                            CollapsibleSectionHeader {
                                                section_type: SectionType::Downloading,
                                            },
                                            Button,
                                            Interaction::default(),
                                            Node {
                                                width: Val::Percent(100.0),
                                                align_items: AlignItems::Center,
                                                padding: UiRect::new(
                                                    Val::Px(8.0),
                                                    Val::Px(8.0),
                                                    Val::Px(6.0),
                                                    Val::Px(6.0),
                                                ),
                                                border: UiRect::all(Val::Px(1.0)),
                                                column_gap: Val::Px(8.0),
                                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                                            BorderColor::all(AppColors::BORDER),
                                        ))
                                        .with_children(|header| {
                                            // 折叠图标
                                            let icon = if downloading_collapsed {
                                                "\u{F0142}" // ▶ nf-md-chevron_right
                                            } else {
                                                "\u{F0140}" // ▼ nf-md-chevron_down
                                            };
                                            header.spawn((
                                                CollapseIcon {
                                                    section_type: SectionType::Downloading,
                                                },
                                                Text::new(icon),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT_MUTED),
                                            ));

                                            // 下载中标题
                                            header.spawn((
                                                DownloadingTitleText,
                                                Text::new(format!(
                                                    "\u{F01DA} 下载中 ({})", // 󰇚 nf-md-download
                                                    downloading_tasks.len()
                                                )),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT),
                                            ));
                                        });

                                    // 任务列表容器（可折叠内容）
                                    section
                                        .spawn((
                                            SectionContent {
                                                section_type: SectionType::Downloading,
                                            },
                                            DownloadTaskList,
                                            Node {
                                                width: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(10.0),
                                                padding: UiRect::top(Val::Px(10.0)),
                                                display: if downloading_collapsed {
                                                    Display::None
                                                } else {
                                                    Display::Flex
                                                },
                                                ..default()
                                            },
                                        ))
                                        .with_children(|list| {
                                            for task in &downloading_tasks {
                                                spawn_download_task_item(list, &font, task);
                                            }
                                        });
                                });

                            // 2. "等待中" 区域 - 排队等待下载的任务
                            scroll
                                .spawn((
                                    WaitingSection,
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::bottom(Val::Px(10.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|section| {
                                    // 可折叠标题
                                    section
                                        .spawn((
                                            CollapsibleSectionHeader {
                                                section_type: SectionType::Waiting,
                                            },
                                            Button,
                                            Interaction::default(),
                                            Node {
                                                width: Val::Percent(100.0),
                                                align_items: AlignItems::Center,
                                                padding: UiRect::new(
                                                    Val::Px(8.0),
                                                    Val::Px(8.0),
                                                    Val::Px(6.0),
                                                    Val::Px(6.0),
                                                ),
                                                border: UiRect::all(Val::Px(1.0)),
                                                column_gap: Val::Px(8.0),
                                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                                            BorderColor::all(AppColors::BORDER),
                                        ))
                                        .with_children(|header| {
                                            // 折叠图标
                                            let icon = if waiting_collapsed {
                                                "\u{F0142}" // ▶ nf-md-chevron_right
                                            } else {
                                                "\u{F0140}" // ▼ nf-md-chevron_down
                                            };
                                            header.spawn((
                                                CollapseIcon {
                                                    section_type: SectionType::Waiting,
                                                },
                                                Text::new(icon),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT_MUTED),
                                            ));

                                            // 等待中标题
                                            header.spawn((
                                                WaitingTitleText,
                                                Text::new(format!(
                                                    "\u{F0520} 等待中 ({})", // 󰔠 nf-md-timer_sand
                                                    waiting_tasks.len()
                                                )),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT),
                                            ));
                                        });

                                    // 任务列表容器（可折叠内容）
                                    section
                                        .spawn((
                                            SectionContent {
                                                section_type: SectionType::Waiting,
                                            },
                                            WaitingTaskList,
                                            Node {
                                                width: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(10.0),
                                                padding: UiRect::top(Val::Px(10.0)),
                                                display: if waiting_collapsed {
                                                    Display::None
                                                } else {
                                                    Display::Flex
                                                },
                                                ..default()
                                            },
                                        ))
                                        .with_children(|list| {
                                            for task in &waiting_tasks {
                                                spawn_download_task_item(list, &font, task);
                                            }
                                        });
                                });

                            // 3. "已停止" 区域 - 暂停和失败的任务
                            scroll
                                .spawn((
                                    StoppedSection,
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::bottom(Val::Px(10.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|section| {
                                    // 标题行容器（包含可折叠标题 + 全部开始按钮）
                                    section
                                        .spawn((
                                            Node {
                                                width: Val::Percent(100.0),
                                                justify_content: JustifyContent::SpaceBetween,
                                                align_items: AlignItems::Center,
                                                column_gap: Val::Px(10.0),
                                                ..default()
                                            },
                                            Transform::default(),
                                        ))
                                        .with_children(|header_row| {
                                            // 可折叠标题（左侧）
                                            header_row
                                                .spawn((
                                                    CollapsibleSectionHeader {
                                                        section_type: SectionType::Stopped,
                                                    },
                                                    Button,
                                                    Interaction::default(),
                                                    Node {
                                                        flex_grow: 1.0,
                                                        align_items: AlignItems::Center,
                                                        padding: UiRect::new(
                                                            Val::Px(8.0),
                                                            Val::Px(8.0),
                                                            Val::Px(6.0),
                                                            Val::Px(6.0),
                                                        ),
                                                        border: UiRect::all(Val::Px(1.0)),
                                                        column_gap: Val::Px(8.0),
                                                        border_radius: BorderRadius::all(Val::Px(
                                                            4.0,
                                                        )),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                                                    BorderColor::all(AppColors::BORDER),
                                                ))
                                                .with_children(|header| {
                                                    // 折叠图标
                                                    let icon = if stopped_collapsed {
                                                        "\u{F0142}" // ▶ nf-md-chevron_right
                                                    } else {
                                                        "\u{F0140}" // ▼ nf-md-chevron_down
                                                    };
                                                    header.spawn((
                                                        CollapseIcon {
                                                            section_type: SectionType::Stopped,
                                                        },
                                                        Text::new(icon),
                                                        TextFont {
                                                            font: font.clone(),
                                                            font_size: 14.0,
                                                            ..default()
                                                        },
                                                        TextColor(AppColors::TEXT_MUTED),
                                                    ));

                                                    // 已停止标题
                                                    header.spawn((
                                                        StoppedTitleText,
                                                        Text::new(format!(
                                                            "\u{F04DB} 已停止 ({})", // 󰓛 nf-md-stop_circle
                                                            stopped_tasks.len()
                                                        )),
                                                        TextFont {
                                                            font: font.clone(),
                                                            font_size: 16.0,
                                                            ..default()
                                                        },
                                                        TextColor(Color::srgb(0.8, 0.6, 0.2)),
                                                    ));
                                                });

                                            // 全部开始按钮（右侧）
                                            header_row
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
                                                        border_radius: BorderRadius::all(Val::Px(
                                                            4.0,
                                                        )),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                                                    BorderColor::all(Color::srgb(0.3, 0.7, 0.4)),
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

                                    // 任务列表容器（可折叠内容）
                                    section
                                        .spawn((
                                            SectionContent {
                                                section_type: SectionType::Stopped,
                                            },
                                            StoppedTaskList,
                                            Node {
                                                width: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(10.0),
                                                padding: UiRect::top(Val::Px(10.0)),
                                                display: if stopped_collapsed {
                                                    Display::None
                                                } else {
                                                    Display::Flex
                                                },
                                                ..default()
                                            },
                                        ))
                                        .with_children(|list| {
                                            for task in &stopped_tasks {
                                                spawn_download_task_item(list, &font, task);
                                            }
                                        });
                                });

                            // 4. "已下载" 区域 - 已完成的任务
                            scroll
                                .spawn((
                                    CompletedSection,
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::bottom(Val::Px(10.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|section| {
                                    // 可折叠标题
                                    section
                                        .spawn((
                                            CollapsibleSectionHeader {
                                                section_type: SectionType::Completed,
                                            },
                                            Button,
                                            Interaction::default(),
                                            Node {
                                                width: Val::Percent(100.0),
                                                align_items: AlignItems::Center,
                                                padding: UiRect::new(
                                                    Val::Px(8.0),
                                                    Val::Px(8.0),
                                                    Val::Px(6.0),
                                                    Val::Px(6.0),
                                                ),
                                                border: UiRect::all(Val::Px(1.0)),
                                                column_gap: Val::Px(8.0),
                                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                                            BorderColor::all(AppColors::BORDER),
                                        ))
                                        .with_children(|header| {
                                            // 折叠图标
                                            let icon = if completed_collapsed {
                                                "\u{F0142}" // ▶ nf-md-chevron_right
                                            } else {
                                                "\u{F0140}" // ▼ nf-md-chevron_down
                                            };
                                            header.spawn((
                                                CollapseIcon {
                                                    section_type: SectionType::Completed,
                                                },
                                                Text::new(icon),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(AppColors::TEXT_MUTED),
                                            ));

                                            // 已下载标题
                                            header.spawn((
                                                CompletedTitleText,
                                                Text::new(format!(
                                                    "\u{F012C} 已下载 ({})", // 󰄬 nf-md-check
                                                    completed_downloads.len()
                                                )),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.3, 0.8, 0.3)),
                                            ));
                                        });

                                    // 列表容器（可折叠内容）
                                    section
                                        .spawn((
                                            SectionContent {
                                                section_type: SectionType::Completed,
                                            },
                                            CompletedDownloadList,
                                            Node {
                                                width: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(10.0),
                                                padding: UiRect::top(Val::Px(10.0)),
                                                display: if completed_collapsed {
                                                    Display::None
                                                } else {
                                                    Display::Flex
                                                },
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

                            // 底部间距，确保最后的内容不会贴着窗口底部
                            scroll.spawn((
                                Node {
                                    height: Val::Px(40.0),
                                    min_height: Val::Px(40.0),
                                    ..default()
                                },
                                Transform::default(),
                            ));
                        })
                        .id();

                    // 滚动条
                    spawn_downloads_scrollbar(content_wrapper, scroll_container);

                    // 浮动标题（固定在顶部，初始隐藏）
                    spawn_floating_header(content_wrapper, &font);
                });
            });
    });

    tracing::info!("下载页面 UI 已创建");
}

/// 创建浮动标题（当区域标题滚出视口时显示）
fn spawn_floating_header(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            FloatingHeader,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(20.0),
                right: Val::Px(32.0), // 给滚动条留空间
                height: Val::Px(36.0),
                display: Display::None, // 初始隐藏，滚动时显示
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.98)),
            ZIndex(100), // 提高 ZIndex 确保可见
        ))
        .with_children(|header| {
            // 可点击按钮
            header
                .spawn((
                    FloatingHeaderButton { section_type: None },
                    Button,
                    Interaction::default(),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        column_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.98)),
                    BorderColor::all(AppColors::BORDER),
                ))
                .with_children(|btn| {
                    // 折叠图标
                    btn.spawn((
                        FloatingHeaderIcon,
                        Text::new("\u{F0140}"), // ▼ nf-md-chevron_down
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(AppColors::TEXT_MUTED),
                    ));

                    // 标题文本
                    btn.spawn((
                        FloatingHeaderText,
                        Text::new(""),
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

            // 按钮组容器
            header
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                },))
                .with_children(|btn_group| {
                    // 打开原图文件夹按钮
                    btn_group
                        .spawn((
                            OpenDownloadFolderButton,
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
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                            BorderColor::all(AppColors::BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F0770} 原图"), // 󰝰 nf-md-folder_open
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });

                    // 打开 CBZ 文件夹按钮
                    btn_group
                        .spawn((
                            OpenCbzFolderButton,
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
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                            BorderColor::all(AppColors::BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F0770} CBZ"), // 󰝰 nf-md-folder_open
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
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
            BorderColor::all(AppColors::BORDER),
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
                    // 漫画标题（可点击跳转详情）
                    row.spawn((
                        DownloadTitleButton {
                            comic_id: task.comic_id.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node::default(),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(task.comic_title.clone()),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

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

            // 分类和标签行（如果有的话）
            let has_categories = !task.categories.is_empty();
            let has_tags = !task.tags.is_empty();
            if has_categories || has_tags {
                item.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|tags_row| {
                    // 显示所有分类
                    for category in task.categories.iter() {
                        spawn_tag_badge(tags_row, category, font, TagColor::Category);
                    }
                    // 显示所有标签
                    for tag in task.tags.iter() {
                        spawn_tag_badge(tags_row, tag, font, TagColor::Tag);
                    }
                });
            }

            // 进度条
            item.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
            ))
            .with_children(|track| {
                track.spawn((
                    DownloadProgressBar {
                        comic_id: task.comic_id.clone(),
                    },
                    Node {
                        width: Val::Percent(progress * 100.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(match &task.status {
                        ComicDownloadStatus::Completed => Color::srgb(0.3, 0.8, 0.3),
                        ComicDownloadStatus::Failed(_) => Color::srgb(0.8, 0.3, 0.3),
                        ComicDownloadStatus::Paused => Color::srgb(0.8, 0.6, 0.2),
                        _ => AppColors::PRIMARY,
                    }),
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

                            // 设置按钮（独立下载设置）
                            spawn_control_button_with_display(
                                btns,
                                font,
                                "\u{F0493}", // 󰒓 nf-md-cog
                                DownloadTaskSettingsButton {
                                    comic_id: task.comic_id.clone(),
                                },
                                Color::srgb(0.5, 0.5, 0.7),
                                true,
                            );
                        });
                });

            // 独立设置标注（如果有自定义设置）
            if task.custom_download_path.is_some() || task.custom_auto_pack_cbz.is_some() {
                let mut settings_parts = Vec::new();
                if let Some(ref path) = task.custom_download_path {
                    settings_parts.push(format!("路径: {}", path));
                }
                if let Some(cbz) = task.custom_auto_pack_cbz {
                    settings_parts.push(format!("CBZ: {}", if cbz { "开" } else { "关" }));
                }
                item.spawn((
                    Text::new(format!("\u{F0493} {}", settings_parts.join(" | "))),
                    TextFont {
                        font: font.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.5, 0.5, 0.7, 0.8)),
                ));
            }
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
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(color.with_alpha(0.2)),
            BorderColor::all(color),
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
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
            BorderColor::all(AppColors::BORDER),
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
                    // 漫画名称（可点击跳转详情）
                    info.spawn((
                        DownloadTitleButton {
                            comic_id: download.comic_id.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node::default(),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(download.folder_name.clone()),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

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

                    // 分类和标签
                    let has_categories = !download.categories.is_empty();
                    let has_tags = !download.tags.is_empty();
                    if has_categories || has_tags {
                        info.spawn((Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(4.0),
                            row_gap: Val::Px(2.0),
                            margin: UiRect::top(Val::Px(2.0)),
                            ..default()
                        },))
                            .with_children(|tags_row| {
                                // 显示所有分类
                                for category in download.categories.iter() {
                                    spawn_tag_badge(tags_row, category, font, TagColor::Category);
                                }
                                // 显示所有标签
                                for tag in download.tags.iter() {
                                    spawn_tag_badge(tags_row, tag, font, TagColor::Tag);
                                }
                            });
                    }
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
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.3).with_alpha(0.2)),
                            BorderColor::all(Color::srgb(0.3, 0.7, 0.4)),
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

                    // 设置按钮（独立下载设置）
                    if has_comic_id {
                        spawn_control_button_with_display(
                            btns,
                            font,
                            "\u{F0493}", // 󰒓 nf-md-cog
                            DownloadTaskSettingsButton {
                                comic_id: download.comic_id.clone(),
                            },
                            Color::srgb(0.5, 0.5, 0.7),
                            true,
                        );
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
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.4).with_alpha(0.2)),
                        BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
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

                    // 删除按钮
                    let delete_color = Color::srgb(0.8, 0.3, 0.3);
                    btns.spawn((
                        DeleteCompletedDownloadButton {
                            comic_id: download.comic_id.clone(),
                            path: download.path.clone(),
                        },
                        Button,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(delete_color.with_alpha(0.2)),
                        BorderColor::all(delete_color),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("\u{F01B4}"), // 󰆴 nf-md-delete
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(delete_color),
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
                    border_radius: BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
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

/// 打开 CBZ 文件夹按钮交互
pub fn open_cbz_folder_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<OpenCbzFolderButton>),
    >,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));

                // 打开 CBZ 文件夹
                let cbz_path = get_download_base_path().join("CBZ");

                // 如果目录不存在则创建
                if !cbz_path.exists() {
                    let _ = std::fs::create_dir_all(&cbz_path);
                }

                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg(&cbz_path)
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open").arg(&cbz_path).spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&cbz_path)
                        .spawn();
                }

                tracing::info!("打开 CBZ 文件夹: {:?}", cbz_path);
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

/// 动态添加新任务的 UI（检测没有 UI 的任务并创建，根据状态添加到正确的区域）
pub fn add_new_task_ui(
    mut commands: Commands,
    download_state: Res<DownloadManagerState>,
    downloading_list_query: Query<Entity, With<DownloadTaskList>>,
    waiting_list_query: Query<Entity, With<WaitingTaskList>>,
    stopped_list_query: Query<Entity, With<StoppedTaskList>>,
    existing_items_query: Query<&DownloadTaskItem>,
    _asset_server: Res<AssetServer>,
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

    let font: Handle<Font> = get_font();

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
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
                BorderColor::all(AppColors::BORDER),
            ))
            .with_children(|item| {
                // 标题和按钮行
                item.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Transform::default(),
                ))
                .with_children(|row| {
                    // 标题（可点击跳转详情）
                    row.spawn((
                        DownloadTitleButton {
                            comic_id: task.comic_id.clone(),
                        },
                        Button,
                        Interaction::default(),
                        Node::default(),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(&task.comic_title),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                        ));
                    });

                    // 按钮容器
                    row.spawn((
                        Node {
                            column_gap: Val::Px(8.0),
                            ..default()
                        },
                        Transform::default(),
                    ))
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
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.5, 0.2)),
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
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
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
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.7, 0.5, 0.2)),
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
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
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
                item.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(4.0),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                ))
                .with_children(|bar_bg| {
                    bar_bg.spawn((
                        DownloadProgressBar {
                            comic_id: task.comic_id.clone(),
                        },
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(AppColors::PRIMARY),
                    ));
                });

                // 分类和标签容器（初始可能为空，API 返回后会更新）
                item.spawn((
                    DownloadTaskTagsContainer {
                        comic_id: task.comic_id.clone(),
                    },
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ))
                .with_children(|tags_row| {
                    // 显示所有分类
                    for category in task.categories.iter() {
                        spawn_tag_badge(tags_row, category, &font, TagColor::Category);
                    }
                    // 显示所有标签
                    for tag in task.tags.iter() {
                        spawn_tag_badge(tags_row, tag, &font, TagColor::Tag);
                    }
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

        // 根据任务状态选择正确的列表容器
        let list_entity = match &task.status {
            ComicDownloadStatus::Downloading => downloading_list_query.single().ok(),
            ComicDownloadStatus::Waiting => waiting_list_query.single().ok(),
            ComicDownloadStatus::Paused | ComicDownloadStatus::Failed(_) => {
                stopped_list_query.single().ok()
            }
            ComicDownloadStatus::Completed => None, // 已完成的不应该出现在这里
        };

        if let Some(list_entity) = list_entity {
            commands.entity(list_entity).add_child(task_entity);
            tracing::debug!("任务 {} 添加到 {:?} 区域", task.comic_title, task.status);
        } else {
            tracing::warn!(
                "无法找到任务 {} 的目标列表容器 (状态: {:?})",
                task.comic_title,
                task.status
            );
            // 清理未添加到列表的实体
            commands.entity(task_entity).despawn();
        }
    }
}

/// 当任务状态变化时，将任务 UI 移动到正确的区域
pub fn move_task_between_sections(
    mut commands: Commands,
    download_state: Res<DownloadManagerState>,
    task_items_query: Query<(Entity, &DownloadTaskItem, &ChildOf)>,
    downloading_list_query: Query<Entity, With<DownloadTaskList>>,
    waiting_list_query: Query<Entity, With<WaitingTaskList>>,
    stopped_list_query: Query<Entity, With<StoppedTaskList>>,
) {
    // 只在状态变化时检查
    if !download_state.is_changed() {
        return;
    }

    // 获取当前任务状态映射
    let task_status_map: std::collections::HashMap<String, ComicDownloadStatus> = download_state
        .tasks()
        .into_iter()
        .map(|t| (t.comic_id.clone(), t.status.clone()))
        .collect();

    // 获取各区域的 Entity
    let downloading_list = downloading_list_query.single().ok();
    let waiting_list = waiting_list_query.single().ok();
    let stopped_list = stopped_list_query.single().ok();

    for (task_entity, task_item, child_of) in task_items_query.iter() {
        let Some(status) = task_status_map.get(&task_item.comic_id) else {
            continue;
        };

        // 确定任务应该在哪个列表
        let target_list = match status {
            ComicDownloadStatus::Downloading => downloading_list,
            ComicDownloadStatus::Waiting => waiting_list,
            ComicDownloadStatus::Paused | ComicDownloadStatus::Failed(_) => stopped_list,
            ComicDownloadStatus::Completed => None, // 已完成的会通过其他系统处理
        };

        let Some(target_list) = target_list else {
            continue;
        };

        // 检查当前父级是否正确
        let current_parent = child_of.parent();
        if current_parent == target_list {
            continue; // 已经在正确的列表中
        }

        // 移动到正确的列表
        tracing::debug!(
            "移动任务 {} 从 {:?} 到 {:?}",
            task_item.comic_id,
            current_parent,
            target_list
        );
        commands
            .entity(task_entity)
            .set_parent_in_place(target_list);
    }
}

/// 更新下载页面标题数字和区域显示状态（基于任务状态分类统计）
pub fn update_download_titles(
    download_state: Res<DownloadManagerState>,
    completed_items_query: Query<&CompletedDownloadItem>,
    // 下载中
    mut downloading_title_query: Query<
        &mut Text,
        (
            With<DownloadingTitleText>,
            Without<WaitingTitleText>,
            Without<StoppedTitleText>,
            Without<CompletedTitleText>,
        ),
    >,
    mut downloading_section_query: Query<
        &mut Node,
        (
            With<DownloadingSection>,
            Without<WaitingSection>,
            Without<StoppedSection>,
            Without<CompletedSection>,
        ),
    >,
    // 等待中
    mut waiting_title_query: Query<
        &mut Text,
        (
            With<WaitingTitleText>,
            Without<DownloadingTitleText>,
            Without<StoppedTitleText>,
            Without<CompletedTitleText>,
        ),
    >,
    mut waiting_section_query: Query<
        &mut Node,
        (
            With<WaitingSection>,
            Without<DownloadingSection>,
            Without<StoppedSection>,
            Without<CompletedSection>,
        ),
    >,
    // 已停止
    mut stopped_title_query: Query<
        &mut Text,
        (
            With<StoppedTitleText>,
            Without<DownloadingTitleText>,
            Without<WaitingTitleText>,
            Without<CompletedTitleText>,
        ),
    >,
    mut stopped_section_query: Query<
        &mut Node,
        (
            With<StoppedSection>,
            Without<DownloadingSection>,
            Without<WaitingSection>,
            Without<CompletedSection>,
        ),
    >,
    // 已下载
    mut completed_title_query: Query<
        &mut Text,
        (
            With<CompletedTitleText>,
            Without<DownloadingTitleText>,
            Without<WaitingTitleText>,
            Without<StoppedTitleText>,
        ),
    >,
    mut completed_section_query: Query<
        &mut Node,
        (
            With<CompletedSection>,
            Without<DownloadingSection>,
            Without<WaitingSection>,
            Without<StoppedSection>,
        ),
    >,
) {
    // 从下载状态中统计各类型任务数量
    let tasks = download_state.tasks();
    let downloading_count = tasks
        .iter()
        .filter(|t| matches!(t.status, ComicDownloadStatus::Downloading))
        .count();
    let waiting_count = tasks
        .iter()
        .filter(|t| matches!(t.status, ComicDownloadStatus::Waiting))
        .count();
    let stopped_count = tasks
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                ComicDownloadStatus::Paused | ComicDownloadStatus::Failed(_)
            )
        })
        .count();
    let completed_count = completed_items_query.iter().count();

    // 更新下载中标题和区域显示状态
    if let Ok(mut title) = downloading_title_query.single_mut() {
        let new_text = format!("\u{F01DA} 下载中 ({})", downloading_count); // 󰇚 nf-md-download
        if **title != new_text {
            **title = new_text;
        }
    }
    if let Ok(mut section) = downloading_section_query.single_mut() {
        let new_display = if downloading_count > 0 {
            Display::Flex
        } else {
            Display::None
        };
        if section.display != new_display {
            section.display = new_display;
        }
    }

    // 更新等待中标题和区域显示状态
    if let Ok(mut title) = waiting_title_query.single_mut() {
        let new_text = format!("\u{F0520} 等待中 ({})", waiting_count); // 󰔠 nf-md-timer_sand
        if **title != new_text {
            **title = new_text;
        }
    }
    if let Ok(mut section) = waiting_section_query.single_mut() {
        let new_display = if waiting_count > 0 {
            Display::Flex
        } else {
            Display::None
        };
        if section.display != new_display {
            section.display = new_display;
        }
    }

    // 更新已停止标题和区域显示状态
    if let Ok(mut title) = stopped_title_query.single_mut() {
        let new_text = format!("\u{F04DB} 已停止 ({})", stopped_count); // 󰓛 nf-md-stop_circle
        if **title != new_text {
            **title = new_text;
        }
    }
    if let Ok(mut section) = stopped_section_query.single_mut() {
        let new_display = if stopped_count > 0 {
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
        let new_text = format!("\u{F012C} 已下载 ({})", completed_count); // 󰄬 nf-md-check
        if **title != new_text {
            **title = new_text;
        }
    }
    if let Ok(mut section) = completed_section_query.single_mut() {
        let new_display = if completed_count > 0 {
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
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    // 滚动容器的上下 padding（各 20px）
    const SCROLL_PADDING_VERTICAL: f32 = 40.0;

    for (scroll_computed, mut content_info, children) in scroll_query.iter_mut() {
        let viewport_height = scroll_computed.size().y / scale_factor;

        let mut content_height = 0.0;
        for child in children.iter() {
            if let Ok(child_computed) = children_query.get(child) {
                content_height += child_computed.size().y / scale_factor;
            }
        }

        // 加上容器的上下 padding
        content_height += SCROLL_PADDING_VERTICAL;

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

/// 折叠区域标题交互
/// 点击标题可以折叠/展开对应区域
pub fn section_header_collapse_interaction(
    mut interaction_query: Query<(&Interaction, &CollapsibleSectionHeader), Changed<Interaction>>,
    mut collapse_state: ResMut<DownloadSectionCollapseState>,
    mut icon_query: Query<(&CollapseIcon, &mut Text)>,
    mut content_query: Query<(&SectionContent, &mut Node)>,
) {
    for (interaction, header) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            // 切换折叠状态
            let is_collapsed = match header.section_type {
                SectionType::Downloading => {
                    collapse_state.downloading_collapsed = !collapse_state.downloading_collapsed;
                    collapse_state.downloading_collapsed
                }
                SectionType::Waiting => {
                    collapse_state.waiting_collapsed = !collapse_state.waiting_collapsed;
                    collapse_state.waiting_collapsed
                }
                SectionType::Stopped => {
                    collapse_state.stopped_collapsed = !collapse_state.stopped_collapsed;
                    collapse_state.stopped_collapsed
                }
                SectionType::Completed => {
                    collapse_state.completed_collapsed = !collapse_state.completed_collapsed;
                    collapse_state.completed_collapsed
                }
            };

            // 更新图标
            let icon = if is_collapsed {
                "\u{F0142}" // ▶ nf-md-chevron_right
            } else {
                "\u{F0140}" // ▼ nf-md-chevron_down
            };
            for (collapse_icon, mut text) in icon_query.iter_mut() {
                if collapse_icon.section_type == header.section_type {
                    *text = Text::new(icon);
                }
            }

            // 更新内容显示
            let display = if is_collapsed {
                Display::None
            } else {
                Display::Flex
            };
            for (content, mut node) in content_query.iter_mut() {
                if content.section_type == header.section_type {
                    node.display = display;
                }
            }

            tracing::debug!(
                "切换 {:?} 区域折叠状态: {}",
                header.section_type,
                if is_collapsed { "折叠" } else { "展开" }
            );
        }
    }
}

/// 更新浮动标题显示
/// 当某个区域的标题滚出视口顶部时，显示浮动标题
pub fn update_floating_header(
    scroll_query: Query<&ScrollPosition, With<DownloadsScrollContainer>>,
    section_query: Query<(
        &ComputedNode,
        Option<&DownloadingSection>,
        Option<&WaitingSection>,
        Option<&StoppedSection>,
        Option<&CompletedSection>,
    )>,
    collapse_state: Res<DownloadSectionCollapseState>,
    mut floating_header_query: Query<&mut Node, With<FloatingHeader>>,
    mut floating_btn_query: Query<&mut FloatingHeaderButton>,
    mut floating_text_query: Query<&mut Text, With<FloatingHeaderText>>,
    mut floating_icon_query: Query<
        &mut Text,
        (With<FloatingHeaderIcon>, Without<FloatingHeaderText>),
    >,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    download_state: Res<DownloadManagerState>,
) {
    let Ok(scroll_pos) = scroll_query.single() else {
        return;
    };

    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    // 滚动容器的 padding
    const SCROLL_PADDING: f32 = 20.0;
    // 标题行高度（大约）
    const HEADER_HEIGHT: f32 = 36.0;

    // 计算各区域的位置（累计高度）
    let mut section_positions: Vec<(SectionType, f32, f32, bool)> = Vec::new(); // (type, start, end, is_collapsed)
    let mut current_y: f32 = 0.0;

    // 获取任务数量用于标题显示
    let all_tasks: Vec<_> = download_state
        .active_tasks()
        .into_iter()
        .map(|fsm| fsm.to_ui_task())
        .collect();

    let downloading_count = all_tasks
        .iter()
        .filter(|t| matches!(t.status, crate::resources::ComicDownloadStatus::Downloading))
        .count();
    let waiting_count = all_tasks
        .iter()
        .filter(|t| matches!(t.status, crate::resources::ComicDownloadStatus::Waiting))
        .count();
    let stopped_count = all_tasks
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                crate::resources::ComicDownloadStatus::Paused
                    | crate::resources::ComicDownloadStatus::Failed(_)
            )
        })
        .count();

    // 遍历区域获取高度（只处理有区域标记的实体）
    for (computed, is_downloading, is_waiting, is_stopped, is_completed) in section_query.iter() {
        let section_type = if is_downloading.is_some() {
            Some(SectionType::Downloading)
        } else if is_waiting.is_some() {
            Some(SectionType::Waiting)
        } else if is_stopped.is_some() {
            Some(SectionType::Stopped)
        } else if is_completed.is_some() {
            Some(SectionType::Completed)
        } else {
            None
        };

        // 只处理有区域标记的实体
        if let Some(st) = section_type {
            let height = computed.size().y / scale_factor;
            let is_collapsed = match st {
                SectionType::Downloading => collapse_state.downloading_collapsed,
                SectionType::Waiting => collapse_state.waiting_collapsed,
                SectionType::Stopped => collapse_state.stopped_collapsed,
                SectionType::Completed => collapse_state.completed_collapsed,
            };
            section_positions.push((st, current_y, current_y + height, is_collapsed));
            current_y += height + 10.0; // 加上区域间距
        }
    }

    // 按 SectionType 排序（确保顺序正确：Downloading -> Waiting -> Stopped ->
    // Completed）
    section_positions.sort_by_key(|(st, _, _, _)| match st {
        SectionType::Downloading => 0,
        SectionType::Waiting => 1,
        SectionType::Stopped => 2,
        SectionType::Completed => 3,
    });

    // 重新计算位置（排序后）
    let mut recalc_y: f32 = 0.0;
    for (_, start, end, _) in section_positions.iter_mut() {
        let height = *end - *start;
        *start = recalc_y;
        *end = recalc_y + height;
        recalc_y += height + 10.0;
    }

    // 调试日志
    if !section_positions.is_empty() && scroll_pos.y > 10.0 {
        tracing::debug!(
            "浮动标题检测: scroll_y={:.1}, sections={:?}",
            scroll_pos.y,
            section_positions
                .iter()
                .map(|(t, s, e, c)| {
                    format!(
                        "{:?}:{:.0}-{:.0}({})",
                        t,
                        s,
                        e,
                        if *c { "折" } else { "展" }
                    )
                })
                .collect::<Vec<_>>()
        );
    }

    // 当前滚动位置
    let scroll_y = scroll_pos.y;

    // 找到当前应该显示浮动标题的区域
    // 条件：区域未折叠，且标题已滚出视口顶部，但区域内容还在视口中
    let mut active_section: Option<SectionType> = None;

    for (section_type, start, end, is_collapsed) in &section_positions {
        if *is_collapsed {
            continue;
        }

        // 标题在滚动位置之上（标题已滚出），但区域底部还在视口中
        let header_scrolled_out = *start < scroll_y;
        let content_still_visible = *end > scroll_y + HEADER_HEIGHT;

        if header_scrolled_out && content_still_visible {
            active_section = Some(*section_type);
            break; // 只显示第一个符合条件的
        }
    }

    // 更新浮动标题
    let floating_result = floating_header_query.single_mut();
    if floating_result.is_err() {
        tracing::trace!("浮动标题查询失败: 找不到 FloatingHeader 实体");
    }

    if let Ok(mut floating_node) = floating_result {
        if let Some(section_type) = active_section {
            tracing::debug!("显示浮动标题: {:?}", section_type);
            floating_node.display = Display::Flex;

            // 更新按钮的 section_type
            if let Ok(mut btn) = floating_btn_query.single_mut() {
                btn.section_type = Some(section_type);
            }

            // 更新标题文本
            let (icon, text, _color) = match section_type {
                SectionType::Downloading => (
                    "\u{F01DA}", // 󰇚 nf-md-download
                    format!("下载中 ({})", downloading_count),
                    AppColors::TEXT,
                ),
                SectionType::Waiting => (
                    "\u{F0520}", // 󰔠 nf-md-timer_sand
                    format!("等待中 ({})", waiting_count),
                    AppColors::TEXT,
                ),
                SectionType::Stopped => (
                    "\u{F04DB}", // 󰓛 nf-md-stop_circle
                    format!("已停止 ({})", stopped_count),
                    Color::srgb(0.8, 0.6, 0.2),
                ),
                SectionType::Completed => {
                    // 需要从数据库获取已完成数量，这里简化处理
                    (
                        "\u{F012C}", // 󰄬 nf-md-check
                        "已下载".to_string(),
                        Color::srgb(0.3, 0.8, 0.3),
                    )
                }
            };

            if let Ok(mut text_component) = floating_text_query.single_mut() {
                *text_component = Text::new(format!("{} {} (点击跳转)", icon, text));
            }

            // 图标始终显示向下箭头（点击跳转到该区域）
            if let Ok(mut icon_component) = floating_icon_query.single_mut() {
                *icon_component = Text::new("\u{F0140}"); // ▼ nf-md-chevron_down
            }
        } else {
            floating_node.display = Display::None;
        }
    }
}

/// 浮动标题点击交互 - 跳转到对应区域
pub fn floating_header_click_interaction(
    mut interaction_query: Query<
        (&Interaction, &FloatingHeaderButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut scroll_query: Query<&mut ScrollPosition, With<DownloadsScrollContainer>>,
    // 分别查询每个区域，确保按固定顺序计算位置
    downloading_query: Query<&ComputedNode, With<DownloadingSection>>,
    waiting_query: Query<&ComputedNode, With<WaitingSection>>,
    stopped_query: Query<&ComputedNode, With<StoppedSection>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    for (interaction, btn, mut bg_color) in interaction_query.iter_mut() {
        let base_color = Color::srgba(0.12, 0.12, 0.16, 0.98);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(base_color.lighter(0.1));

                if let Some(target_section) = btn.section_type {
                    // 计算目标区域的滚动位置
                    let scale_factor = window_query
                        .single()
                        .ok()
                        .map(|w| w.scale_factor())
                        .unwrap_or(1.0);

                    // 区域间距
                    const SECTION_GAP: f32 = 10.0;

                    // 按固定顺序获取每个区域的高度
                    let downloading_height = downloading_query
                        .single()
                        .ok()
                        .map(|n| n.size().y / scale_factor)
                        .unwrap_or(0.0);
                    let waiting_height = waiting_query
                        .single()
                        .ok()
                        .map(|n| n.size().y / scale_factor)
                        .unwrap_or(0.0);
                    let stopped_height = stopped_query
                        .single()
                        .ok()
                        .map(|n| n.size().y / scale_factor)
                        .unwrap_or(0.0);

                    // 按布局顺序计算目标位置：Downloading → Waiting → Stopped → Completed
                    let target_y = match target_section {
                        SectionType::Downloading => 0.0,
                        SectionType::Waiting => downloading_height + SECTION_GAP,
                        SectionType::Stopped => {
                            downloading_height + SECTION_GAP + waiting_height + SECTION_GAP
                        }
                        SectionType::Completed => {
                            downloading_height
                                + SECTION_GAP
                                + waiting_height
                                + SECTION_GAP
                                + stopped_height
                                + SECTION_GAP
                        }
                    };

                    // 跳转到目标位置
                    if let Ok(mut scroll_pos) = scroll_query.single_mut() {
                        scroll_pos.y = target_y;
                        tracing::debug!(
                            "跳转到 {:?} 区域, scroll_y = {}",
                            target_section,
                            target_y
                        );
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(base_color.lighter(0.05));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(base_color);
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
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &DeleteDownloadButton),
        Changed<Interaction>,
    >,
    task_item_query: Query<(Entity, &DownloadTaskItem)>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let delete_color = Color::srgb(0.8, 0.3, 0.3);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(delete_color.with_alpha(0.4));

                // 删除元数据文件
                if let Some(fsm) = download_state.find_task(&btn.comic_id)
                    && let Err(e) = fsm.meta.delete()
                {
                    tracing::warn!("删除元数据文件失败: {}", e);
                }

                // 从任务列表中删除
                download_state.remove_task(&btn.comic_id);

                // 从 UI 中移除对应的卡片
                for (entity, task_item) in task_item_query.iter() {
                    if task_item.comic_id == btn.comic_id {
                        commands.entity(entity).despawn();
                        break;
                    }
                }

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
    _asset_server: Res<AssetServer>,
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

        let (episode_count, categories, tags) = download_state
            .find_task(comic_id)
            .map(|fsm| {
                (
                    fsm.meta.episode_orders.len(),
                    fsm.meta.categories.clone(),
                    fsm.meta.tags.clone(),
                )
            })
            .unwrap_or((0, vec![], vec![]));

        // 4. 添加到已下载列表
        if let Ok(list_entity) = completed_list_query.single() {
            let font: Handle<Font> = get_font();
            let download = CompletedDownload {
                comic_id: comic_id.clone(),
                folder_name: comic_title,
                path: save_path.clone(),
                episode_count,
                categories,
                tags,
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

                // 恢复所有已停止的任务（暂停和失败状态）
                let mut resumed_count = 0;
                for fsm in &download_state.fsm_tasks {
                    let task = fsm.to_ui_task();
                    // 只恢复已停止区域的任务（暂停和失败）
                    if matches!(
                        task.status,
                        crate::resources::ComicDownloadStatus::Paused
                            | crate::resources::ComicDownloadStatus::Failed(_)
                    ) {
                        resume_messages.write(ResumeDownloadRequest {
                            comic_id: task.comic_id.clone(),
                        });
                        resumed_count += 1;
                    }
                }

                if resumed_count > 0 {
                    tracing::info!("开始全部下载: 恢复 {} 个已停止任务", resumed_count);
                } else {
                    tracing::info!("没有已停止的下载任务需要恢复");
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

/// 更新下载任务的分类标签显示
/// 当 API 返回新数据后，刷新标签容器
pub fn update_download_task_tags(
    mut commands: Commands,
    download_state: Res<DownloadManagerState>,
    tags_container_query: Query<(Entity, &DownloadTaskTagsContainer, Option<&Children>)>,
    _asset_server: Res<AssetServer>,
) {
    // 只有当下载状态发生变化时才检查
    if !download_state.is_changed() {
        return;
    }

    let font: Handle<Font> = get_font();

    for (container_entity, tags_container, children) in tags_container_query.iter() {
        // 查找对应的任务
        let Some(fsm) = download_state.find_task(&tags_container.comic_id) else {
            continue;
        };

        let task_categories = &fsm.meta.categories;
        let task_tags = &fsm.meta.tags;

        // 如果任务有分类或标签数据
        let has_data = !task_categories.is_empty() || !task_tags.is_empty();

        // 检查当前是否已有子元素
        let has_children = children.map(|c| !c.is_empty()).unwrap_or(false);

        // 如果任务有数据但容器为空，则添加标签
        if has_data && !has_children {
            commands.entity(container_entity).with_children(|tags_row| {
                // 显示所有分类
                for category in task_categories.iter() {
                    spawn_tag_badge(tags_row, category, &font, TagColor::Category);
                }
                // 显示所有标签
                for tag in task_tags.iter() {
                    spawn_tag_badge(tags_row, tag, &font, TagColor::Tag);
                }
            });
            tracing::debug!(
                "更新下载任务标签: {} - {} 分类, {} 标签",
                tags_container.comic_id,
                task_categories.len(),
                task_tags.len()
            );
        }
    }
}

/// 下载列表标题点击交互（跳转到漫画详情）
pub fn download_title_interaction(
    mut interaction_query: Query<
        (&Interaction, &DownloadTitleButton, &Children),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut TextColor>,
    mut navigate_messages: MessageWriter<NavigateToComicDetailEvent>,
) {
    for (interaction, title_btn, children) in interaction_query.iter_mut() {
        // 更新子文本颜色
        for child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                match *interaction {
                    Interaction::Pressed => {
                        *text_color = TextColor(Color::srgb(0.5, 0.7, 1.0));
                        navigate_messages.write(NavigateToComicDetailEvent {
                            comic_id: title_btn.comic_id.clone(),
                        });
                        tracing::info!("点击下载标题跳转详情: {}", title_btn.comic_id);
                    }
                    Interaction::Hovered => {
                        *text_color = TextColor(Color::srgb(0.6, 0.8, 1.0));
                    }
                    Interaction::None => {
                        *text_color = TextColor(AppColors::TEXT);
                    }
                }
            }
        }
    }
}

/// 下载列表分类标签点击交互（跳转到分类列表）
pub fn download_category_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &DownloadCategoryTag),
        Changed<Interaction>,
    >,
    mut navigate_messages: MessageWriter<NavigateToComicsListEvent>,
) {
    for (interaction, mut bg_color, tag) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.5, 0.9, 0.5));
                navigate_messages.write(NavigateToComicsListEvent {
                    category: tag.category.clone(),
                });
                tracing::info!("点击下载分类标签: {}", tag.category);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.25, 0.45, 0.85, 0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.4, 0.8, 0.3));
            }
        }
    }
}

/// 下载列表标签点击交互（跳转到搜索）
pub fn download_tag_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &DownloadTagButton),
        Changed<Interaction>,
    >,
    mut search_state: ResMut<SearchState>,
    mut next_route: ResMut<NextState<AppRoute>>,
    mut history: ResMut<NavigationHistory>,
    current_route: Res<State<AppRoute>>,
    mut search_messages: MessageWriter<SearchComicsRequestEvent>,
) {
    for (interaction, mut bg_color, tag_btn) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.7, 0.5, 0.5));

                // 设置搜索状态
                search_state.keyword = tag_btn.tag.clone();
                search_state.results.clear();
                search_state.page = 1;
                search_state.total_pages = 0;
                search_state.is_loading = true;
                search_state.has_searched = true;
                search_state.error = None;

                // 记录导航历史
                history.push(current_route.get().clone());

                // 跳转到搜索页面
                next_route.set(AppRoute::Search);

                // 发送搜索请求
                search_messages.write(SearchComicsRequestEvent {
                    keyword: tag_btn.tag.clone(),
                    page: 1,
                    sort: search_state.sort.clone(),
                    categories: search_state.selected_categories.clone(),
                });

                tracing::info!("点击下载标签搜索: {}", tag_btn.tag);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.25, 0.65, 0.45, 0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.6, 0.4, 0.3));
            }
        }
    }
}

/// 下载任务独立设置按钮交互
///
/// 点击设置按钮后，展开/收起内联设置面板（路径选择 + CBZ 开关）
pub fn task_settings_button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &DownloadTaskSettingsButton), Changed<Interaction>>,
    panel_query: Query<(Entity, &DownloadTaskSettingsPanel)>,
    download_state: Res<DownloadManagerState>,
    _asset_server: Res<AssetServer>,
    task_item_query: Query<(Entity, &DownloadTaskItem)>,
    completed_item_query: Query<(Entity, &CompletedDownloadItem)>,
) {
    for (interaction, settings_btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let comic_id = &settings_btn.comic_id;

        // 检查是否已有面板，有则关闭
        let existing_panel = panel_query
            .iter()
            .find(|(_, panel)| panel.comic_id == *comic_id);

        if let Some((panel_entity, _)) = existing_panel {
            commands.entity(panel_entity).despawn();
            continue;
        }

        // 找到父任务项 Entity
        let parent_entity = task_item_query
            .iter()
            .find(|(_, item)| item.comic_id == *comic_id)
            .map(|(e, _)| e)
            .or_else(|| {
                completed_item_query
                    .iter()
                    .find(|(_, item)| item.comic_id == *comic_id)
                    .map(|(e, _)| e)
            });

        let Some(parent_entity) = parent_entity else {
            continue;
        };

        // 获取当前任务的设置
        let task = download_state.find_task(comic_id);
        let custom_path = task.and_then(|t| t.meta.custom_download_path.clone());
        let custom_cbz = task.and_then(|t| t.meta.custom_auto_pack_cbz);

        let font: Handle<Font> = get_font();

        // 创建设置面板
        let panel_entity = commands
            .spawn((
                DownloadTaskSettingsPanel {
                    comic_id: comic_id.clone(),
                },
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(8.0),
                    border: UiRect::top(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
                BorderColor::all(Color::srgba(0.3, 0.3, 0.5, 0.5)),
            ))
            .with_children(|panel| {
                // 标题
                panel.spawn((
                    Text::new("独立设置"),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.7)),
                ));

                // 下载路径行
                panel
                    .spawn((Node {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },))
                    .with_children(|row| {
                        row.spawn((
                            Text::new("路径:"),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));

                        let path_text = custom_path.as_deref().unwrap_or("(使用全局设置)");
                        row.spawn((
                            Text::new(path_text),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                        ));

                        // 选择路径按钮
                        row.spawn((
                            TaskPathSelectButton {
                                comic_id: comic_id.clone(),
                            },
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::new(
                                    Val::Px(8.0),
                                    Val::Px(8.0),
                                    Val::Px(4.0),
                                    Val::Px(4.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                            BorderColor::all(AppColors::BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("\u{F0770} 选择"), // 󰝰
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                    });

                // CBZ 打包开关行
                panel
                    .spawn((Node {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },))
                    .with_children(|row| {
                        row.spawn((
                            Text::new("CBZ 打包:"),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(AppColors::TEXT_SECONDARY),
                        ));

                        let cbz_text = match custom_cbz {
                            Some(true) => "开启",
                            Some(false) => "关闭",
                            None => "(使用全局设置)",
                        };

                        // 三态切换按钮：全局 → 开启 → 关闭 → 全局
                        row.spawn((
                            TaskCbzToggle {
                                comic_id: comic_id.clone(),
                            },
                            Button,
                            Interaction::default(),
                            Node {
                                padding: UiRect::new(
                                    Val::Px(12.0),
                                    Val::Px(12.0),
                                    Val::Px(4.0),
                                    Val::Px(4.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(match custom_cbz {
                                Some(true) => Color::srgb(0.2, 0.5, 0.3).with_alpha(0.3),
                                Some(false) => Color::srgb(0.5, 0.2, 0.2).with_alpha(0.3),
                                None => Color::srgb(0.15, 0.15, 0.2),
                            }),
                            BorderColor::all(AppColors::BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(cbz_text),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(AppColors::TEXT),
                            ));
                        });
                    });
            })
            .id();

        commands.entity(parent_entity).add_child(panel_entity);
    }
}

/// 下载任务路径选择按钮交互
pub fn task_path_select_interaction(
    mut interaction_query: Query<(&Interaction, &TaskPathSelectButton), Changed<Interaction>>,
    mut download_state: ResMut<DownloadManagerState>,
) {
    for (interaction, path_btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let comic_id = &path_btn.comic_id;

        // 使用 rfd 文件对话框选择文件夹
        let dialog = rfd::FileDialog::new();
        let Some(selected_path) = dialog.pick_folder() else {
            continue;
        };
        let new_path = selected_path.to_string_lossy().to_string();

        // 获取当前任务
        let Some(fsm) = download_state.find_task_mut(comic_id) else {
            continue;
        };

        let old_path = fsm.meta.save_path.clone();

        // 更新路径
        fsm.meta.custom_download_path = Some(new_path.clone());

        // 如果旧路径存在，尝试移动文件
        let old_dir = std::path::Path::new(&old_path);
        if old_dir.exists() && old_dir.is_dir() {
            let new_dir = std::path::Path::new(&new_path);
            // 获取旧路径的文件夹名
            if let Some(folder_name) = old_dir.file_name() {
                let target_dir = new_dir.join(folder_name);
                if !target_dir.exists() {
                    if let Err(e) = move_dir_recursive(old_dir, &target_dir) {
                        tracing::error!("移动下载文件失败: {}", e);
                    } else {
                        // 更新 save_path 为新目标
                        fsm.meta.save_path = target_dir.to_string_lossy().to_string();
                        tracing::info!("已移动下载文件: {} -> {}", old_path, fsm.meta.save_path);
                    }
                }
            }
        }

        // 保存到数据库
        if let Err(e) = fsm.meta.save() {
            tracing::error!("保存独立路径设置失败: {}", e);
        }
    }
}

/// 递归移动目录
fn move_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    // 先尝试 rename（同文件系统的情况下最快）
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }

    // rename 失败（可能跨文件系统），逐文件复制
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            move_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    // 删除原目录
    std::fs::remove_dir_all(src)?;
    Ok(())
}

/// 下载任务 CBZ 开关交互（三态循环：全局 → 开启 → 关闭 → 全局）
pub fn task_cbz_toggle_interaction(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &TaskCbzToggle), Changed<Interaction>>,
    mut download_state: ResMut<DownloadManagerState>,
    panel_query: Query<(Entity, &DownloadTaskSettingsPanel)>,
) {
    for (interaction, cbz_toggle) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let comic_id = &cbz_toggle.comic_id;

        let Some(fsm) = download_state.find_task_mut(comic_id) else {
            continue;
        };

        // 三态切换：None → Some(true) → Some(false) → None
        fsm.meta.custom_auto_pack_cbz = match fsm.meta.custom_auto_pack_cbz {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        };

        // 保存到数据库
        if let Err(e) = fsm.meta.save() {
            tracing::error!("保存 CBZ 设置失败: {}", e);
        }

        // 关闭并重新打开面板以刷新显示
        for (panel_entity, panel) in panel_query.iter() {
            if panel.comic_id == *comic_id {
                commands.entity(panel_entity).despawn();
            }
        }
    }
}

/// 删除已下载漫画按钮交互 — 弹出确认面板
pub fn delete_completed_download_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &DeleteCompletedDownloadButton,
        ),
        Changed<Interaction>,
    >,
    panel_query: Query<(Entity, &DeleteConfirmPanel)>,
    completed_item_query: Query<(Entity, &CompletedDownloadItem)>,
    _asset_server: Res<AssetServer>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let delete_color = Color::srgb(0.8, 0.3, 0.3);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(delete_color.with_alpha(0.4));

                let comic_id = &btn.comic_id;

                // 已有面板则关闭
                let existing = panel_query.iter().find(|(_, p)| p.comic_id == *comic_id);
                if let Some((panel_entity, _)) = existing {
                    commands.entity(panel_entity).despawn();
                    continue;
                }

                // 找到父项 Entity
                let Some(parent_entity) = completed_item_query
                    .iter()
                    .find(|(_, item)| item.comic_id == *comic_id)
                    .map(|(e, _)| e)
                else {
                    continue;
                };

                let font: Handle<Font> = get_font();

                // 创建确认面板
                let panel_entity = commands
                    .spawn((
                        DeleteConfirmPanel {
                            comic_id: comic_id.clone(),
                            path: btn.path.clone(),
                        },
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(10.0)),
                            row_gap: Val::Px(8.0),
                            border: UiRect::top(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.06, 0.06)),
                        BorderColor::all(Color::srgba(0.5, 0.2, 0.2, 0.5)),
                    ))
                    .with_children(|panel| {
                        // 警告文本
                        panel.spawn((
                            Text::new("确认删除此下载记录？"),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.5, 0.5)),
                        ));

                        // 勾选框行：同时删除磁盘文件
                        panel
                            .spawn((Node {
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.0),
                                ..default()
                            },))
                            .with_children(|row| {
                                // 勾选框按钮
                                row.spawn((
                                    DeleteFilesCheckbox {
                                        comic_id: comic_id.clone(),
                                        checked: false,
                                    },
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        width: Val::Px(18.0),
                                        height: Val::Px(18.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(1.5)),
                                        border_radius: BorderRadius::all(Val::Px(3.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.1, 0.1, 0.14)),
                                    BorderColor::all(Color::srgb(0.5, 0.3, 0.3)),
                                ))
                                .with_children(|cb| {
                                    // 初始状态：空（未勾选）
                                    cb.spawn((
                                        Text::new(""),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.4, 0.4)),
                                    ));
                                });

                                row.spawn((
                                    Text::new("同时删除磁盘文件"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.6, 0.6)),
                                ));
                            });

                        // 按钮行
                        panel
                            .spawn((Node {
                                column_gap: Val::Px(8.0),
                                margin: UiRect::top(Val::Px(4.0)),
                                ..default()
                            },))
                            .with_children(|row| {
                                // 确认删除按钮
                                let confirm_color = Color::srgb(0.8, 0.3, 0.3);
                                row.spawn((
                                    ConfirmDeleteButton {
                                        comic_id: comic_id.clone(),
                                        path: btn.path.clone(),
                                    },
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        padding: UiRect::new(
                                            Val::Px(12.0),
                                            Val::Px(12.0),
                                            Val::Px(4.0),
                                            Val::Px(4.0),
                                        ),
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(confirm_color.with_alpha(0.3)),
                                    BorderColor::all(confirm_color),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("\u{F01B4} 确认删除"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.4, 0.4)),
                                    ));
                                });

                                // 取消按钮
                                row.spawn((
                                    CancelDeleteButton {
                                        comic_id: comic_id.clone(),
                                    },
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        padding: UiRect::new(
                                            Val::Px(12.0),
                                            Val::Px(12.0),
                                            Val::Px(4.0),
                                            Val::Px(4.0),
                                        ),
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                                    BorderColor::all(AppColors::BORDER),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("取消"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextColor(AppColors::TEXT_SECONDARY),
                                    ));
                                });
                            });
                    })
                    .id();

                commands.entity(parent_entity).add_child(panel_entity);
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

/// "同时删除磁盘文件" 勾选框交互
pub fn delete_files_checkbox_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut DeleteFilesCheckbox, &Children),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut checkbox, children) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        checkbox.checked = !checkbox.checked;

        // 更新勾选图标
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = if checkbox.checked {
                    "\u{F012C}".to_string() // 󰄬 nf-md-check
                } else {
                    String::new()
                };
            }
        }
    }
}

/// 确认删除按钮交互
pub fn confirm_delete_button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ConfirmDeleteButton),
        Changed<Interaction>,
    >,
    checkbox_query: Query<&DeleteFilesCheckbox>,
    completed_item_query: Query<(Entity, &CompletedDownloadItem)>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let confirm_color = Color::srgb(0.8, 0.3, 0.3);
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(confirm_color.with_alpha(0.5));

                let comic_id = &btn.comic_id;

                // 检查是否勾选了删除磁盘文件
                let delete_files = checkbox_query
                    .iter()
                    .any(|cb| cb.comic_id == *comic_id && cb.checked);

                // 从数据库中删除记录
                {
                    use picacg_db::{delete_download_task_async, get_pool, run_db_operation};
                    let cid = comic_id.clone();
                    let pool = get_pool();
                    if let Err(e) = run_db_operation(async move {
                        delete_download_task_async(&pool, &cid)
                            .await
                            .map_err(|e| format!("删除下载记录失败: {}", e))
                    }) {
                        tracing::error!("删除已下载记录失败: {}", e);
                    }
                }

                // 删除磁盘文件（如果勾选）
                if delete_files {
                    let path = std::path::Path::new(&btn.path);
                    if path.exists() {
                        if let Err(e) = std::fs::remove_dir_all(path) {
                            tracing::error!("删除磁盘文件失败: {} - {}", btn.path, e);
                        } else {
                            tracing::info!("已删除磁盘文件: {}", btn.path);
                        }
                    }
                }

                // 从 UI 中移除整个项
                for (entity, item) in completed_item_query.iter() {
                    if item.comic_id == *comic_id {
                        commands.entity(entity).despawn();
                        break;
                    }
                }

                tracing::info!(
                    "删除已下载记录: {} ({}){}",
                    comic_id,
                    btn.path,
                    if delete_files {
                        " [含磁盘文件]"
                    } else {
                        ""
                    }
                );
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(confirm_color.with_alpha(0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(confirm_color.with_alpha(0.3));
            }
        }
    }
}

/// 取消删除按钮交互
pub fn cancel_delete_button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &CancelDeleteButton), Changed<Interaction>>,
    panel_query: Query<(Entity, &DeleteConfirmPanel)>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 关闭确认面板
        for (panel_entity, panel) in panel_query.iter() {
            if panel.comic_id == btn.comic_id {
                commands.entity(panel_entity).despawn();
            }
        }
    }
}
