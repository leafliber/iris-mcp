#!/usr/bin/env python3
"""
iris-mcp Python 客户端示例
演示如何在 Python 中调用 iris-mcp 的工具
"""

import json
import subprocess
import sys
from typing import Any, Dict, Optional


class IrisMCPClient:
    """iris-mcp 客户端封装"""
    
    def __init__(self, executable_path: str = "./target/release/iris-mcp"):
        self.executable_path = executable_path
        self.request_id = 0
        self.proc = None
        self._start_server()
    
    def _start_server(self):
        """启动 MCP 服务器进程"""
        self.proc = subprocess.Popen(
            [self.executable_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1  # 行缓冲
        )
    
    def _call(self, method: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """发送 JSON-RPC 请求"""
        if self.proc is None or self.proc.poll() is not None:
            raise Exception("Server process not running")
        
        self.request_id += 1
        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params or {}
        }
        
        # 发送请求
        self.proc.stdin.write(json.dumps(request) + "\n")
        self.proc.stdin.flush()
        
        # 读取响应
        response_line = self.proc.stdout.readline()
        if not response_line:
            raise Exception("No response from server")
        
        response = json.loads(response_line)
        
        if "error" in response:
            raise Exception(f"RPC Error: {response['error']}")
        
        return response.get("result", {})
    
    def initialize(self) -> Dict[str, Any]:
        """初始化 MCP 服务器"""
        return self._call("initialize")
    
    def list_tools(self) -> list:
        """列出所有可用工具"""
        result = self._call("tools/list")
        return result.get("tools", [])
    
    def call_tool(self, tool_name: str, arguments: Dict[str, Any]) -> Any:
        """调用指定工具"""
        result = self._call("tools/call", {
            "name": tool_name,
            "arguments": arguments
        })
        return result
    
    # 便捷方法
    
    def mouse_move(self, x: int, y: int) -> str:
        """移动鼠标"""
        result = self.call_tool("mouse_move", {"x": x, "y": y})
        return result["content"][0]["text"]
    
    def mouse_click(self, x: int, y: int, button: str = "left") -> str:
        """点击鼠标"""
        result = self.call_tool("mouse_click", {"x": x, "y": y, "button": button})
        return result["content"][0]["text"]
    
    def type_text(self, text: str) -> str:
        """输入文本"""
        result = self.call_tool("type_text", {"text": text})
        return result["content"][0]["text"]
    
    def monitor_screen_events(self) -> Dict[str, Any]:
        """截取当前屏幕画面（按需捕获）"""
        result = self.call_tool("monitor_screen_events", {})
        events_data = result["content"][1]["json"]
        return {
            "event": events_data.get("event"),
            "image_base64": events_data.get("image_base64", ""),
            "width": events_data.get("width", 0),
            "height": events_data.get("height", 0)
        }
    
    def monitor_keyboard_events(self, cursor: int = 0) -> Dict[str, Any]:
        """获取键盘监控事件"""
        result = self.call_tool("monitor_keyboard_events", {"cursor": cursor})
        events_data = result["content"][1]["json"]
        return {
            "events": events_data["events"],
            "next_cursor": events_data["next_cursor"],
            "count": len(events_data["events"])
        }
    
    def monitor_mouse_events(self, cursor: int = 0) -> Dict[str, Any]:
        """获取鼠标监控事件"""
        result = self.call_tool("monitor_mouse_events", {"cursor": cursor})
        events_data = result["content"][1]["json"]
        return {
            "events": events_data["events"],
            "next_cursor": events_data["next_cursor"],
            "count": len(events_data["events"])
        }
    
    def close(self):
        """关闭服务器进程"""
        if self.proc and self.proc.poll() is None:
            self.proc.terminate()
            self.proc.wait(timeout=2)
    
    def __enter__(self):
        """上下文管理器入口"""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """上下文管理器退出"""
        self.close()


def main():
    """演示示例"""
    print("=== iris-mcp Python 客户端演示 ===\n")
    
    # 使用上下文管理器确保进程正确关闭
    with IrisMCPClient() as client:
        
        # 1. 初始化
        print("1. 初始化服务器...")
        info = client.initialize()
        print(f"   服务器: {info['serverInfo']['name']} v{info['serverInfo']['version']}")
        print(f"   协议版本: {info['protocolVersion']}\n")
        
        # 2. 列出工具
        print("2. 列出所有工具...")
        tools = client.list_tools()
        print(f"   共 {len(tools)} 个工具:")
        
        # 显示输入操作工具
        input_tools = [t for t in tools if not t['name'].startswith('monitor_')]
        print(f"   输入操作工具 ({len(input_tools)}个):")
        for tool in input_tools[:3]:
            print(f"   - {tool['name']}: {tool['description']}")
        if len(input_tools) > 3:
            print(f"   ... 还有 {len(input_tools) - 3} 个输入工具")
        
        # 显示监控工具
        monitor_tools = [t for t in tools if t['name'].startswith('monitor_')]
        print(f"   监控工具 ({len(monitor_tools)}个):")
        for tool in monitor_tools:
            print(f"   - {tool['name']}: {tool['description']}")
        print()
        
        # 3. 输入操作示例
        print("3. 输入操作示例...")
        try:
            # 注意：这会实际移动鼠标！
            print("   提示: 以下操作会实际控制输入设备")
            response = input("   是否继续？(y/N): ")
            if response.lower() == 'y':
                print(f"   {client.mouse_move(500, 300)}")
                print(f"   {client.type_text('Hello from Python!')}")
        except Exception as e:
            print(f"   操作失败: {e}\n")
        
        # 4. 监控示例
        print("\n4. 监控工具示例...")
        try:
            # 屏幕监控（按需截图）
            print("   捕获屏幕截图...")
            screen_data = client.monitor_screen_events()
            event = screen_data['event']
            print(f"   - 事件类型: {event['kind']['type']}")
            print(f"   - 分辨率: {screen_data['width']}x{screen_data['height']}")
            print(f"   - 图像数据: {len(screen_data['image_base64'])} 字节 (base64)")
            
            # 键盘监控（事件驱动，累积事件）
            print("\n   获取键盘事件（从启动开始累积）...")
            print("   提示: 请在接下来按几个键...")
            import time
            time.sleep(2)
            try:
                keyboard_data = client.monitor_keyboard_events(cursor=0)
                print(f"   ✅ 获得 {keyboard_data['count']} 条键盘事件")
                if keyboard_data['count'] > 0:
                    print(f"   前3个事件:")
                    for evt in keyboard_data['events'][:3]:
                        print(f"     - {evt['key']} ({evt['event_type']})")
            except Exception as e:
                error_msg = str(e).replace('RPC Error: ', '')
                print(f"   ⚠️ {error_msg}")
                print(f"   提示: 需要在系统设置中授予辅助功能权限")
            
            # 鼠标监控（事件驱动，累积事件）
            print("\n   获取鼠标事件...")
            try:
                mouse_data = client.monitor_mouse_events(cursor=0)
                print(f"   ✅ 获得 {mouse_data['count']} 条鼠标事件")
                if mouse_data['count'] > 0:
                    print(f"   前3个事件:")
                    for evt in mouse_data['events'][:3]:
                        kind = evt['kind']
                        if kind.get('type') == 'Move':
                            print(f"     - 移动到 ({kind['x']}, {kind['y']})")
                        elif kind.get('type') == 'Button':
                            print(f"     - {kind['button']} {kind['state']}")
            except Exception as e:
                error_msg = str(e).replace('RPC Error: ', '')
                print(f"   ⚠️ {error_msg}")

            
        except Exception as e:
            print(f"   监控失败: {e}")
    
    print("\n=== 演示完成 ===")


if __name__ == "__main__":
    main()
