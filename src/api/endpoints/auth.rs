use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::{client::ApiRequest, models::User};

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

impl ApiRequest for GetUserInfoRequest {
    type Response = User;

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
    pub docs: Vec<crate::api::models::Comment>,
    pub page: crate::api::models::PageInfo,
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

// 忘记密码（发送重置邮件）
#[derive(Debug, Serialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
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
