//! 下载管理界面系统
//!
//! 实现下载任务列表和进度显示

use bevy::prelude::*;
use picacg_config::AppSettings;

use crate::{
    components::ContentArea,
    events::{
        DownloadCompletedEvent, NavigateToComicDetailEvent, NavigateToComicsListEvent,
        RedownloadConfirmed, RedownloadRequest, RedownloadSkipped, ResumeDownloadRequest,
        SearchComicsRequestEvent,
    },
    resources::{
        AppRoute, ComicDownloadStatus, DownloadManagerState, DownloadState, DownloadedComicsIndex,
        SearchState,
    },
    systems::{
        login::AppColors,
        navigation::NavigationHistory,
        scrollbar::{ScrollArea, scrollbar},
        theme::Scale,
        ui_common::truncate_text,
        widgets::ButtonStyle,
    },
    utils::{icons::*, tokio_tasks::TokioTasksRuntime},
};

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

/// 下载页面根标记
#[derive(Component, Default, Clone)]
pub struct DownloadsRoot;

/// 下载滚动容器标记
#[derive(Component, Default, Clone)]
pub struct DownloadsScrollContainer;

/// 下载任务列表容器标记
#[derive(Component, Default, Clone)]
pub struct DownloadTaskList;

/// "下载中" 区域容器（包含标题和任务列表）
#[derive(Component, Default, Clone)]
pub struct DownloadingSection;

/// "下载中" 标题文本
#[derive(Component, Default, Clone)]
pub struct DownloadingTitleText;

/// "全部更新" 按钮标记（检查已下载漫画的新章节）
#[derive(Component, Default, Clone)]
pub struct UpdateAllDownloadsButton {
    /// true = 全部强制更新（逐图校验），false = 普通更新（章节数一致即跳过）
    pub force: bool,
}

/// "开始全部下载" 按钮标记
#[derive(Component, Default, Clone)]
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
#[derive(Component, Default, Clone)]
pub struct CompletedDownloadItem {
    pub comic_id: String,
    pub path: String,
}

/// 已下载列表容器标记
#[derive(Component, Default, Clone)]
pub struct CompletedDownloadList;

/// "已下载" 区域容器
#[derive(Component, Default, Clone)]
pub struct CompletedSection;

/// "已下载" 标题文本
#[derive(Component, Default, Clone)]
pub struct CompletedTitleText;

/// "等待中" 区域容器（排队等待下载的任务）
#[derive(Component, Default, Clone)]
pub struct WaitingSection;

/// "等待中" 标题文本
#[derive(Component, Default, Clone)]
pub struct WaitingTitleText;

/// "等待中" 任务列表容器
#[derive(Component, Default, Clone)]
pub struct WaitingTaskList;

/// "已停止" 区域容器（暂停和失败的任务）
#[derive(Component, Default, Clone)]
pub struct StoppedSection;

/// "已停止" 标题文本
#[derive(Component, Default, Clone)]
pub struct StoppedTitleText;

/// "已停止" 任务列表容器
#[derive(Component, Default, Clone)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SectionType {
    #[default]
    Downloading,
    Waiting,
    Stopped,
    Completed,
}

/// 可折叠的区域标题按钮
#[derive(Component, Default, Clone)]
pub struct CollapsibleSectionHeader {
    pub section_type: SectionType,
}

/// 折叠图标标记
#[derive(Component, Default, Clone)]
pub struct CollapseIcon {
    pub section_type: SectionType,
}

/// 区域内容容器标记（用于折叠/展开）
#[derive(Component, Default, Clone)]
pub struct SectionContent {
    pub section_type: SectionType,
}

/// 浮动标题容器（固定在滚动区域顶部）
#[derive(Component, Default, Clone)]
pub struct FloatingHeader;

/// 浮动标题文本
#[derive(Component, Default, Clone)]
pub struct FloatingHeaderText;

/// 浮动标题折叠图标
#[derive(Component, Default, Clone)]
pub struct FloatingHeaderIcon;

/// 浮动标题按钮（可点击折叠）
#[derive(Component, Default, Clone)]
pub struct FloatingHeaderButton {
    pub section_type: Option<SectionType>,
}

/// 重新下载按钮标记
#[derive(Component, Default, Clone)]
pub struct RedownloadButton {
    pub comic_id: String,
    /// true = 强制更新（逐章拉图片列表、逐图补全），false = 普通更新
    /// （只比对章节数，一致即跳过）
    pub force: bool,
}

/// 打开已下载漫画文件夹按钮
#[derive(Component, Default, Clone)]
pub struct OpenCompletedFolderButton {
    pub path: String,
}

/// 移动已下载漫画按钮（同时移动 Images 和 CBZ）
#[derive(Component, Default, Clone)]
pub struct MoveCompletedButton {
    pub comic_id: String,
    pub comic_title: String,
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

        // 修复可能被污染的路径，然后使用
        let mut meta = crate::resources::DownloadTaskMeta::from_db_task(&db_task);
        crate::resources::repair_save_path(&mut meta);
        let effective_path = meta.save_path.clone();

        // 从有效路径提取文件夹名称
        let folder_name = std::path::Path::new(&effective_path)
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
            path: effective_path,
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
#[derive(Component, Default, Clone)]
pub struct DownloadTaskItem {
    pub comic_id: String,
}

/// 下载任务分类标签容器标记
#[derive(Component, Default, Clone)]
pub struct DownloadTaskTagsContainer {
    pub comic_id: String,
}

/// 标签颜色类型
#[derive(Clone, Copy)]
enum TagColor {
    /// 分类（蓝色）
    Category,
    /// 标签（绿色）
    Tag,
}

/// 下载列表中可点击的标题（跳转到漫画详情）
#[derive(Component, Default, Clone)]
pub struct DownloadTitleButton {
    pub comic_id: String,
}

/// 下载列表中可点击的分类标签（跳转到分类列表）
#[derive(Component, Default, Clone)]
pub struct DownloadCategoryTag {
    pub category: String,
}

/// 下载列表中可点击的标签（跳转到搜索）
#[derive(Component, Default, Clone)]
pub struct DownloadTagButton {
    pub tag: String,
}

/// 标签徽章场景（可点击）
///
/// 分类与标签的标记组件类型不同（`DownloadCategoryTag` /
/// `DownloadTagButton`）， 用 `Box<dyn Scene>` 在外层分派，内部结构由
/// `tag_badge_with_marker` 共用。
fn tag_badge(text: &str, color_type: TagColor) -> Box<dyn Scene> {
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

    // 根据类型添加不同的组件
    match color_type {
        TagColor::Category => Box::new(tag_badge_with_marker(
            DownloadCategoryTag {
                category: text_owned,
            },
            display_text,
            bg_color,
            text_color,
        )),
        TagColor::Tag => Box::new(tag_badge_with_marker(
            DownloadTagButton { tag: text_owned },
            display_text,
            bg_color,
            text_color,
        )),
    }
}

/// 标签徽章公共结构（标记组件由调用方决定）
fn tag_badge_with_marker<M: Component + Default + Clone + Unpin>(
    marker: M,
    display_text: String,
    bg_color: Color,
    text_color: Color,
) -> impl Scene + use<M> {
    bsn! {
        template_value(marker)
        Button
        Node {
            padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(2.0), Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
        }
        BackgroundColor(bg_color)
        Children [
            (
                Text({display_text})
                TextFont { font_size: FontSize::Px(10.0) }
                TextColor(text_color)
            )
        ]
    }
}

/// 下载进度条标记
#[derive(Component, Default, Clone)]
pub struct DownloadProgressBar {
    pub comic_id: String,
}

/// 下载状态文本标记
#[derive(Component, Default, Clone)]
pub struct DownloadStatusText {
    pub comic_id: String,
}

/// 打开下载文件夹按钮标记
#[derive(Component, Default, Clone)]
pub struct OpenDownloadFolderButton;

/// "全部移动"按钮标记
#[derive(Component, Default, Clone)]
pub struct MoveAllDownloadsButton;

/// 标题栏"全部开始"按钮
#[derive(Component, Default, Clone)]
pub struct StartAllHeaderButton;

/// 标题栏"全部暂停"按钮
#[derive(Component, Default, Clone)]
pub struct PauseAllHeaderButton;

/// 打开 CBZ 文件夹按钮标记
#[derive(Component, Default, Clone)]
pub struct OpenCbzFolderButton;

/// 暂停下载按钮标记
#[derive(Component, Default, Clone)]
pub struct PauseDownloadButton {
    pub comic_id: String,
}

/// 继续下载按钮标记
#[derive(Component, Default, Clone)]
pub struct ResumeDownloadButton {
    pub comic_id: String,
}

/// 删除下载按钮标记
#[derive(Component, Default, Clone)]
pub struct DeleteDownloadButton {
    pub comic_id: String,
}

/// 重试下载按钮标记
#[derive(Component, Default, Clone)]
pub struct RetryDownloadButton {
    pub comic_id: String,
}

/// 删除已下载漫画按钮标记
#[derive(Component, Default, Clone)]
pub struct DeleteCompletedDownloadButton {
    pub comic_id: String,
    pub path: String,
}

/// 删除确认面板标记
#[derive(Component, Default, Clone)]
pub struct DeleteConfirmPanel {
    pub comic_id: String,
}

/// "同时删除磁盘文件" 勾选框标记
#[derive(Component, Default, Clone)]
pub struct DeleteFilesCheckbox {
    pub comic_id: String,
    pub checked: bool,
}

/// 确认删除按钮标记
#[derive(Component, Default, Clone)]
pub struct ConfirmDeleteButton {
    pub comic_id: String,
    pub path: String,
}

/// 取消删除按钮标记
#[derive(Component, Default, Clone)]
pub struct CancelDeleteButton {
    pub comic_id: String,
}

/// 下载任务独立设置按钮标记
#[derive(Component, Default, Clone)]
pub struct DownloadTaskSettingsButton {
    pub comic_id: String,
}

/// 下载任务独立设置面板标记
#[derive(Component, Default, Clone)]
pub struct DownloadTaskSettingsPanel {
    pub comic_id: String,
}

/// 下载任务独立路径选择按钮
#[derive(Component, Default, Clone)]
pub struct TaskPathSelectButton {
    pub comic_id: String,
}

/// 下载任务独立 CBZ 开关
#[derive(Component, Default, Clone)]
pub struct TaskCbzToggle {
    pub comic_id: String,
}

/// 下载统计类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadStatType {
    /// 总计
    #[default]
    Total,
    /// 下载中
    Downloading,
    /// 等待中
    Waiting,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed,
}

