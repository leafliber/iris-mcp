# iris-mcp

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)

面向 LLM 的模型上下文协议 (MCP) 输入控制服务器，基于 Rust + Enigo，提供鼠标、键盘控制与监控工具。

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP 客户端 (Claude Desktop)                │
│                                                               │
│  LLM 通过 MCP 协议调用工具执行输入操作或获取监控事件              │
└───────────────────────┬─────────────────────────────────────┘
                        │ JSON-RPC 2.0 over stdin/stdout
                        ↓
┌─────────────────────────────────────────────────────────────┐
│                     iris-mcp 服务器                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              MCP 工具接口层 (server.rs)               │   │
│  │  - 工具注册与调用分发                                   │   │
│  │  - 参数验证与错误处理                                   │   │
│  │  - 监控状态管理 (OnceLock + Mutex)                     │   │
│  └────────────┬──────────────────────┬───────────────────┘   │
│               ↓                      ↓                       │
│  ┌─────────────────────┐  ┌──────────────────────────────┐  │
│  │   输入操作模块        │  │       监控模块                │  │
│  │  ┌────────────────┐ │  │  ┌────────────────────────┐ │  │
│  │  │ operator_      │ │  │  │ monitor_screen         │ │  │
│  │  │  - keyboard    │ │  │  │  (macOS: CGDisplay)   │ │  │
│  │  │  - mouse       │ │  │  ├────────────────────────┤ │  │
│  │  │                │ │  │  │ monitor_keyboard       │ │  │
│  │  │  基于 Enigo     │ │  │  │  (存根: 各平台待实现)   │ │  │
│  │  └────────────────┘ │  │  ├────────────────────────┤ │  │
│  └─────────────────────┘  │  │ monitor_mouse          │ │  │
│                            │  │  (存根: 各平台待实现)   │ │  │
│                            │  └────────────────────────┘ │  │
│                            └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│                     操作系统 API                              │
│  macOS: Accessibility API, CGEvent, CGDisplayStream          │
│  Windows: SendInput, Windows Hooks (计划)                    │
│  Linux: evdev, X11 (计划)                                    │
└─────────────────────────────────────────────────────────────┘
```

## 特点
- 纯 JSON-RPC 2.0 + MCP 工具接口，标准 stdin/stdout 传输，易于嵌入任意 MCP 客户端。
- **输入操作**：覆盖常用鼠标/键盘操作：移动、点击、滚动、拖拽、路径移动、按键控制、系统快捷键、文本输入、位置获取。
- **输入监控**：支持实时监控屏幕、键盘、鼠标事件，基于游标的增量读取机制，适合 LLM 持续观察用户操作。
- 零额外运行时依赖（Enigo 直接驱动系统输入，原生 API 实现监控）。
- 提供集成测试，启动二进制后验证 `initialize` / `tools/list` / `tools/call` 基本流程。

## 安装与构建
```bash
# 克隆仓库
git clone https://github.com/yourusername/iris-mcp.git
cd iris-mcp

# 构建发布版
cargo build --release

# 运行集成测试（含服务器启动与调用验证）
cargo test --release --tests server_starts_and_handles_basic_requests
```

生成的可执行文件：`target/release/iris-mcp`

## 快速开始

### 1. 命令行测试
```bash
# 启动服务器
./target/release/iris-mcp

# 在另一个终端中测试
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/iris-mcp
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | ./target/release/iris-mcp
```

### 2. 与 Claude Desktop 集成
编辑 Claude Desktop 配置文件：
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

添加以下配置：
```json
{
  "mcpServers": {
    "iris-mcp": {
      "command": "/path/to/iris-mcp/target/release/iris-mcp"
    }
  }
}
```

重启 Claude Desktop，在对话中即可使用输入控制功能。

### 3. 使用示例对话
```
你: "请移动鼠标到屏幕中央并点击"
Claude: [调用 mouse_move 和 mouse_click]

你: "帮我在记事本里输入 Hello World"
Claude: [调用 type_text]

