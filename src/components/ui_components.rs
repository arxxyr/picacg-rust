//! UI 组件标记
//!
//! 定义用于标识 UI 实体的组件，部分预留

#![allow(dead_code)]

use bevy::prelude::*;

// ==================== 页面根节点标记 ====================

/// 登录页面根节点
#[derive(Component)]
pub struct LoginRoot;

/// 主布局根节点
#[derive(Component)]
pub struct MainLayoutRoot;

/// 分类页面根节点
#[derive(Component)]
pub struct CategoriesRoot;

/// 漫画列表根节点
#[derive(Component)]
pub struct ComicsListRoot;

/// 漫画详情根节点
#[derive(Component)]
pub struct ComicDetailRoot;

/// 阅读器根节点
#[derive(Component)]
pub struct ReaderRoot;

/// 代理设置根节点
#[derive(Component)]
pub struct ProxySettingsRoot;

// ==================== 登录页面组件 ====================

/// 用户名输入框
#[derive(Component)]
pub struct UsernameInput;

/// 密码输入框
#[derive(Component)]
pub struct PasswordInput;

/// 登录按钮
#[derive(Component)]
pub struct LoginButton;

/// 代理设置按钮（登录页）
#[derive(Component)]
pub struct ProxySettingsButton;

/// 注册按钮（登录页）
#[derive(Component)]
pub struct RegisterButton;

// ==================== 注册页面组件 ====================

/// 注册页面根节点
#[derive(Component)]
pub struct RegisterRoot;

/// 注册页面输入框类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterInputType {
    Email,
    Password,
    ConfirmPassword,
    Name,
    Birthday,
    Question1,
    Question2,
    Question3,
    Answer1,
    Answer2,
    Answer3,
}

/// 注册页面输入框
#[derive(Component)]
pub struct RegisterInputField {
    pub input_type: RegisterInputType,
    pub focused: bool,
}

/// 注册页面性别按钮
#[derive(Component)]
pub struct RegisterGenderButton {
    pub gender: crate::resources::Gender,
}

/// 注册提交按钮
#[derive(Component)]
pub struct RegisterSubmitButton;

/// 返回登录按钮
#[derive(Component)]
pub struct BackToLoginButton;

/// 注册错误消息
#[derive(Component)]
pub struct RegisterErrorText;

/// 注册成功消息
#[derive(Component)]
pub struct RegisterSuccessText;

/// 注册页面滚动容器
#[derive(Component)]
pub struct RegisterScrollContainer;

/// 登录错误消息
#[derive(Component)]
pub struct LoginErrorText;

/// 登录复选框类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginCheckboxType {
    SavePassword,
    AutoLogin,
    AutoPunchIn,
}

/// 登录复选框组件
#[derive(Component)]
pub struct LoginCheckbox {
    pub checkbox_type: LoginCheckboxType,
}

/// 复选框图标（用于标记勾选状态的文本）
#[derive(Component)]
pub struct CheckboxIcon {
    pub checkbox_type: LoginCheckboxType,
}

// ==================== 主布局组件 ====================

/// 侧边栏
#[derive(Component)]
pub struct Sidebar;

/// 侧边栏按钮
#[derive(Component)]
pub struct SidebarButton {
    pub route: SidebarRoute,
}

/// 侧边栏路由
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidebarRoute {
    Home,
    Categories,
    Search,
    Rankings,
    Favorites,
    Downloads,
    Settings,
}

/// 内容区域
#[derive(Component)]
pub struct ContentArea;

// ==================== 分类页面组件 ====================

/// 分类卡片
#[derive(Component)]
pub struct CategoryCard {
    pub title: String,
}

/// 分类图片
#[derive(Component)]
pub struct CategoryImage {
    pub url: String,
}

// ==================== 漫画列表组件 ====================

/// 漫画卡片
#[derive(Component)]
pub struct ComicCard {
    pub comic_id: String,
}

/// 漫画封面图片
#[derive(Component)]
pub struct ComicThumbnail {
    pub url: String,
}

/// 分页控件
#[derive(Component)]
pub struct PaginationControl;

/// 上一页按钮
#[derive(Component)]
pub struct PrevPageButton;

/// 下一页按钮
#[derive(Component)]
pub struct NextPageButton;

/// 页码文本
#[derive(Component)]
pub struct PageNumberText;

// ==================== 漫画详情组件 ====================

/// 漫画封面
#[derive(Component)]
pub struct CoverImage;

/// 章节卡片
#[derive(Component)]
pub struct EpisodeCard {
    pub episode_order: i32,
}

/// 点赞按钮
#[derive(Component)]
pub struct LikeButton;

/// 收藏按钮
#[derive(Component)]
pub struct FavoriteButton;