/// 下载统计面板标记
#[derive(Component, Default, Clone)]
pub struct DownloadStatsPanel;

/// 下载统计文本标记
#[derive(Component, Default, Clone)]
pub struct DownloadStatText {
    pub stat_type: DownloadStatType,
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
    existing_query: Query<Entity, With<DownloadsRoot>>,
) {
    // 下载状态频繁变化，每次进入都重建 UI
    for entity in existing_query.iter() {
        commands.entity(entity).despawn();
    }

    // 查找内容区域
    let content_area = match content_area_query.iter().next() {
        Some(entity) => entity,
        None => {
            tracing::warn!("下载页面：找不到内容区域");
            return;
        }
    };

    // 在内容区域下创建下载页面
    let downloads_root = commands
        .spawn_scene(downloads_page(&download_state, &collapse_state))
        .id();
    commands.entity(content_area).add_child(downloads_root);

    tracing::info!("下载页面 UI 已创建");
}

/// 下载页面场景
fn downloads_page(
    download_state: &DownloadManagerState,
    collapse_state: &DownloadSectionCollapseState,
) -> impl Scene + use<> {
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
                ComicDownloadStatus::Paused | ComicDownloadStatus::Failed(_)
            )
        })
        .collect();

    // 获取折叠状态
    let downloading_collapsed = collapse_state.downloading_collapsed;
    let waiting_collapsed = collapse_state.waiting_collapsed;
    let stopped_collapsed = collapse_state.stopped_collapsed;
    let completed_collapsed = collapse_state.completed_collapsed;

    bsn! {
        DownloadsRoot
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            // 标题栏
            downloads_header(),
            // 下载统计面板（固定在标题栏下方，不随内容滚动）
            download_stats_panel(download_state),
            (
                // 下载内容（可滚动）
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                }
                Children [
                    (
                        // 滚动容器
                        #DownloadsScroll
                        DownloadsScrollContainer
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(20.0)),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [
                            // 1. "下载中" 区域 - 始终显示标题，内容可折叠
                            downloading_section(downloading_collapsed, &downloading_tasks),
                            // 2. "等待中" 区域 - 排队等待下载的任务
                            waiting_section(waiting_collapsed, &waiting_tasks),
                            // 3. "已停止" 区域 - 暂停和失败的任务
                            stopped_section(stopped_collapsed, &stopped_tasks),
                            // 4. "已下载" 区域 - 已完成的任务
                            completed_section(completed_collapsed, &completed_downloads),
                            (
                                // 底部间距，确保最后的内容不会贴着窗口底部
                                Node {
                                    height: Val::Px(40.0),
                                    min_height: Val::Px(40.0),
                                }
                            ),
                        ]
                    ),
                    // 滚动条
                    scrollbar(#DownloadsScroll),
                    // 浮动标题（固定在顶部，初始隐藏）
                    floating_header(),
                ]
            ),
        ]
    }
}

/// 可折叠区域标题按钮场景（折叠图标 + 标题文本）
///
/// `grow`: 标题位于标题行内时占据剩余宽度（"已停止" 区域），否则铺满整行
fn collapsible_section_header<T: Component + Default + Clone + Unpin>(
    section_type: SectionType,
    collapsed: bool,
    title_marker: T,
    title: String,
    title_color: Color,
    grow: bool,
) -> impl Scene + use<T> {
    // 折叠图标
    let icon = if collapsed {
        ICON_CHEVRON_RIGHT
    } else {
        ICON_CHEVRON_DOWN
    };
    let (width, flex_grow) = if grow {
        (Val::Auto, 1.0_f32)
    } else {
        (Val::Percent(100.0), 0.0_f32)
    };

    bsn! {
        CollapsibleSectionHeader { section_type: {section_type} }
        Button
        Node {
            width: {width},
            flex_grow: {flex_grow},
            align_items: AlignItems::Center,
            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(6.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            column_gap: Val::Px(8.0),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(AppColors::SURFACE_SUNKEN)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 折叠图标
                CollapseIcon { section_type: {section_type} }
                Text({icon})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_MUTED)
            ),
            (
                // 区域标题
                template_value(title_marker)
                Text({title})
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(title_color)
            ),
        ]
    }
}

/// 区域任务列表容器场景（可折叠内容）
fn section_content_list<L: Component + Default + Clone + Unpin, C: SceneList>(
    section_type: SectionType,
    list_marker: L,
    collapsed: bool,
    items: C,
) -> impl Scene + use<L, C> {
    let display = if collapsed {
        Display::None
    } else {
        Display::Flex
    };

    bsn! {
        SectionContent { section_type: {section_type} }
        template_value(list_marker)
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::top(Val::Px(10.0)),
            display: {display},
        }
        Children [ {items} ]
    }
}

/// 区域外框场景（标题 + 可折叠内容）
fn section_frame<S: Component + Default + Clone + Unpin, C: SceneList>(
    section_marker: S,
    contents: C,
) -> impl Scene + use<S, C> {
    bsn! {
        template_value(section_marker)
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            margin: UiRect::bottom(Val::Px(10.0)),
        }
        Children [ {contents} ]
    }
}

/// "下载中" 区域场景
fn downloading_section(
    collapsed: bool,
    tasks: &[&crate::resources::ComicDownloadTask],
) -> impl Scene + use<> {
    let title = format!("{ICON_DOWNLOAD} 下载中 ({})", tasks.len());
    let items: Vec<_> = tasks.iter().copied().map(download_task_item).collect();

    section_frame(
        DownloadingSection,
        bsn_list![
            // 可折叠标题
            collapsible_section_header(
                SectionType::Downloading,
                collapsed,
                DownloadingTitleText,
                title,
                AppColors::TEXT,
                false,
            ),
            // 任务列表容器（可折叠内容）
            section_content_list(SectionType::Downloading, DownloadTaskList, collapsed, items),
        ],
    )
}

/// "等待中" 区域场景
fn waiting_section(
    collapsed: bool,
    tasks: &[&crate::resources::ComicDownloadTask],
) -> impl Scene + use<> {
    let title = format!("{ICON_TIMER_SAND} 等待中 ({})", tasks.len());
    let items: Vec<_> = tasks.iter().copied().map(download_task_item).collect();

    section_frame(
        WaitingSection,
        bsn_list![
            // 可折叠标题
            collapsible_section_header(
                SectionType::Waiting,
                collapsed,
                WaitingTitleText,
                title,
                AppColors::TEXT,
                false,
            ),
            // 任务列表容器（可折叠内容）
            section_content_list(SectionType::Waiting, WaitingTaskList, collapsed, items),
        ],
    )
}

/// "已停止" 区域场景（标题行额外带 "全部开始" 按钮）
fn stopped_section(
    collapsed: bool,
    tasks: &[&crate::resources::ComicDownloadTask],
) -> impl Scene + use<> {
    let title = format!("{ICON_STOP} 已停止 ({})", tasks.len());
    let start_all_label = format!("{ICON_PLAY} 全部开始");
    let items: Vec<_> = tasks.iter().copied().map(download_task_item).collect();

    section_frame(
        StoppedSection,
        bsn_list![
            (
                // 标题行容器（包含可折叠标题 + 全部开始按钮）
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                }
                Children [
                    // 可折叠标题（左侧）
                    collapsible_section_header(
                        SectionType::Stopped,
                        collapsed,
                        StoppedTitleText,
                        title,
                        PAUSED_COLOR,
                        true,
                    ),
                    (
                        // 全部开始按钮（右侧）
                        StartAllDownloadsButton
                        Button
                        Node {
                            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(START_ALL_COLOR)
                        template_value(BorderColor::all(REDOWNLOAD_COLOR))
                        Children [
                            (
                                Text({start_all_label})
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
            // 任务列表容器（可折叠内容）
            section_content_list(SectionType::Stopped, StoppedTaskList, collapsed, items),
        ],
    )
}

/// "已下载" 区域场景
fn completed_section(collapsed: bool, downloads: &[CompletedDownload]) -> impl Scene + use<> {
    let title = format!("{ICON_CHECK} 已下载 ({})", downloads.len());
    let items: Vec<_> = downloads.iter().map(completed_download_item).collect();

    section_frame(
        CompletedSection,
        bsn_list![
            // 可折叠标题
            collapsible_section_header(
                SectionType::Completed,
                collapsed,
                CompletedTitleText,
                title,
                COMPLETED_COLOR,
                false,
            ),
            // 列表容器（可折叠内容）
            section_content_list(
                SectionType::Completed,
                CompletedDownloadList,
                collapsed,
                items,
            ),
        ],
    )
}

/// 浮动标题场景（当区域标题滚出视口时显示）
fn floating_header() -> impl Scene {
    bsn! {
        FloatingHeader
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(20.0),
            right: Val::Px(32.0), // 给滚动条留空间
            height: Val::Px(36.0),
            display: Display::None, // 初始隐藏，滚动时显示
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.98))
        ZIndex(100) // 提高 ZIndex 确保可见
        Children [
            (
                // 可点击按钮
                FloatingHeaderButton
                Button
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    column_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.98))
                template_value(BorderColor::all(AppColors::BORDER))
                Children [
                    (
                        // 折叠图标
                        FloatingHeaderIcon
                        Text(ICON_CHEVRON_DOWN)
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT_MUTED)
                    ),
                    (
                        // 标题文本
                        FloatingHeaderText
                        Text(" ")
                        TextFont { font_size: FontSize::Px(14.0) }
                        TextColor(AppColors::TEXT)
                    ),
                ]
            )
        ]
    }
}

