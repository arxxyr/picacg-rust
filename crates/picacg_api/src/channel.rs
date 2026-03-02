//! 分流通道路由
//!
//! 支持直连、CDN IP 覆盖、反代等多种通道

use std::net::{IpAddr, SocketAddr};

use picacg_config::ChannelType;
use reqwest::ClientBuilder;

/// API 原始域名
pub const API_DOMAIN: &str = "picaapi.picacomic.com";

/// CDN IP 1
pub const CDN_IP_1: &str = "104.21.91.145";

/// CDN IP 2
pub const CDN_IP_2: &str = "188.114.98.153";

/// 日本反代 API 域名
pub const JP_PROXY_API: &str = "bika-api.jpacg.cc";

/// 日本反代图片域名
pub const JP_PROXY_IMG: &str = "bika-img.jpacg.cc";

/// 美国反代 API 域名
pub const US_PROXY_API: &str = "bika2-api.jpacg.cc";

/// 美国反代图片域名
pub const US_PROXY_IMG: &str = "bika21-img.jpacg.cc";

/// 图片服务器域名列表
pub const IMAGE_DOMAINS: &[&str] = &[
    "s3.picacomic.com",
    "storage1.picacomic.com",
    "storage-b.picacomic.com",
    "img.picacomic.com",
];

/// 通道路由配置
#[derive(Debug, Clone)]
pub struct ChannelRoute {
    /// 实际请求的 API 基础 URL
    pub api_base_url: String,
    /// 签名用的 API 基础 URL（始终用原始域名）
    pub sign_base_url: String,
    /// 是否为反代模式（反代 URL 格式不同）
    pub is_proxy_mode: bool,
}

impl Default for ChannelRoute {
    fn default() -> Self {
        Self {
            api_base_url: format!("https://{}", API_DOMAIN),
            sign_base_url: format!("https://{}", API_DOMAIN),
            is_proxy_mode: false,
        }
    }
}

/// 根据通道类型解析 API 路由
pub fn resolve_api_route(channel: ChannelType, _custom_ip: &str) -> ChannelRoute {
    let sign_base_url = format!("https://{}", API_DOMAIN);

    match channel {
        ChannelType::Direct => ChannelRoute {
            api_base_url: sign_base_url.clone(),
            sign_base_url,
            is_proxy_mode: false,
        },
        ChannelType::CdnIp1 | ChannelType::CdnIp2 | ChannelType::CustomCdnIp => {
            // CDN 模式：URL 不变，通过 DNS 覆盖指向 CDN IP
            ChannelRoute {
                api_base_url: sign_base_url.clone(),
                sign_base_url,
                is_proxy_mode: false,
            }
        }
        ChannelType::JpProxy => ChannelRoute {
            api_base_url: format!("https://{}/{}", JP_PROXY_API, API_DOMAIN),
            sign_base_url,
            is_proxy_mode: true,
        },
        ChannelType::UsProxy => ChannelRoute {
            api_base_url: format!("https://{}/{}", US_PROXY_API, API_DOMAIN),
            sign_base_url,
            is_proxy_mode: true,
        },
    }
}

/// 为 ClientBuilder 添加 API 通道的 DNS 覆盖
pub fn apply_api_dns_override(
    mut builder: ClientBuilder,
    channel: ChannelType,
    custom_ip: &str,
) -> ClientBuilder {
    if let Some(ip) = get_cdn_ip(channel, custom_ip) {
        let addr = SocketAddr::new(ip, 443);
        builder = builder.resolve(API_DOMAIN, addr);
    }
    builder
}

/// 为 ClientBuilder 添加图片通道的 DNS 覆盖
pub fn apply_image_dns_override(
    mut builder: ClientBuilder,
    channel: ChannelType,
    custom_ip: &str,
) -> ClientBuilder {
    if let Some(ip) = get_cdn_ip(channel, custom_ip) {
        let addr = SocketAddr::new(ip, 443);
        for domain in IMAGE_DOMAINS {
            builder = builder.resolve(domain, addr);
        }
    }
    builder
}

/// 转换图片 URL（反代模式下替换域名）
pub fn transform_image_url(url: &str, channel: ChannelType) -> String {
    match channel {
        ChannelType::JpProxy => replace_image_domain(url, JP_PROXY_IMG),
        ChannelType::UsProxy => replace_image_domain(url, US_PROXY_IMG),
        _ => url.to_string(),
    }
}

/// 获取 CDN IP 地址
fn get_cdn_ip(channel: ChannelType, custom_ip: &str) -> Option<IpAddr> {
    match channel {
        ChannelType::CdnIp1 => CDN_IP_1.parse().ok(),
        ChannelType::CdnIp2 => CDN_IP_2.parse().ok(),
        ChannelType::CustomCdnIp => {
            if custom_ip.is_empty() {
                None
            } else {
                custom_ip.parse().ok()
            }
        }
        _ => None,
    }
}

/// 替换图片 URL 中的域名
fn replace_image_domain(url: &str, new_host: &str) -> String {
    for domain in IMAGE_DOMAINS {
        if url.contains(domain) {
            return url.replace(domain, new_host);
        }
    }
    // 域名不在已知列表中，尝试通用替换
    if let Some(after_scheme) = url.strip_prefix("https://")
        && let Some(slash_pos) = after_scheme.find('/')
    {
        let original_host = &after_scheme[..slash_pos];
        let path = &after_scheme[slash_pos..];
        return format!("https://{}/{}{}", new_host, original_host, path);
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_route() {
        let route = ChannelRoute::default();
        assert_eq!(route.api_base_url, "https://picaapi.picacomic.com");
        assert!(!route.is_proxy_mode);
    }

    #[test]
    fn test_jp_proxy_route() {
        let route = resolve_api_route(ChannelType::JpProxy, "");
        assert_eq!(
            route.api_base_url,
            "https://bika-api.jpacg.cc/picaapi.picacomic.com"
        );
        assert!(route.is_proxy_mode);
        assert_eq!(route.sign_base_url, "https://picaapi.picacomic.com");
    }

    #[test]
    fn test_transform_image_url_jp() {
        let url = "https://s3.picacomic.com/static/some/path.jpg";
        let result = transform_image_url(url, ChannelType::JpProxy);
        assert_eq!(result, "https://bika-img.jpacg.cc/static/some/path.jpg");
    }

    #[test]
    fn test_transform_image_url_direct() {
        let url = "https://s3.picacomic.com/static/some/path.jpg";
        let result = transform_image_url(url, ChannelType::Direct);
        assert_eq!(result, url);
    }

    #[test]
    fn test_get_cdn_ip() {
        assert!(get_cdn_ip(ChannelType::CdnIp1, "").is_some());
        assert!(get_cdn_ip(ChannelType::CdnIp2, "").is_some());
        assert!(get_cdn_ip(ChannelType::CustomCdnIp, "1.2.3.4").is_some());
        assert!(get_cdn_ip(ChannelType::CustomCdnIp, "").is_none());
        assert!(get_cdn_ip(ChannelType::Direct, "").is_none());
    }
}
