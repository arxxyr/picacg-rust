//! UI 组件标记
//!
//! 定义用于标识 UI 实体的组件

use bevy::prelude::*;

// ==================== 页面根节点标记 ====================

/// 登录页面根节点
#[derive(Component, Default, Clone)]
pub struct LoginRoot;

/// 主布局根节点
#[derive(Component, Default, Clone)]
pub struct MainLayoutRoot;

/// 分类页面根节点
#[derive(Component, Default, Clone)]
pub struct CategoriesRoot;

/// 漫画列表根节点
#[derive(Component, Default, Clone)]
pub struct ComicsListRoot;

/// 漫画详情根节点
#[derive(Component, Default, Clone)]
pub struct ComicDetailRoot;

// ==================== 登录页面组件 ====================

/// 登录按钮
#[derive(Component, Default, Clone)]
pub struct LoginButton;

/// 代理设置按钮（登录页）
#[derive(Component, Default, Clone)]
pub struct ProxySettingsButton;

/// 注册按钮（登录页）
#[derive(Component, Default, Clone)]
pub struct RegisterButton;

// ==================== 注册页面组件 ====================

/// 注册页面根节点
#[derive(Component, Default, Clone)]
pub struct RegisterRoot;

/// 注册页面输入框类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RegisterInputType {
    #[default]
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

/// 注册页面输入框标记（配合 TextInput 使用）
#[derive(Component, Default, Clone)]
pub struct RegisterInputField {
    pub input_type: RegisterInputType,
}

/// 注册页面性别按钮
#[derive(Component, Default, Clone)]
pub struct RegisterGenderButton {
    pub gender: crate::resources::Gender,
}

/// 注册提交按钮
#[derive(Component, Default, Clone)]
pub struct RegisterSubmitButton;

/// 返回登录按钮
#[derive(Component, Default, Clone)]
pub struct BackToLoginButton;

/// 注册错误消息
#[derive(Component, Default, Clone)]
pub struct RegisterErrorText;

/// 注册成功消息
#[derive(Component, Default, Clone)]
pub struct RegisterSuccessText;

/// 注册页面滚动容器
#[derive(Component, Default, Clone)]
pub struct RegisterScrollContainer;

/// 登录错误消息
#[derive(Component, Default, Clone)]
pub struct LoginErrorText;

/// 登录复选框类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoginCheckboxType {
    #[default]
    SavePassword,
    AutoLogin,
    AutoPunchIn,
}

/// 登录复选框组件
#[derive(Component, Default, Clone)]
pub struct LoginCheckbox {
    pub checkbox_type: LoginCheckboxType,
}

/// 复选框图标（用于标记勾选状态的文本）
#[derive(Component, Default, Clone)]
pub struct CheckboxIcon {
    pub checkbox_type: LoginCheckboxType,
}

// ==================== 主布局组件 ====================

/// 侧边栏
#[derive(Component, Default, Clone)]
pub struct Sidebar;

/// 侧边栏按钮
#[derive(Component, Default, Clone)]
pub struct SidebarButton {
    pub route: SidebarRoute,
}

/// 侧边栏路由
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SidebarRoute {
    #[default]
    Home,
    Categories,
    Search,
    Rankings,
    Games,
    Fried,
    Favorites,
    History,
    LikeRecords,
    Profile,
    LocalRead,
    Downloads,
    Settings,
    ImageConvert,
    Waifu2x,
    Nas,
    Chat,
}

/// 内容区域
#[derive(Component, Default, Clone)]
pub struct ContentArea;

// ==================== 分类页面组件 ====================

/// 分类卡片
#[derive(Component, Default, Clone)]
pub struct CategoryCard {
    pub title: String,
}

/// 分类图片
#[derive(Component, Default, Clone)]
pub struct CategoryImage {
    pub url: String,
}

// ==================== 漫画列表组件 ====================

/// 漫画卡片
#[derive(Component, Default, Clone)]
pub struct ComicCard {
    pub comic_id: String,
}

/// 漫画封面图片
#[derive(Component, Default, Clone)]
pub struct ComicThumbnail {
    pub url: String,
}

// ==================== 漫画详情组件 ====================

/// 漫画封面（`url` 供图片替换系统直接取用，避免每帧回查详情状态重算 URL）
#[derive(Component, Default, Clone)]
pub struct CoverImage {
    pub url: String,
}

/// 章节卡片
#[derive(Component, Default, Clone)]
pub struct EpisodeCard {
    pub episode_order: i32,
}

/// 点赞按钮
#[derive(Component, Default, Clone)]
pub struct LikeButton;

/// 收藏按钮
#[derive(Component, Default, Clone)]
pub struct FavoriteButton;

/// 开始阅读按钮
#[derive(Component, Default, Clone)]
pub struct StartReadButton;

// ==================== 右键菜单目标 ====================

/// 右键菜单目标（添加到所有漫画卡片上，供全局右键菜单系统使用）
#[derive(Component, Default, Clone)]
pub struct ContextMenuTarget {
    pub comic_id: String,
    pub comic_title: String,
    /// 列表接口给的 `epsCount`（0 = 未知）
    ///
    /// 顺着右键下载一路带到 `DownloadTaskMeta.remote_eps_count`，作为封面角标的
    /// 更新基准。卡片本来就持有 `Comic`，捎带这个字段是零成本；不带的话右键
    /// 下载的漫画永远没有基准，之后再有新章节也不会亮更新角标。
    pub eps_count: i32,
}

// ==================== 通用组件 ====================

/// 加载指示器
#[derive(Component, Default, Clone)]
pub struct LoadingIndicator;

/// 错误消息
#[derive(Component, Default, Clone)]
pub struct ErrorMessage;

/// 占位符图片
#[derive(Component, Default, Clone)]
pub struct PlaceholderImage;

// ==================== 滚动容器组件 ====================

/// 分类页面滚动容器
#[derive(Component, Default, Clone)]
pub struct CategoriesScrollContainer;

/// 漫画列表滚动容器
#[derive(Component, Default, Clone)]
pub struct ComicsScrollContainer;

/// 漫画详情滚动容器
#[derive(Component, Default, Clone)]
pub struct DetailScrollContainer;

// ==================== 忘记密码页面组件 ====================

/// 忘记密码页面根节点
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordRoot;

/// 忘记密码输入框类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ForgotPasswordInputType {
    /// 邮箱/用户名
    #[default]
    Email,
    /// 安全问题答案
    Answer,
}

/// 忘记密码输入框标记（配合 TextInput 使用）
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordInputField {
    pub input_type: ForgotPasswordInputType,
}

/// 忘记密码安全问题选择按钮
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordQuestionButton {
    pub question_no: i32,
}

/// 忘记密码提交按钮
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordSubmitButton;

/// 忘记密码返回登录按钮
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordBackButton;

/// 忘记密码错误消息文本
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordErrorText;

/// 忘记密码成功消息文本
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordSuccessText;

/// 忘记密码安全问题显示区域
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordQuestionsArea;

/// 忘记密码"忘记密码"链接（登录页）
#[derive(Component, Default, Clone)]
pub struct ForgotPasswordLink;

// ==================== 个人资料页面组件 ====================

/// 个人资料页面根节点
#[derive(Component, Default, Clone)]
pub struct ProfileRoot;

/// 个人资料头像图片
#[derive(Component, Default, Clone)]
pub struct ProfileAvatarImage {
    pub url: String,
}