/// 各统计类型对应的数字颜色
fn stat_value_color(stat_type: DownloadStatType) -> Color {
    match stat_type {
        DownloadStatType::Total => Color::WHITE,
        DownloadStatType::Downloading => Color::srgb(0.3, 0.5, 0.9),
        DownloadStatType::Waiting => Color::srgb(0.8, 0.7, 0.2),
        DownloadStatType::Paused => Color::srgb(0.8, 0.5, 0.2),
        DownloadStatType::Completed => Color::srgb(0.3, 0.7, 0.3),
        DownloadStatType::Failed => AppColors::ERROR,
    }
}

/// 各统计类型对应的中文标签
fn stat_label(stat_type: DownloadStatType) -> &'static str {
    match stat_type {
        DownloadStatType::Total => "总计",
        DownloadStatType::Downloading => "下载中",
        DownloadStatType::Waiting => "等待",
        DownloadStatType::Paused => "暂停",
        DownloadStatType::Completed => "已完成",
        DownloadStatType::Failed => "失败",
    }
}

/// 从 DownloadManagerState 中统计各状态任务数量
fn count_download_stats(download_state: &DownloadManagerState) -> [(DownloadStatType, usize); 6] {
    let mut downloading = 0usize;
    let mut waiting = 0usize;
    let mut paused = 0usize;
    let mut completed = 0usize;
    let mut failed = 0usize;

    for fsm in &download_state.fsm_tasks {
        match &fsm.meta.state {
            DownloadState::Downloading { .. } => downloading += 1,
            DownloadState::Queued => waiting += 1,
            DownloadState::Paused { .. } => paused += 1,
            DownloadState::Completed => completed += 1,
            DownloadState::Failed(_) => failed += 1,
        }
    }

    let total = download_state.fsm_tasks.len();

    [
        (DownloadStatType::Total, total),
        (DownloadStatType::Downloading, downloading),
        (DownloadStatType::Waiting, waiting),
        (DownloadStatType::Paused, paused),
        (DownloadStatType::Completed, completed),
        (DownloadStatType::Failed, failed),
    ]
}

/// 单个统计项场景（"标签: 数字"）
fn stat_item(stat_type: DownloadStatType, value: usize) -> impl Scene {
    let label = format!("{}: ", stat_label(stat_type));
    let value_text = format!("{value}");
    let value_color = stat_value_color(stat_type);

    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.0),
            margin: UiRect::right(Val::Px(16.0)),
        }
        Children [
            (
                // 标签文本
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 数字文本（带颜色 + 标记组件，用于动态更新）
                DownloadStatText { stat_type: {stat_type} }
                Text({value_text})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(value_color)
            ),
        ]
    }
}

/// 下载统计面板场景
fn download_stats_panel(download_state: &DownloadManagerState) -> impl Scene + use<> {
    let stats = count_download_stats(download_state);
    let items: Vec<_> = stats
        .iter()
        .map(|(stat_type, value)| stat_item(*stat_type, *value))
        .collect();

    bsn! {
        DownloadStatsPanel
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(8.0), Val::Px(8.0)),
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            border: UiRect::bottom(Val::Px(1.0)),
        }
        BackgroundColor(AppColors::SURFACE_SUNKEN)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [ {items} ]
    }
}

/// 标题栏小按钮场景
fn header_button<T: Component + Default + Clone + Unpin>(
    marker: T,
    label: String,
    color: Color,
) -> impl Scene + use<T> {
    bsn! {
        template_value(marker)
        Button
        Node {
            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(color)
        template_value(BorderColor::all(color))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(Color::WHITE)
            )
        ]
    }
}

/// 标题栏按钮场景（交互态交给全局 `apply_button_interaction`）
///
/// `rest_bg` 只是静止色的初值，避免建好到首次配色之间闪一帧。
fn styled_header_button<T: Component + Default + Clone + Unpin>(
    marker: T,
    label: String,
    style: ButtonStyle,
    rest_bg: Color,
    border: Color,
) -> impl Scene + use<T> {
    bsn! {
        template_value(marker)
        Button
        template_value(style)
        Node {
            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(rest_bg)
        template_value(BorderColor::all(border))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(AppColors::TEXT)
            )
        ]
    }
}

/// 下载标题栏场景
fn downloads_header() -> impl Scene {
    let title = format!("{ICON_DOWNLOAD} 下载管理");
    let update_all_label = format!("{ICON_REFRESH} 全部更新");
    let force_update_all_label = format!("{ICON_SYNC} 全部强制更新");
    let start_all_label = format!("{ICON_PLAY} 全部开始");
    let pause_all_label = format!("{ICON_PAUSE} 全部暂停");
    let open_images_label = format!("{ICON_FOLDER_OPEN} 原图");
    let open_cbz_label = format!("{ICON_FOLDER_OPEN} CBZ");

    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            padding: UiRect::horizontal(Val::Px(20.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::bottom(Val::Px(1.0)),
        }
        BackgroundColor(AppColors::HEADER_BG)
        template_value(BorderColor::all(AppColors::BORDER))
        Children [
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 按钮组容器
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                }
                Children [
                    // 全部更新按钮（章节数一致即跳过）
                    styled_header_button(
                        UpdateAllDownloadsButton { force: false },
                        update_all_label,
                        ButtonStyle::primary(),
                        AppColors::PRIMARY,
                        AppColors::PRIMARY,
                    ),
                    // 全部强制更新按钮（逐图校验，慢）
                    styled_header_button(
                        UpdateAllDownloadsButton { force: true },
                        force_update_all_label,
                        ButtonStyle::card(),
                        AppColors::SURFACE,
                        FORCE_REDOWNLOAD_COLOR,
                    ),
                    // 全部开始按钮
                    header_button(StartAllHeaderButton, start_all_label, START_ALL_COLOR),
                    // 全部暂停按钮
                    header_button(PauseAllHeaderButton, pause_all_label, RETRY_COLOR),
                    // 打开原图文件夹按钮
                    styled_header_button(
                        OpenDownloadFolderButton,
                        open_images_label,
                        ButtonStyle::card(),
                        AppColors::SURFACE,
                        AppColors::BORDER,
                    ),
                    // 打开 CBZ 文件夹按钮
                    styled_header_button(
                        OpenCbzFolderButton,
                        open_cbz_label,
                        ButtonStyle::card(),
                        AppColors::SURFACE,
                        AppColors::BORDER,
                    ),
                ]
            ),
        ]
    }
}

// ==================== 语义色 ====================
//
// 状态色与操作按钮的角色色。**建卡与交互系统必须引用同一常量**——
// 此前两边各写一份字面量，按钮一旦 hover 过就被交互系统改成另一套色，
// 视觉上"走样"且回不去。失败/删除类统一走 `AppColors::ERROR`。

/// 已暂停状态 / 暂停按钮（琥珀）
const PAUSED_COLOR: Color = Color::srgb(0.8, 0.6, 0.2);
/// 已完成状态（绿）
const COMPLETED_COLOR: Color = Color::srgb(0.3, 0.8, 0.3);
/// 继续按钮（绿）
const RESUME_COLOR: Color = Color::srgb(0.3, 0.7, 0.3);
/// 重试按钮 / 全部暂停按钮（橙）
const RETRY_COLOR: Color = Color::srgb(0.7, 0.5, 0.2);
/// 设置按钮（靛）
const SETTINGS_COLOR: Color = Color::srgb(0.5, 0.5, 0.7);
/// 打开文件夹按钮（灰）
const FOLDER_COLOR: Color = Color::srgb(0.5, 0.5, 0.6);
/// 移动按钮（蓝灰）
const MOVE_COLOR: Color = Color::srgb(0.5, 0.6, 0.8);
/// 更新 / 重新下载按钮（青绿）
const REDOWNLOAD_COLOR: Color = Color::srgb(0.3, 0.7, 0.4);
/// 强制更新按钮（琥珀——与普通更新区分，提示这是慢路径）
const FORCE_REDOWNLOAD_COLOR: Color = Color::srgb(0.85, 0.6, 0.25);
/// 全部开始按钮（深绿）
const START_ALL_COLOR: Color = Color::srgb(0.2, 0.5, 0.3);

/// 任务状态 → (显示文案, 文字颜色)
///
/// 建卡（`download_task_item`）与刷新（`refresh_downloads_ui`）共用，
/// 避免两条路径的文案/配色各写一份后走样。
fn task_status_label(task: &crate::resources::ComicDownloadTask) -> (String, Color) {
    match &task.status {
        ComicDownloadStatus::Waiting => ("等待中".to_string(), AppColors::TEXT_SECONDARY),
        ComicDownloadStatus::Downloading => (
            format!(
                "下载中 第{}/{}章 {}/{}",
                task.current_episode, task.total_episodes, task.current_page, task.total_pages
            ),
            AppColors::PRIMARY,
        ),
        ComicDownloadStatus::Paused => ("已暂停".to_string(), PAUSED_COLOR),
        ComicDownloadStatus::Completed => ("已完成".to_string(), COMPLETED_COLOR),
        ComicDownloadStatus::Failed(err) => (format!("失败: {}", err), AppColors::ERROR),
    }
}

