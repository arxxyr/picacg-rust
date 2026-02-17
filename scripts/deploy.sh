#!/usr/bin/env bash
# Bash 部署脚本 - 将编译产物收集到 bin 目录
# 用法: ./scripts/deploy.sh [release|debug]

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

# 目标目录
BIN_DIR="$ROOT_DIR/bin"
BIN_ASSETS_DIR="$BIN_DIR/assets"
BIN_FONTS_DIR="$BIN_ASSETS_DIR/fonts"

# 源目录
TARGET_DIR="$ROOT_DIR/target/$PROFILE"
ASSETS_DIR="$ROOT_DIR/assets"

echo -e "\n${YELLOW}[1/5] 清理旧的 bin 目录...${NC}"
if [ -d "$BIN_DIR" ]; then
    rm -rf "$BIN_DIR"
    echo -e "${GRAY}已删除旧目录: $BIN_DIR${NC}"
fi

echo -e "\n${YELLOW}[2/5] 创建目录结构...${NC}"
mkdir -p "$BIN_DIR"
mkdir -p "$BIN_FONTS_DIR"
echo -e "${GRAY}已创建: bin/${NC}"
echo -e "${GRAY}已创建: bin/assets/fonts/${NC}"

echo -e "\n${YELLOW}[3/5] 复制可执行文件...${NC}"
EXE_NAME="picacg"
# Windows 下可执行文件带 .exe 后缀
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    EXE_NAME="picacg.exe"
fi
EXE_PATH="$TARGET_DIR/$EXE_NAME"
if [ -f "$EXE_PATH" ]; then
    cp "$EXE_PATH" "$BIN_DIR/"
    echo -e "${GREEN}已复制: $EXE_NAME${NC}"
else
    echo -e "${RED}错误: 找不到可执行文件 $EXE_PATH${NC}"
    echo -e "${YELLOW}请先运行: cargo build --${PROFILE} -p picacg${NC}"
    exit 1
fi

echo -e "\n${YELLOW}[4/5] 复制字体文件...${NC}"
if [ -d "$ASSETS_DIR/fonts" ] && [ -r "$ASSETS_DIR/fonts" ]; then
    FONT_COUNT=0
    # 只复制 Regular 字体（运行时必须）
    FONT_FILE="$ASSETS_DIR/fonts/SarasaTermSCNerd-Regular.ttf"
    if [ -f "$FONT_FILE" ]; then
        cp "$FONT_FILE" "$BIN_FONTS_DIR/"
        echo -e "${GREEN}已复制: $(basename "$FONT_FILE")${NC}"
        FONT_COUNT=$((FONT_COUNT + 1))
    fi

    if [ "$FONT_COUNT" -eq 0 ]; then
        echo -e "${YELLOW}警告: 未找到字体文件${NC}"
    else
        echo -e "${GREEN}成功复制 $FONT_COUNT 个字体文件${NC}"
    fi
else
    echo -e "${YELLOW}警告: 字体目录不存在 $ASSETS_DIR/fonts${NC}"
fi

echo -e "\n${CYAN}=== 部署完成 ===${NC}"
echo -e "\n${GREEN}目录结构:${NC}"
echo "bin/"
echo "├── $EXE_NAME"
echo "└── assets/"
echo "    └── fonts/"
echo "        └── SarasaTermSCNerd-Regular.ttf"

echo -e "\n${YELLOW}[5/5] 创建版本压缩包...${NC}"
ZIP_NAME="picacg-v${VERSION}.zip"

# 进入 bin 目录创建 zip（排除 zip 文件本身）
cd "$BIN_DIR"
rm -f picacg-v*.zip
zip -r "$ZIP_NAME" . -x "*.zip"
cd "$ROOT_DIR"

echo -e "${GREEN}已创建: bin/$ZIP_NAME${NC}"

echo -e "\n${GREEN}运行方式:${NC}"
echo "  cd bin"
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    echo "  ./picacg.exe"
else
    echo "  ./picacg"
fi
echo ""
echo -e "${GREEN}分发方式:${NC}"
echo "  将 bin/$ZIP_NAME 发送给用户"
echo ""
