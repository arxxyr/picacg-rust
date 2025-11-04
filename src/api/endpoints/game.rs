use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::{
    client::ApiRequest,
    models::{Comment, Game, PageInfo},
};

// 获取游戏列表
#[derive(Debug, Serialize)]
pub struct GetGamesRequest {
    #[serde(skip_serializing)]
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct GamesData {
    pub games: Vec<Game>,
    pub page: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct GetGamesResponse {
    pub games: GamesData,
}

impl ApiRequest for GetGamesRequest {
    type Response = GetGamesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/games".to_string()
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("page".to_string(), self.page.to_string())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取游戏详情
#[derive(Debug, Serialize)]
pub struct GetGameDetailRequest {
    #[serde(skip_serializing)]
    pub game_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GetGameDetailResponse {
    pub game: Game,
}

impl ApiRequest for GetGameDetailRequest {
    type Response = GetGameDetailResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!("/games/{}", self.game_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取游戏评论
#[derive(Debug, Serialize)]
pub struct GetGameCommentsRequest {
    #[serde(skip_serializing)]
    pub game_id: String,
    #[serde(skip_serializing)]
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct GameCommentsData {
    pub docs: Vec<Comment>,
    pub page: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct GetGameCommentsResponse {
    pub comments: GameCommentsData,
}

impl ApiRequest for GetGameCommentsRequest {
    type Response = GetGameCommentsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!("/games/{}/comments", self.game_id)
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("page".to_string(), self.page.to_string())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 发送游戏评论
#[derive(Debug, Serialize)]
pub struct PostGameCommentRequest {
    #[serde(skip_serializing)]
    pub game_id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct PostGameCommentResponse {
    #[serde(rename = "_id")]
    pub id: String,
}

impl ApiRequest for PostGameCommentRequest {
    type Response = PostGameCommentResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/games/{}/comments", self.game_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "content": self.content,
        }))
    }
}
