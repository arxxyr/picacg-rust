//! 聊天室 API 端点
//!
//! 聊天服务使用独立域名 `live-server.bidobido.xyz`，不走 PicACG 主 API
//! 签名体系。 流程：
//! 1. `POST /auth/signin` 使用 PicACG 账号密码登录，获取聊天 token
//! 2. `GET /user/profile` 获取聊天用户资料
//! 3. `GET /room/list` 获取房间列表
//! 4. WebSocket 连接
//!    `wss://live-server.bidobido.xyz?token={token}&room={roomId}`
//! 5. `POST /message/send-message` 发送文本消息

use serde::{Deserialize, Serialize};

/// 聊天服务基础 URL
pub const CHAT_API_BASE: &str = "https://live-server.bidobido.xyz";

/// 聊天服务 WebSocket 基础 URL
pub const CHAT_WS_BASE: &str = "wss://live-server.bidobido.xyz";

/// 聊天 API 请求头（模拟 Dart 客户端）
///
/// 注意：`accept-encoding` 由 reqwest 的 `gzip(true)` / `brotli(true)`
/// 自动处理， 不在此处手动设置，否则 reqwest 不会自动解压响应体。
pub fn chat_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("user-agent", "Dart/2.19 (dart:io)"),
        ("api-version", "1.0.3"),
        ("content-type", "application/json; charset=UTF-8"),
    ]
}

// ==================== 数据模型 ====================

/// 聊天房间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    /// 房间 ID
    #[serde(default)]
    pub id: String,
    /// 房间标题
    #[serde(default)]
    pub title: String,
    /// 房间描述
    #[serde(default)]
    pub description: String,
    /// 最低等级要求
    #[serde(rename = "minLevel", default)]
    pub min_level: i32,
    /// 最低注册天数要求
    #[serde(rename = "minRegisterDays", default)]
    pub min_register_days: i32,
    /// 是否公开
    #[serde(rename = "isPublic", default)]
    pub is_public: bool,
    /// 是否可用
    #[serde(rename = "isAvailable", default)]
    pub is_available: bool,
    /// 房间图标 URL
    #[serde(default)]
    pub icon: String,
    /// 允许的角色类型
    #[serde(rename = "allowedCharacters", default)]
    pub allowed_characters: Vec<String>,
}

/// 聊天用户资料
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatProfile {
    /// 用户 ID
    #[serde(default)]
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
    #[serde(rename = "avatarUrl", default)]
    pub avatar_url: String,
    /// 角色标签（vip、girl、manager、official 等）
    #[serde(default)]
    pub characters: Vec<String>,
}

/// 聊天消息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 消息 ID
    #[serde(default)]
    pub id: String,
    /// 引用 ID
    #[serde(rename = "referenceId", default)]
    pub reference_id: String,
    /// 消息类型：TEXT_MESSAGE / IMAGE_MESSAGE / CONNECTED / INITIAL_MESSAGES 等
    #[serde(rename = "type", default)]
    pub msg_type: String,
    /// 创建时间
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
    /// 是否被屏蔽
    #[serde(rename = "isBlocked", default)]
    pub is_blocked: bool,
    /// 消息数据（不同类型有不同结构）
    #[serde(default)]
    pub data: serde_json::Value,
}

/// 聊天消息中的发送者资料（嵌套在 data.profile 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageProfile {
    /// 用户 ID
    #[serde(default)]
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
    #[serde(rename = "avatarUrl", default)]
    pub avatar_url: String,
    /// 角色标签
    #[serde(default)]
    pub characters: Vec<String>,
}

/// 解析后的聊天消息（UI 显示用）
#[derive(Debug, Clone)]
pub struct ParsedChatMessage {
    /// 消息 ID
    pub id: String,
    /// 消息类型
    pub msg_type: ChatMessageType,
    /// 发送者名称
    pub sender_name: String,
    /// 发送者等级
    pub sender_level: i32,
    /// 发送者称号
    pub sender_title: String,
    /// 发送者角色标签
    pub sender_characters: Vec<String>,
    /// 发送者头像 URL
    pub sender_avatar_url: String,
    /// 消息文本内容
    pub message: String,
    /// 发送平台
    pub platform: String,
    /// 创建时间
    pub created_at: String,
    /// 图片 URL 列表（IMAGE_MESSAGE 时有值）
    pub media_urls: Vec<String>,
    /// 回复信息
    pub reply: Option<ChatReplyInfo>,
    /// @提及的用户
    pub mentions: Vec<String>,
}

