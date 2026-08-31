# CBZ 打包功能设计文档

> 文档版本: 1.1
> 创建日期: 2026-02-05
> 状态: **已实现**（后台线程打包 + `CbzPackagingState.in_flight` 计数配合
> 「下载全部完成后退出」；实际落盘目录为 `{下载目录}/image/` 与 `{下载目录}/cbz/`，
> 与本文草案中的 `Downloads/Images|CBZ/` 命名略有出入，以代码为准）

## 1. 需求概述

下载完成后自动打包成 CBZ 格式：
- **原图目录**: `Downloads/Images/<漫画标题>/<章节>/`
- **CBZ 目录**: `Downloads/CBZ/<漫画标题>.cbz`

CBZ 本质是 ZIP 文件（扩展名改为 `.cbz`），图片（JPG/PNG/WebP）本身已是压缩格式，使用 `Stored`（存储模式）打包即可，速度最快且无损。

## 2. 目录结构变更

### 2.1 当前结构
```
Downloads/
└── <漫画标题>/
    ├── 第1章/
    │   ├── 0001.jpg
    │   └── ...
    └── 第2章/
```

### 2.2 新结构
```
Downloads/                      # 下载根目录（用户可配置）
├── Images/                     # 原图存放目录
│   └── <漫画标题>/
│       ├── 第1章/
│       │   ├── 0001.jpg
│       │   └── ...
│       └── 第2章/
└── CBZ/                        # CBZ 文件存放目录
    └── <漫画标题>.cbz
```

## 3. 文件修改清单

### 3.1 依赖添加

| 文件 | 修改 |
|------|------|
| `Cargo.toml` (workspace) | 添加 `zip = { version = "2", default-features = false }` |
| `crates/picacg_app/Cargo.toml` | 添加 `zip.workspace = true` |

### 3.2 配置项

**文件**: `crates/picacg_config/src/settings.rs`

在 `AppSettings` 中添加：
```rust
/// 下载完成后自动打包为 CBZ 格式
#[serde(default)]
pub auto_pack_cbz: bool,

/// 打包 CBZ 后删除原图文件夹
#[serde(default)]
pub delete_images_after_cbz: bool,
```

### 3.3 事件定义

**文件**: `crates/picacg_app/src/events/api_events.rs`

新增消息：
```rust
/// CBZ 打包请求
#[derive(Message)]
pub struct CbzPackageRequest {
    pub comic_id: String,
    pub comic_title: String,
    pub source_path: String,
}

/// CBZ 打包完成事件
#[derive(Message)]
pub struct CbzPackageCompletedEvent {
    pub comic_id: String,
    pub cbz_path: String,
}

/// CBZ 打包失败事件
#[derive(Message)]
pub struct CbzPackageFailedEvent {
    pub comic_id: String,
    pub error: String,
}
```

### 3.4 核心逻辑

**文件**: `crates/picacg_app/src/plugins/api_plugin.rs`

| 修改点 | 说明 |
|--------|------|
| `get_download_base_path()` | 保持不变（返回基础路径） |
| 新增 `get_images_download_path()` | 返回 `base/Images` |
| 新增 `get_cbz_output_path()` | 返回 `base/CBZ` |
| 修改 `handle_download_comic()` | 使用 `get_images_download_path()` |
| 修改 `handle_download_completed()` | 检查配置，发送 `CbzPackageRequest` |
| 新增 `handle_cbz_package_request()` | 异步打包（spawn_blocking） |
| 新增 `create_cbz_package()` | ZIP 打包核心函数（Stored 模式） |
| 新增 `collect_image_files()` | 递归收集图片文件 |
| 新增 `is_image_file()` | 判断是否为图片文件 |
| 新增 `handle_cbz_package_completed()` | 打包完成处理 |
| 新增 `handle_cbz_package_failed()` | 打包失败处理 |

### 3.5 设置 UI

