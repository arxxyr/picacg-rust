# PicACG Rust 客户端

基于 **Bevy 0.19** ECS 架构的 PicACG 漫画客户端，提供原生性能和内存安全性。使用 **mimalloc** 全局内存分配器优化多线程性能。

## 功能概览

### 核心功能
- 登录 / 注册 / 找回密码
- 分类浏览、搜索（排序 + 分类过滤）、排行榜（含骑士榜）、首页推荐
- 漫画详情（点赞、收藏、评论、分类/标签/作者跳转）
- 阅读器（单页 / 条漫双模式、手动缩放、Shift 横向平移、本地文件优先加载、失败点击重试）
- **阅读进度与继续阅读**（自动记录到章 + 页，详情页一键续读，历史页管理）
- 图片列表三级加载（DB 缓存 → 本地下载目录重建 → 网络并发拉取，已下载漫画零网络秒开）
- 收藏管理、点赞记录、个人资料（自动加载头像）
- 下载管理（并发控制、断点续传、CBZ 打包、移动/打开、单本独立设置、批量选择下载）
- 下载更新两档（普通更新走 epsCount 快照前置比对，强制更新逐图校验补缺）+ 封面下载角标（已下载 ✓ / 有更新 ⟳）
- 右键菜单（下载 / 屏蔽，全局生效）与快速下载（自动获取章节列表，无需进详情页）
- 内容过滤（按分类/标签/标题屏蔽，繁简转换，分类快速选择面板）
- 分流通道（6 种：直连 / CDN ×2 / 自定义 IP / 日本反代 / 美国反代，API 与图片独立配置）
- 代理支持（HTTP/SOCKS5 + 用户名密码认证）与网络诊断（测速 + Ping）
- 检查更新与自动更新（Linux 原地自替换 / macOS 下载校验后挂载 dmg / Windows 下载校验后打开，强制 SHA-256 校验，走代理）
- 「下载全部完成后退出」挂机模式（等 CBZ 打包收尾后退出）
- 性能追踪（F3 帧时叠加层、F4 系统耗时榜、掉帧自动打榜并落盘）
- 设置页面（全部自动保存：代理、日志等级、下载路径、内容过滤、分流通道、更新检查等）
- 游戏区、锅贴社区、聊天室、本地阅读、NAS 同步

### UI 特性
- 漫画列表虚拟滚动 + 节点复用（滚动路径零 spawn/despawn，实体数 4200 → ~300）
- 条漫锚点滚动模型（图片高度陆续就位时视觉位置纹丝不动）
- 图片占位骨架屏微光动画（失败即停 + 点击重试）
- BSN 场景化 UI（Bevy 0.19 `bsn!`）、上游 ScrollArea/Scrollbar（VSCode 风格外观）
- 通用分页组件（值组件 + 内联观察者）、瀑布流布局
- 页面缓存（Display 显隐切换，页面切换零延迟）
- 全局焦点体系（上游 InputFocus + TabIndex 导航）、IME 中文输入
- 下载计数徽章、侧边栏用户头像
- 哔咔官方应用图标（窗口 / 任务栏 / macOS Dock + .icns bundle 图标）
- 合盖保活（窗口销毁自动重建，下载不中断）
- CJK 字体内置（更纱黑体，系统字体回退）、Unicode 通用符号图标

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

采用纯 Cargo Workspace 架构（根目录无 `[package]`，版本统一管理）：

```
picacg-rust/
├── Cargo.toml                    # Workspace 配置
├── assets/                       # 静态资源
│   ├── fonts/SarasaTermSCNerd/   # 内置更纱黑体 CJK 字体
│   ├── icons/                    # 应用图标（官方素材）
│   └── dmg/                      # macOS dmg 安装窗口背景
├── docs/                         # 文档（API 协议等）
├── scripts/                      # 部署与工具脚本
│   ├── deploy.sh / deploy-windows.ps1
│   ├── make-icon.sh              # 图标尺寸生成
│   ├── make-dmg-background.sh    # dmg 背景图生成
│   └── check_query_conflicts.py  # Bevy B0001 查询冲突静态检查
└── crates/
    ├── picacg_app/               # 主应用 (Bevy ECS)
    │   └── src/
    │       ├── main.rs           # 入口（mimalloc 全局分配器）
    │       ├── components/       # ECS 组件
    │       ├── events/           # 事件定义（请求-响应事件携带身份防串页）
    │       ├── resources/        # 资源（状态、图片缓存状态机）
    │       ├── systems/          # 系统函数（页面逻辑 + 共享 UI 层）
    │       ├── plugins/          # 插件（UI、API）
    │       └── utils/            # 工具（内容过滤、自然排序、性能追踪、Bevy-tokio 集成）
    ├── picacg_core/              # 核心类型（错误定义）
    ├── picacg_api/               # API 客户端（签名、分流通道、28+ 端点）
    ├── picacg_db/                # 数据库层（SQLite，内嵌迁移 + 独立异步函数）
    └── picacg_config/            # 配置管理（TOML，含代理/分流/过滤设置）
```

### Crate 依赖关系

```
picacg_core          ← 无依赖（错误类型）
    ↑
picacg_config        ← 依赖 picacg_core
    ↑
picacg_api           ← 依赖 picacg_core, picacg_config
    ↑
picacg_db            ← 依赖 picacg_core, picacg_api

picacg_app (主应用)  ← 依赖以上所有 crate
```

## 技术栈

| 类别 | 技术 |
|------|------|
| UI 框架 | Bevy 0.19 (ECS + BSN 场景系统) |
| 异步运行时 | tokio |
| HTTP 客户端 | reqwest (HTTP/2, SOCKS, rustls-tls) |
| 序列化 | serde + serde_json + toml |
| 数据库 | sqlx + SQLite（WAL） |
| 日志 | tracing + tracing-subscriber |
| 密码学 | hmac, sha2, uuid, hex |
| 并发 | parking_lot, tokio-util |
| 内存分配 | mimalloc |
| 文本处理 | zhconv（繁简转换） |
| 自更新 | self-replace（Linux 原地替换） |

## 编译优化

Release 模式启用：
- **LTO**: fat（全局链接时优化）
- **codegen-units**: 1（单编译单元）
- **opt-level**: 3（最高优化级别）
- **strip**: true（剥离调试符号）
- **panic**: abort

## CI/CD

项目使用 **GitHub Actions + GitLab CI** 双轨实现自动化构建和发布：

- **代码检查**：`cargo fmt` → `cargo clippy -D warnings` → B0001 静态检查 → `cargo test`（严格串行）
- **多平台构建**：Linux x64 + Windows x64 + macOS ARM64（bundle 级 ad-hoc 签名 + dmg 安装窗口）
- **产物校验**：每个包附 `.sha256`（自动更新强制校验）
- **产物压缩**：UPX（--best --lzma，Windows/macOS 跳过）
- **自动发布**：推送 `v*` 标签或 master 推送自动发布 GitHub Release
- **版本格式**：Release `v{版本号}+{commit短哈希}` / Dev `v{版本号}+{日期}.{commit短哈希}`

## 开发

```bash
# 格式化 + Lint（提交前必须零警告）
cargo fmt --all && cargo clippy --all --all-targets -- -D warnings

# 运行测试
cargo test --workspace

# Bevy B0001 查询冲突静态检查
python3 scripts/check_query_conflicts.py
```

开发规范、架构约定与常见陷阱见 [CLAUDE.md](CLAUDE.md)。

## 许可证

Apache-2.0
