//! 分类相关 API 端点

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{client::ApiRequest, models::Category};

// 获取分类列表
#[derive(Debug, Serialize)]
pub struct GetCategoriesRequest;

#[derive(Debug, Deserialize)]
pub struct GetCategoriesResponse {
    pub categories: Vec<Category>,
}

impl ApiRequest for GetCategoriesRequest {
    type Response = GetCategoriesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/categories".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取收藏夹
#[derive(Debug, Serialize)]
pub struct GetFavoritesRequest {
    pub page: i32,
    #[serde(skip_serializing)]
    pub sort: String,
}

impl ApiRequest for GetFavoritesRequest {
    type Response = super::comic::GetComicsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/users/favourite".to_string()
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![
            ("page".to_string(), self.page.to_string()),
            ("s".to_string(), self.sort.clone()),
        ])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取推荐漫画
#[derive(Debug, Serialize)]
pub struct GetRecommendationsRequest;

#[derive(Debug, Deserialize)]
pub struct GetRecommendationsResponse {
    pub comics: Vec<crate::models::Comic>,
}

impl ApiRequest for GetRecommendationsRequest {
    type Response = GetRecommendationsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/comics/random".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取排行榜（旧接口，保留兼容）
#[derive(Debug, Serialize)]
pub struct GetRankingRequest {
    #[serde(skip_serializing)]
    pub time_type: String, // H24, D7, D30
}

#[derive(Debug, Deserialize)]
pub struct GetRankingResponse {
    pub comics: Vec<crate::models::Comic>,
}

impl ApiRequest for GetRankingRequest {
    type Response = GetRankingResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/comics/leaderboard".to_string()
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("tt".to_string(), self.time_type.clone())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}