**文件**: `crates/picacg_app/src/systems/settings.rs`

在下载设置分组中添加两个勾选框：
- "下载完成后自动打包 CBZ"
- "打包 CBZ 后删除原图文件夹"

### 3.6 下载 UI

**文件**: `crates/picacg_app/src/systems/downloads.rs`

- 新增 `OpenCbzFolderButton` 按钮组件
- 新增 `handle_open_cbz_folder()` 系统函数

## 4. 事件流程

```
DownloadCompletedEvent
    │
    ▼
handle_download_completed()
    │
    ├── 检查 auto_pack_cbz 配置
    │
    ▼ (启用时)
CbzPackageRequest
    │
    ▼
handle_cbz_package_request()
    │
    ├── spawn_blocking(create_cbz_package)
    │       │
    │       ├── 创建 CBZ 输出目录
    │       ├── 递归收集图片文件
    │       ├── 按文件名排序
    │       └── 写入 ZIP（Stored 模式）
    │
    ▼
    ├─ 成功 ─► CbzPackageCompletedEvent
    │              │
    │              ▼ (如果 delete_images_after_cbz)
    │              删除原图目录
    │
    └─ 失败 ─► CbzPackageFailedEvent
```

## 5. 核心实现

### 5.1 路径函数

```rust
/// 获取原图下载保存路径
fn get_images_download_path() -> PathBuf {
    get_download_base_path().join("Images")
}

/// 获取 CBZ 文件保存路径
fn get_cbz_output_path() -> PathBuf {
    get_download_base_path().join("CBZ")
}
```

### 5.2 CBZ 打包函数

```rust
async fn create_cbz_package(
    source_path: &str,
    comic_title: &str,
) -> Result<String, String> {
    // 使用 spawn_blocking 避免阻塞异步运行时
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::create(&cbz_path)?;
        let mut zip = ZipWriter::new(file);

        // 使用 Stored 模式（图片已压缩）
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored);

        // 递归收集并排序图片文件
        let mut entries = collect_image_files(source_dir)?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        // 写入 ZIP
        for (archive_name, file_path) in entries {
            zip.start_file(&archive_name, options)?;
            let data = std::fs::read(&file_path)?;
            zip.write_all(&data)?;
        }

        zip.finish()?;
        Ok(cbz_path.to_string_lossy().to_string())
    }).await?
}
```

### 5.3 CBZ 内部结构

```
漫画标题.cbz
├── 第1章/
│   ├── 0001.jpg
│   ├── 0002.jpg
│   └── ...
├── 第2章/
│   ├── 0001.jpg
│   └── ...
└── ...
```

## 6. 错误处理

| 错误类型 | 处理方式 |
|---------|---------|
| 源目录不存在 | 返回 `CbzPackageFailedEvent`，记录错误日志 |
| 创建 CBZ 目录失败 | 返回 `CbzPackageFailedEvent`，记录错误日志 |
| ZIP 写入失败 | 返回 `CbzPackageFailedEvent`，记录错误日志 |
| 删除原图失败 | 仅记录警告日志，不影响打包结果 |
| 读取图片文件失败 | 跳过该文件，继续打包，记录警告 |

## 7. 向后兼容

- 旧下载目录（直接在 Downloads 下）不自动迁移
- 仅新下载使用新目录结构
- 设置默认关闭，用户需手动启用

## 8. 验证方法

1. **编译检查**: `cargo build --release`
2. **功能测试**:
   - 启用自动打包设置
   - 下载一部漫画
   - 检查 `Downloads/Images/` 是否有原图
   - 检查 `Downloads/CBZ/` 是否有 .cbz 文件
   - 用 CBZ 阅读器（如 CDisplayEx、Kavita）打开验证结构正确
3. **删除原图测试**: 启用"打包后删除原图"，验证 Images 目录被清理
4. **错误恢复测试**: 模拟打包失败，验证原图不受影响
