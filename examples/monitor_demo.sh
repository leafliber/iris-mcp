#!/bin/bash
# 监控工具演示脚本
# 展示如何使用 iris-mcp 的监控功能

set -e

IRIS_MCP="./target/release/iris-mcp"

if [ ! -f "$IRIS_MCP" ]; then
    echo "错误：找不到 iris-mcp 可执行文件"
    echo "请先运行: cargo build --release"
    exit 1
fi

echo "=== iris-mcp 监控工具演示 ==="
echo ""

# 初始化
echo "1. 初始化 MCP 服务器..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | $IRIS_MCP 2>/dev/null | tail -1 | jq -r '.result.serverInfo.name'
echo ""

# 获取屏幕监控事件
echo "2. 获取屏幕监控事件（首次获取，cursor=0）..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"monitor_screen_events","arguments":{"cursor":0}}}' | $IRIS_MCP 2>/dev/null | tail -1)
echo "$RESPONSE" | jq -r '.result.content[0].text'
NEXT_CURSOR=$(echo "$RESPONSE" | jq -r '.result.content[1].json.next_cursor')
echo "下一个游标: $NEXT_CURSOR"
echo ""

# 模拟一段时间后再次获取
echo "3. 等待 2 秒后增量获取新事件..."
sleep 2
RESPONSE=$(echo "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"monitor_screen_events\",\"arguments\":{\"cursor\":$NEXT_CURSOR}}}" | $IRIS_MCP 2>/dev/null | tail -1)
echo "$RESPONSE" | jq -r '.result.content[0].text'
echo ""

# 尝试键盘监控（预期返回 NotImplemented）
echo "4. 尝试启动键盘监控（当前为存根实现）..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"monitor_keyboard_events","arguments":{"cursor":0}}}' | $IRIS_MCP 2>/dev/null | tail -1)
if echo "$RESPONSE" | jq -e '.error' > /dev/null; then
    echo "预期错误: $(echo "$RESPONSE" | jq -r '.error.message')"
else
    echo "成功: $(echo "$RESPONSE" | jq -r '.result.content[0].text')"
fi
echo ""

echo "=== 演示完成 ==="
echo ""
echo "提示："
echo "  - 屏幕监控在 macOS 上完全可用"
echo "  - 键盘和鼠标监控需要完成平台实现"
echo "  - 使用 cursor 参数实现增量读取"
