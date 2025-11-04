use crate::api::client::ApiRequest;
use crate::api::models::{Comment, PageInfo};
use reqwest::Method;
use serde::{Deserialize, Serialize};

// 获取漫画评论列表
#[derive(Debug, Serialize)]
pub struct GetCommentsRequest {
    #[serde(skip_serializing)]
    pub comic_id: String,
    #[serde(skip_serializing)]
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct CommentsData {
    pub docs: Vec<Comment>,
    pub page: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct GetCommentsResponse {
    pub comments: CommentsData,
}

impl ApiRequest for GetCommentsRequest {
    type Response = GetCommentsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!("/comics/{}/comments", self.comic_id)
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("page".to_string(), self.page.to_string())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 发表评论
#[derive(Debug, Serialize)]
pub struct PostCommentRequest {
    #[serde(skip_serializing)]
    pub comic_id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct PostCommentResponse {
    #[serde(rename = "_id")]
    pub id: String,
}

impl ApiRequest for PostCommentRequest {
    type Response = PostCommentResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/comics/{}/comments", self.comic_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "content": self.content,
        }))
    }
}

// 回复评论（发送子评论）
#[derive(Debug, Serialize)]
pub struct PostCommentReplyRequest {
    #[serde(skip_serializing)]
    pub comment_id: String,
    pub content: String,
}

impl ApiRequest for PostCommentReplyRequest {
    type Response = PostCommentResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/comments/{}", self.comment_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "content": self.content,
        }))
    }
}

// 获取子评论列表
#[derive(Debug, Serialize)]
pub struct GetCommentChildrenRequest {
    #[serde(skip_serializing)]
    pub comment_id: String,
    #[serde(skip_serializing)]
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetCommentChildrenResponse {
    pub comments: CommentsData,
}

impl ApiRequest for GetCommentChildrenRequest {
    type Response = GetCommentChildrenResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!("/comments/{}/childrens", self.comment_id)
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("page".to_string(), self.page.to_string())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 点赞评论
#[derive(Debug, Serialize)]
pub struct LikeCommentRequest {
    #[serde(skip_serializing)]
    pub comment_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LikeCommentResponse {
    pub action: String, // "like" or "unlike"
}

impl ApiRequest for LikeCommentRequest {
    type Response = LikeCommentResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/comments/{}/like", self.comment_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 举报评论
#[derive(Debug, Serialize)]
pub struct ReportCommentRequest {
    #[serde(skip_serializing)]
    pub comment_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ReportCommentResponse {
    pub message: String,
}

impl ApiRequest for ReportCommentRequest {
    type Response = ReportCommentResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/comments/{}/report", self.comment_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}
