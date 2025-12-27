#!/bin/bash
# 监控工具演示脚本（rdev 事件驱动实现）
# 展示如何使用 iris-mcp 的监控功能

set -e

IRIS_MCP="./target/release/iris-mcp"

if [ ! -f "$IRIS_MCP" ]; then
    echo "错误：找不到 iris-mcp 可执行文件"
    echo "请先运行: cargo build --release"
    exit 1
fi

echo "=== iris-mcp 监控工具演示 (rdev 事件驱动) ==="
echo ""

# 启动服务器并保持运行
echo "🚀 启动 MCP 服务器..."
exec 3< <($IRIS_MCP)
SERVER_PID=$!

# 清理函数
cleanup() {
    echo ""
    echo "🛑 关闭服务器..."
    kill $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# 等待服务器启动
sleep 0.5

# 初始化
echo "1. 初始化服务器..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | $IRIS_MCP 2>/dev/null | tail -1 | jq -r '.result.serverInfo.name + " v" + .result.serverInfo.version'
echo ""

# 屏幕监控（按需截图）
echo "2. 屏幕监控（按需截图）..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"monitor_screen_events","arguments":{}}}' | $IRIS_MCP 2>/dev/null | tail -1)
echo "$RESPONSE" | jq -r '.result.content[0].text'

# 提取事件信息
EVENT_TYPE=$(echo "$RESPONSE" | jq -r '.result.content[1].json.event.kind.type')
WIDTH=$(echo "$RESPONSE" | jq -r '.result.content[1].json.width')
HEIGHT=$(echo "$RESPONSE" | jq -r '.result.content[1].json.height')
IMAGE_SIZE=$(echo "$RESPONSE" | jq -r '.result.content[1].json.image_base64' | wc -c)

echo "   事件类型: $EVENT_TYPE"
echo "   分辨率: ${WIDTH}x${HEIGHT}"
echo "   图像大小: $IMAGE_SIZE 字节 (base64)"
echo ""

# 键盘监控（事件驱动，累积事件）
echo "3. 键盘监控（事件驱动，自动累积）..."
echo "   请在接下来的3秒内按几个键..."
echo ""

# 倒计时
for i in 3 2 1; do
    echo -n "   ⏳ ${i}秒...  "
    sleep 1
    echo -ne "\r"
done
echo ""

RESPONSE=$(echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"monitor_keyboard_events","arguments":{"cursor":0}}}' | $IRIS_MCP 2>/dev/null | tail -1)

if echo "$RESPONSE" | jq -e '.error' > /dev/null; then
    ERROR_MSG=$(echo "$RESPONSE" | jq -r '.error.message')
    echo "   ⚠️  错误: $ERROR_MSG"
    echo ""
    echo "   💡 提示: 需要在系统设置中授予辅助功能权限"
    echo "      系统设置 > 隐私与安全性 > 辅助功能"
else
    EVENT_COUNT=$(echo "$RESPONSE" | jq -r '.result.content[1].json.events | length')
    NEXT_CURSOR=$(echo "$RESPONSE" | jq -r '.result.content[1].json.next_cursor')
    
    echo "   ✅ 获得 $EVENT_COUNT 条键盘事件"
    echo "   next_cursor: $NEXT_CURSOR"
    
    if [ "$EVENT_COUNT" -gt 0 ]; then
        echo ""
        echo "   📋 前5个事件:"
        echo "$RESPONSE" | jq -r '.result.content[1].json.events[:5][] | "      " + .event_type + " " + .key'
    fi
fi
echo ""

# 鼠标监控（事件驱动，累积事件）
echo "4. 鼠标监控（事件驱动，自动累积）..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"monitor_mouse_events","arguments":{"cursor":0}}}' | $IRIS_MCP 2>/dev/null | tail -1)

if echo "$RESPONSE" | jq -e '.error' > /dev/null; then
    ERROR_MSG=$(echo "$RESPONSE" | jq -r '.error.message')
    echo "   ⚠️  错误: $ERROR_MSG"
else
    EVENT_COUNT=$(echo "$RESPONSE" | jq -r '.result.content[1].json.events | length')
    NEXT_CURSOR=$(echo "$RESPONSE" | jq -r '.result.content[1].json.next_cursor')
    
    echo "   ✅ 获得 $EVENT_COUNT 条鼠标事件"
    echo "   next_cursor: $NEXT_CURSOR"
    
    if [ "$EVENT_COUNT" -gt 0 ]; then
        echo ""
        echo "   📋 前5个事件:"
        echo "$RESPONSE" | jq -r '.result.content[1].json.events[:5][] | 
            if .kind.type == "Move" then
                "      移动到 (" + (.kind.x|tostring) + ", " + (.kind.y|tostring) + ")"
            elif .kind.type == "Button" then
                "      " + .kind.state + " " + .kind.button
            else
                "      " + .kind.type
            end'
    fi
fi
echo ""

echo "=== 演示完成 ==="
echo ""
echo "💡 特性说明："
echo "  ✅ 屏幕监控：按需捕获（零后台占用）"
echo "  ✅ 键盘监控：事件驱动（零 CPU 占用）"
echo "  ✅ 鼠标监控：事件驱动（零 CPU 占用）"
echo "  ✅ 增量读取：使用 cursor 参数"
echo "  ✅ 跨平台：基于 rdev（macOS/Windows/Linux）"
echo ""
echo "⚠️  权限要求："
echo "  macOS 需要在 系统设置 > 隐私与安全性 > 辅助功能 中授权"
