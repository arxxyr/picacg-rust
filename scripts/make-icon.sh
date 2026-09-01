#!/usr/bin/env bash
# 从官方素材生成 PicACG 应用图标
#
# 权威源：assets/icons/picacg-official-192.png
#   = 哔咔漫画官网 PWA 的 https://manhuabika.com/assets/logo_round-*.png
#     （192×192，官方 app 图标，粉底圆角 + 哔咔娘，自带 alpha）
#   已入库，构建与本脚本默认都不依赖网络；官方换图时用 --fetch 重新拉取。
#
# 产物（由权威源放大而来，勿手改）：
#   assets/icons/icon.png      512×512  macOS dock / ⌘Tab（运行时 AppKit 设置）
#   assets/icons/icon-256.png  256×256  Windows 任务栏 / Linux 标题栏
#   assets/icons/icon.icns              macOS .app bundle 图标（Finder / 应用程序 / 启动台）
#
# 为什么 .app 还要单独一份 icns：运行时 AppKit 那套只管**进程活着时**的 dock 图标，
# Finder、「应用程序」列表、启动台读的是 bundle 里的 icns。缺了它，装进
# /Applications 后就是一个白板图标——dmg 拖拽窗口里看到的也是白板。
#
# 依赖 ImageMagick 7（brew install imagemagick）。
# 放大用 Catrom：官方素材只有 192，双三次类滤镜在这种平涂+网点的插画上
# 不会像 Lanczos 那样在网点边缘振铃。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/assets/icons/picacg-official-192.png"
OFFICIAL_PAGE="https://manhuabika.com/"

# --fetch：从官网重新抓取图标素材（走 socks5 代理时设 ALL_PROXY）
if [[ "${1:-}" == "--fetch" ]]; then
    echo "从官网解析图标地址..."
    logo_path="$(curl -fsS "$OFFICIAL_PAGE" \
        | grep -oE '/assets/logo_round-[A-Za-z0-9_-]+\.png' \
        | head -1)"
    if [[ -z "$logo_path" ]]; then
        echo "未能在官网首页找到 logo_round 资源，站点结构可能已变" >&2
        exit 1
    fi
    echo "下载 ${OFFICIAL_PAGE%/}$logo_path"
    curl -fsS "${OFFICIAL_PAGE%/}$logo_path" -o "$SRC"
fi

if [[ ! -f "$SRC" ]]; then
    echo "缺少官方素材 $SRC，先跑 $0 --fetch" >&2
    exit 1
fi

# macOS 图标网格：1024 画布里主体约占 824（≈80.5%），四周留透明边。
# 官方素材是满幅圆角方形，直接铺满画布会在 dock 里显得比别的 app 大一圈
# （实测截图对比明显）。凡 macOS 用途（dock 的 icon.png、bundle 的 icns）
# 都按此比例缩进；Windows/Linux 的 icon-256 保持满幅——那边的惯例就是满的。
MACOS_ICON_SCALE_PERMILLE=805

# 生成 macOS 风格图标：主体缩到画布的 80.5% 居中，四周透明
make_macos_icon() {
    local size="$1" out="$2"
    local inner=$((size * MACOS_ICON_SCALE_PERMILLE / 1000))
    magick "$SRC" -filter Catrom -resize "${inner}x${inner}" \
        -background none -gravity center -extent "${size}x${size}" "$out"
}

make_macos_icon 512 "$ROOT/assets/icons/icon.png"
magick "$SRC" -filter Catrom -resize 256x256 "$ROOT/assets/icons/icon-256.png"

echo "已生成: $ROOT/assets/icons/icon.png"
echo "已生成: $ROOT/assets/icons/icon-256.png"

# ---- macOS .app bundle 图标（icns）----
# iconutil 只有 macOS 有；其他平台跳过，仓库里已入库的 icns 照样可用。
if ! command -v iconutil >/dev/null 2>&1; then
    echo "跳过 icon.icns：当前系统没有 iconutil（仅 macOS 提供）"
    exit 0
fi

ICONSET="$(mktemp -d)/icon.iconset"
mkdir -p "$ICONSET"

# iconutil 认死这套文件名，少一个尺寸就会拒绝整个 iconset
for spec in "16 icon_16x16" "32 icon_16x16@2x" "32 icon_32x32" "64 icon_32x32@2x" \
            "128 icon_128x128" "256 icon_128x128@2x" "256 icon_256x256" \
            "512 icon_256x256@2x" "512 icon_512x512" "1024 icon_512x512@2x"; do
    size="${spec%% *}"
    name="${spec#* }"
    make_macos_icon "$size" "$ICONSET/${name}.png"
done

iconutil -c icns "$ICONSET" -o "$ROOT/assets/icons/icon.icns"
rm -rf "$(dirname "$ICONSET")"

echo "已生成: $ROOT/assets/icons/icon.icns"
