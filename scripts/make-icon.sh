#!/usr/bin/env bash
# 从官方素材生成 PicACG 应用图标
#
# 权威源：assets/icons/picacg-official-192.png
#   = 哔咔漫画官网 PWA 的 https://manhuabika.com/assets/logo_round-*.png
#     （192×192，官方 app 图标，粉底圆角 + 哔咔娘，自带 alpha）
#   已入库，构建与本脚本默认都不依赖网络；官方换图时用 --fetch 重新拉取。
#
# 产物（由权威源放大而来，勿手改）：
#   assets/icons/icon.png      512×512  macOS dock / ⌘Tab
#   assets/icons/icon-256.png  256×256  Windows 任务栏 / Linux 标题栏
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

magick "$SRC" -filter Catrom -resize 512x512 "$ROOT/assets/icons/icon.png"
magick "$SRC" -filter Catrom -resize 256x256 "$ROOT/assets/icons/icon-256.png"

echo "已生成: $ROOT/assets/icons/icon.png"
echo "已生成: $ROOT/assets/icons/icon-256.png"
