# PicACG Rust 客户端

基于 **Bevy 0.18** ECS 架构的 PicACG 漫画客户端，提供原生性能和内存安全性。

## 功能概览

### 核心功能
- 登录 / 注册
- 分类浏览、搜索（排序 + 分类过滤）、排行榜
- 漫画详情（点赞、收藏、分类/标签跳转）
- 阅读器（单页模式、条漫模式、缩放控制）
- 收藏管理（分页浏览）
- 下载管理（并发控制、断点续传、CBZ 打包、独立下载设置）
- 内容过滤（按分类/标签/标题屏蔽关键词，设置页面管理，分类快速选择面板）
- 设置页面（代理、日志等级、下载路径、内容过滤、自动保存）

### UI 特性
- 瀑布流卡片布局
- VSCode 风格自定义滚动条
- 通用分页组件
- 无限滚动加载
- 滚动位置保存
- IME 中文输入法支持
- Nerd Font 图标
- 中文字体集成（Sarasa Term SC Nerd）

## 快速开始

### 环境要求

- Rust nightly (edition 2024)
- Cargo
- Vulkan / DirectX 12 / Metal（Bevy 渲染后端）

### Linux 系统依赖

```bash
sudo apt-get install -y \
    libasound2-dev \
    libudev-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libvulkan-dev
```

### 编译运行

```bash
cargo run --release
```

### 部署打包

```bash
# Linux / macOS
./scripts/deploy.sh

# Windows (PowerShell)
.\scripts\deploy-windows.ps1
```

部署脚本会自动：收集编译产物 → 复制字体资源 → 创建版本压缩包到 `bin/` 目录。

## 项目结构

采用 Cargo Workspace 架构：

```
picacg-rust/
├── Cargo.toml                    # Workspace 配置
├── assets/                       # 静态资源（字体、图片）
├── docs/                         # 文档
├── scripts/                      # 部署脚本
│   ├── deploy.sh                 # Bash 部署脚本
│   └── deploy-windows.ps1        # PowerShell 部署脚本
└── crates/
    ├── picacg_app/               # 主应用 (Bevy ECS)
    │   └── src/
    │       ├── main.rs           # 入口
    │       ├── components/       # ECS 组件
    │       ├── events/           # 事件定义
    │       ├── resources/        # 资源
    │       ├── systems/          # 系统函数（页面逻辑）
    │       └── plugins/          # 插件（UI、API）
    ├── picacg_core/              # 核心类型（错误定义）
    ├── picacg_api/               # API 客户端（28+ 端点）
    ├── picacg_db/                # 数据库层（SQLite + Moka 缓存）
    ├── picacg_config/            # 配置管理（TOML）
    └── bevy_ui_toolkit/          # 通用 UI 组件库
        └── src/
            ├── theme.rs          # 主题系统
            ├── scrollbar/        # 滚动条组件
            ├── pagination/       # 分页组件
            └── waterfall/        # 瀑布流布局
```

### Crate 依赖关系

```
picacg_core          ← 无依赖（错误类型）
    ↑
picacg_api           ← 依赖 picacg_core
    ↑
picacg_db            ← 依赖 picacg_core, picacg_api

picacg_config        ← 依赖 picacg_core

bevy_ui_toolkit      ← 依赖 bevy（独立 UI 库）

picacg_app (主应用)  ← 依赖以上所有 crate
```

## 技术栈

| 类别 | 技术 |
|------|------|
| UI 框架 | Bevy 0.18 (ECS 架构) |
| 异步运行时 | tokio |
| HTTP 客户端 | reqwest (HTTP/2, SOCKS, rustls-tls) |
| 序列化 | serde + serde_json + toml |
| 数据库 | sqlx + SQLite |
| 缓存 | moka (async) |
| 日志 | tracing + tracing-subscriber |
| 密码学 | hmac, sha2, uuid, hex |
| 并发 | parking_lot, tokio-util |

## 编译优化

Release 模式启用：
- **LTO**: fat（全局链接时优化）
- **codegen-units**: 1（单编译单元）
- **opt-level**: 3（最高优化级别）
- **strip**: true（剥离调试符号）
- **panic**: abort

## CI/CD

项目使用 GitHub Actions 实现自动化构建和发布：

- **代码检查**：`cargo fmt` + `cargo clippy`
- **多平台构建**：Linux x64 + Windows x64
- **产物压缩**：UPX（--best --lzma）
- **自动发布**：推送 `v*` 标签时创建 GitHub Release
- **版本格式**：`v{版本号}+{日期}.{commit短哈希}`

## 开发

```bash
# 格式化
cargo fmt --all

# Lint 检查
cargo clippy --all

# 运行测试
cargo test
```

## 许可证

GPL-3.0