/// 消息类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatMessageType {
    /// 文本消息
    Text,
    /// 图片消息
    Image,
    /// 已连接通知
    Connected,
    /// 初始消息列表
    InitialMessages,
    /// 删除消息
    DeleteMessage,
    /// 在线人数更新
    UpdateOnlineCount,
    /// 其他未知类型
    Unknown(String),
}

/// 回复信息
#[derive(Debug, Clone)]
pub struct ChatReplyInfo {
    /// 被回复消息的 ID
    pub id: String,
    /// 被回复者名称
    pub name: String,
    /// 被回复的消息内容
    pub message: String,
    /// 回复类型
    pub reply_type: String,
}

impl ChatMessage {
    /// 解析为 UI 可用的消息结构
    pub fn parse(&self) -> ParsedChatMessage {
        let data = &self.data;

        // 提取发送者资料
        let profile = data.get("profile").cloned().unwrap_or_default();
        let sender_name = profile
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let sender_level = profile.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let sender_title = profile
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let sender_avatar_url = profile
            .get("avatarUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let sender_characters = profile
            .get("characters")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // 消息文本
        let message = data
            .get("message")
            .and_then(|v| v.as_str())
            .or_else(|| data.get("caption").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();

        // 平台
        let platform = data
            .get("platform")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 消息类型
        let msg_type = match self.msg_type.as_str() {
            "TEXT_MESSAGE" => ChatMessageType::Text,
            "IMAGE_MESSAGE" => ChatMessageType::Image,
            "CONNECTED" => ChatMessageType::Connected,
            "INITIAL_MESSAGES" => ChatMessageType::InitialMessages,
            "DELETE_MESSAGE_ACTION" => ChatMessageType::DeleteMessage,
            "UPDATE_ROOM_ONLINE_USERS_COUNT_ACTION" => ChatMessageType::UpdateOnlineCount,
            other => ChatMessageType::Unknown(other.to_string()),
        };

        // 图片 URL
        let media_urls = data
            .get("medias")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // 回复信息
        let reply = data.get("reply").and_then(|r| {
            let id = r.get("id")?.as_str()?.to_string();
            let reply_type = r
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let reply_data = r.get("data").cloned().unwrap_or_default();
            let name = reply_data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let message = reply_data
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(ChatReplyInfo {
                id,
                name,
                message,
                reply_type,
            })
        });

        // @提及
        let mentions = data
            .get("userMentions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        v.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        ParsedChatMessage {
            id: self.id.clone(),
            msg_type,
            sender_name,
            sender_level,
            sender_title,
            sender_characters,
            sender_avatar_url,
            message,
            platform,
            created_at: self.created_at.clone(),
            media_urls,
            reply,
            mentions,
        }
    }
}

// ==================== 聊天 REST API 客户端 ====================

/// 聊天 API 客户端（独立于 PicACG 主 API，不使用签名机制）
#[derive(Clone)]
pub struct ChatApiClient {
    client: reqwest::Client,
    /// 聊天服务 token（通过 /auth/signin 获取）
    token: Option<String>,
}

impl ChatApiClient {
    /// 创建新的聊天 API 客户端
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(15))
            .gzip(true)
            .brotli(true)
            .build()
            .expect("创建聊天 HTTP 客户端失败");

        Self {
            client,
            token: None,
        }
    }

