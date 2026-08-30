//! 自动更新
//!
//! 三平台走三条路，取决于各自的安全模型：
//!
//! | 平台 | 产物 | 策略 | 为什么 |
//! |------|------|------|--------|
//! | Linux | `tar.gz` 里的裸可执行文件 | **原地自替换** | 无 Gatekeeper / SmartScreen，换文件即可 |
//! | macOS | `.dmg`（内含 `.app`） | **下载 + 校验 + `open`** | 挂载后弹出拖拽安装窗口，由用户拖进 Applications |
//! | Windows | `.zip` | **下载 + 校验 + 打开所在目录** | 无 Authenticode 签名，自动替换极易被 Defender 拦 |
//!
//! macOS / Windows 不做原地替换的原因不是懒：`.app` bundle 被就地改写会破坏
//! 完整性，Gatekeeper 直接判「应用已损坏」；Windows 上自动下载并执行的无签名
//! exe 是杀软的典型画像。这两条路的真正门槛是**代码签名证书**，不是代码。
//!
//! ## 安全前提
//!
//! 三条路都**强制校验 SHA-256**（CI 的 `Generate checksums` 步骤产出
//! `*.sha256`）。 校验和缺失或对不上一律中止——宁可不更新，
//! 也不能把损坏或被篡改的产物交给用户。

#[cfg(not(target_os = "linux"))]
use std::path::PathBuf;

/// 更新流程的产出
///
/// 变体按平台 cfg 门控：CI 带 `-D warnings`，留一个当前平台永远构造不出来的
/// 变体会直接把该平台的构建挂掉。
pub enum UpdateOutcome {
    /// 已就地替换，重启生效（Linux）
    #[cfg(target_os = "linux")]
    Replaced { version: String },
    /// 已下载并交给系统打开，等用户手动完成安装（macOS / Windows）
    #[cfg(not(target_os = "linux"))]
    Downloaded { path: PathBuf },
}

impl UpdateOutcome {
    /// 给用户看的一句话结果
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            #[cfg(target_os = "linux")]
            Self::Replaced { version } => format!("已更新到 v{version}，重启后生效"),
            #[cfg(not(target_os = "linux"))]
            Self::Downloaded { path } => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                if cfg!(target_os = "macos") {
                    format!("已下载 {name}，请在弹出的窗口中拖入「应用程序」")
                } else {
                    format!("已下载 {name}，请解压后覆盖当前程序")
                }
            }
        }
    }
}

/// 本平台的更新按钮该叫什么
#[must_use]
pub fn action_label() -> &'static str {
    if cfg!(target_os = "linux") {
        "立即更新"
    } else {
        "下载并打开"
    }
}

/// 执行更新（阻塞，调用方须放到后台线程）
///
/// # Errors
///
/// 网络失败、校验和缺失或不匹配、替换/打开失败时返回错误描述。
pub fn run_update(
    asset_url: Option<&str>,
    checksum_url: Option<&str>,
) -> Result<UpdateOutcome, String> {
    let asset_url = asset_url.ok_or_else(|| "该版本没有提供本平台的安装包".to_string())?;
    let checksum_url = checksum_url
        .ok_or_else(|| "该版本缺少 SHA-256 校验和，出于安全考虑不自动更新".to_string())?;

    let (bytes, file_name) = download(asset_url)?;
    verify_checksum(&bytes, checksum_url)?;

    install(bytes, &file_name)
}

/// 下载产物，返回 (内容, 文件名)
fn download(url: &str) -> Result<(Vec<u8>, String), String> {
    let file_name = url
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("picacg-update")
        .to_string();

    let response = http_get(url)?;
    Ok((response, file_name))
}

/// 取 URL 内容（阻塞）
///
/// 复用 reqwest 的阻塞客户端——项目已依赖它，不必为此再引一个 HTTP 库。
///
/// **走应用的代理设置**：产物托管在 GitHub，部分地区不可直连；
/// 检查更新能走代理而下载不走，就会出现「提示有新版本但永远下不下来」。
fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let proxy_url = picacg_config::AppSettings::global()
        .read()
        .proxy
        .to_proxy_url();

    let mut builder = reqwest::blocking::Client::builder()
        .user_agent("picacg-rust")
        .timeout(std::time::Duration::from_secs(300));
    if let Some(ref url) = proxy_url {
        let proxy = reqwest::Proxy::all(url).map_err(|e| format!("代理配置无效: {e}"))?;
        builder = builder.proxy(proxy);
    }
    let client = builder
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("下载失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| format!("读取下载内容失败: {e}"))
}

/// 校验 SHA-256
///
/// `.sha256` 的格式是 `sha256sum` / `shasum -a 256` 的两列输出，取第一列。
fn verify_checksum(bytes: &[u8], checksum_url: &str) -> Result<(), String> {
    use sha2::{Digest, Sha256};

    let raw = http_get(checksum_url)?;
    let text = String::from_utf8_lossy(&raw);
    let expected = text
        .split_whitespace()
        .next()
        .ok_or_else(|| "校验和文件为空".to_string())?
        .to_ascii_lowercase();

    let actual = hex::encode(Sha256::digest(bytes));
    if actual != expected {
        return Err(format!(
            "校验和不匹配，已中止更新（期望 {}…，实际 {}…）",
            &expected[..expected.len().min(12)],
            &actual[..actual.len().min(12)]
        ));
    }
    Ok(())
}