/// 任务整体进度（0.0 ~ 1.0）：已完成章节占比 + 当前章节内页占比
fn task_progress(task: &crate::resources::ComicDownloadTask) -> f32 {
    if task.total_episodes == 0 {
        return 0.0;
    }
    let episode_progress = (task.current_episode - 1) as f32 / task.total_episodes as f32;
    let page_progress = if task.total_pages > 0 {
        task.current_page as f32 / task.total_pages as f32 / task.total_episodes as f32
    } else {
        0.0
    };
    (episode_progress + page_progress).clamp(0.0, 1.0)
}

/// 进行中进度条的亮度渐变端点（Hsla lightness；PRIMARY 基准亮度 0.5）
///
/// 刚开始浅（淡蓝），临近完成深（比基准色再沉一点）——色相/饱和度不动，
/// 只走亮度，一眼能读出进度段位
const PROGRESS_LIGHTNESS_START: f32 = 0.72;
const PROGRESS_LIGHTNESS_END: f32 = 0.42;

/// 进度条填充色（建卡与刷新共用同一映射）
///
/// 终态（完成/失败/暂停）用各自的固定状态色；进行中保持 PRIMARY 色相，
/// 按进度在 Hsla 亮度上从浅到深线性渐变
fn progress_fill_color(status: &ComicDownloadStatus, progress: f32) -> Color {
    match status {
        ComicDownloadStatus::Completed => COMPLETED_COLOR,
        ComicDownloadStatus::Failed(_) => AppColors::ERROR,
        ComicDownloadStatus::Paused => PAUSED_COLOR,
        _ => {
            let base = Hsla::from(AppColors::PRIMARY);
            let lightness = PROGRESS_LIGHTNESS_START
                + (PROGRESS_LIGHTNESS_END - PROGRESS_LIGHTNESS_START) * progress.clamp(0.0, 1.0);
            Color::from(Hsla { lightness, ..base })
        }
    }
}

/// 下载任务项场景
///
/// 页面初建（各区域列表）与动态新增任务（`add_new_task_ui`）共用同一实现，
/// 任何样式改动自动对两条路径同时生效。
fn download_task_item(task: &crate::resources::ComicDownloadTask) -> impl Scene + use<> {
    let (status_text, status_color) = task_status_label(task);

    let progress = task_progress(task);
    let progress_width = Val::Percent(progress * 100.0);
    let progress_color = progress_fill_color(&task.status, progress);

    // 判断是否可以暂停/继续/重试
    let can_pause = matches!(
        task.status,
        ComicDownloadStatus::Downloading | ComicDownloadStatus::Waiting
    );
    let can_resume = matches!(task.status, ComicDownloadStatus::Paused);
    let can_retry = matches!(task.status, ComicDownloadStatus::Failed(_));
    // 删除按钮始终可见（下载中时作为"取消"功能）
    let can_delete = true;

    let comic_title = task.comic_title.clone();
    let path_label = format!("{ICON_FOLDER} {}", task.save_path);
    // 组件字段值在 bsn! 内是延迟求值的 move 闭包，只能引用 owned 局部变量
    let item_comic_id = task.comic_id.clone();
    let title_comic_id = task.comic_id.clone();
    let status_comic_id = task.comic_id.clone();
    let bar_comic_id = task.comic_id.clone();

    // 分类和标签行（容器始终存在：分类/标签常在建卡后才由 API 返回，
    // update_download_task_tags 靠这个 marker 回填）
    let tags_comic_id = task.comic_id.clone();
    let badges: Vec<_> = task
        .categories
        .iter()
        .map(|category| tag_badge(category, TagColor::Category))
        .chain(task.tags.iter().map(|tag| tag_badge(tag, TagColor::Tag)))
        .collect();

    // 独立设置标注（如果有自定义设置）
    let settings_note: Box<dyn SceneList> =
        if task.custom_download_path.is_some() || task.custom_auto_pack_cbz.is_some() {
            let mut settings_parts = Vec::new();
            if let Some(ref path) = task.custom_download_path {
                settings_parts.push(format!("路径: {}", path));
            }
            if let Some(cbz) = task.custom_auto_pack_cbz {
                settings_parts.push(format!("CBZ: {}", if cbz { "开" } else { "关" }));
            }
            let note = format!("{ICON_COG} {}", settings_parts.join(" | "));
            Box::new(bsn_list![(
                Text({note})
                TextFont { font_size: FontSize::Px(10.0) }
                TextColor(Color::srgba(0.5, 0.5, 0.7, 0.8))
            )])
        } else {
            Box::new(bsn_list![])
        };

    bsn! {
        DownloadTaskItem { comic_id: {item_comic_id} }
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(Scale::SP_MD)),
            row_gap: Val::Px(Scale::SP_SM),
            border_radius: BorderRadius::all(Val::Px(Scale::R_MD)),
        }
        BackgroundColor(AppColors::SURFACE)
        Children [
            (
                // 标题行
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        // 漫画标题（可点击跳转详情）
                        DownloadTitleButton { comic_id: {title_comic_id} }
                        Button
                        Node
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text({comic_title})
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 状态标签
                        DownloadStatusText { comic_id: {status_comic_id} }
                        Text({status_text})
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(status_color)
                    ),
                ]
            ),
            (
                // 分类和标签容器（初始可能为空，API 返回后由
                // update_download_task_tags 补齐）
                DownloadTaskTagsContainer { comic_id: {tags_comic_id} }
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                }
                Children [ {badges} ]
            ),
            (
                // 进度条
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(4.0),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                }
                BackgroundColor(AppColors::SURFACE_HOVER)
                Children [
                    (
                        DownloadProgressBar { comic_id: {bar_comic_id} }
                        Node {
                            width: {progress_width},
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                        }
                        BackgroundColor(progress_color)
                    )
                ]
            ),
            (
                // 路径和控制按钮行
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        // 保存路径
                        Text({path_label})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        // 控制按钮组
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                        }
                        Children [
                            // 暂停按钮（始终创建，通过 display 控制可见性）
                            labeled_button_with_visibility(
                                format!("{ICON_PAUSE} 暂停"),
                                PauseDownloadButton { comic_id: task.comic_id.clone() },
                                PAUSED_COLOR,
                                can_pause,
                            ),
                            // 继续按钮（暂停状态显示）
                            labeled_button_with_visibility(
                                format!("{ICON_PLAY} 继续"),
                                ResumeDownloadButton { comic_id: task.comic_id.clone() },
                                RESUME_COLOR,
                                can_resume,
                            ),
                            // 重试按钮（失败状态显示）
                            labeled_button_with_visibility(
                                format!("{ICON_REFRESH} 重试"),
                                RetryDownloadButton { comic_id: task.comic_id.clone() },
                                RETRY_COLOR,
                                can_retry,
                            ),
                            // 删除按钮（始终可见）
                            labeled_button_with_visibility(
                                format!("{ICON_DELETE} 删除"),
                                DeleteDownloadButton { comic_id: task.comic_id.clone() },
                                AppColors::ERROR,
                                can_delete,
                            ),
                            // 设置按钮（独立下载设置）
                            labeled_button_with_visibility(
                                format!("{ICON_COG} 设置"),
                                DownloadTaskSettingsButton { comic_id: task.comic_id.clone() },
                                SETTINGS_COLOR,
                                true,
                            ),
                        ]
                    ),
                ]
            ),
            {settings_note},
        ]
    }
}

/// 带标签的按钮场景（已下载列表使用）
fn labeled_button<T: Component + Default + Clone + Unpin>(
    label: String,
    marker: T,
    color: Color,
) -> impl Scene + use<T> {
    let bg_color = color.with_alpha(0.2);

    bsn! {
        template_value(marker)
        Button
        Node {
            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(bg_color)
        template_value(BorderColor::all(color))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(color)
            )
        ]
    }
}