    /// 设置 token
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// 获取 token
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// 构建带认证头的请求
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in chat_headers() {
            headers.insert(
                reqwest::header::HeaderName::from_static(key),
                reqwest::header::HeaderValue::from_static(value),
            );
        }
        if let Some(ref token) = self.token {
            let auth_value = format!("Bearer {}", token);
            if let Ok(v) = reqwest::header::HeaderValue::from_str(&auth_value) {
                headers.insert(reqwest::header::AUTHORIZATION, v);
            }
        }
        headers
    }

    /// 登录聊天服务（使用 PicACG 账号密码）
    pub async fn signin(&mut self, email: &str, password: &str) -> Result<String, String> {
        let url = format!("{}/auth/signin", CHAT_API_BASE);
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let headers = {
            let mut h = reqwest::header::HeaderMap::new();
            for (key, value) in chat_headers() {
                h.insert(
                    reqwest::header::HeaderName::from_static(key),
                    reqwest::header::HeaderValue::from_static(value),
                );
            }
            h
        };

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("聊天登录网络错误: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("聊天登录失败 ({}): {}", status, text));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("解析聊天登录响应失败: {}", e))?;

        let token = data
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "聊天登录响应中无 token".to_string())?
            .to_string();

        self.token = Some(token.clone());
        Ok(token)
    }

    /// 获取用户资料
    pub async fn get_profile(&self) -> Result<ChatProfile, String> {
        let url = format!("{}/user/profile", CHAT_API_BASE);
        let headers = self.build_headers();

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("获取聊天资料网络错误: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("获取聊天资料失败 ({}): {}", status, text));
        }

        // 先获取原始文本，方便调试
        let text = resp
            .text()
            .await
            .map_err(|e| format!("读取聊天资料响应体失败: {}", e))?;

        let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("解析聊天资料 JSON 失败: {}, 响应体: {}", e, text);
            format!("解析聊天资料 JSON 失败: {}", e)
        })?;

        // 尝试从 "profile" 字段提取，若不存在则尝试 "user" 或整个 data
        let profile_value = data
            .get("profile")
            .or_else(|| data.get("user"))
            .unwrap_or(&data);

        serde_json::from_value(profile_value.clone()).map_err(|e| {
            tracing::error!(
                "反序列化 ChatProfile 失败: {}, 原始数据: {}",
                e,
                profile_value
            );
            format!("解析聊天资料失败: {}", e)
        })
    }

    /// 获取房间列表
    pub async fn get_rooms(&self) -> Result<Vec<ChatRoom>, String> {
        let url = format!("{}/room/list", CHAT_API_BASE);
        let headers = self.build_headers();

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("获取房间列表网络错误: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("获取房间列表失败 ({}): {}", status, text));
        }

        // 先获取原始文本，方便调试
        let text = resp
            .text()
            .await
            .map_err(|e| format!("读取房间列表响应体失败: {}", e))?;

        let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("解析房间列表 JSON 失败: {}, 响应体: {}", e, text);
            format!("解析房间列表 JSON 失败: {}", e)
        })?;

        // 尝试从 "rooms" 字段提取，若不存在则尝试整个 data 作为数组
        let rooms_value = data.get("rooms").unwrap_or(&data);

        serde_json::from_value(rooms_value.clone()).map_err(|e| {
            tracing::error!("反序列化房间列表失败: {}, 原始数据: {}", e, rooms_value);
            format!("解析房间列表失败: {}", e)
        })
    }

    /// 发送文本消息
    pub async fn send_message(
        &self,
        room_id: &str,
        message: &str,
        reply_id: Option<&str>,
    ) -> Result<(), String> {
        let url = format!("{}/message/send-message", CHAT_API_BASE);
        let headers = self.build_headers();

        let reference_id = uuid::Uuid::new_v4().to_string();
        let mut body = serde_json::json!({
            "roomId": room_id,
            "message": message,
            "referenceId": reference_id,
            "userMentions": [],
        });

        if let Some(reply) = reply_id {
            body.as_object_mut().unwrap().insert(
                "replyId".to_string(),
                serde_json::Value::String(reply.to_string()),
            );
        }

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("发送消息网络错误: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("发送消息失败 ({}): {}", status, text));
        }

        Ok(())
    }
}

impl Default for ChatApiClient {
    fn default() -> Self {
        Self::new()
    }
}
