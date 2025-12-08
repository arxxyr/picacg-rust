//! API 数据模型

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// 图片信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    #[serde(rename = "originalName")]
    pub original_name: String,
    pub path: String,
    #[serde(rename = "fileServer")]
    pub file_server: String,
}

impl ImageInfo {
    pub fn url(&self) -> String {
        format!("{}/static/{}", self.file_server, self.path)
    }
}

// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub name: String,
    pub level: i32,
    pub exp: i64,
    pub gender: String,
    pub avatar: ImageInfo,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(rename = "isPunched", skip_serializing_if = "Option::is_none")]
    pub is_punched: Option<bool>,
    #[serde(rename = "created_at")]
    pub created_at: String,
}

// 漫画信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(rename = "pagesCount", default)]
    pub pages_count: i32,
    #[serde(rename = "epsCount", default)]
    pub eps_count: i32,
    #[serde(default)]
    pub finished: bool,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub thumb: ImageInfo,
    #[serde(rename = "likesCount", default)]
    pub likes_count: i64,
    #[serde(rename = "viewsCount", default)]
    pub views_count: i64,
    #[serde(rename = "commentsCount", default)]
    pub comments_count: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "chineseTeam", skip_serializing_if = "Option::is_none")]
    pub chinese_team: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "updated_at", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(rename = "allowDownload", default)]
    pub allow_download: bool,
    #[serde(
        rename = "allowComment",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_comment: Option<bool>,
    #[serde(rename = "isFavourite", skip_serializing_if = "Option::is_none")]
    pub is_favourite: Option<bool>,
    #[serde(rename = "isLiked", skip_serializing_if = "Option::is_none")]
    pub is_liked: Option<bool>,
}

// 章节信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub order: i32,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

// 图片信息(章节内)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
    #[serde(rename = "_id")]
    pub id: String,
    pub media: ImageInfo,
}

// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub total: i32,
    pub limit: i32,
    pub page: i32,
    pub pages: i32,
}

// 评论信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    #[serde(rename = "_id")]
    pub id: String,
    pub content: String,
    #[serde(rename = "_user")]
    pub user: User,
    #[serde(rename = "_comic", skip_serializing_if = "Option::is_none")]
    pub comic: Option<String>,
    #[serde(
        rename = "totalComments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_comments: Option<i32>,
    #[serde(rename = "isTop", default, skip_serializing_if = "Option::is_none")]
    pub is_top: Option<bool>,
    pub hide: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "likesCount")]
    pub likes_count: i64,
    #[serde(rename = "commentsCount")]
    pub comments_count: i64,
    #[serde(rename = "isLiked", skip_serializing_if = "Option::is_none")]
    pub is_liked: Option<bool>,
}

// 分类信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
    pub thumb: ImageInfo,
    #[serde(rename = "isWeb", skip_serializing_if = "Option::is_none")]
    pub is_web: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// 游戏信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: ImageInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(rename = "ios", skip_serializing_if = "Option::is_none")]
    pub ios_link: Option<String>,
    #[serde(rename = "android", skip_serializing_if = "Option::is_none")]
    pub android_link: Option<String>,
    #[serde(rename = "iosLinks", skip_serializing_if = "Option::is_none")]
    pub ios_links: Option<Vec<String>>,
    #[serde(rename = "androidLinks", skip_serializing_if = "Option::is_none")]
    pub android_links: Option<Vec<String>>,
    #[serde(rename = "updateContent", skip_serializing_if = "Option::is_none")]
    pub update_content: Option<String>,
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggest: Option<bool>,
    #[serde(rename = "screenshots", skip_serializing_if = "Option::is_none")]
    pub screenshots: Option<Vec<ImageInfo>>,
    #[serde(rename = "likesCount", skip_serializing_if = "Option::is_none")]
    pub likes_count: Option<i64>,
    #[serde(rename = "commentsCount", skip_serializing_if = "Option::is_none")]
    pub comments_count: Option<i64>,
}
