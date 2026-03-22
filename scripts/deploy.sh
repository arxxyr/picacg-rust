#!/usr/bin/env bash
# Bash 部署脚本 - 将编译产物收集到 bin 目录
# 用法: ./scripts/deploy.sh [release|debug]
# macOS: 打包为 .app Bundle + .dmg（双击启动无终端窗口）
# Linux/Windows: 打包为可执行文件 + 资源目录

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# 默认编译配置
PROFILE="${1:-release}"

echo -e "${CYAN}=== PicACG 部署脚本 ===${NC}"
echo -e "${GREEN}编译配置: $PROFILE${NC}"

# 项目根目录
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 从 Cargo.toml 提取版本号
VERSION=$(grep -A5 '^\[workspace\.package\]' "$ROOT_DIR/Cargo.toml" | grep '^version' | sed 's/.*= *"\([^"]*\)".*/\1/')
if [ -z "$VERSION" ]; then
    echo -e "${RED}错误: 无法从 Cargo.toml 提取版本号${NC}"
    exit 1
fi
echo -e "${GREEN}版本号: v$VERSION${NC}"

cd "$ROOT_DIR"

# 平台检测
EXE_NAME="picacg"
IS_MACOS=false
IS_WINDOWS=false

if [[ "$OSTYPE" == "darwin"* ]]; then
    IS_MACOS=true
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    IS_WINDOWS=true
    EXE_NAME="picacg.exe"
fi

# 目标目录
BIN_DIR="$ROOT_DIR/bin"

# 源目录
TARGET_DIR="$ROOT_DIR/target/$PROFILE"

echo -e "\n${YELLOW}[1/4] 清理旧的 bin 目录...${NC}"
if [ -d "$BIN_DIR" ]; then
    rm -rf "$BIN_DIR"
    echo -e "${GRAY}已删除旧目录: $BIN_DIR${NC}"
fi

echo -e "\n${YELLOW}[2/4] 复制可执行文件...${NC}"
mkdir -p "$BIN_DIR"
EXE_PATH="$TARGET_DIR/$EXE_NAME"
if [ ! -f "$EXE_PATH" ]; then
    echo -e "${RED}错误: 找不到可执行文件 $EXE_PATH${NC}"
    echo -e "${YELLOW}请先运行: cargo build --${PROFILE} -p picacg${NC}"
    exit 1
fi

# 字体路径
FONT_SRC="$ROOT_DIR/assets/fonts/SarasaTermSCNerd/SarasaTermSCNerd-Regular.ttf"

if $IS_MACOS; then
    # ========== macOS: 创建 .app Bundle + .dmg ==========
    APP_NAME="PicACG.app"
    APP_DIR="$BIN_DIR/$APP_NAME"

    # 创建 .app 目录结构
    mkdir -p "$APP_DIR/Contents/MacOS"
    mkdir -p "$APP_DIR/Contents/Resources"

    # 复制可执行文件
    cp "$EXE_PATH" "$APP_DIR/Contents/MacOS/picacg"
    echo -e "${GREEN}已复制: picacg → $APP_NAME/Contents/MacOS/${NC}"

    # 复制内置字体到 Resources
    echo -e "\n${YELLOW}[3/4] 复制内置字体...${NC}"
    if [ -f "$FONT_SRC" ]; then
        FONT_DEST_DIR="$APP_DIR/Contents/Resources/assets/fonts/SarasaTermSCNerd"
        mkdir -p "$FONT_DEST_DIR"
        cp "$FONT_SRC" "$FONT_DEST_DIR/"
        FONT_SIZE=$(du -h "$FONT_SRC" | cut -f1)
        echo -e "${GREEN}已复制: SarasaTermSCNerd-Regular.ttf ($FONT_SIZE)${NC}"
    else
        echo -e "${YELLOW}警告: 未找到内置字体 $FONT_SRC${NC}"
        echo -e "${GRAY}程序将回退到系统字体${NC}"
    fi

    # 生成 Info.plist
    cat > "$APP_DIR/Contents/Info.plist" << PLIST_EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>PicACG</string>
    <key>CFBundleDisplayName</key>
    <string>PicACG</string>
    <key>CFBundleIdentifier</key>
    <string>com.picacg</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>picacg</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
