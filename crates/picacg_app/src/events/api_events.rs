//! API 相关消息
//!
//! 定义与 API 请求/响应相关的消息 (Bevy 0.17 使用 Message 替代 Event)

use bevy::prelude::*;
use picacg_api::{
    endpoints::{RankTimeType, rank::KnightUser},
    models::{Category, Comic, Comment, Episode, Game, Picture},
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

// ==================== 热词消息 ====================

/// 加载热词请求
#[derive(Message)]
pub struct LoadKeywordsRequest;

/// 热词加载完成
#[derive(Message)]
pub struct KeywordsLoadedEvent {
    pub keywords: Vec<String>,
}

/// 热词加载失败
#[derive(Message)]
pub struct KeywordsLoadFailedEvent {
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
    pub comic_id: String,
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
    /// 新的基础目录（原路径不存在时由用户选择）
    /// 实际保存路径 = new_base_path/Images/漫画文件夹名
    pub new_base_path: Option<String>,
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

// ==================== 骑士榜消息 ====================

/// 加载骑士榜请求
#[derive(Message)]
pub struct LoadKnightRankingsRequest;

/// 骑士榜加载完成
#[derive(Message)]
pub struct KnightRankingsLoadedEvent {
    pub users: Vec<KnightUser>,
}

/// 骑士榜加载失败
#[derive(Message)]
pub struct KnightRankingsLoadFailedEvent {
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

// ==================== 历史记录消息 ====================

/// 加载历史记录请求
#[derive(Message)]
pub struct LoadHistoryRequest;

/// 历史记录加载完成
#[derive(Message)]
pub struct HistoryLoadedEvent {
    pub records: Vec<picacg_db::DbHistory>,
    pub total_count: i64,
}

/// 历史记录加载失败
#[derive(Message)]
pub struct HistoryLoadFailedEvent {
    pub error: String,
}

/// 保存历史记录请求（阅读器发出）
#[derive(Message)]
pub struct SaveHistoryRequest {
    pub comic_id: String,
    pub comic_title: String,
    pub thumb_url: String,
    pub last_eps_order: i32,
    pub last_eps_title: String,
    pub last_page: i32,
}

/// 删除历史记录请求
#[derive(Message)]
pub struct DeleteHistoryRequest {
    pub comic_id: String,
}

/// 清空所有历史记录请求
#[derive(Message)]
pub struct ClearAllHistoryRequest;

// ==================== 点赞记录消息 ====================

/// 加载点赞记录请求
#[derive(Message)]
pub struct LoadLikeRecordsRequest;

/// 点赞记录加载完成
#[derive(Message)]
pub struct LikeRecordsLoadedEvent {
    pub records: Vec<picacg_db::DbLikeRecord>,
    pub total_count: i64,
}

/// 点赞记录加载失败
#[derive(Message)]
pub struct LikeRecordsLoadFailedEvent {
    pub error: String,
}

/// 保存点赞记录请求（点赞成功时触发）
#[derive(Message)]
pub struct SaveLikeRecordRequest {
    pub comic_id: String,
    pub comic_title: String,
    pub thumb_url: String,
}

/// 删除点赞记录请求（取消点赞时触发）
#[derive(Message)]
pub struct DeleteLikeRecordRequest {
    pub comic_id: String,
}

// ==================== 评论消息 ====================

/// 加载评论列表请求
#[derive(Message)]
pub struct LoadCommentsRequest {
    pub comic_id: String,
    pub page: i32,
}

/// 评论列表加载完成
#[derive(Message)]
pub struct CommentsLoadedEvent {
    pub comments: Vec<Comment>,
    pub total_pages: i32,
    pub page: i32,
}

/// 评论列表加载失败
#[derive(Message)]
pub struct CommentsLoadFailedEvent {
    pub error: String,
}

/// 发表评论请求
#[derive(Message)]
pub struct PostCommentRequest {
    pub comic_id: String,
    pub content: String,
}

/// 发表评论响应
#[derive(Message)]
pub struct PostCommentResponseEvent {
    pub success: bool,
    pub error: Option<String>,
}

/// 回复评论请求
#[derive(Message)]
pub struct PostCommentReplyRequest {
    pub comment_id: String,
    pub content: String,
}

/// 回复评论响应
#[derive(Message)]
pub struct PostCommentReplyResponseEvent {
    pub success: bool,
    pub error: Option<String>,
}

/// 点赞评论请求
#[derive(Message)]
pub struct LikeCommentRequestEvent {
    pub comment_id: String,
}

/// 点赞评论响应
#[derive(Message)]
pub struct LikeCommentResponseEvent {
    pub comment_id: String,
    pub action: String,
}

/// 加载子评论请求
#[derive(Message)]
pub struct LoadChildCommentsRequest {
    pub comment_id: String,
    pub page: i32,
}

/// 子评论加载完成
#[derive(Message)]
pub struct ChildCommentsLoadedEvent {
    pub comment_id: String,
    pub comments: Vec<Comment>,
    pub total_pages: i32,
    pub page: i32,
}

// ==================== 个人资料消息 ====================

/// 加载用户个人资料请求
#[derive(Message)]
pub struct LoadUserProfileRequest;

/// 用户个人资料加载完成
#[derive(Message)]
pub struct UserProfileLoadedEvent {
    pub user: picacg_api::models::User,
}

/// 用户个人资料加载失败
#[derive(Message)]
pub struct UserProfileLoadFailedEvent {
    pub error: String,
}

// ==================== 版本更新消息 ====================

/// 检查更新请求
#[derive(Message)]
pub struct CheckUpdateRequest;

/// 检查更新响应
#[derive(Message)]
pub struct CheckUpdateResponse {
    /// 最新版本号
    pub latest_version: String,
    /// 当前版本号
    pub current_version: String,
    /// 是否有更新
    pub has_update: bool,
    /// 更新说明（如果有）
    pub release_notes: Option<String>,
    /// 下载链接
    pub download_url: Option<String>,
}

/// 检查更新失败事件
#[derive(Message)]
pub struct CheckUpdateFailedEvent {
    pub error: String,
}

// ==================== 忘记/重置密码消息 ====================

/// 忘记密码请求（获取安全问题）
#[derive(Message)]
pub struct ForgotPasswordRequestEvent {
    pub email: String,
}

/// 忘记密码响应（返回安全问题）
#[derive(Message)]
pub struct ForgotPasswordResponseEvent {
    pub result: Result<(String, String, String), String>,
}

/// 重置密码请求（通过安全问题）
#[derive(Message)]
pub struct ResetPasswordRequestEvent {
    pub email: String,
    pub question_no: i32,
    pub answer: String,
}

/// 重置密码响应
#[derive(Message)]
pub struct ResetPasswordResponseEvent {
    pub result: Result<String, String>,
}

// ==================== 游戏消息 ====================

/// 加载游戏列表请求
#[derive(Message)]
pub struct LoadGamesRequest {
    pub page: i32,
}

/// 游戏列表加载完成
#[derive(Message)]
pub struct GamesLoadedEvent {
    pub games: Vec<Game>,
    pub total_pages: i32,
}

/// 游戏列表加载失败
#[derive(Message)]
pub struct GamesLoadFailedEvent {
    pub error: String,
}

/// 加载游戏详情请求
#[derive(Message)]
pub struct LoadGameDetailRequest {
    pub game_id: String,
}

/// 游戏详情加载完成
#[derive(Message)]
pub struct GameDetailLoadedEvent {
    pub game: Game,
}

/// 游戏详情加载失败
#[derive(Message)]
pub struct GameDetailLoadFailedEvent {
    pub error: String,
}

// ==================== 本地阅读消息 ====================

/// 扫描本地已下载漫画请求
#[derive(Message)]
pub struct ScanLocalComicsRequest;

/// 扫描本地漫画完成
#[derive(Message)]
pub struct ScanLocalComicsCompletedEvent {
    pub entries: Vec<crate::resources::LocalComicEntry>,
}

/// 扫描本地漫画失败
#[derive(Message)]
pub struct ScanLocalComicsFailedEvent {
    pub error: String,
}

// ==================== 网络诊断消息 ====================

/// 网速测试请求（下载固定图片测速）
#[derive(Message)]
pub struct SpeedTestRequest;

/// 网速测试结果
#[derive(Message)]
pub struct SpeedTestResultEvent {
    /// 下载速度（KB/s）
    pub download_speed: f64,
    /// 耗时（毫秒）
    pub elapsed_ms: u64,
}

/// Ping 测试请求（请求 /categories API 测延迟）
#[derive(Message)]
pub struct PingTestRequest;

/// Ping 测试结果
#[derive(Message)]
pub struct PingTestResultEvent {
    /// 延迟（毫秒）
    pub latency_ms: u64,
}

/// 网络测试失败事件
#[derive(Message)]
pub struct NetworkTestFailedEvent {
    /// 错误信息
    pub error: String,
}

// ==================== 锅贴社区消息 ====================

/// 加载小程序列表请求
#[derive(Message)]
pub struct LoadAppsRequest;

/// 小程序列表加载完成
#[derive(Message)]
pub struct AppsLoadedEvent {
    pub apps: Vec<picacg_api::endpoints::fried::AppInfo>,
}

/// 小程序列表加载失败
#[derive(Message)]
pub struct AppsLoadFailedEvent {
    pub error: String,
}

/// 加载锅贴帖子列表请求
#[derive(Message)]
pub struct LoadFriedPostsRequest {
    pub page: i32,
}

/// 锅贴帖子列表加载完成
#[derive(Message)]
pub struct FriedPostsLoadedEvent {
    pub posts: Vec<picacg_api::endpoints::fried::FriedPost>,
    pub total: i32,
    pub limit: i32,
}

/// 锅贴帖子列表加载失败
#[derive(Message)]
pub struct FriedPostsLoadFailedEvent {
    pub error: String,
}

// ==================== NAS 远程存储消息 ====================

/// NAS 测试连接请求
#[derive(Message)]
pub struct NasTestConnectionRequest;

/// NAS 测试连接响应
#[derive(Message)]
pub struct NasTestConnectionResponse {
    pub success: bool,
    pub message: String,
}

/// NAS 上传下载目录请求
#[derive(Message)]
pub struct NasUploadRequest;

/// NAS 上传进度更新
#[derive(Message)]
pub struct NasUploadProgressEvent {
    /// 漫画标题
    pub comic_title: String,
    /// 已上传文件数
    pub uploaded_files: u32,
    /// 总文件数
    pub total_files: u32,
}

/// NAS 上传完成
#[derive(Message)]
pub struct NasUploadCompletedEvent {
    pub message: String,
}

/// NAS 上传失败
#[derive(Message)]
pub struct NasUploadFailedEvent {
    pub error: String,
}

/// NAS 浏览远程目录请求
#[derive(Message)]
pub struct NasBrowseRequest {
    pub path: String,
}

/// NAS 浏览远程目录响应
#[derive(Message)]
pub struct NasBrowseResponse {
    pub entries: Vec<crate::resources::NasRemoteEntry>,
    pub path: String,
}

/// NAS 浏览失败
#[derive(Message)]
pub struct NasBrowseFailedEvent {
    pub error: String,
}

// ==================== 聊天室消息 ====================

/// 加载聊天房间列表请求
#[derive(Message)]
pub struct LoadChatRoomsRequest;

/// 聊天房间列表加载完成
#[derive(Message)]
pub struct ChatRoomsLoadedEvent {
    pub rooms: Vec<picacg_api::endpoints::chat::ChatRoom>,
    pub token: String,
    pub profile: Option<picacg_api::endpoints::chat::ChatProfile>,
}

/// 聊天房间列表加载失败
#[derive(Message)]
pub struct ChatRoomsLoadFailedEvent {
    pub error: String,
}

/// 连接聊天室 WebSocket 请求
#[derive(Message)]
pub struct ConnectChatRoomRequest {
    pub room_id: String,
    pub token: String,
}

/// 发送聊天消息请求（通过 REST API）
#[derive(Message)]
pub struct SendChatMessageRequest {
    pub room_id: String,
    pub message: String,
}

/// 发送聊天消息响应
#[derive(Message)]
pub struct SendChatMessageResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// 断开聊天室 WebSocket
#[derive(Message)]
pub struct DisconnectChatRoomRequest;
