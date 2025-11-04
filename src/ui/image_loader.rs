use iced::widget::image;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 图片加载状态
#[derive(Debug, Clone)]
pub enum ImageState {
    /// 未加载
    NotLoaded,
    /// 加载中
    Loading,
    /// 加载成功
    Loaded(image::Handle),
    /// 加载失败
    Failed(String),
}

/// 图片缓存管理器
#[derive(Debug, Clone)]
pub struct ImageCache {
    cache: Arc<RwLock<HashMap<String, ImageState>>>,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取图片状态
    pub async fn get(&self, url: &str) -> ImageState {
        let cache = self.cache.read().await;
        cache
            .get(url)
            .cloned()
            .unwrap_or(ImageState::NotLoaded)
    }

    /// 设置图片状态
    pub async fn set(&self, url: String, state: ImageState) {
        let mut cache = self.cache.write().await;
        cache.insert(url, state);
    }

    /// 检查图片是否已加载
    pub async fn is_loaded(&self, url: &str) -> bool {
        let cache = self.cache.read().await;
        matches!(cache.get(url), Some(ImageState::Loaded(_)))
    }

    /// 检查图片是否正在加载
    pub async fn is_loading(&self, url: &str) -> bool {
        let cache = self.cache.read().await;
        matches!(cache.get(url), Some(ImageState::Loading))
    }
}

/// 下载图片
pub async fn download_image(
    _client: crate::api::ApiClient,
    url: String,
) -> Result<image::Handle, String> {
    use crate::config::settings::AppSettings;
    use reqwest::{Client, Proxy};
    use std::time::Duration;

    // 创建 HTTP 客户端（使用全局配置的代理）
    let proxy_url = {
        let settings = AppSettings::global().read();
        settings.proxy.to_proxy_url()
    }; // 在这里释放 settings 锁

    let mut builder = Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10));

    if let Some(proxy_url) = proxy_url {
        let proxy = Proxy::all(&proxy_url)
            .map_err(|e| format!("创建代理失败: {}", e))?;
        builder = builder.proxy(proxy);
    }

    let http_client = builder
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 获取图片数据
    let response = http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取数据失败: {}", e))?;

    // 转换为 iced 的 Handle
    Ok(image::Handle::from_bytes(bytes))
}
