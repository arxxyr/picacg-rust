//! API 客户端

use std::{sync::Arc, time::Duration};

use parking_lot::RwLock;
use picacg_config::ChannelType;
use picacg_core::{PicacgError, Result};
use reqwest::{Client, Method, Proxy};
use serde::{Deserialize, Serialize};

use crate::{
    channel::{ChannelRoute, apply_api_dns_override, resolve_api_route},
    signer::Signer,
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
    channel_route: ChannelRoute,
}

impl ApiClient {
    /// 创建新的 API 客户端（不使用代理，直连通道）
    pub fn new() -> Result<Self> {
        Self::with_config(None, ChannelType::Direct, "", false, false)
    }

    /// 创建带代理的 API 客户端（直连通道，向后兼容）
    pub fn with_proxy(proxy_url: Option<String>) -> Result<Self> {
        Self::with_config(proxy_url, ChannelType::Direct, "", false, false)
    }

    /// 创建带完整配置的 API 客户端
    pub fn with_config(
        proxy_url: Option<String>,
        api_channel: ChannelType,
        custom_cdn_api_ip: &str,
        use_sni_pretend: bool,
        prefer_ipv6: bool,
    ) -> Result<Self> {
        let channel_route = resolve_api_route(api_channel, custom_cdn_api_ip);
        let client = Self::build_client(
            proxy_url,
            api_channel,
            custom_cdn_api_ip,
            use_sni_pretend,
            prefer_ipv6,
        )?;

        Ok(ApiClient {
            client,
            token: Arc::new(RwLock::new(None)),
            signer: Signer::new(),
            channel_route,
        })
    }

    /// 重新加载代理配置（向后兼容）
    pub fn reload_proxy(&mut self, proxy_url: Option<String>) -> Result<()> {
        self.reload_config(proxy_url, ChannelType::Direct, "", false, false)
    }

    /// 重新加载完整配置（代理 + 分流通道 + SNI/IPv6）
    pub fn reload_config(
        &mut self,
        proxy_url: Option<String>,
        api_channel: ChannelType,
        custom_cdn_api_ip: &str,
        use_sni_pretend: bool,
        prefer_ipv6: bool,
    ) -> Result<()> {
        tracing::info!(
            "重新加载 API 客户端配置: 通道={:?}, 代理={:?}, SNI伪装={}, IPv6优先={}",
            api_channel,
            proxy_url.as_deref().unwrap_or("无"),
            use_sni_pretend,
            prefer_ipv6,
        );

        self.channel_route = resolve_api_route(api_channel, custom_cdn_api_ip);
        self.client = Self::build_client(
            proxy_url,
            api_channel,
            custom_cdn_api_ip,
            use_sni_pretend,
            prefer_ipv6,
        )?;
        Ok(())
    }

    /// 构建 reqwest::Client
    fn build_client(
        proxy_url: Option<String>,
        api_channel: ChannelType,
        custom_cdn_api_ip: &str,
        use_sni_pretend: bool,
        prefer_ipv6: bool,
    ) -> Result<Client> {
        let mut builder = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));

        // SNI 伪装：使用 rustls 配置禁用 SNI 扩展，绕过 SNI 封锁
        if use_sni_pretend {
            tracing::info!("启用 SNI 伪装模式");
            builder = builder.tls_sni(false);
        }

        // IPv6 优先：绑定本地 IPv6 地址，强制优先使用 IPv6
        if prefer_ipv6 {
            tracing::info!("启用 IPv6 优先模式");
            builder = builder.local_address("::".parse::<std::net::IpAddr>().ok());
        }

        // 添加代理
        if let Some(url) = proxy_url {
            tracing::info!("使用代理: {}", url);
            let proxy = Proxy::all(&url).map_err(|e| {
                tracing::error!("创建代理失败: {}", e);
                PicacgError::ConfigError(format!("无效的代理配置: {}", e))
            })?;
            builder = builder.proxy(proxy);
        }

        // 添加 CDN DNS 覆盖
        builder = apply_api_dns_override(builder, api_channel, custom_cdn_api_ip);

        builder.build().map_err(|e| {
            tracing::error!("创建 HTTP 客户端失败: {}", e);
            PicacgError::ConfigError(format!("创建 HTTP 客户端失败: {}", e))
        })
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

        // 构建签名 URL（始终使用原始域名）
        let mut sign_url = format!("{}{}", self.channel_route.sign_base_url, req.path());
        // 构建请求 URL（使用分流后域名）
        let mut request_url = format!("{}{}", self.channel_route.api_base_url, req.path());

        if let Some(query) = req.query() {
            let query_string = query
                .iter()
                .map(|(k, v)| {
                    let encoded_value = urlencoding::encode(v);
                    format!("{}={}", k, encoded_value)
                })
                .collect::<Vec<_>>()
                .join("&");
            sign_url = format!("{}?{}", sign_url, query_string);
            request_url = format!("{}?{}", request_url, query_string);
            tracing::debug!("签名 URL: {}", sign_url);
        }

        tracing::debug!("发送请求: {} {}", method, request_url);

        // 使用签名 URL 进行签名（始终用原始域名）
        let mut headers = self.signer.sign(&sign_url, &method);

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

        // 构建请求（使用实际请求 URL）
        let mut builder = self
            .client
            .request(method.clone(), &request_url)
            .headers(headers);

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

        // 安全地截取前 500 个字符（处理多字节 UTF-8）
        let preview: String = response_text.chars().take(500).collect();
        tracing::debug!("响应体(前 500 字符): {}", preview);

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

    #[test]
    fn test_client_with_channel() {
        let client = ApiClient::with_config(None, ChannelType::JpProxy, "", false, false);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_cdn() {
        let client = ApiClient::with_config(None, ChannelType::CdnIp1, "", false, false);
        assert!(client.is_ok());
    }
}