/// 带标签的按钮场景（等待/已停止区域使用，带可见性控制）
fn labeled_button_with_visibility<T: Component + Default + Clone + Unpin>(
    label: String,
    marker: T,
    color: Color,
    visible: bool,
) -> impl Scene + use<T> {
    let bg_color = color.with_alpha(0.2);
    let display = if visible {
        Display::Flex
    } else {
        Display::None
    };

    bsn! {
        template_value(marker)
        Button
        Node {
            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            display: {display},
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(bg_color)
        template_value(BorderColor::all(color))
        Children [
            (
                Text({label})
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(color)
            )
        ]
    }
}

/// 已下载漫画项场景
fn completed_download_item(download: &CompletedDownload) -> impl Scene + use<> {
    let path = download.path.clone();
    let has_comic_id = !download.comic_id.is_empty();
    let folder_name = download.folder_name.clone();
    let episode_label = format!("{} 章节 • {}", download.episode_count, path);

    // 分类和标签
    let has_categories = !download.categories.is_empty();
    let has_tags = !download.tags.is_empty();
    let tags_row: Box<dyn SceneList> = if has_categories || has_tags {
        // 显示所有分类 + 所有标签
        let badges: Vec<_> = download
            .categories
            .iter()
            .map(|category| tag_badge(category, TagColor::Category))
            .chain(
                download
                    .tags
                    .iter()
                    .map(|tag| tag_badge(tag, TagColor::Tag)),
            )
            .collect();
        Box::new(bsn_list![(
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(2.0),
                margin: UiRect::top(Val::Px(2.0)),
            }
            Children [ {badges} ]
        )])
    } else {
        Box::new(bsn_list![])
    };

    // 按钮组
    let mut buttons: Vec<Box<dyn Scene>> = Vec::new();
    // 更新按钮（快：只比对章节数，一致即跳过）
    if has_comic_id {
        buttons.push(Box::new(labeled_button(
            format!("{ICON_REFRESH} 更新"),
            RedownloadButton {
                comic_id: download.comic_id.clone(),
                force: false,
            },
            REDOWNLOAD_COLOR,
        )));
        // 强制更新按钮（慢：逐章拉图片列表、逐图补全缺失）
        buttons.push(Box::new(labeled_button(
            format!("{ICON_SYNC} 强制更新"),
            RedownloadButton {
                comic_id: download.comic_id.clone(),
                force: true,
            },
            FORCE_REDOWNLOAD_COLOR,
        )));
    }
    // 设置按钮（独立下载设置）
    if has_comic_id {
        buttons.push(Box::new(labeled_button(
            format!("{ICON_COG} 设置"),
            DownloadTaskSettingsButton {
                comic_id: download.comic_id.clone(),
            },
            SETTINGS_COLOR,
        )));
    }
    // 打开文件夹按钮
    buttons.push(Box::new(labeled_button(
        format!("{ICON_FOLDER_OPEN} 打开"),
        OpenCompletedFolderButton {
            path: download.path.clone(),
        },
        FOLDER_COLOR,
    )));
    // 移动按钮
    buttons.push(Box::new(labeled_button(
        format!("{ICON_FILE_MOVE} 移动"),
        MoveCompletedButton {
            comic_id: download.comic_id.clone(),
            comic_title: download.folder_name.clone(),
            path: download.path.clone(),
        },
        MOVE_COLOR,
    )));
    // 删除按钮
    buttons.push(Box::new(labeled_button(
        format!("{ICON_DELETE} 删除"),
        DeleteCompletedDownloadButton {
            comic_id: download.comic_id.clone(),
            path: download.path.clone(),
        },
        AppColors::ERROR,
    )));

    let item_marker = CompletedDownloadItem {
        comic_id: download.comic_id.clone(),
        path: download.path.clone(),
    };
    let title_marker = DownloadTitleButton {
        comic_id: download.comic_id.clone(),
    };

    bsn! {
        template_value(item_marker)
        // 行卡本身可点击（打开所在文件夹）——
        // completed_download_item_interaction 一直在等这个 Interaction
        Button
        template_value(ButtonStyle::card())
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(Scale::SP_MD)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(Scale::SP_MD),
            border_radius: BorderRadius::all(Val::Px(Scale::R_MD)),
        }
        BackgroundColor(AppColors::SURFACE)
        Children [
            (
                // 图标
                Text(ICON_BOOK)
                TextFont { font_size: FontSize::Px(24.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 信息列
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(4.0),
                }
                Children [
                    (
                        // 漫画名称（可点击跳转详情）
                        template_value(title_marker)
                        Button
                        Node
                        BackgroundColor(Color::NONE)
                        Children [
                            (
                                Text({folder_name})
                                TextFont { font_size: FontSize::Px(14.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                    (
                        // 章节数和路径
                        Text({episode_label})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    {tags_row},
                ]
            ),
            (
                // 按钮组
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                }
                Children [ {buttons} ]
            ),
        ]
    }
}

/// 清理下载页面（隐藏而非销毁）
pub fn cleanup_downloads_ui(mut commands: Commands, query: Query<Entity, With<DownloadsRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 打开下载文件夹按钮交互
pub fn open_download_folder_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<OpenDownloadFolderButton>)>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

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
}

/// 打开 CBZ 文件夹按钮交互
pub fn open_cbz_folder_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<OpenCbzFolderButton>)>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 打开 CBZ 文件夹
        let cbz_path = get_download_base_path().join("cbz");

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
}

/// 更新下载统计面板数字
///
/// 只在 `DownloadManagerState` 变化时重新统计并更新文本节点。
pub fn update_download_stats(
    download_state: Res<DownloadManagerState>,
    mut stat_text_query: Query<(&DownloadStatText, &mut Text)>,
) {
    if !download_state.is_changed() {
        return;
    }

    let stats = count_download_stats(&download_state);

    for (stat_text, mut text) in stat_text_query.iter_mut() {
        // 查找对应类型的统计值
        if let Some((_, value)) = stats.iter().find(|(t, _)| *t == stat_text.stat_type) {
            **text = format!("{value}");
        }
    }
}

/// 更新下载任务列表 UI（监听状态变化并实时更新进度条、
/// 状态文本和按钮可见性）（FSM 架构）
pub fn refresh_downloads_ui(
    download_state: Res<DownloadManagerState>,
    mut progress_bar_query: Query<(&DownloadProgressBar, &mut Node, &mut BackgroundColor)>,
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
        // 更新进度条宽度与填充色（进行中按进度做亮度渐变：开始浅、临完成深）
        let progress = task_progress(task);
        for (bar, mut node, mut bg_color) in progress_bar_query.iter_mut() {
            if bar.comic_id == task.comic_id {
                node.width = Val::Percent(progress * 100.0);
                *bg_color = BackgroundColor(progress_fill_color(&task.status, progress));
            }
        }

        // 更新状态文本（与建卡共用同一映射）
        let (status_text, status_color) = task_status_label(task);

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

    // 为每个新任务添加 UI
    for task in new_tasks {
        tracing::info!("动态添加下载任务 UI: {}", task.comic_title);

        // 与页面初建走同一个场景函数，样式不会因入口不同而分叉
        let task_entity = commands.spawn_scene(download_task_item(task)).id();

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
    // 下载状态未变化时零开销（此前每帧深拷贝全部任务 + 4 次 format!）
    if !download_state.is_changed() {
        return;
    }
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
        let new_text = format!("{ICON_DOWNLOAD} 下载中 ({})", downloading_count);
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
        let new_text = format!("{ICON_TIMER_SAND} 等待中 ({})", waiting_count);
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
        let new_text = format!("{ICON_STOP} 已停止 ({})", stopped_count);
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
        let new_text = format!("{ICON_CHECK} 已下载 ({})", completed_count);
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

/// 已下载项点击交互（点击打开文件夹）
pub fn completed_download_item_interaction(
    interaction_query: Query<(&Interaction, &CompletedDownloadItem), Changed<Interaction>>,
) {
    for (interaction, item) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 打开文件夹
        let path = std::path::Path::new(&item.path);
        if !path.exists() {
            tracing::warn!("文件夹不存在: {:?}", path);
            continue;
        }
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
                ICON_CHEVRON_RIGHT
            } else {
                ICON_CHEVRON_DOWN
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
    scroll_changed: Query<(), (With<DownloadsScrollContainer>, Changed<ScrollPosition>)>,
    download_state_probe: Res<DownloadManagerState>,
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
    // 只在滚动/下载状态/折叠状态变化时工作（此前每帧深拷贝全部任务）
    if scroll_changed.is_empty()
        && !download_state_probe.is_changed()
        && !collapse_state.is_changed()
    {
        return;
    }
    let Ok(scroll_pos) = scroll_query.single() else {
        return;
    };

    let scale_factor = window_query
        .single()
        .ok()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    // 标题行高度（大约）
    const HEADER_HEIGHT: f32 = 36.0;

    // 计算各区域的位置（累计高度）
    let mut section_positions: Vec<(SectionType, f32, f32, bool)> = Vec::new(); // (type, start, end, is_collapsed)
    let mut current_y: f32 = 0.0;

    // 获取任务数量用于标题显示（直接在 FSM 状态上计数，不做 UI 深拷贝）
    let all_tasks = download_state.active_tasks();

    let downloading_count = all_tasks
        .iter()
        .filter(|fsm| {
            matches!(
                fsm.meta.state,
                crate::resources::DownloadState::Downloading { .. }
            )
        })
        .count();
    let waiting_count = all_tasks
        .iter()
        .filter(|fsm| matches!(fsm.meta.state, crate::resources::DownloadState::Queued))
        .count();
    let stopped_count = all_tasks
        .iter()
        .filter(|fsm| {
            matches!(
                fsm.meta.state,
                crate::resources::DownloadState::Paused { .. }
                    | crate::resources::DownloadState::Failed(_)
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
                    ICON_DOWNLOAD,
                    format!("下载中 ({})", downloading_count),
                    AppColors::TEXT,
                ),
                SectionType::Waiting => (
                    ICON_TIMER_SAND,
                    format!("等待中 ({})", waiting_count),
                    AppColors::TEXT,
                ),
                SectionType::Stopped => (
                    ICON_STOP,
                    format!("已停止 ({})", stopped_count),
                    Color::srgb(0.8, 0.6, 0.2),
                ),
                SectionType::Completed => {
                    // 需要从数据库获取已完成数量，这里简化处理
                    (ICON_CHECK, "已下载".to_string(), Color::srgb(0.3, 0.8, 0.3))
                }
            };

            if let Ok(mut text_component) = floating_text_query.single_mut() {
                let label = format!("{} {} (点击跳转)", icon, text);
                if **text_component != label {
                    **text_component = label;
                }
            }

            // 图标始终显示向下箭头（点击跳转到该区域；
            // 比较后写避免每次触发文字整形）
            if let Ok(mut icon_component) = floating_icon_query.single_mut()
                && **icon_component != ICON_CHEVRON_DOWN
            {
                **icon_component = ICON_CHEVRON_DOWN.to_string();
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

                    // 按布局顺序计算目标位置：Downloading → Waiting → Stopped →
                    // Completed
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
        let pause_color = PAUSED_COLOR;
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
        let resume_color = RESUME_COLOR;
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
        let delete_color = AppColors::ERROR;
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
        let retry_color = RETRY_COLOR;
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

/// 重新下载按钮交互（普通更新 / 强制更新共用，按 `RedownloadButton.force`
/// 分流）
///
/// 点击更新时检查原下载目录是否存在：
/// - 存在 → 直接发送重新下载请求（原地更新/补全）
/// - 不存在 → 弹出文件选择对话框让用户选择新目录，从新目录开始下载
///
/// **不在这里摘除已下载列表项**：普通更新可能被前置检查判定为「已是最新」，
/// 摘了就要再补回来。摘除统一交给 `remove_completed_item_on_redownload`
/// （只在下载真正启动时触发）。
pub fn redownload_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &RedownloadButton),
        Changed<Interaction>,
    >,
    mut redownload_messages: MessageWriter<RedownloadRequest>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    use crate::resources::DownloadTaskMeta;

    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let base_color = if btn.force {
            FORCE_REDOWNLOAD_COLOR
        } else {
            REDOWNLOAD_COLOR
        };
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(base_color.with_alpha(0.4));

                // 检查路径是否存在，不存在时在后台弹对话框选择
                let comic_id = btn.comic_id.clone();
                let force = btn.force;
                let need_pick =
                    if let Ok(old_meta) = DownloadTaskMeta::load_by_comic_id(&btn.comic_id) {
                        let path = std::path::Path::new(old_meta.effective_download_path());
                        !path.exists()
                    } else {
                        false
                    };

                if need_pick {
                    // 需要选目录——放到后台避免卡死
                    runtime.spawn_background_task(move |mut ctx| async move {
                        let selected =
                            tokio::task::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
                                .await
                                .ok()
                                .flatten();

                        let Some(path) = selected else {
                            tracing::info!("用户取消选择目录，放弃重新下载");
                            return;
                        };
                        let new_base = path.to_string_lossy().to_string();
                        ctx.run_on_main_thread(move |ctx| {
                            ctx.world.write_message(RedownloadRequest {
                                comic_id,
                                new_base_path: Some(new_base),
                                force,
                            });
                        })
                        .await;
                    });
                } else {
                    // 路径存在，直接发送重新下载请求
                    redownload_messages.write(RedownloadRequest {
                        comic_id: comic_id.clone(),
                        new_base_path: None,
                        force,
                    });
                    tracing::info!(
                        "请求{}: {}",
                        if force {
                            "强制更新"
                        } else {
                            "检查更新"
                        },
                        comic_id
                    );
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(base_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(base_color.with_alpha(0.2));
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
        let folder_color = FOLDER_COLOR;
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

/// 移动已下载漫画按钮交互
///
/// 弹出目录选择对话框，将 Images 目录和 CBZ 文件同时移动到新位置，
/// 并更新数据库记录。移动完成后销毁旧的 UI 项并重建。
pub fn move_completed_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MoveCompletedButton),
        Changed<Interaction>,
    >,
    download_state: Res<DownloadManagerState>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let move_color = MOVE_COLOR;
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(move_color.with_alpha(0.4));

                // 如果任务正在下载中，不允许移动
                if download_state.downloading_ids.contains(&btn.comic_id) {
                    tracing::warn!("漫画 {} 正在下载中，无法移动", btn.comic_id);
                    continue;
                }

                let comic_id = btn.comic_id.clone();
                let comic_title = btn.comic_title.clone();
                let old_path = btn.path.clone();

                // 全部放到后台：对话框 + 文件移动
                runtime.spawn_background_task(move |_ctx| async move {
                    let selected =
                        tokio::task::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
                            .await
                            .ok()
                            .flatten();

                    let Some(selected_path) = selected else {
                        return;
                    };
                    let new_base = selected_path.to_string_lossy().to_string();
                    move_comic_files_async(&comic_id, &comic_title, &old_path, &new_base).await;
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(move_color.with_alpha(0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(move_color.with_alpha(0.2));
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
            let download = CompletedDownload {
                comic_id: comic_id.clone(),
                folder_name: comic_title,
                path: save_path.clone(),
                episode_count,
                categories,
                tags,
            };

            commands
                .spawn_scene(completed_download_item(&download))
                .insert(ChildOf(list_entity));

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
        let btn_color = START_ALL_COLOR;
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
            // 显示所有分类
            for category in task_categories.iter() {
                commands
                    .spawn_scene(tag_badge(category, TagColor::Category))
                    .insert(ChildOf(container_entity));
            }
            // 显示所有标签
            for tag in task_tags.iter() {
                commands
                    .spawn_scene(tag_badge(tag, TagColor::Tag))
                    .insert(ChildOf(container_entity));
            }
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

        // 创建设置面板
        let panel_entity = commands
            .spawn_scene(task_settings_panel(comic_id, custom_path, custom_cbz))
            .id();

        commands.entity(parent_entity).add_child(panel_entity);
    }
}

/// 下载任务独立设置面板场景（路径选择 + CBZ 三态开关）
fn task_settings_panel(
    comic_id: &str,
    custom_path: Option<String>,
    custom_cbz: Option<bool>,
) -> impl Scene + use<> {
    let panel_marker = DownloadTaskSettingsPanel {
        comic_id: comic_id.to_string(),
    };
    let path_select_marker = TaskPathSelectButton {
        comic_id: comic_id.to_string(),
    };
    let cbz_toggle_marker = TaskCbzToggle {
        comic_id: comic_id.to_string(),
    };
    let path_text = custom_path.unwrap_or_else(|| "(使用全局设置)".to_string());
    let select_label = format!("{ICON_FOLDER_OPEN} 选择");
    let cbz_text = match custom_cbz {
        Some(true) => "开启",
        Some(false) => "关闭",
        None => "(使用全局设置)",
    };
    let cbz_bg = match custom_cbz {
        Some(true) => Color::srgb(0.2, 0.5, 0.3).with_alpha(0.3),
        Some(false) => Color::srgb(0.5, 0.2, 0.2).with_alpha(0.3),
        None => AppColors::SURFACE,
    };

    bsn! {
        template_value(panel_marker)
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(8.0),
            border: UiRect::top(Val::Px(1.0)),
        }
        BackgroundColor(AppColors::HEADER_BG)
        template_value(BorderColor::all(Color::srgba(0.3, 0.3, 0.5, 0.5)))
        Children [
            (
                // 标题
                Text("独立设置")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(Color::srgb(0.5, 0.5, 0.7))
            ),
            (
                // 下载路径行
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                }
                Children [
                    (
                        Text("路径:")
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        Text({path_text})
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT)
                        Node { flex_grow: 1.0 }
                    ),
                    (
                        // 选择路径按钮
                        template_value(path_select_marker)
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text({select_label})
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
            (
                // CBZ 打包开关行
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                }
                Children [
                    (
                        Text("CBZ 打包:")
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(AppColors::TEXT_SECONDARY)
                    ),
                    (
                        // 三态切换按钮：全局 → 开启 → 关闭 → 全局
                        template_value(cbz_toggle_marker)
                        Button
                        Node {
                            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(4.0), Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(cbz_bg)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text({cbz_text})
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(AppColors::TEXT)
                            )
                        ]
                    ),
                ]
            ),
        ]
    }
}

/// 下载任务路径选择按钮交互（全异步：对话框 + 文件移动 + 路径更新）
pub fn task_path_select_interaction(
    mut interaction_query: Query<(&Interaction, &TaskPathSelectButton), Changed<Interaction>>,
    download_state: Res<DownloadManagerState>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (interaction, path_btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let comic_id = path_btn.comic_id.clone();

        // 从内存获取当前任务信息
        let Some(fsm) = download_state.find_task(&comic_id) else {
            continue;
        };
        let old_path = fsm.meta.save_path.clone();
        let comic_title = fsm.meta.comic_title.clone();

        // 全部放到后台线程
        runtime.spawn_background_task(move |mut ctx| async move {
            // 弹出目录选择（spawn_blocking 不阻塞 tokio runtime）
            let selected = tokio::task::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
                .await
                .ok()
                .flatten();

            let Some(selected_path) = selected else {
                return;
            };
            let new_base = selected_path.to_string_lossy().to_string();
            let folder_name = crate::utils::sanitize_filename(&comic_title);
            let new_full_path = std::path::Path::new(&new_base)
                .join("image")
                .join(&folder_name)
                .to_string_lossy()
                .to_string();

            // 移动文件
            move_comic_files_async(&comic_id, &comic_title, &old_path, &new_base).await;

            // 回主线程更新内存中的 FSM 路径
            let cid = comic_id.clone();
            let nfp = new_full_path.clone();
            let nb = new_base.clone();
            ctx.run_on_main_thread(move |ctx| {
                let mut state = ctx
                    .world
                    .get_resource_mut::<DownloadManagerState>()
                    .unwrap();
                if let Some(fsm) = state.find_task_mut(&cid) {
                    fsm.meta.save_path = nfp;
                    fsm.meta.custom_download_path = Some(nb);
                }
            })
            .await;
        });
    }
}

/// 递归移动目录
/// 异步移动漫画文件（Images 目录 + CBZ 文件 + 更新数据库）
async fn move_comic_files_async(comic_id: &str, comic_title: &str, old_path: &str, new_base: &str) {
    use crate::resources::DownloadTaskMeta;

    let old_images_path = std::path::Path::new(old_path);
    // 始终用漫画标题构建文件夹名（不依赖旧路径，防止历史数据污染）
    let folder_name = crate::utils::sanitize_filename(comic_title);

    let new_images_dir = std::path::Path::new(new_base)
        .join("image")
        .join(&folder_name);
    let new_images_path = new_images_dir.to_string_lossy().to_string();

    // 移动 Images 目录（在 blocking 线程中执行）
    let old_img = old_images_path.to_path_buf();
    let new_img = new_images_dir.clone();
    if old_img.exists() && old_img.is_dir() {
        if new_img.exists() {
            tracing::warn!("目标 Images 目录已存在: {}", new_images_path);
        } else {
            let result =
                tokio::task::spawn_blocking(move || move_dir_recursive(&old_img, &new_img)).await;
            match result {
                Ok(Ok(())) => {
                    tracing::info!("Images 目录已移动: {} -> {}", old_path, new_images_path)
                }
                Ok(Err(e)) => tracing::error!("移动 Images 目录失败: {}", e),
                Err(e) => tracing::error!("移动任务 panic: {}", e),
            }
        }
    }

    // 移动 CBZ 文件
    let old_cbz_dir = std::path::Path::new(old_path)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("cbz"));
    let cbz_filename = format!("{}.cbz", crate::utils::sanitize_filename(comic_title));
    let new_cbz_dir = std::path::Path::new(new_base).join("cbz");

    if let Some(ref old_cbz_parent) = old_cbz_dir {
        let old_cbz_file = old_cbz_parent.join(&cbz_filename);
        if old_cbz_file.exists() {
            let _ = tokio::fs::create_dir_all(&new_cbz_dir).await;
            let new_cbz_file = new_cbz_dir.join(&cbz_filename);
            if new_cbz_file.exists() {
                tracing::warn!("目标 CBZ 文件已存在: {}", new_cbz_file.display());
            } else if let Err(e) = tokio::fs::rename(&old_cbz_file, &new_cbz_file).await {
                // rename 失败（跨文件系统），用 copy + delete
                match tokio::fs::copy(&old_cbz_file, &new_cbz_file).await {
                    Ok(_) => {
                        let _ = tokio::fs::remove_file(&old_cbz_file).await;
                        tracing::info!(
                            "CBZ 文件已移动: {} -> {}",
                            old_cbz_file.display(),
                            new_cbz_file.display()
                        );
                    }
                    Err(copy_err) => {
                        tracing::error!("复制 CBZ 文件失败: {} (rename: {})", copy_err, e);
                    }
                }
            } else {
                tracing::info!(
                    "CBZ 文件已移动: {} -> {}",
                    old_cbz_file.display(),
                    new_cbz_file.display()
                );
            }
        }
    }

    // 更新数据库记录
    let cid = comic_id.to_string();
    let nip = new_images_path.clone();
    let nb = new_base.to_string();
    if let Ok(mut meta) = DownloadTaskMeta::load_by_comic_id(&cid) {
        meta.save_path = nip;
        meta.custom_download_path = Some(nb);
        if let Err(e) = meta.save() {
            tracing::error!("更新数据库下载路径失败: {}", e);
        } else {
            tracing::info!("数据库路径已更新: {}", new_images_path);
        }
    }
}

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
        let delete_color = AppColors::ERROR;
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

                // 创建确认面板
                let panel_entity = commands
                    .spawn_scene(delete_confirm_panel(comic_id, &btn.path))
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

/// 删除已下载漫画的确认面板场景
fn delete_confirm_panel(comic_id: &str, path: &str) -> impl Scene + use<> {
    let panel_marker = DeleteConfirmPanel {
        comic_id: comic_id.to_string(),
    };
    let checkbox_marker = DeleteFilesCheckbox {
        comic_id: comic_id.to_string(),
        checked: false,
    };
    let confirm_marker = ConfirmDeleteButton {
        comic_id: comic_id.to_string(),
        path: path.to_string(),
    };
    let cancel_marker = CancelDeleteButton {
        comic_id: comic_id.to_string(),
    };
    let confirm_color = AppColors::ERROR;
    let confirm_bg = confirm_color.with_alpha(0.3);
    let confirm_label = format!("{ICON_DELETE} 确认删除");

    bsn! {
        template_value(panel_marker)
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(8.0),
            border: UiRect::top(Val::Px(1.0)),
        }
        BackgroundColor(Color::srgb(0.12, 0.06, 0.06))
        template_value(BorderColor::all(Color::srgba(0.5, 0.2, 0.2, 0.5)))
        Children [
            (
                // 警告文本
                Text("确认删除此下载记录？")
                TextFont { font_size: FontSize::Px(12.0) }
                TextColor(Color::srgb(0.9, 0.5, 0.5))
            ),
            (
                // 勾选框行：同时删除磁盘文件
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                }
                Children [
                    (
                        // 勾选框按钮
                        template_value(checkbox_marker)
                        Button
                        Node {
                            width: Val::Px(18.0),
                            height: Val::Px(18.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.5)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                        }
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.14))
                        template_value(BorderColor::all(Color::srgb(0.5, 0.3, 0.3)))
                        Children [
                            (
                                // 初始状态：空（未勾选）
                                Text(" ")
                                TextFont { font_size: FontSize::Px(13.0) }
                                TextColor(Color::srgb(0.9, 0.4, 0.4))
                            )
                        ]
                    ),
                    (
                        Text("同时删除磁盘文件")
                        TextFont { font_size: FontSize::Px(11.0) }
                        TextColor(Color::srgb(0.9, 0.6, 0.6))
                    ),
                ]
            ),
            (
                // 按钮行
                Node {
                    column_gap: Val::Px(8.0),
                    margin: UiRect::top(Val::Px(4.0)),
                }
                Children [
                    (
                        // 确认删除按钮
                        template_value(confirm_marker)
                        Button
                        Node {
                            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(4.0), Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(confirm_bg)
                        template_value(BorderColor::all(confirm_color))
                        Children [
                            (
                                Text({confirm_label})
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(Color::srgb(0.9, 0.4, 0.4))
                            )
                        ]
                    ),
                    (
                        // 取消按钮
                        template_value(cancel_marker)
                        Button
                        template_value(ButtonStyle::card())
                        Node {
                            padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(4.0), Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BackgroundColor(AppColors::SURFACE)
                        template_value(BorderColor::all(AppColors::BORDER))
                        Children [
                            (
                                Text("取消")
                                TextFont { font_size: FontSize::Px(11.0) }
                                TextColor(AppColors::TEXT_SECONDARY)
                            )
                        ]
                    ),
                ]
            ),
        ]
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
                    ICON_CHECK.to_string()
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
    mut downloaded_index: ResMut<DownloadedComicsIndex>,
) {
    for (interaction, mut bg_color, btn) in interaction_query.iter_mut() {
        let confirm_color = AppColors::ERROR;
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

                // 同步封面角标索引（各列表页的对号随之消失）
                downloaded_index.remove(comic_id);

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

/// "全部更新" / "全部强制更新"按钮交互：对所有已下载漫画发起更新
///
/// 两个按钮共用本系统，由 `UpdateAllDownloadsButton.force` 区分：
/// - `false`：逐个走前置检查，章节数与本地一致的直接跳过（每本 1 个请求）
/// - `true`：跳过前置检查，逐章拉图片列表补全缺失文件（慢，用于修本地损坏）
pub fn update_all_downloads_button_interaction(
    interaction_query: Query<(&Interaction, &UpdateAllDownloadsButton), Changed<Interaction>>,
    download_state: Res<DownloadManagerState>,
    mut redownload_messages: MessageWriter<RedownloadRequest>,
    mut toast_state: ResMut<DownloadsToastState>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 从数据库获取所有已完成下载（不依赖 UI 实体）
        let active_ids: std::collections::HashSet<String> = download_state.downloading_ids.clone();
        let completed = scan_completed_downloads(&active_ids);

        for dl in &completed {
            redownload_messages.write(RedownloadRequest {
                comic_id: dl.comic_id.clone(),
                new_base_path: None,
                force: btn.force,
            });
        }

        let action = if btn.force {
            "全部强制更新"
        } else {
            "全部更新"
        };
        tracing::info!("{}：已对 {} 个已下载漫画发起请求", action, completed.len());
        toast_state.show(
            format!("{}：已提交 {} 本漫画", action, completed.len()),
            false,
        );
    }
}

// ==================== 更新结果反馈 ====================

/// 下载页 Toast 状态（更新检查结果提示）
#[derive(Resource, Default)]
pub struct DownloadsToastState {
    /// 待显示的消息（None = 无新消息）
    pub message: Option<String>,
    /// 是否为错误提示（红底）
    pub is_error: bool,
}

impl DownloadsToastState {
    /// 提交一条待显示的提示
    pub fn show(&mut self, message: impl Into<String>, is_error: bool) {
        self.message = Some(message.into());
        self.is_error = is_error;
    }
}

/// 下载页 Toast 标记
#[derive(Component, Default, Clone)]
pub struct DownloadsToast;

/// 下载页 Toast 自动消失计时器
#[derive(Resource)]
pub struct DownloadsToastTimer(pub Timer);

/// Toast 成功背景色（绿）
const TOAST_SUCCESS_COLOR: Color = Color::srgb(0.2, 0.55, 0.3);
/// Toast 失败背景色（红）
const TOAST_ERROR_COLOR: Color = Color::srgb(0.65, 0.2, 0.2);

/// 「已是最新 / 检查失败」→ 汇总成 Toast 文案
///
/// 「全部更新」会一次涌入几十条 `RedownloadSkipped`，逐条弹提示没法看，
/// 故按帧聚合：同一帧内多条只出一条汇总。
pub fn handle_redownload_skipped(
    mut messages: MessageReader<RedownloadSkipped>,
    mut toast_state: ResMut<DownloadsToastState>,
) {
    let mut up_to_date: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();

    for event in messages.read() {
        if event.error.is_some() {
            failed.push(event.comic_title.clone());
        } else {
            up_to_date.push(event.comic_title.clone());
        }
    }

    if up_to_date.is_empty() && failed.is_empty() {
        return;
    }

    // 单本时报书名，多本时报数量
    let describe = |titles: &[String], suffix: &str| -> String {
        match titles {
            [only] => format!("《{}》{}", truncate_text(only, 16), suffix),
            _ => format!("{} 本漫画{}", titles.len(), suffix),
        }
    };

    let message = match (up_to_date.as_slice(), failed.as_slice()) {
        ([], f) => describe(f, "更新检查失败"),
        (u, []) => describe(u, "已是最新"),
        (u, f) => format!("{}，{}", describe(u, "已是最新"), describe(f, "检查失败")),
    };
    toast_state.show(message, !failed.is_empty());
}

/// 更新确认后从「已下载」列表摘除该项（避免与新建的下载任务重复显示）
pub fn remove_completed_item_on_redownload(
    mut commands: Commands,
    mut confirmed: MessageReader<RedownloadConfirmed>,
    mut forced: MessageReader<RedownloadRequest>,
    completed_item_query: Query<(Entity, &CompletedDownloadItem)>,
) {
    let ids: Vec<String> = confirmed
        .read()
        .map(|event| event.comic_id.clone())
        .chain(
            forced
                .read()
                .filter(|event| event.force)
                .map(|event| event.comic_id.clone()),
        )
        .collect();

    if ids.is_empty() {
        return;
    }

    for (entity, item) in completed_item_query.iter() {
        if ids.contains(&item.comic_id) {
            commands.entity(entity).despawn();
            tracing::debug!("已从已下载列表移除: {}", item.comic_id);
        }
    }
}

/// 显示下载页 Toast（结构照搬首页签到 Toast）
pub fn display_downloads_toast(
    mut commands: Commands,
    mut toast_state: ResMut<DownloadsToastState>,
    toast_query: Query<Entity, With<DownloadsToast>>,
    root_query: Query<Entity, With<DownloadsRoot>>,
) {
    let Some(message) = toast_state.message.take() else {
        return;
    };

    // 旧 Toast 与计时器先清掉，避免叠加
    for entity in toast_query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DownloadsToastTimer>();

    let Ok(root) = root_query.single() else {
        return;
    };

    let bg_color = if toast_state.is_error {
        TOAST_ERROR_COLOR
    } else {
        TOAST_SUCCESS_COLOR
    };

    let toast = commands
        .spawn_scene(downloads_toast(message.as_str(), bg_color))
        .id();
    commands.entity(root).insert_children(0, &[toast]);
    commands.insert_resource(DownloadsToastTimer(Timer::from_seconds(
        3.0,
        TimerMode::Once,
    )));
}

/// 下载页 Toast 场景
fn downloads_toast(message: &str, bg_color: Color) -> impl Scene + use<> {
    let message = message.to_string();

    bsn! {
        DownloadsToast
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        BackgroundColor(bg_color)
        ZIndex(100)
        Children [
            (
                Text({message})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::WHITE)
            )
        ]
    }
}

/// 计时器到期后移除下载页 Toast
pub fn auto_hide_downloads_toast(
    mut commands: Commands,
    time: Res<Time>,
    timer: Option<ResMut<DownloadsToastTimer>>,
    toast_query: Query<Entity, With<DownloadsToast>>,
) {
    let Some(mut timer) = timer else {
        return;
    };

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        for entity in toast_query.iter() {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<DownloadsToastTimer>();
    }
}

/// 标题栏"全部开始"按钮交互：恢复所有已暂停/已失败的任务
pub fn start_all_header_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartAllHeaderButton>),
    >,
    download_state: Res<DownloadManagerState>,
    mut resume_messages: MessageWriter<ResumeDownloadRequest>,
) {
    let btn_color = START_ALL_COLOR;
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(btn_color.lighter(0.1));
                let mut count = 0;
                for task in download_state.active_tasks() {
                    if task.meta.state.can_resume() {
                        resume_messages.write(ResumeDownloadRequest {
                            comic_id: task.meta.comic_id.clone(),
                        });
                        count += 1;
                    }
                }
                tracing::info!("全部开始：恢复 {} 个任务", count);
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

/// 标题栏"全部暂停"按钮交互：暂停所有下载中的任务
pub fn pause_all_header_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PauseAllHeaderButton>),
    >,
    mut download_state: ResMut<DownloadManagerState>,
) {
    let btn_color = RETRY_COLOR;
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(btn_color.lighter(0.1));
                let mut count = 0;
                let ids: Vec<String> = download_state
                    .active_tasks()
                    .iter()
                    .filter(|t| t.meta.state.is_downloading())
                    .map(|t| t.meta.comic_id.clone())
                    .collect();
                for id in &ids {
                    if let Some(fsm) = download_state.find_task_mut(id) {
                        fsm.request_pause();
                        let _ = fsm.pause();
                        count += 1;
                    }
                }
                tracing::info!("全部暂停：暂停 {} 个任务", count);
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

/// "迁移全部"按钮交互：暂停活跃任务，弹出目录选择，后台串行迁移
pub fn move_all_downloads_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<MoveAllDownloadsButton>)>,
    mut download_state: ResMut<DownloadManagerState>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // 1. 暂停所有正在下载的任务，收集 control 句柄
            let mut controls = Vec::new();
            let active_ids: Vec<String> = download_state
                .active_tasks()
                .iter()
                .filter(|t| t.meta.state.is_downloading())
                .map(|t| t.meta.comic_id.clone())
                .collect();
            for id in &active_ids {
                if let Some(fsm) = download_state.find_task_mut(id) {
                    fsm.request_pause(); // 通知后台线程停止
                    let _ = fsm.pause(); // 更新状态为 Paused
                    controls.push(fsm.get_control());
                }
            }

            // 记住需要恢复的任务 ID
            let paused_ids = active_ids.clone();

            // 2. 全部放到后台：等待停止 + 数据库查询 + 目录选择 + 文件迁移
            runtime.spawn_background_task(move |mut ctx| async move {
                // 等待下载线程响应暂停（每张图片下载后会检查 pause_requested）
                if !controls.is_empty() {
                    tracing::info!("等待 {} 个下载任务停止...", controls.len());
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    tracing::info!("下载任务已停止");
                }
                // 从数据库获取所有任务
                let all_tasks = {
                    use picacg_db::{get_all_download_tasks_async, get_pool};
                    let pool = get_pool();
                    let db_tasks = get_all_download_tasks_async(&pool)
                        .await
                        .unwrap_or_default();
                    let tasks: Vec<(String, String, String)> = db_tasks
                        .into_iter()
                        .filter_map(|t| {
                            // 使用 save_path（完整图片路径，如
                            // /base/image/漫画名）
                            // 不用 custom_download_path（只是基础目录）
                            if t.save_path.is_empty() {
                                None
                            } else {
                                Some((t.comic_id, t.comic_title, t.save_path))
                            }
                        })
                        .collect();
                    tasks
                };

                if all_tasks.is_empty() {
                    tracing::info!("没有下载任务可迁移");
                    return;
                }

                // 使用设置中已配置的下载路径
                let new_base = AppSettings::global().read().download_path.clone();
                if new_base.is_empty() {
                    tracing::warn!("下载路径未设置，请先在上方选择目录");
                    return;
                }

                // 串行迁移
                let total = all_tasks.len();
                tracing::info!("开始迁移 {} 个漫画到 {}", total, new_base);
                for (i, (comic_id, comic_title, old_path)) in all_tasks.into_iter().enumerate() {
                    tracing::info!("迁移 [{}/{}]: {}", i + 1, total, comic_title);
                    move_comic_files_async(&comic_id, &comic_title, &old_path, &new_base).await;
                }
                tracing::info!("迁移全部完成：共 {} 个", total);

                // 更新内存中 FSM 的 save_path + 恢复下载
                let nb2 = new_base.clone();
                let paused = paused_ids;
                ctx.run_on_main_thread(move |ctx| {
                    // 更新所有 FSM 任务的 save_path
                    let mut state = ctx
                        .world
                        .get_resource_mut::<DownloadManagerState>()
                        .unwrap();
                    for fsm in &mut state.fsm_tasks {
                        let new_save = std::path::Path::new(&nb2)
                            .join("image")
                            .join(crate::utils::sanitize_filename(&fsm.meta.comic_title))
                            .to_string_lossy()
                            .to_string();
                        fsm.meta.save_path = new_save;
                        fsm.meta.custom_download_path = Some(nb2.clone());
                    }

                    // 恢复之前暂停的下载任务
                    if !paused.is_empty() {
                        tracing::info!("恢复 {} 个之前暂停的下载任务", paused.len());
                        for id in paused {
                            ctx.world
                                .write_message(crate::events::ResumeDownloadRequest {
                                    comic_id: id,
                                });
                        }
                    }
                })
                .await;
            });
        }
    }
}