你: "监控我的键盘操作"
Claude: [调用 monitor_keyboard_events 并定期轮询]
```

### 4. 编程示例
项目提供了完整的示例代码：
- **Shell 脚本**: [`examples/monitor_demo.sh`](examples/monitor_demo.sh) - 演示监控工具的使用
- **Python 客户端**: [`examples/client_example.py`](examples/client_example.py) - 完整的 Python 封装和示例

运行示例：
```bash
# Shell 演示
./examples/monitor_demo.sh

# Python 演示
python3 examples/client_example.py
```

## 运行
```bash
./target/release/iris-mcp
```
进程通过 stdin/stdout 交互，stderr 输出日志。

**日志示例**：
```
Iris MCP Server 启动中...
Received: {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
Sending: {"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05",...}}
```

## 已暴露的工具

📖 **完整工具速查表**: [`TOOL_REFERENCE.md`](TOOL_REFERENCE.md) - 包含所有工具的详细参数、示例和事件格式

### 输入操作工具
| 工具名称 | 功能描述 | 主要参数 |
|---------|---------|---------|
| `mouse_move` | 移动鼠标到指定坐标 | `x`, `y` |
| `mouse_click` | 在指定坐标点击鼠标按钮 | `x`, `y`, `button` (left/right/middle) |
| `mouse_double_click` | 在指定坐标双击鼠标按钮 | `x`, `y`, `button` |
| `mouse_scroll` | 滚动鼠标滚轮 | `lines_x`, `lines_y` |
| `mouse_get_position` | 获取当前鼠标位置 | 无 |
| `mouse_drag` | 拖拽鼠标从当前位置到目标位置 | `target_x`, `target_y`, `button` |
| `mouse_button_control` | 控制鼠标按钮按下或释放 | `button`, `direction` (press/release/click) |
| `mouse_move_path` | 按指定路径移动鼠标 | `points` (数组), `speed_ms` |
| `type_text` | 使用键盘输入文本 | `text` |
| `key_control` | 控制键盘按键按下或释放 | `key`, `direction` (press/release/click) |
| `system_command` | 执行系统命令快捷键 | `command` (copy/paste/cut/undo/save/select_all) |

### 监控工具
| 工具名称 | 功能描述 | 主要参数 | 返回值 |
|---------|---------|---------|--------|
| `monitor_screen_events` | 获取屏幕监控事件（自动启动监控） | `cursor` (可选，默认0) | 事件数组 + next_cursor |
| `monitor_keyboard_events` | 获取键盘监控事件（自动启动监控） | `cursor` (可选，默认0) | 事件数组 + next_cursor |
| `monitor_mouse_events` | 获取鼠标监控事件（自动启动监控） | `cursor` (可选，默认0) | 事件数组 + next_cursor |

## 手工调用示例（JSON-RPC 逐行）

### 基础初始化与工具列表
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```

### 输入操作示例
```json
// 移动鼠标
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"mouse_move","arguments":{"x":100,"y":200}}}

// 点击左键
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"mouse_click","arguments":{"x":100,"y":200,"button":"left"}}}

// 输入文本
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"type_text","arguments":{"text":"Hello World"}}}

// 按键控制（按下并释放回车键）
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"key_control","arguments":{"key":"return","direction":"click"}}}

// 执行系统命令（复制）
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"system_command","arguments":{"command":"copy"}}}
```

### 监控工具使用示例

#### 屏幕监控
```json
// 首次获取所有事件
{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"monitor_screen_events","arguments":{"cursor":0}}}

// 响应示例
{
  "jsonrpc":"2.0",
  "id":8,
  "result":{
    "content":[
      {
        "type":"text",
        "text":"返回2条屏幕事件，next_cursor=2 (total=2)"
      },
      {
        "type":"json",
        "json":{
          "events":[
            {
              "timestamp_micros":1234567890,
              "kind":{"type":"geometry_changed","width":1920,"height":1080,"scale":2.0}
            },
            {
              "timestamp_micros":1234567891,
              "kind":{"type":"frame_captured","width":1920,"height":1080,"format":"BGRA8"}
            }
          ],
          "next_cursor":2
        }
      }
    ]
  }
}

// 增量获取新事件（使用上次返回的 next_cursor）
{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"monitor_screen_events","arguments":{"cursor":2}}}
```

