#!/usr/bin/env bash
# 生成 macOS dmg 安装窗口的背景图
#
# dmg 里那个「App → Applications」的箭头**是画在背景图上的**，不是控件；
# Finder 只负责把两个图标摆到指定坐标上。所以背景图的尺寸必须与
# create-dmg 的 --window-size 完全一致，箭头位置也要和图标坐标对得上，
# 否则就会歪。三者的约定值集中写在下面，改一处必须同步改 ci.yml。
#
# 产物（已入库，CI 不依赖 ImageMagick）：
#   assets/dmg/background.png     660×400   1x
#   assets/dmg/background@2x.png  1320×800  Retina
#
# 依赖 ImageMagick 7（brew install imagemagick）。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/assets/dmg"
mkdir -p "$OUT"

# ---- 与 ci.yml 的 create-dmg 参数保持一致 ----
W=660; H=400          # --window-size
ARROW_X=330           # 窗口水平中点（两个图标之间）
ARROW_Y=165           # 与图标中心同高（--icon 的 y 值）

BG="#f0f0f4"          # 浅底：dmg 背景图是固定图片，不跟随深色模式
FG="#1d1d1f"          # 箭头色

render() {
    local scale="$1" out="$2"
    local w=$((W * scale)) h=$((H * scale))
    local ax=$((ARROW_X * scale)) ay=$((ARROW_Y * scale))
    local arm=$((22 * scale))     # 箭头单臂长度
    local sw=$((9 * scale))       # 线宽

    magick -size "${w}x${h}" "xc:${BG}" \
        -stroke "$FG" -strokewidth "$sw" -fill none \
        -draw "stroke-linecap round stroke-linejoin round \
               polyline $((ax - arm / 2)),$((ay - arm)) \
                        $((ax + arm / 2)),${ay} \
                        $((ax - arm / 2)),$((ay + arm))" \
        "$out"
}

render 1 "$OUT/background.png"
render 2 "$OUT/background@2x.png"

echo "已生成: $OUT/background.png     (${W}x${H})"
echo "已生成: $OUT/background@2x.png  ($((W * 2))x$((H * 2)))"
