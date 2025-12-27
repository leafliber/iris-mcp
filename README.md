# iris-mcp

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)

**基于 Rust 的 MCP 输入控制服务器** - 为 LLM 提供系统级的鼠标、键盘控制与实时监控能力。

## ✨ 核心特性

- 🎯 **输入控制** - 完整的鼠标/键盘操作：移动、点击、拖拽、滚动、文本输入、系统快捷键
- 👀 **实时监控** - 捕获屏幕、键盘、鼠标事件，支持增量读取，适合 LLM 观察用户操作
- 🔌 **MCP 标准** - JSON-RPC 2.0 over stdin/stdout，无缝集成 Claude Desktop 等客户端
- 🚀 **零依赖运行** - 基于原生系统 API（Enigo + rdev），无需额外运行时
- 📦 **跨平台** - 支持 macOS、Linux、Windows（部分功能开发中）

## 🏗️ 项目结构

```
src/
├── server/           # MCP 服务器核心
│   ├── jsonrpc.rs    # JSON-RPC 协议实现
│   ├── mouse.rs      # 鼠标工具处理
│   ├── keyboard.rs   # 键盘工具处理
│   ├── monitor.rs    # 监控工具处理
│   └── tools_list.rs # 工具列表定义
├── operator/         # 输入操作层
│   ├── keyboard.rs   # 键盘控制器
│   └── mouse.rs      # 鼠标控制器
└── monitor/          # 监控实现层
    ├── key_mouse.rs  # 键鼠监控（rdev）
    ├── screen.rs     # 屏幕监控
    └── state.rs      # 监控状态管理
```

## 🚀 快速开始

### 安装
```bash
git clone https://github.com/yourusername/iris-mcp.git
cd iris-mcp

# 构建 release 版本
cargo build --release
# 或使用 make
make release-macos
```

二进制文件位于：`target/release/iris-mcp` 或 `release-builds/iris-mcp-macos-arm64`

### 与 Claude Desktop 集成

编辑配置文件（重启 Claude Desktop 生效）：

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "iris-mcp": {
      "command": "/absolute/path/to/iris-mcp"
    }
  }
}
```

### 使用示例

```
你: "移动鼠标到屏幕中央并点击"
Claude: [调用 mouse_move 和 mouse_click]

你: "输入 Hello World"
Claude: [调用 type_text]

你: "监控我的键盘，记录我接下来的操作"
Claude: [调用 monitor_keyboard_events 持续获取事件]
```

## 🛠️ 可用工具

完整工具列表和详细文档：[TOOL_REFERENCE.md](TOOL_REFERENCE.md)

### 鼠标控制 (8 个工具)
- `mouse_move` - 移动鼠标
- `mouse_click` - 点击
- `mouse_double_click` - 双击
- `mouse_scroll` - 滚动
- `mouse_get_position` - 获取位置
- `mouse_drag` - 拖拽
- `mouse_button_control` - 按钮控制
- `mouse_move_path` - 路径移动

### 键盘控制 (3 个工具)
- `type_text` - 输入文本
- `key_control` - 按键控制
- `system_command` - 系统快捷键 (复制/粘贴/剪切/撤销/保存/全选)

### 监控工具 (3 个工具)
- `monitor_screen_events` - 屏幕监控
- `monitor_keyboard_events` - 键盘监控
- `monitor_mouse_events` - 鼠标监控

**监控增量读取**：使用 `cursor` 参数实现增量读取，避免重复处理事件
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{
  "name":"monitor_keyboard_events",
  "arguments":{"cursor":0}
}}
// 返回：events + next_cursor，下次使用 next_cursor 继续读取
```

## 💻 平台支持

| 功能 | macOS | Windows | Linux |
|-----|-------|---------|-------|
| 鼠标控制 | ✅ | ✅ | ✅ |
| 键盘控制 | ✅ | ✅ | ✅ |
| 屏幕监控 | ✅ | ⏳ | ⏳ |
| 键盘监控 | ✅ (rdev) | ✅ (rdev) | ✅ (rdev) |
| 鼠标监控 | ✅ (rdev) | ✅ (rdev) | ✅ (rdev) |

