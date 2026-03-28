//! API 数据模型

#![allow(dead_code)]

use serde::{Deserialize, Deserializer, Serialize};

/// 兼容 API 返回数字或字符串格式的 i64
fn deserialize_i64_or_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum I64OrString {
        I64(i64),
        Str(String),
    }

    match I64OrString::deserialize(deserializer)? {
        I64OrString::I64(v) => Ok(v),
        I64OrString::Str(s) => s.parse().map_err(serde::de::Error::custom),
    }
}

/// 同上，带 default
fn deserialize_i64_or_string_default<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_i64_or_string(deserializer).or(Ok(0))
}

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
    /// 用户 ID，部分 API（如 /users/profile）可能不返回此字段
    #[serde(rename = "_id", default)]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub name: String,
    pub level: i32,
    #[serde(deserialize_with = "deserialize_i64_or_string_default")]
    pub exp: i64,
    pub gender: String,
    #[serde(default)]
    pub avatar: Option<ImageInfo>,
    #[serde(default)]
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(rename = "isPunched", skip_serializing_if = "Option::is_none")]
    pub is_punched: Option<bool>,
    #[serde(rename = "created_at", default)]
    pub created_at: String,
    /// 自我简介（/users/profile 返回）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slogan: Option<String>,
    /// 角色标签（/users/profile 返回，如 knight）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub characters: Option<Vec<String>>,
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
    #[serde(
        rename = "likesCount",
        default,
        deserialize_with = "deserialize_i64_or_string_default"
    )]
    pub likes_count: i64,
    #[serde(
        rename = "viewsCount",
        default,
        deserialize_with = "deserialize_i64_or_string_default"
    )]
    pub views_count: i64,
    #[serde(
        rename = "commentsCount",
        default,
        deserialize_with = "deserialize_i64_or_string_default"
    )]
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
    #[serde(
        rename = "likesCount",
        deserialize_with = "deserialize_i64_or_string_default"
    )]
    pub likes_count: i64,
    #[serde(
        rename = "commentsCount",
        deserialize_with = "deserialize_i64_or_string_default"
    )]
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