#[cfg(test)]
mod progress_color_tests {
    use bevy::prelude::*;

    use super::{
        COMPLETED_COLOR, PAUSED_COLOR, PROGRESS_LIGHTNESS_END, PROGRESS_LIGHTNESS_START,
        progress_fill_color,
    };
    use crate::resources::ComicDownloadStatus;

    /// 进行中：进度越大亮度越低（开始浅、临完成深），端点精确命中配置值
    #[test]
    fn downloading_darkens_with_progress() {
        let at = |p: f32| Hsla::from(progress_fill_color(&ComicDownloadStatus::Downloading, p));
        assert!((at(0.0).lightness - PROGRESS_LIGHTNESS_START).abs() < 1e-5);
        assert!((at(1.0).lightness - PROGRESS_LIGHTNESS_END).abs() < 1e-5);
        assert!(at(0.2).lightness > at(0.8).lightness);
        // 越界进度钳位，不产生离谱颜色
        assert!((at(1.5).lightness - PROGRESS_LIGHTNESS_END).abs() < 1e-5);
    }

    /// 色相/饱和度必须与 PRIMARY 一致——只动亮度，不动色调
    #[test]
    fn downloading_keeps_primary_hue() {
        let base = Hsla::from(crate::systems::login::AppColors::PRIMARY);
        for p in [0.0, 0.5, 1.0] {
            let fill = Hsla::from(progress_fill_color(&ComicDownloadStatus::Downloading, p));
            assert!((fill.hue - base.hue).abs() < 0.5);
            assert!((fill.saturation - base.saturation).abs() < 1e-3);
        }
    }

    /// 终态颜色不随进度变化，仍是各自的固定状态色
    #[test]
    fn terminal_states_use_fixed_colors() {
        for p in [0.0, 0.5, 1.0] {
            assert_eq!(
                progress_fill_color(&ComicDownloadStatus::Completed, p),
                COMPLETED_COLOR
            );
            assert_eq!(
                progress_fill_color(&ComicDownloadStatus::Paused, p),
                PAUSED_COLOR
            );
        }
    }
}