### macOS 权限设置

**辅助功能权限**（必需）：
1. **系统设置** → **隐私与安全性** → **辅助功能**
2. 点击 **+** 添加：
   - 终端.app（如从终端运行）
   - VS Code.app（如从 VS Code 运行）
   - 或运行程序的应用
3. 确保开关已启用 ✅
4. 重启应用/终端

测试权限：`python3 examples/test_keyboard.py`

## 📦 跨平台编译

详见：[BUILD.md](BUILD.md)

```bash
# 安装工具
brew install zig
cargo install cargo-zigbuild

# 构建所有平台
make release-all

# 或单独构建
make release-linux    # Linux x86_64
make release-windows  # Windows x86_64
```

生成的文件位于 `release-builds/` 目录。

## 📚 示例代码

- **Shell 演示**: [`examples/monitor_demo.sh`](examples/monitor_demo.sh)
- **Python 客户端**: [`examples/client_example.py`](examples/client_example.py)
- **键盘测试**: [`examples/test_keyboard.py`](examples/test_keyboard.py)

```bash
./examples/monitor_demo.sh  # Shell 演示
python3 examples/client_example.py  # Python 演示
```

## 💡 使用场景

### 1. 自动化操作
LLM 理解意图后自动执行复杂操作序列
```
用户："打开计算器并计算 2+3"
→ mouse_move + click + type_text + key_control
```

### 2. 实时监控与响应
LLM 观察用户操作并智能响应
```
用户："监控我的操作，提醒我保存文件"
→ monitor_keyboard_events + 检测 Cmd+S
```

### 3. 操作录制与回放
记录用户操作并生成可重放脚本
```
→ monitor_keyboard/mouse_events 持续记录
→ 分析事件序列生成脚本
→ 通过输入工具回放
```

## 🔧 开发

### 测试
```bash
cargo test              # 所有测试
cargo test --release    # Release 模式测试
```

### 添加新工具
1. 在 `src/server/` 对应模块添加处理函数
2. 在 `src/server/tools_list.rs` 添加工具定义
3. 在 `src/server/mod.rs` 的 `handle_call_tool` 添加路由

### 项目文档
- [BUILD.md](BUILD.md) - 跨平台编译指南
- [TOOL_REFERENCE.md](TOOL_REFERENCE.md) - 完整工具文档
- [REFACTORING.md](REFACTORING.md) - 重构记录

## ⚠️ 故障排除

| 问题 | 解决方案 |
|------|----------|
| 权限被拒绝 | macOS: 系统设置 → 辅助功能 → 添加应用 |
| Claude Desktop 无法连接 | 检查配置文件路径，使用绝对路径，重启 Claude |
| 编译失败 | 确保 Rust >= 1.70，macOS 需要 Xcode Command Line Tools |
| 监控无响应 | 检查权限设置，重启应用 |

## 🗺️ 路线图

- [x] 基础鼠标/键盘输入操作
- [x] 键盘/鼠标监控（rdev）
- [x] 屏幕监控（macOS）
- [x] 跨平台编译支持
- [x] 模块化重构
- [ ] 屏幕监控扩展到 Windows/Linux
- [ ] 屏幕截图返回（base64）
- [ ] 事件过滤与条件触发
- [ ] GitHub Actions 自动发布

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🤝 贡献

欢迎 Issues 和 Pull Requests！

**贡献方向**：
- 完善 Windows/Linux 屏幕监控
- 增加测试覆盖率
- 改进文档
- 性能优化

## 🙏 致谢

- [Enigo](https://github.com/enigo-rs/enigo) - 跨平台输入控制
- [rdev](https://github.com/Narsil/rdev) - 跨平台事件监听
- [MCP](https://modelcontextprotocol.io/) - 模型上下文协议

---

**⭐ 如果这个项目对你有帮助，请给个 Star！**