#### 键盘监控
```json
// 获取键盘事件
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"monitor_keyboard_events","arguments":{"cursor":0}}}

// 响应示例（包含按键事件）
{
  "jsonrpc":"2.0",
  "id":10,
  "result":{
    "content":[
      {
        "type":"text",
        "text":"返回3条键盘事件，next_cursor=3 (total=3)"
      },
      {
        "type":"json",
        "json":{
          "events":[
            {
              "timestamp_micros":1234567900,
              "code":{"type":"char","value":"a"},
              "state":"press"
            },
            {
              "timestamp_micros":1234567901,
              "code":{"type":"char","value":"a"},
              "state":"release"
            },
            {
              "timestamp_micros":1234567902,
              "code":{"type":"named","value":"shift"},
              "state":"press"
            }
          ],
          "next_cursor":3
        }
      }
    ]
  }
}
```

#### 鼠标监控
```json
// 获取鼠标事件
{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"monitor_mouse_events","arguments":{"cursor":0}}}

// 响应示例（包含移动、点击、滚动事件）
{
  "jsonrpc":"2.0",
  "id":11,
  "result":{
    "content":[
      {
        "type":"text",
        "text":"返回4条鼠标事件，next_cursor=4 (total=4)"
      },
      {
        "type":"json",
        "json":{
          "events":[
            {
              "timestamp_micros":1234567910,
              "kind":{"type":"move","x":500,"y":300}
            },
            {
              "timestamp_micros":1234567911,
              "kind":{"type":"button","button":"left","state":"press"}
            },
            {
              "timestamp_micros":1234567912,
              "kind":{"type":"button","button":"left","state":"release"}
            },
            {
              "timestamp_micros":1234567913,
              "kind":{"type":"scroll","delta_x":0,"delta_y":-3}
            }
          ],
          "next_cursor":4
        }
      }
    ]
  }
}
```

### 监控工具的增量读取模式
监控工具使用游标（cursor）机制实现增量读取：
1. **首次调用**：`cursor=0`，返回所有已捕获的事件
2. **后续调用**：使用上次返回的 `next_cursor`，只获取新增事件
3. **无新事件**：返回空数组，`next_cursor` 保持不变
4. **自动启动**：首次调用任何监控工具时，自动启动对应的监控器

这种机制使得 LLM 可以：
- 定期轮询获取最新事件
- 避免重复处理已读事件
- 实现实时观察用户操作的能力

## 注意事项

### 通用要求
- 必须使用 `"jsonrpc": "2.0"`，否则服务器返回 `-32600`。
- 工具参数缺失或非法时返回 `-32602`，其他内部错误返回 `-32603`。
- 需要在具备图形环境与输入权限的系统上运行（macOS 已验证）。

### 监控功能的平台支持
| 功能 | macOS | Windows | Linux | 说明 |
|-----|-------|---------|-------|------|
| 屏幕监控 | ✅ 已实现 | ❌ 未实现 | ❌ 未实现 | macOS 使用 CGDisplayStream API |
| 键盘监控 | ⚠️ 存根 | ⚠️ 存根 | ⚠️ 存根 | 返回 NotImplemented 错误 |
| 鼠标监控 | ⚠️ 存根 | ⚠️ 存根 | ⚠️ 存根 | 返回 NotImplemented 错误 |

**当前状态**：
- 屏幕监控在 macOS 上完全可用，可捕获屏幕几何变化和帧截图事件
- 键盘和鼠标监控的架构已就绪，但各平台的具体实现尚未完成
- 调用未实现的监控器时，会返回详细的错误信息（错误码 -32002/-32003）