</dict>
</plist>
PLIST_EOF
    echo -e "${GREEN}已生成: Info.plist (v$VERSION)${NC}"

    # 同时保留裸二进制（方便终端调试）
    cp "$EXE_PATH" "$BIN_DIR/"

    echo -e "\n${YELLOW}[4/4] 创建 DMG 镜像...${NC}"
    DMG_NAME="picacg-v${VERSION}.dmg"
    DMG_PATH="$BIN_DIR/$DMG_NAME"

    # 创建临时目录作为 DMG 内容
    DMG_STAGING="$BIN_DIR/.dmg_staging"
    mkdir -p "$DMG_STAGING"
    cp -R "$APP_DIR" "$DMG_STAGING/"

    # 创建指向 /Applications 的符号链接（方便用户拖拽安装）
    ln -s /Applications "$DMG_STAGING/Applications"

    # 生成 DMG
    rm -f "$DMG_PATH"
    hdiutil create \
        -volname "PicACG v${VERSION}" \
        -srcfolder "$DMG_STAGING" \
        -ov \
        -format UDZO \
        "$DMG_PATH"

    # 清理临时目录
    rm -rf "$DMG_STAGING"

    echo -e "${GREEN}已创建: bin/$DMG_NAME${NC}"

    echo -e "\n${CYAN}=== 部署完成 ===${NC}"
    echo -e "\n${GREEN}目录结构:${NC}"
    echo "bin/"
    echo "├── picacg                          (终端调试用)"
    echo "├── $DMG_NAME              (分发用)"
    echo "└── PicACG.app/"
    echo "    └── Contents/"
    echo "        ├── Info.plist"
    echo "        ├── MacOS/"
    echo "        │   └── picacg"
    echo "        └── Resources/"
    echo "            └── assets/fonts/SarasaTermSCNerd/"
    echo "                └── SarasaTermSCNerd-Regular.ttf"
    echo ""

    echo -e "${GREEN}运行方式:${NC}"
    echo "  open bin/PicACG.app"
    echo "  # 或终端调试: bin/picacg"
    echo ""
    echo -e "${GREEN}分发方式:${NC}"
    echo "  将 bin/$DMG_NAME 发送给用户"
    echo "  用户双击 DMG → 拖拽 PicACG.app 到 Applications 文件夹"
    echo ""
else
    # ========== Linux / Windows: 裸二进制 + 资源目录 ==========
    cp "$EXE_PATH" "$BIN_DIR/"
    echo -e "${GREEN}已复制: $EXE_NAME${NC}"

    echo -e "\n${YELLOW}[3/4] 复制内置字体...${NC}"
    FONT_DEST_DIR="$BIN_DIR/assets/fonts/SarasaTermSCNerd"
    if [ -f "$FONT_SRC" ]; then
        mkdir -p "$FONT_DEST_DIR"
        cp "$FONT_SRC" "$FONT_DEST_DIR/"
        FONT_SIZE=$(du -h "$FONT_SRC" | cut -f1)
        echo -e "${GREEN}已复制: SarasaTermSCNerd-Regular.ttf ($FONT_SIZE)${NC}"
    else
        echo -e "${YELLOW}警告: 未找到内置字体 $FONT_SRC${NC}"
        echo -e "${GRAY}程序将回退到系统字体${NC}"
    fi

    echo -e "\n${CYAN}=== 部署完成 ===${NC}"
    echo -e "\n${GREEN}目录结构:${NC}"
    echo "bin/"
    echo "├── $EXE_NAME"
    echo "└── assets/"
    echo "    └── fonts/"
    echo "        └── SarasaTermSCNerd/"
    echo "            └── SarasaTermSCNerd-Regular.ttf"
    echo ""

    echo -e "\n${YELLOW}[4/4] 创建版本压缩包...${NC}"
    ZIP_NAME="picacg-v${VERSION}.zip"
    cd "$BIN_DIR"
    rm -f picacg-v*.zip
    zip -r "$ZIP_NAME" . -x "*.zip"
    cd "$ROOT_DIR"
    echo -e "${GREEN}已创建: bin/$ZIP_NAME${NC}"

    echo -e "\n${GREEN}运行方式:${NC}"
    echo "  cd bin"
    if $IS_WINDOWS; then
        echo "  ./picacg.exe"
    else
        echo "  ./picacg"
    fi
    echo ""
    echo -e "${GREEN}分发方式:${NC}"
    echo "  将 bin/$ZIP_NAME 发送给用户"
    echo ""
fi
