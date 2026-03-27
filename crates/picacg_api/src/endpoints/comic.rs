//! 漫画相关 API 端点

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    client::ApiRequest,
    models::{Comic, Episode, Picture},
};

// 获取漫画列表（按分类）
#[derive(Debug, Serialize)]
pub struct GetComicsRequest {
    pub category: String,
    pub page: i32,
    #[serde(skip_serializing)]
    pub sort: String, // dd, da, ld, vd (更新时间降序、升序、点赞数、浏览数)
}

#[derive(Debug, Deserialize)]
pub struct ComicsData {
    pub docs: Vec<Comic>,
    pub total: i32,
    pub limit: i32,
    pub page: i32,
    pub pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetComicsResponse {
    pub comics: ComicsData,
}

impl ApiRequest for GetComicsRequest {
    type Response = GetComicsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/comics".to_string()
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![
            ("c".to_string(), self.category.clone()),
            ("page".to_string(), self.page.to_string()),
            ("s".to_string(), self.sort.clone()),
        ])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取漫画详情
#[derive(Debug, Serialize)]
pub struct GetComicDetailRequest {
    pub comic_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GetComicDetailResponse {
    pub comic: Comic,
}

impl ApiRequest for GetComicDetailRequest {
    type Response = GetComicDetailResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!("/comics/{}", self.comic_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取章节列表
#[derive(Debug, Serialize)]
pub struct GetEpisodesRequest {
    pub comic_id: String,
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct EpisodesData {
    pub docs: Vec<Episode>,
    pub total: i32,
    pub limit: i32,
    pub page: i32,
    pub pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetEpisodesResponse {
    pub eps: EpisodesData,
}

impl ApiRequest for GetEpisodesRequest {
    type Response = GetEpisodesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!("/comics/{}/eps", self.comic_id)
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("page".to_string(), self.page.to_string())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 获取图片列表
#[derive(Debug, Serialize)]
pub struct GetPicturesRequest {
    pub comic_id: String,
    pub episode_order: i32,
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct PicturesData {
    pub docs: Vec<Picture>,
    pub total: i32,
    pub limit: i32,
    pub page: i32,
    pub pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetPicturesResponse {
    pub pages: PicturesData,
}

impl ApiRequest for GetPicturesRequest {
    type Response = GetPicturesResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!(
            "/comics/{}/order/{}/pages",
            self.comic_id, self.episode_order
        )
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("page".to_string(), self.page.to_string())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 点赞漫画
#[derive(Debug, Serialize)]
pub struct LikeComicRequest {
    pub comic_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LikeComicResponse {
    pub action: String, // "like" or "unlike"
}

impl ApiRequest for LikeComicRequest {
    type Response = LikeComicResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/comics/{}/like", self.comic_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 收藏漫画
#[derive(Debug, Serialize)]
pub struct FavoriteComicRequest {
    pub comic_id: String,
}

#[derive(Debug, Deserialize)]
pub struct FavoriteComicResponse {
    pub action: String, // "favorite" or "un_favorite"
}

impl ApiRequest for FavoriteComicRequest {
    type Response = FavoriteComicResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/comics/{}/favourite", self.comic_id)
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 搜索漫画
#[derive(Debug, Serialize)]
pub struct SearchComicsRequest {
    pub keyword: String,
    pub page: i32,
    #[serde(skip_serializing)]
    pub sort: String,
    /// 分类过滤（空列表表示不过滤）
    #[serde(skip_serializing)]
    pub categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchComicsResponse {
    pub comics: ComicsData,
}

impl ApiRequest for SearchComicsRequest {
    type Response = SearchComicsResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        "/comics/advanced-search".to_string()
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![
            ("page".to_string(), self.page.to_string()),
            ("s".to_string(), self.sort.clone()),
        ])
    }

    fn body(&self) -> Option<serde_json::Value> {
        let mut body = serde_json::json!({
            "keyword": self.keyword,
        });
        if !self.categories.is_empty() {
            body["categories"] = serde_json::json!(self.categories);
        }
        Some(body)
    }
}

// 获取热门搜索关键词
#[derive(Debug, Serialize)]
pub struct GetKeywordsRequest;

#[derive(Debug, Deserialize)]
pub struct GetKeywordsResponse {
    pub keywords: Vec<String>,
}

impl ApiRequest for GetKeywordsRequest {
    type Response = GetKeywordsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/keywords".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}
