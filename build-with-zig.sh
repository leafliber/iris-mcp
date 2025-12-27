#!/bin/bash
# 使用 cargo-zigbuild 进行跨平台编译（推荐方案）
# 需要先安装: brew install zig && cargo install cargo-zigbuild

set -e

echo "🚀 使用 cargo-zigbuild 进行跨平台编译"
echo ""

# 检查 cargo-zigbuild 是否安装
if ! command -v cargo-zigbuild &> /dev/null; then
    echo "❌ cargo-zigbuild 未安装"
    echo "请运行以下命令安装："
    echo "  brew install zig"
    echo "  cargo install cargo-zigbuild"
    exit 1
fi

# 目标平台
TARGETS=(
    "x86_64-unknown-linux-gnu"      # Linux x86_64
    "x86_64-pc-windows-gnu"          # Windows x86_64
    "aarch64-unknown-linux-gnu"      # Linux ARM64
    "aarch64-apple-darwin"           # macOS ARM64
)

OUTPUT_DIR="release-builds"
mkdir -p "$OUTPUT_DIR"

for target in "${TARGETS[@]}"; do
    echo "📦 编译 $target..."
    
    if cargo zigbuild --release --target "$target"; then
        if [[ $target == *"windows"* ]]; then
            binary="target/$target/release/iris-mcp.exe"
            output="$OUTPUT_DIR/iris-mcp-${target}.exe"
        else
            binary="target/$target/release/iris-mcp"
            output="$OUTPUT_DIR/iris-mcp-${target}"
        fi
        
        if [ -f "$binary" ]; then
            cp "$binary" "$output"
            echo "✓ $target 编译成功"
        fi
    else
        echo "✗ $target 编译失败"
    fi
    echo ""
done

echo "✨ 编译完成！生成的文件："
ls -lh "$OUTPUT_DIR/" | tail -n +2 | awk '{print "  " $9 " (" $5 ")"}'
