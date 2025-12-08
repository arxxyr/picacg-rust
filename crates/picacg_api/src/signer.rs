//! API 签名

use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use reqwest::{
    Method,
    header::{HeaderMap, HeaderValue},
};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const API_KEY: &str = "C69BAF41DA5ABD1FFEDC6D2FEA56B";
const SECRET_KEY: &str = r"~d}$Q7$eIni=V)9\RK/P.RM4;9[7|@/CA}b~OW!3?EV`:<>M7pddUBL5n|0/*Cn";
const VERSION: &str = "2.2.1.3.3.4";
const BUILD_VERSION: &str = "45";
const BASE_URL: &str = "https://picaapi.picacomic.com/";
const APP_UUID: &str = "defaultUuid";

#[derive(Clone, Copy)]
pub struct Signer;

impl Signer {
    pub fn new() -> Self {
        Signer
    }

    pub fn sign(&self, url: &str, method: &Method) -> HeaderMap {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let nonce = Uuid::new_v4().simple().to_string();

        // 提取相对路径（去掉 BASE_URL 前缀）
        let relative_path = url.strip_prefix(BASE_URL).unwrap_or_else(|| {
            url.strip_prefix("https://picaapi.picacomic.com/")
                .unwrap_or(url)
        });

        // 构造签名源字符串
        let src = format!(
            "{}{}{}{}{}",
            relative_path,
            now,
            nonce,
            method.as_str(),
            API_KEY,
        );

        // HMAC-SHA256 签名
        let mut mac = HmacSha256::new_from_slice(SECRET_KEY.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(src.to_lowercase().as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // 构造 Header
        let mut headers = HeaderMap::new();
        headers.insert("api-key", HeaderValue::from_static(API_KEY));
        headers.insert(
            "accept",
            HeaderValue::from_static("application/vnd.picacomic.com.v1+json"),
        );
        headers.insert("app-channel", HeaderValue::from_static("3"));
        headers.insert("time", HeaderValue::from_str(&now).unwrap());
        headers.insert("app-uuid", HeaderValue::from_static(APP_UUID));
        headers.insert("nonce", HeaderValue::from_str(&nonce).unwrap());
        headers.insert("signature", HeaderValue::from_str(&signature).unwrap());
        headers.insert("app-version", HeaderValue::from_static(VERSION));
        headers.insert("image-quality", HeaderValue::from_static("original"));
        headers.insert("app-platform", HeaderValue::from_static("android"));
        headers.insert("app-build-version", HeaderValue::from_static(BUILD_VERSION));
        headers.insert("user-agent", HeaderValue::from_static("okhttp/3.8.1"));

        if method == Method::POST || method == Method::PUT {
            headers.insert(
                "content-type",
                HeaderValue::from_static("application/json; charset=UTF-8"),
            );
        }

        headers
    }
}

impl Default for Signer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer() {
        let signer = Signer::new();
        let url = "https://picaapi.picacomic.com/auth/sign-in";
        let headers = signer.sign(url, &Method::POST);

        assert!(headers.contains_key("api-key"));
        assert!(headers.contains_key("signature"));
        assert!(headers.contains_key("time"));
        assert!(headers.contains_key("nonce"));
    }
}