/// 落盘到下载目录
#[cfg(not(target_os = "linux"))]
fn write_to_downloads(bytes: &[u8], file_name: &str) -> Result<PathBuf, String> {
    let dir = directories::UserDirs::new()
        .and_then(|d| d.download_dir().map(std::path::Path::to_path_buf))
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建下载目录失败: {e}"))?;

    let path = dir.join(file_name);
    std::fs::write(&path, bytes).map_err(|e| format!("写入 {} 失败: {e}", path.display()))?;
    Ok(path)
}

/// 从 tar.gz 里取出指定名字的文件
///
/// 平台无关 + 有单测：这段逻辑本身与 Linux 无关，但只有 Linux 分支用得到；
/// 若写在 `#[cfg(target_os = "linux")]` 里，写错了要等 CI 的 Linux 构建才知道。
///
/// 非 Linux 的**非测试**构建里它确实没有调用方，但不能因此 cfg 掉——那样就回到了
/// 「只有 Linux 编译这段」的老问题。单测在所有平台都跑得到。
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn extract_from_targz(bytes: &[u8], want: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;

    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive
        .entries()
        .map_err(|e| format!("读取压缩包失败: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("读取压缩包条目失败: {e}"))?;
        let is_match = entry
            .path()
            .map_err(|e| format!("解析条目路径失败: {e}"))?
            .file_name()
            .is_some_and(|n| n == want);
        if is_match {
            let mut out = Vec::new();
            entry
                .read_to_end(&mut out)
                .map_err(|e| format!("解出 {want} 失败: {e}"))?;
            return Ok(out);
        }
    }

    Err(format!("压缩包里没有找到 {want}"))
}

/// Linux：解出 tar.gz 里的可执行文件并原地替换自身
#[cfg(target_os = "linux")]
fn install(bytes: Vec<u8>, _file_name: &str) -> Result<UpdateOutcome, String> {
    // tar.gz 里是 `./picacg` 加资源目录，只取可执行文件
    let binary = extract_from_targz(&bytes, "picacg")?;

    // 先落到自身同目录的临时文件，再原子替换——跨文件系统 rename 会失败，
    // 同目录能保证在同一个卷上
    let current = std::env::current_exe().map_err(|e| format!("定位自身失败: {e}"))?;
    let staged = current.with_extension("new");
    std::fs::write(&staged, &binary).map_err(|e| format!("写入新版本失败: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置可执行权限失败: {e}"))?;
    }

    self_replace::self_replace(&staged).map_err(|e| format!("替换自身失败: {e}"))?;
    let _ = std::fs::remove_file(&staged);

    Ok(UpdateOutcome::Replaced {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// macOS / Windows：落盘后交给系统打开
///
/// macOS 上 `open` 一个 `.dmg` 会挂载并弹出拖拽安装窗口，用户拖进「应用程序」
/// 即完成升级——绕开了自替换破坏 `.app` 签名的问题。
/// Windows 上打开 `.zip` 所在位置，由用户解压覆盖。
#[cfg(not(target_os = "linux"))]
fn install(bytes: Vec<u8>, file_name: &str) -> Result<UpdateOutcome, String> {
    let path = write_to_downloads(&bytes, file_name)?;
    open::that(&path).map_err(|e| format!("打开 {} 失败: {e}", path.display()))?;
    Ok(UpdateOutcome::Downloaded { path })
}

#[cfg(test)]
mod tests {
    use super::extract_from_targz;

    /// 造一个内存里的 tar.gz，验证能按文件名取出内容
    fn make_targz(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;

        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            for (name, data) in entries {
                let mut header = tar::Header::new_gnu();
                header.set_size(data.len() as u64);
                header.set_mode(0o755);
                header.set_cksum();
                builder.append_data(&mut header, name, *data).unwrap();
            }
            builder.finish().unwrap();
        }

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn extracts_named_entry() {
        let gz = make_targz(&[("./picacg", b"BINARY"), ("./VERSION", b"v0.5.0")]);
        assert_eq!(extract_from_targz(&gz, "picacg").unwrap(), b"BINARY");
        assert_eq!(extract_from_targz(&gz, "VERSION").unwrap(), b"v0.5.0");
    }

    /// 产物里可执行文件带 `./` 前缀，按 file_name 匹配才找得到
    #[test]
    fn matches_by_file_name_not_full_path() {
        let gz = make_targz(&[("./assets/fonts/x.ttf", b"FONT"), ("./picacg", b"BIN")]);
        assert_eq!(extract_from_targz(&gz, "picacg").unwrap(), b"BIN");
    }

    #[test]
    fn reports_missing_entry() {
        let gz = make_targz(&[("./VERSION", b"v0.5.0")]);
        let err = extract_from_targz(&gz, "picacg").unwrap_err();
        assert!(err.contains("picacg"), "错误信息应指出缺什么: {err}");
    }
}
