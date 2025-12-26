# iris-mcp

面向 LLM 的模型上下文协议 (MCP) 输入控制服务器，基于 Rust + Enigo，提供鼠标与键盘控制工具。

## 特点
- 纯 JSON-RPC 2.0 + MCP 工具接口，标准 stdin/stdout 传输，易于嵌入任意 MCP 客户端。
- 覆盖常用鼠标/键盘操作：移动、点击、滚动、拖拽、路径移动、按键控制、系统快捷键、文本输入、位置获取。
- 零额外运行时依赖（Enigo 直接驱动系统输入）。
- 提供集成测试，启动二进制后验证 `initialize` / `tools/list` / `tools/call` 基本流程。

## 安装与构建
```bash
# 构建发布版
cargo build --release

# 运行集成测试（含服务器启动与调用验证）
cargo test --release --tests server_starts_and_handles_basic_requests
```

生成的可执行文件：`target/release/iris-mcp`

## 运行
```bash
./target/release/iris-mcp
```
进程通过 stdin/stdout 交互，stderr 输出日志。

## MCP 客户端配置示例（Claude Desktop）
在 Claude 配置中添加：
```json
{
  "mcpServers": {
    "iris-mcp": {
      "command": "/Users/cassia/Code/iris-mcp/target/release/iris-mcp"
    }
  }
}
```

## 已暴露的工具
- mouse_move
- mouse_click
- mouse_double_click
- mouse_scroll
- mouse_get_position
- mouse_drag
- mouse_button_control
- mouse_move_path
- type_text
- key_control
- system_command

## 手工调用示例（JSON-RPC 逐行）
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"mouse_move","arguments":{"x":100,"y":200}}}
```

## 注意事项
- 必须使用 `"jsonrpc": "2.0"`，否则服务器返回 `-32600`。
- 工具参数缺失或非法时返回 `-32602`，其他内部错误返回 `-32603`。
- 需要在具备图形环境与输入权限的系统上运行（macOS 已验证）。
