#!/bin/bash
# 跨平台编译脚本 - 为 Windows 和 Linux 生成 release 文件

set -e

echo "🚀 开始跨平台编译..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 目标平台列表
TARGETS=(
    "x86_64-unknown-linux-gnu"      # Linux x86_64
    "x86_64-pc-windows-gnu"          # Windows x86_64
    "aarch64-unknown-linux-gnu"      # Linux ARM64
)

# 检查并安装目标平台
echo -e "${BLUE}检查编译目标...${NC}"
for target in "${TARGETS[@]}"; do
    if rustup target list | grep -q "$target (installed)"; then
        echo -e "  ✓ $target 已安装"
    else
        echo -e "  + 正在安装 $target..."
        rustup target add "$target" || echo -e "  ${RED}✗ 无法安装 $target，跳过${NC}"
    fi
done
echo ""

# 创建输出目录
OUTPUT_DIR="release-builds"
mkdir -p "$OUTPUT_DIR"

# 当前平台（macOS）
echo -e "${BLUE}📦 编译当前平台 (macOS)...${NC}"
cargo build --release
if [ -f "target/release/iris-mcp" ]; then
    cp target/release/iris-mcp "$OUTPUT_DIR/iris-mcp-macos-$(uname -m)"
    echo -e "${GREEN}✓ macOS 版本已生成${NC}"
fi
echo ""

# 交叉编译其他平台
for target in "${TARGETS[@]}"; do
    echo -e "${BLUE}📦 编译 $target...${NC}"
    
    if cargo build --release --target "$target" 2>/dev/null; then
        # 确定可执行文件名和输出名
        if [[ $target == *"windows"* ]]; then
            binary_name="iris-mcp.exe"
            output_name="iris-mcp-${target}.exe"
        else
            binary_name="iris-mcp"
            output_name="iris-mcp-${target}"
        fi
        
        # 复制到输出目录
        if [ -f "target/$target/release/$binary_name" ]; then
            cp "target/$target/release/$binary_name" "$OUTPUT_DIR/$output_name"
            echo -e "${GREEN}✓ $target 编译成功${NC}"
        fi
    else
        echo -e "${RED}✗ $target 编译失败（可能需要安装交叉编译工具）${NC}"
    fi
    echo ""
done

# 显示结果
echo -e "${GREEN}===========================================${NC}"
echo -e "${GREEN}✨ 编译完成！生成的文件：${NC}"
echo -e "${GREEN}===========================================${NC}"
ls -lh "$OUTPUT_DIR/" | tail -n +2 | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo -e "${BLUE}输出目录: $OUTPUT_DIR/${NC}"
