//! API 相关消息
//!
//! 定义与 API 请求/响应相关的消息 (Bevy 0.17 使用 Message 替代 Event)

use bevy::prelude::*;
use picacg_api::{
    endpoints::RankTimeType,
    models::{Category, Comic, Episode, Picture},
};

// ==================== 登录消息 ====================

/// 登录请求消息
#[derive(Message)]
pub struct LoginRequestEvent {
    pub email: String,
    pub password: String,
}

/// 登录响应消息
#[derive(Message)]
pub struct LoginResponseEvent {
    pub result: Result<String, String>,
}

/// 用户登录成功事件
///
/// 当用户成功登录后发送，用于通知需要等待登录的系统（如自动恢复下载）
#[derive(Message)]
pub struct UserLoggedInEvent;

// ==================== 注册消息 ====================

/// 注册请求消息
#[derive(Message)]
pub struct RegisterRequestEvent {
    pub email: String,
    pub password: String,
    pub name: String,
    pub birthday: String,
    pub gender: String,
    pub question1: String,
    pub question2: String,
    pub question3: String,
    pub answer1: String,
    pub answer2: String,
    pub answer3: String,
}

/// 注册响应消息
#[derive(Message)]
pub struct RegisterResponseEvent {
    pub result: Result<String, String>,
}

// ==================== 打卡消息 ====================

/// 打卡请求消息
#[derive(Message)]
pub struct PunchInRequestEvent;

/// 打卡响应消息
#[derive(Message)]
pub struct PunchInResponseEvent {
    pub result: Result<String, String>,
}

// ==================== 分类消息 ====================

/// 加载分类请求
#[derive(Message)]
pub struct LoadCategoriesRequest;

/// 分类加载完成
#[derive(Message)]
pub struct CategoriesLoadedEvent {
    pub categories: Vec<Category>,
}

/// 分类加载失败
#[derive(Message)]
pub struct CategoriesLoadFailedEvent {
    pub error: String,
}

// ==================== 漫画列表消息 ====================

/// 加载漫画列表请求
#[derive(Message)]
pub struct LoadComicsRequest {
    pub category: String,
    pub page: i32,
    pub sort: String,
}

/// 漫画列表加载完成
#[derive(Message)]
pub struct ComicsLoadedEvent {
    pub comics: Vec<Comic>,
    pub total_pages: i32,
    /// 当前加载的页码（用于区分首次加载和追加）
    pub page: i32,
}

/// 漫画列表加载失败
#[derive(Message)]
pub struct ComicsLoadFailedEvent {
    pub error: String,
}

// ==================== 漫画详情消息 ====================

/// 加载漫画详情请求
#[derive(Message)]
pub struct LoadComicDetailRequest {
    pub comic_id: String,
}

/// 漫画详情加载完成
#[derive(Message)]
pub struct ComicDetailLoadedEvent {
    pub comic: Comic,
}

/// 漫画详情加载失败
#[derive(Message)]
pub struct ComicDetailLoadFailedEvent {
    pub error: String,
}

// ==================== 章节列表消息 ====================

/// 加载章节列表请求（自动获取所有页）
#[derive(Message)]
pub struct LoadEpisodesRequest {
    pub comic_id: String,
}

/// 章节列表加载完成
#[derive(Message)]
pub struct EpisodesLoadedEvent {
    pub episodes: Vec<Episode>,
    pub total_pages: i32,
}

/// 章节列表加载失败
#[derive(Message)]
pub struct EpisodesLoadFailedEvent {
    pub error: String,
}

// ==================== 图片列表消息 ====================

/// 加载图片列表请求
#[derive(Message)]
pub struct LoadPicturesRequest {
    pub comic_id: String,
    pub episode_order: i32,
    pub page: i32,
}

/// 图片列表加载完成
#[derive(Message)]
pub struct PicturesLoadedEvent {
    pub pictures: Vec<Picture>,
    pub total_pages: i32,
}

/// 图片列表加载失败
#[derive(Message)]
pub struct PicturesLoadFailedEvent {
    pub error: String,
}

// ==================== 搜索消息 ====================

/// 搜索漫画请求
#[derive(Message)]
pub struct SearchComicsRequestEvent {
    pub keyword: String,
    pub page: i32,
    pub sort: String,
    /// 分类过滤
    pub categories: Vec<String>,
}

/// 搜索结果加载完成
#[derive(Message)]
pub struct SearchResultsLoadedEvent {
    pub comics: Vec<Comic>,
    pub total_pages: i32,
    pub keyword: String,
}

/// 搜索失败
#[derive(Message)]
pub struct SearchFailedEvent {
    pub error: String,
}

// ==================== 点赞/收藏消息 ====================

/// 点赞漫画请求
#[derive(Message)]
pub struct LikeComicRequest {
    pub comic_id: String,
}

