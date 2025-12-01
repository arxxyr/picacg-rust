//! API 相关消息
//!
//! 定义与 API 请求/响应相关的消息 (Bevy 0.17 使用 Message 替代 Event)

use bevy::prelude::*;

use crate::api::models::{Category, Comic, Episode, Picture};

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

/// 加载章节列表请求
#[derive(Message)]
pub struct LoadEpisodesRequest {
    pub comic_id: String,
    pub page: i32,
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
