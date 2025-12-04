//! 排行榜 API 端点

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::{client::ApiRequest, models::Comic};

/// 排行榜时间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum RankTimeType {
    /// 日榜（24小时）
    #[default]
    H24,
    /// 周榜（7天）
    D7,
    /// 月榜（30天）
    D30,
}

impl RankTimeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RankTimeType::H24 => "H24",
            RankTimeType::D7 => "D7",
            RankTimeType::D30 => "D30",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RankTimeType::H24 => "日榜",
            RankTimeType::D7 => "周榜",
            RankTimeType::D30 => "月榜",
        }
    }
}

/// 获取漫画排行榜
#[derive(Debug, Serialize)]
pub struct GetRankingsRequest {
    pub time_type: RankTimeType,
}

#[derive(Debug, Deserialize)]
pub struct GetRankingsResponse {
    pub comics: Vec<Comic>,
}

impl ApiRequest for GetRankingsRequest {
    type Response = GetRankingsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/comics/leaderboard".to_string()
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![
            ("tt".to_string(), self.time_type.as_str().to_string()),
            ("ct".to_string(), "VC".to_string()),
        ])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

/// 骑士榜用户信息
#[derive(Debug, Clone, Deserialize)]
pub struct KnightUser {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub title: String,
    pub level: i32,
    #[serde(rename = "comicsUploaded")]
    pub comics_uploaded: i32,
    pub character: Option<String>,
}

/// 获取骑士榜
#[derive(Debug, Serialize)]
pub struct GetKnightRankingsRequest;

#[derive(Debug, Deserialize)]
pub struct GetKnightRankingsResponse {
    pub users: Vec<KnightUser>,
}

impl ApiRequest for GetKnightRankingsRequest {
    type Response = GetKnightRankingsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/comics/knight-leaderboard".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}
