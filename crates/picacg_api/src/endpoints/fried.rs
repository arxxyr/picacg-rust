//! 锅贴社区 API 端点
//!
//! 锅贴使用独立域名 `post-api.wikawika.xyz`，需要通过 PicACG 主 API 获取 token
//! 后登录。 流程：
//! 1. `GET /pica-apps` 获取小程序列表（主 API）
//! 2. 从列表中找到锅贴入口，使用 PicACG token 换取锅贴 token
//! 3. 使用锅贴 token 访问帖子列表/评论等

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::ApiRequest;

/// 锅贴 API 基础域名
pub const FRIED_API_BASE: &str = "https://post-api.wikawika.xyz";

// ==================== 小程序列表（主 API） ====================

/// 获取小程序列表请求（包含锅贴入口）
#[derive(Debug, Serialize)]
pub struct GetAppsRequest;

/// 小程序信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// 标题
    #[serde(default)]
    pub title: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 图标 URL
    #[serde(default)]
    pub icon: String,
    /// 链接地址
    #[serde(default)]
    pub url: String,
    /// 是否显示
    #[serde(rename = "isShow", default)]
    pub is_show: bool,
}

/// 小程序列表响应
#[derive(Debug, Deserialize)]
pub struct GetAppsResponse {
    pub apps: Vec<AppInfo>,
}

impl ApiRequest for GetAppsRequest {
    type Response = GetAppsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/pica-apps".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// ==================== 锅贴帖子相关数据结构 ====================

/// 锅贴用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriedUser {
    /// 用户 ID
    #[serde(rename = "_id", default)]
    pub id: String,
    /// 用户名
    #[serde(default)]
    pub name: String,
    /// 等级
    #[serde(default)]
    pub level: i32,
    /// 称号
    #[serde(default)]
    pub title: String,
    /// 头像 URL
    #[serde(default)]
    pub avatar: String,
    /// 角色头像框 URL
    #[serde(default)]
    pub character: String,
}

/// 锅贴帖子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriedPost {
    /// 帖子 ID
    #[serde(rename = "_id")]
    pub id: String,
    /// 帖子内容
    #[serde(default)]
    pub content: String,
    /// 发帖用户
    #[serde(rename = "_user", default)]
    pub user: Option<FriedUser>,
    /// 媒体附件（图片 URL 列表）
    #[serde(default)]
    pub medias: Vec<String>,
    /// 点赞数
    #[serde(rename = "totalLikes", default)]
    pub total_likes: i32,
    /// 评论数
    #[serde(rename = "totalComments", default)]
    pub total_comments: i32,
    /// 是否已点赞
    #[serde(default)]
    pub liked: bool,
    /// 创建时间
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
}

/// 锅贴评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriedComment {
    /// 评论 ID
    #[serde(rename = "_id")]
    pub id: String,
    /// 评论内容
    #[serde(default)]
    pub content: String,
    /// 评论用户
    #[serde(rename = "_user", default)]
    pub user: Option<FriedUser>,
    /// 点赞数
    #[serde(rename = "totalLikes", default)]
    pub total_likes: i32,
    /// 是否已点赞
    #[serde(default)]
    pub liked: bool,
    /// 创建时间
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
}

/// 锅贴帖子列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct FriedPostsData {
    /// 帖子列表
    pub posts: Vec<FriedPost>,
    /// 总数
    #[serde(default)]
    pub total: i32,
    /// 每页条数
    #[serde(default)]
    pub limit: i32,
}

/// 锅贴帖子列表外层响应
#[derive(Debug, Deserialize)]
pub struct FriedPostsResponse {
    pub data: FriedPostsData,
}

/// 锅贴评论列表响应数据
#[derive(Debug, Clone, Deserialize)]
pub struct FriedCommentsData {
    /// 评论列表
    pub comments: Vec<FriedComment>,
    /// 总数
    #[serde(default)]
    pub total: i32,
    /// 每页条数
    #[serde(default)]
    pub limit: i32,
}

/// 锅贴评论列表外层响应
#[derive(Debug, Deserialize)]
pub struct FriedCommentsResponse {
    pub data: FriedCommentsData,
}
