# PicACG API 协议文档

> 最后更新: 2026-08-31

## API 基础信息

### 服务器地址

| 服务 | URL |
|------|-----|
| API 基础 | `https://picaapi.picacomic.com/` |
| 图片服务器 | `https://s3.picacomic.com/` |
| 聊天服务器 | `https://live-server.bidobido.xyz/` |

### 请求头

所有 API 请求需要以下 Header：

```http
api-key: C69BAF41DA5ABD1FFEDC6D2FEA56B
accept: application/vnd.picacomic.com.v1+json
app-channel: 3
time: {Unix 时间戳}
app-uuid: {设备 UUID}
nonce: {随机 UUID}
signature: {HMAC-SHA256 签名}
app-version: 2.2.1.3.3.4
image-quality: original
app-platform: android
app-build-version: 45
user-agent: okhttp/3.8.1
Content-Type: application/json; charset=UTF-8  # POST/PUT 请求
authorization: {JWT Token}  # 需要认证的请求
```

### 签名算法

**签名源字符串**:
```
raw_url + relative_path + timestamp + nonce + method + api_key + version + build_version
```

**签名计算**:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

const SECRET_KEY: &str = "~d}$Q7$eIni=V)9\\RK/P.RM4;9[7|@/CA}b~OW!3?EV`:<>M7pddUBL5n|0/*Cn";

fn sign(src: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(SECRET_KEY.as_bytes()).unwrap();
    mac.update(src.to_lowercase().as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

---

## 通用响应格式

```json
{
  "code": 200,
  "message": "success",
  "data": { ... }
}
```

**错误码**: 200=成功, 401=未认证, 404=不存在, 500=服务器错误

---

## 核心 API 端点

### 认证

| 端点 | 方法 | 说明 |
|------|------|------|
| `/auth/sign-in` | POST | 登录 |
| `/auth/register` | POST | 注册 |
| `/auth/forgot-password` | POST | 获取密保问题 |
| `/auth/reset-password` | POST | 重置密码 |

**登录请求**:
```json
{ "email": "user@example.com", "password": "password123" }
```

**登录响应**:
```json
{ "code": 200, "data": { "token": "eyJ..." } }
```

### 用户

| 端点 | 方法 | 说明 |
|------|------|------|
| `/users/profile` | GET | 获取用户信息 |
| `/users/punch-in` | POST | 签到 |
| `/users/avatar` | PUT | 设置头像 |
| `/users/password` | PUT | 修改密码 |
| `/users/my-comments` | GET | 获取我的评论 |
| `/users/favourite` | GET | 获取收藏列表 |

### 漫画

| 端点 | 方法 | 说明 |
|------|------|------|
| `/categories` | GET | 获取分类 |
| `/comics` | GET | 分类搜索 |
| `/comics/advanced-search` | POST | 高级搜索 |
| `/comics/{id}` | GET | 漫画详情 |
| `/comics/{id}/eps` | GET | 章节列表 |
| `/comics/{id}/order/{order}/pages` | GET | 章节图片 |
| `/comics/{id}/recommendation` | GET | 相关推荐 |
| `/comics/{id}/favourite` | POST | 收藏/取消收藏 |
| `/comics/{id}/like` | POST | 点赞/取消点赞 |
| `/comics/{id}/comments` | GET/POST | 评论 |
| `/comics/leaderboard` | GET | 排行榜 |
| `/comics/random` | GET | 随机漫画 |

**分类搜索参数**:
- `page`: 页码
- `c`: 分类名
- `s`: 排序 (ua=默认, dd=新到旧, da=旧到新, ld=最多爱心, vd=最多浏览)

**图片 URL 拼接**:
```
完整 URL = fileServer + "static/" + path
例: https://s3.picacomic.com/static/path/to/image.jpg
```

### 实测协议行为（踩坑记录）

- ⚠️ **高级搜索的排序只认请求体**：`POST /comics/advanced-search` 必须把
  `sort` 放在 JSON body（`{ "keyword": ..., "sort": ... }`）；查询串上的
  `s=` 会被服务端静默忽略（只有 `GET /comics` 认 `s`）。历史实现只发查询串，
  表现为四种排序返回完全相同的列表
- ⚠️ **`epsCount` 是漂移的冗余计数**：与 `GET /comics/{id}/eps` 的真实条数
  长期对不上且两个方向都会偏（实测 48↔49、46↔48、12↔15、55↔53）。不能拿它
  与本地章节数直比；同一漫画的 epsCount 随时间自增，「今天的 epsCount >
  历史快照」才是可靠的更新信号（同字段自比，系统偏差相消）
- `GET /comics/{id}/order/{order}/pages` 按 40 张/页分页，取整章必须循环拉完
  `pages` 页；章节列表接口返回顺序不可信，消费前按 `order` 显式排序

### 评论

| 端点 | 方法 | 说明 |
|------|------|------|
| `/comments/{id}/like` | POST | 评论点赞 |
| `/comments/{id}/childrens` | GET | 子评论 |
| `/comments/{id}` | POST | 回复评论 |

### 游戏

| 端点 | 方法 | 说明 |
|------|------|------|
| `/games` | GET | 游戏列表 |
| `/games/{id}` | GET | 游戏详情 |
| `/games/{id}/comments` | GET/POST | 游戏评论 |

### 其他

| 端点 | 方法 | 说明 |
|------|------|------|
| `/keywords` | GET | 热词 |
| `/collections` | GET | 神魔推荐 |
| `/comics/knight-leaderboard` | GET | 骑士榜 |
| `/chat` | GET | 聊天室列表 |

---

## 数据模型 (Rust)

### 基础类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "fileServer")]
    pub file_server: String,
    pub path: String,
    #[serde(rename = "originalName")]
    pub original_name: String,
}

impl Image {
    pub fn full_url(&self) -> String {
        format!("{}static/{}", self.file_server, self.path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub title: String,
    pub thumb: Option<Image>,
    #[serde(rename = "isWeb")]
    pub is_web: Option<bool>,
    pub active: Option<bool>,
    pub link: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub author: String,
    pub thumb: Image,
    pub categories: Vec<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "pagesCount")]
    pub pages_count: i32,
    #[serde(rename = "epsCount")]
    pub eps_count: i32,
    pub finished: bool,
    #[serde(rename = "likesCount")]
    pub likes_count: i64,
    #[serde(rename = "totalViews")]
    pub total_views: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub order: i32,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
    #[serde(rename = "_id")]
    pub id: String,
    pub media: Image,
}
```

### 分页响应

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination<T> {
    pub docs: Vec<T>,
    pub total: i64,
    pub limit: i32,
    pub page: i32,
    pub pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}
```

---

## 实现参考

当前 Rust 实现位于 `crates/picacg_api/src/`：
- `client.rs` - HTTP 客户端（代理 / 分流路由接入）
- `signer.rs` - 请求签名
- `channel.rs` - 分流通道路由（CDN DNS 覆盖 / 反代 URL 重写，签名始终用原始域名）
- `models.rs` - 数据模型
- `endpoints/` - API 端点定义
