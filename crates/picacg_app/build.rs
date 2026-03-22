//! 编译时提取依赖版本信息，暴露为环境变量供 env!() 使用

use std::{fs, path::Path};

fn main() {
    // CARGO_MANIFEST_DIR = crates/picacg_app/，往上两层是 workspace 根目录
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    let lock_path = workspace_root.join("Cargo.lock");

    let bevy_version = fs::read_to_string(&lock_path)
        .ok()
        .and_then(|content| {
            // 查找 name = "bevy"\nversion = "x.y.z" 模式
            let mut lines = content.lines();
            while let Some(line) = lines.next() {
                if line.trim() == r#"name = "bevy""#
                    && let Some(ver_line) = lines.next()
                {
                    let ver_line = ver_line.trim();
                    if let Some(ver) = ver_line.strip_prefix("version = \"") {
                        return ver.strip_suffix('"').map(String::from);
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=BEVY_VERSION={bevy_version}");
    println!("cargo:rerun-if-changed={}", lock_path.display());
}