/// 点赞漫画响应
#[derive(Message)]
pub struct LikeComicResponse {
    pub action: String,
}

/// 收藏漫画请求
#[derive(Message)]
pub struct FavoriteComicRequest {
    pub comic_id: String,
}

/// 收藏漫画响应
#[derive(Message)]
pub struct FavoriteComicResponse {
    pub action: String,
}

// ==================== 收藏列表消息 ====================

/// 加载收藏列表请求
#[derive(Message)]
pub struct LoadFavoritesRequest {
    pub page: i32,
    pub sort: String,
}

/// 收藏列表加载完成
#[derive(Message)]
pub struct FavoritesLoadedEvent {
    pub comics: Vec<Comic>,
    pub total_pages: i32,
}

/// 收藏列表加载失败
#[derive(Message)]
pub struct FavoritesLoadFailedEvent {
    pub error: String,
}

// ==================== 首页消息 ====================

/// 加载推荐漫画请求
#[derive(Message)]
pub struct LoadRecommendationsRequest;

/// 推荐漫画加载完成
#[derive(Message)]
pub struct RecommendationsLoadedEvent {
    pub comics: Vec<Comic>,
}

/// 推荐漫画加载失败
#[derive(Message)]
pub struct RecommendationsLoadFailedEvent {
    pub error: String,
}

// ==================== 图片加载消息 ====================

/// 加载图片请求
#[derive(Message)]
pub struct LoadImageRequest {
    pub url: String,
}

/// 图片加载完成
#[derive(Message)]
pub struct ImageLoadedEvent {
    pub url: String,
    pub handle: Handle<Image>,
}

/// 图片加载失败
#[derive(Message)]
pub struct ImageLoadFailedEvent {
    pub url: String,
    pub error: String,
}

// ==================== 下载消息 ====================

/// 下载漫画请求
#[derive(Message)]
pub struct DownloadComicRequest {
    /// 漫画 ID
    pub comic_id: String,
    /// 漫画标题（用于创建目录）
    pub comic_title: String,
    /// 要下载的章节列表（空表示下载全部）
    pub episodes: Vec<i32>,
}

/// 下载进度更新事件
#[derive(Message)]
pub struct DownloadProgressEvent {
    /// 漫画 ID
    pub comic_id: String,
    /// 当前章节
    pub current_episode: i32,
    /// 总章节数
    pub total_episodes: i32,
    /// 当前章节已下载图片数
    pub current_page: i32,
    /// 当前章节总图片数
    pub total_pages: i32,
    /// 下载状态描述
    pub status: String,
}

/// 下载完成事件
#[derive(Message)]
pub struct DownloadCompletedEvent {
    pub comic_id: String,
    pub save_path: String,
}

/// 下载失败事件
#[derive(Message)]
pub struct DownloadFailedEvent {
    pub comic_id: String,
    pub error: String,
}

/// 下载暂停事件（后台任务发出，通知主线程下载已暂停）
#[derive(Message)]
pub struct DownloadPausedEvent {
    pub comic_id: String,
}

/// 恢复下载请求（主线程发出，请求从断点继续下载）
#[derive(Message)]
pub struct ResumeDownloadRequest {
    pub comic_id: String,
}

/// 重新下载请求（检查更新/补全缺失）
#[derive(Message)]
pub struct RedownloadRequest {
    pub comic_id: String,
}

// ==================== 排行榜消息 ====================

/// 加载排行榜请求
#[derive(Message)]
pub struct LoadRankingsRequest {
    pub time_type: RankTimeType,
}

/// 排行榜加载完成
#[derive(Message)]
pub struct RankingsLoadedEvent {
    pub time_type: RankTimeType,
    pub comics: Vec<Comic>,
}

/// 排行榜加载失败
#[derive(Message)]
pub struct RankingsLoadFailedEvent {
    pub error: String,
}

// ==================== API 客户端重载消息 ====================

/// 重新加载 API 客户端配置（通道/代理变更时触发）
#[derive(Message)]
pub struct ReloadApiClientEvent;

// ==================== CBZ 打包消息 ====================

/// CBZ 打包请求
#[derive(Message)]
pub struct CbzPackageRequest {
    /// 漫画 ID
    pub comic_id: String,
    /// 漫画标题
    pub comic_title: String,
    /// 原图文件夹路径
    pub source_path: String,
}

/// CBZ 打包完成事件
#[derive(Message)]
pub struct CbzPackageCompletedEvent {
    /// 漫画 ID
    pub comic_id: String,
    /// CBZ 文件路径
    pub cbz_path: String,
}

/// CBZ 打包失败事件
#[derive(Message)]
pub struct CbzPackageFailedEvent {
    /// 漫画 ID
    pub comic_id: String,
    /// 错误信息
    pub error: String,
}