/// 开始阅读按钮
#[derive(Component)]
pub struct StartReadButton;

// ==================== 阅读器组件 ====================

/// 阅读器图片
#[derive(Component)]
pub struct ReaderImage;

/// 阅读器工具栏
#[derive(Component)]
pub struct ReaderToolbar;

/// 返回按钮
#[derive(Component)]
pub struct BackButton;

/// 上一页按钮（阅读器）
#[derive(Component)]
pub struct PrevPictureButton;

/// 下一页按钮（阅读器）
#[derive(Component)]
pub struct NextPictureButton;

/// 缩放按钮
#[derive(Component)]
pub struct ZoomButton {
    pub zoom_type: ZoomType,
}

/// 缩放类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomType {
    In,
    Out,
    Reset,
}

// ==================== 通用组件 ====================

/// 加载指示器
#[derive(Component)]
pub struct LoadingIndicator;

/// 错误消息
#[derive(Component)]
pub struct ErrorMessage;

/// 成功消息
#[derive(Component)]
pub struct SuccessMessage;

/// 占位符图片
#[derive(Component)]
pub struct PlaceholderImage;

// ==================== 滚动容器组件 ====================

/// 分类页面滚动容器
#[derive(Component)]
pub struct CategoriesScrollContainer;

/// 漫画列表滚动容器
#[derive(Component)]
pub struct ComicsScrollContainer;

/// 漫画详情滚动容器
#[derive(Component)]
pub struct DetailScrollContainer;

// ==================== 自定义滚动条组件 ====================

/// 滚动条容器（包含轨道和滑块）
#[derive(Component)]
pub struct ScrollbarContainer {
    /// 关联的滚动容器实体
    pub scroll_container: Entity,
}

/// 滚动条轨道（可点击跳转）
#[derive(Component)]
pub struct ScrollbarTrack {
    /// 关联的滚动容器实体
    pub scroll_container: Entity,
}

/// 滚动条滑块（可拖拽）
#[derive(Component)]
pub struct ScrollbarThumb {
    /// 关联的滚动容器实体
    pub scroll_container: Entity,
}

/// 滚动条拖拽状态
#[derive(Resource, Default)]
pub struct ScrollbarDragState {
    /// 是否正在拖拽
    pub is_dragging: bool,
    /// 正在拖拽的滚动容器实体
    pub dragging_thumb: Option<Entity>,
    /// 拖拽开始时的鼠标 Y 坐标
    pub drag_start_y: f32,
    /// 拖拽开始时的滚动位置
    pub drag_start_scroll: f32,
}

impl ScrollbarDragState {
    /// 开始拖拽
    pub fn start_drag(&mut self, scroll_container: Entity, mouse_y: f32, scroll_y: f32) {
        self.is_dragging = true;
        self.dragging_thumb = Some(scroll_container);
        self.drag_start_y = mouse_y;
        self.drag_start_scroll = scroll_y;
    }

    /// 结束拖拽
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.dragging_thumb = None;
        self.drag_start_y = 0.0;
        self.drag_start_scroll = 0.0;
    }
}

/// 内容尺寸信息（用于计算滚动条）
#[derive(Component, Default, Clone, Copy)]
pub struct ContentSizeInfo {
    /// 内容总高度
    pub content_height: f32,
    /// 可视区域高度
    pub viewport_height: f32,
}

impl ContentSizeInfo {
    /// 计算网格布局的内容高度
    ///
    /// # 参数
    /// - `item_count`: 项目数量
    /// - `item_width`: 每个项目的宽度
    /// - `item_height`: 每个项目的高度
    /// - `container_width`: 容器宽度（不含滚动条）
    /// - `column_gap`: 列间距
    /// - `row_gap`: 行间距
    /// - `padding_top`: 上内边距
    /// - `padding_bottom`: 下内边距
    pub fn calculate_grid_content_height(
        item_count: usize,
        item_width: f32,
        item_height: f32,
        container_width: f32,
        column_gap: f32,
        row_gap: f32,
        padding_top: f32,
        padding_bottom: f32,
    ) -> f32 {
        if item_count == 0 || container_width <= 0.0 {
            return 0.0;
        }

        // 计算每行能放多少个项目
        let items_per_row = ((container_width + column_gap) / (item_width + column_gap))
            .floor()
            .max(1.0) as usize;

        // 计算需要多少行
        let row_count = (item_count + items_per_row - 1) / items_per_row;

        // 计算内容高度
        if row_count == 0 {
            padding_top + padding_bottom
        } else {
            padding_top
                + (row_count as f32 * item_height)
                + ((row_count - 1) as f32 * row_gap)
                + padding_bottom
        }
    }
}