### 权限要求
- **macOS**：需要授予"辅助功能"权限（系统偏好设置 → 安全性与隐私 → 辅助功能）
- **Windows**（计划）：需要管理员权限或输入设备访问权限
- **Linux**（计划）：需要 evdev 访问权限或 X11 显示服务器权限

## 使用场景

### 1. LLM 自动化操作
```
用户: "请帮我打开计算器并输入 2+3"
LLM: 
  1. mouse_move 到 Spotlight 位置
  2. mouse_click 打开 Spotlight
  3. type_text "Calculator"
  4. key_control "return" 打开应用
  5. type_text "2+3"
  6. key_control "return" 执行计算
```

### 2. LLM 观察与响应
```
用户: "监控我的操作，如果我点击了保存按钮，提醒我填写备注"
LLM:
  1. 调用 monitor_screen_events 和 monitor_mouse_events
  2. 定期轮询新事件
  3. 检测到鼠标点击特定区域时
  4. 检查屏幕截图确认是保存按钮
  5. 提醒用户
```

### 3. 操作录制与回放
```
用户: "记录我接下来的操作步骤"
LLM:
  1. 启动 monitor_keyboard_events 和 monitor_mouse_events
  2. 持续获取事件并记录
  3. 用户完成后，分析事件序列
  4. 生成可重放的操作脚本
  5. 通过输入操作工具回放
```

## 开发与扩展

### 添加新的监控平台支持
监控模块位于 `src/monitor_*.rs`，每个模块包含：
- 事件类型定义（已实现 Serialize）
- `start_monitor()` 函数接口
- 平台特定的 `platform` 模块

扩展步骤：
1. 在对应的 `platform` 模块中实现具体监控逻辑
2. 使用回调函数 `on_event` 报告捕获的事件
3. 返回 `MonitorHandle` 用于生命周期管理

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test monitor_screen_macos_tests
cargo test server_integration
```

## 故障排除

### 问题：权限被拒绝
**症状**：监控工具返回错误或输入操作无效
**解决**：
1. macOS: 系统偏好设置 → 安全性与隐私 → 辅助功能 → 添加终端或 iris-mcp
2. 重启应用程序或终端

### 问题：监控器返回 NotImplemented
**症状**：调用 monitor_keyboard_events 或 monitor_mouse_events 返回 -32002/-32003
**原因**：当前平台的监控实现尚未完成
**现状**：仅屏幕监控在 macOS 上可用

### 问题：Claude Desktop 无法连接
**检查清单**：
1. 确认配置文件路径正确
2. 确认可执行文件路径使用绝对路径
3. 重启 Claude Desktop
4. 检查 Claude Desktop 日志

### 问题：无法编译
**常见原因**：
- 确保 Rust 工具链版本 >= 1.70
- macOS 需要 Xcode Command Line Tools
- 运行 `cargo clean && cargo build` 清理缓存

## 路线图

- [x] 基础鼠标/键盘输入操作
- [x] 屏幕监控（macOS）
- [x] 监控工具 MCP 接口
- [ ] 键盘监控实现（macOS/Windows/Linux）
- [ ] 鼠标监控实现（macOS/Windows/Linux）
- [ ] 屏幕监控扩展到 Windows/Linux
- [ ] 事件过滤与条件触发
- [ ] 操作录制与回放增强
- [ ] 屏幕截图返回（base64 编码）

## 许可证

MIT License - 详见 LICENSE 文件

## 贡献

欢迎提交 Issue 和 Pull Request！

**贡献指南**：
1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

**优先级高的贡献方向**：
- 完善键盘/鼠标监控的平台实现
- 增加更多测试用例
- 改进错误处理和日志
- 文档翻译（英文等）

## 致谢

- [Enigo](https://github.com/enigo-rs/enigo) - 跨平台输入控制库
- [MCP](https://modelcontextprotocol.io/) - 模型上下文协议标准
