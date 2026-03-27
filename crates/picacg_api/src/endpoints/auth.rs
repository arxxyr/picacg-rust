//! 认证相关 API 端点

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{client::ApiRequest, models::User};

// 登录请求
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

impl ApiRequest for LoginRequest {
    type Response = LoginResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        "/auth/sign-in".to_string()
    }

    fn need_auth(&self) -> bool {
        false
    }
}

// 获取用户信息请求
#[derive(Debug, Serialize)]
pub struct GetUserInfoRequest;

/// `/users/profile` 响应的 data 层
///
/// API 返回格式：`{ code: 200, data: { user: { ... } } }`，
/// 用户信息嵌套在 `user` 字段中。
#[derive(Debug, Deserialize)]
pub struct GetUserInfoResponse {
    pub user: User,
}

impl ApiRequest for GetUserInfoRequest {
    type Response = GetUserInfoResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/users/profile".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 打卡请求
#[derive(Debug, Serialize)]
pub struct PunchInRequest;

/// 打卡响应内层结构
#[derive(Debug, Deserialize)]
pub struct PunchInRes {
    pub status: String,
}

/// 打卡响应（匹配 API 嵌套结构 data.res.status）
#[derive(Debug, Deserialize)]
pub struct PunchInResponse {
    pub res: PunchInRes,
}

impl ApiRequest for PunchInRequest {
    type Response = PunchInResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        "/users/punch-in".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 注册请求
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub birthday: String,
    pub gender: String, // "m", "f", "bot"
    pub question1: String,
    pub question2: String,
    pub question3: String,
    pub answer1: String,
    pub answer2: String,
    pub answer3: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub message: String,
}

impl ApiRequest for RegisterRequest {
    type Response = RegisterResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        "/auth/register".to_string()
    }

    fn need_auth(&self) -> bool {
        false
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "email": self.email,
            "password": self.password,
            "name": self.name,
            "birthday": self.birthday,
            "gender": self.gender,
            "question1": self.question1,
            "question2": self.question2,
            "question3": self.question3,
            "answer1": self.answer1,
            "answer2": self.answer2,
            "answer3": self.answer3,
        }))
    }
}

// 修改密码请求
#[derive(Debug, Serialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}

impl ApiRequest for ChangePasswordRequest {
    type Response = ChangePasswordResponse;

    fn method(&self) -> Method {
        Method::PUT
    }

    fn path(&self) -> String {
        "/users/password".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "old_password": self.old_password,
            "new_password": self.new_password,
        }))
    }
}

// 获取我的评论
#[derive(Debug, Serialize)]
pub struct GetMyCommentsRequest {
    #[serde(skip_serializing)]
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct MyCommentsData {
    pub docs: Vec<crate::models::Comment>,
    pub page: crate::models::PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct GetMyCommentsResponse {
    pub comments: MyCommentsData,
}

impl ApiRequest for GetMyCommentsRequest {
    type Response = GetMyCommentsResponse;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        "/users/my-comments".to_string()
    }

    fn query(&self) -> Option<Vec<(String, String)>> {
        Some(vec![("page".to_string(), self.page.to_string())])
    }

    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

// 忘记密码（获取安全问题）
#[derive(Debug, Serialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordResponse {
    pub question1: String,
    pub question2: String,
    pub question3: String,
}

impl ApiRequest for ForgotPasswordRequest {
    type Response = ForgotPasswordResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        "/auth/forgot-password".to_string()
    }

    fn need_auth(&self) -> bool {
        false
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "email": self.email,
        }))
    }
}

// 设置头像请求
#[derive(Debug, Serialize)]
pub struct SetAvatarRequest {
    /// base64 data URI 格式的头像数据
    pub avatar: String,
}

#[derive(Debug, Deserialize)]
pub struct SetAvatarResponse {
    #[serde(default)]
    pub message: Option<String>,
}

impl ApiRequest for SetAvatarRequest {
    type Response = SetAvatarResponse;

    fn method(&self) -> Method {
        Method::PUT
    }

    fn path(&self) -> String {
        "/users/avatar".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "avatar": self.avatar,
        }))
    }
}

// 设置称号请求
#[derive(Debug, Serialize)]
pub struct SetTitleRequest {
    /// 称号名称
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct SetTitleResponse {
    #[serde(default)]
    pub message: Option<String>,
}

impl ApiRequest for SetTitleRequest {
    type Response = SetTitleResponse;

    fn method(&self) -> Method {
        Method::PUT
    }

    fn path(&self) -> String {
        "/users/title".to_string()
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "title": self.title,
        }))
    }
}

// 重置密码（通过安全问题）
#[derive(Debug, Serialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    /// 安全问题编号 (1, 2, 或 3)
    #[serde(rename = "questionNo")]
    pub question_no: i32,
    pub answer: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordResponse {
    /// API 返回的新临时密码
    pub password: Option<String>,
}

impl ApiRequest for ResetPasswordRequest {
    type Response = ResetPasswordResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        "/auth/reset-password".to_string()
    }

    fn need_auth(&self) -> bool {
        false
    }

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "email": self.email,
            "questionNo": self.question_no,
            "answer": self.answer,
        }))
    }
}
