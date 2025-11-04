use std::{sync::Arc, time::Duration};

use parking_lot::RwLock;
use reqwest::{Client, Method, Proxy};
use serde::{Deserialize, Serialize};

use crate::{
    api::signer::Signer,
    config::settings::AppSettings,
    error::{PicacgError, Result},
};

// API 基础配置
pub const API_BASE_URL: &str = "https://picaapi.picacomic.com";

// API 响应包装
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub error: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> Result<T> {
        if self.code != 200 {
            return Err(PicacgError::ApiError {
                code: self.code,
                message: self.message,
                error: self.error,
            });
        }

        self.data
            .ok_or_else(|| PicacgError::NotFound("响应中无数据".to_string()))
    }
}

// API 客户端
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    token: Arc<RwLock<Option<String>>>,
    signer: Signer,
}

impl ApiClient {
    /// 创建新的 API 客户端（使用全局配置中的代理设置）
    pub fn new() -> Result<Self> {
        let settings = AppSettings::global().read();
        Self::with_proxy(settings.proxy.to_proxy_url())
    }

    /// 创建带代理的 API 客户端
    pub fn with_proxy(proxy_url: Option<String>) -> Result<Self> {
        let mut builder = Client::builder()
            // 移除 http2_prior_knowledge()，因为 SOCKS5 代理不支持
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));

        // 如果有代理配置，添加代理
        if let Some(url) = proxy_url {
            tracing::info!("使用代理: {}", url);
            let proxy = Proxy::all(&url).map_err(|e| {
                tracing::error!("创建代理失败: {}", e);
                PicacgError::ConfigError(format!("无效的代理配置: {}", e))
            })?;
            builder = builder.proxy(proxy);
        } else {
            tracing::info!("不使用代理");
        }

        let client = builder.build().map_err(|e| {
            tracing::error!("创建 HTTP 客户端失败: {}", e);
            PicacgError::ConfigError(format!("创建 HTTP 客户端失败: {}", e))
        })?;

        Ok(ApiClient {
            client,
            token: Arc::new(RwLock::new(None)),
            signer: Signer::new(),
        })
    }

    /// 重新加载配置并更新客户端
    pub fn reload_config(&mut self) -> Result<()> {
        let settings = AppSettings::global().read();
        let proxy_url = settings.proxy.to_proxy_url();

        let mut builder = Client::builder()
            // 移除 http2_prior_knowledge()，因为 SOCKS5 代理不支持
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));

        if let Some(url) = proxy_url {
            tracing::info!("重新加载代理配置: {}", url);
            let proxy = Proxy::all(&url).map_err(|e| {
                tracing::error!("创建代理失败: {}", e);
                PicacgError::ConfigError(format!("无效的代理配置: {}", e))
            })?;
            builder = builder.proxy(proxy);
        } else {
            tracing::info!("重新加载配置：不使用代理");
        }

        self.client = builder.build().map_err(|e| {
            tracing::error!("重新创建 HTTP 客户端失败: {}", e);
            PicacgError::ConfigError(format!("重新创建 HTTP 客户端失败: {}", e))
        })?;
        Ok(())
    }

    pub fn set_token(&self, token: impl Into<String>) {
        *self.token.write() = Some(token.into());
    }

    pub fn get_token(&self) -> Option<String> {
        self.token.read().clone()
    }

    pub fn clear_token(&self) {
        *self.token.write() = None;
    }

    pub fn is_logged_in(&self) -> bool {
        self.token.read().is_some()
    }

    // 通用请求方法
    pub async fn request<R>(&self, req: R) -> Result<R::Response>
    where
        R: ApiRequest,
    {
        let method = req.method();

        // 构建完整 URL(包含查询参数)用于签名
        // Python 版本在签名前会构建完整 URL,包括查询参数
        let mut url_with_query = format!("{}{}", API_BASE_URL, req.path());
        if let Some(query) = req.query() {
            // 手动构建查询字符串,确保格式与 Python 版本一致
            let query_string = query
                .iter()
                .map(|(k, v)| {
                    // URL 编码参数值(与 Python 的 quote() 对应)
                    let encoded_value = urlencoding::encode(v);
                    format!("{}={}", k, encoded_value)
                })
                .collect::<Vec<_>>()
                .join("&");
            url_with_query = format!("{}?{}", url_with_query, query_string);
            tracing::debug!("完整 URL(用于签名): {}", url_with_query);
        }

        tracing::debug!("发送请求: {} {}", method, url_with_query);

        // 使用包含查询参数的完整 URL 进行签名
        let mut headers = self.signer.sign(&url_with_query, &method);

        // 添加 Token
        if req.need_auth() {
            let token = self.token.read();
            if let Some(ref token_str) = *token {
                headers.insert(
                    "authorization",
                    token_str
                        .parse()
                        .map_err(|_| PicacgError::AuthError("无效的 token 格式".to_string()))?,
                );
                tracing::debug!("使用 token 认证");
            } else {
                tracing::error!("需要认证但未登录");
                return Err(PicacgError::NotLoggedIn);
            }
        }

        // 构建请求(使用完整 URL)
        let mut builder = self
            .client
            .request(method.clone(), &url_with_query)
            .headers(headers);

        // 注意:查询参数已经包含在 URL 中了,不需要再通过 builder.query() 添加

        // 添加 Body
        if let Some(body) = req.body() {
            tracing::debug!("添加请求体");
            builder = builder.json(&body);
        }

        // 发送请求
        tracing::debug!("正在发送 HTTP 请求...");
        let response = builder.send().await.map_err(|e| {
            tracing::error!("发送请求失败: {}", e);
            PicacgError::NetworkError(e.to_string())
        })?;
        let status = response.status();
        tracing::debug!("收到响应: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PicacgError::HttpError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        // 先获取响应体文本用于调试
        let response_text = response.text().await.map_err(|e| {
            tracing::error!("读取响应体失败: {}", e);
            PicacgError::NetworkError(format!("读取响应体失败: {}", e))
        })?;

        tracing::debug!(
            "响应体(前 500 字符): {}",
            &response_text[..response_text.len().min(500)]
        );

        // 解析响应
        let api_response: ApiResponse<R::Response> =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!("解析响应失败: {}", e);
                tracing::error!("完整响应体: {}", response_text);
                PicacgError::SerializationError(format!("解析响应失败: {}", e))
            })?;

        api_response.into_result()
    }

    // 流式下载
    pub async fn download(&self, url: &str) -> Result<reqwest::Response> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(PicacgError::HttpError {
                status: response.status().as_u16(),
                message: "下载失败".to_string(),
            });
        }

        Ok(response)
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new().expect("Failed to create API client")
    }
}

// API 请求 Trait
pub trait ApiRequest: Serialize + Send + Sync {
    type Response: for<'de> Deserialize<'de> + Send;

    fn method(&self) -> Method;
    fn path(&self) -> String;
    fn need_auth(&self) -> bool {
        true
    }
    fn body(&self) -> Option<serde_json::Value> {
        serde_json::to_value(self).ok()
    }
    fn query(&self) -> Option<Vec<(String, String)>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ApiClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_token_management() {
        let client = ApiClient::new().unwrap();
        assert!(!client.is_logged_in());

        client.set_token("test-token");
        assert!(client.is_logged_in());
        assert_eq!(client.get_token(), Some("test-token".to_string()));

        client.clear_token();
        assert!(!client.is_logged_in());
    }
}
