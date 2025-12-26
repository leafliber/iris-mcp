#!/usr/bin/env python3
"""测试键盘监控功能"""

import json
import subprocess
import sys
import time

def test_keyboard_monitor():
    """测试键盘监控"""
    print("🎹 测试键盘监控功能")
    print("=" * 60)
    
    # 启动 MCP 服务器
    process = subprocess.Popen(
        ['./target/release/iris-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    try:
        # 初始化
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "keyboard-test", "version": "1.0.0"}
            }
        }
        
        process.stdin.write(json.dumps(init_request) + '\n')
        process.stdin.flush()
        
        response = process.stdout.readline()
        init_result = json.loads(response)
        
        if 'result' not in init_result:
            print(f"❌ 初始化失败: {init_result}")
            return
        
        print(f"✅ MCP 服务器初始化成功")
        print(f"   协议版本: {init_result['result']['protocolVersion']}")
        print()
        
        # 调用 monitor_keyboard_events
        print("📝 调用 monitor_keyboard_events 工具...")
        print("   请在接下来的5秒内按几个键...")
        print()
        
        monitor_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "monitor_keyboard_events",
                "arguments": {}
            }
        }
        
        process.stdin.write(json.dumps(monitor_request) + '\n')
        process.stdin.flush()
        
        # 等待5秒让用户按键
        print("⏳ 监控中... (5秒)")
        for i in range(5, 0, -1):
            print(f"   {i}...", end='\r')
            time.sleep(1)
        print()
        
        response = process.stdout.readline()
        monitor_result = json.loads(response)
        
        if 'result' in monitor_result:
            content = monitor_result['result']['content']
            if content:
                result_text = content[0]['text']
                result_data = json.loads(result_text)
                
                print(f"✅ 键盘监控成功!")
                print(f"   捕获的事件数: {result_data.get('count', 0)}")
                print(f"   next_cursor: {result_data.get('next_cursor', 'N/A')}")
                print()
                
                events = result_data.get('events', [])
                if events:
                    print(f"📋 捕获的键盘事件 (前10个):")
                    for i, event in enumerate(events[:10], 1):
                        code = event.get('code', {})
                        state = event.get('state', '')
                        timestamp = event.get('timestamp_micros', 0)
                        
                        if 'Char' in code:
                            key_str = f"字符 '{code['Char']}'"
                        elif 'Named' in code:
                            key_str = f"按键 '{code['Named']}'"
                        else:
                            key_str = f"扫描码 {code.get('ScanCode', 'unknown')}"
                        
                        print(f"   {i}. {key_str} - {state} @ {timestamp}")
                else:
                    print("⚠️  未捕获到键盘事件")
                    print("   可能需要授予辅助功能权限:")
                    print("   系统设置 > 隐私与安全性 > 辅助功能")
            else:
                print("❌ 响应内容为空")
        elif 'error' in monitor_result:
            error = monitor_result['error']
            print(f"❌ 错误: [{error['code']}] {error['message']}")
            
            if error['code'] == -32002:
                print()
                print("💡 这是预期的错误 - 键盘监控需要辅助功能权限")
                print("   请按照以下步骤授予权限:")
                print("   1. 打开 系统设置 > 隐私与安全性 > 辅助功能")
                print("   2. 点击 + 按钮添加终端或 VSCode")
                print("   3. 重新运行此测试")
        else:
            print(f"❌ 未预期的响应: {monitor_result}")
    
    except Exception as e:
        print(f"❌ 测试失败: {e}")
        import traceback
        traceback.print_exc()
    
    finally:
        process.stdin.close()
        process.terminate()
        process.wait()
        print()
        print("=" * 60)
        print("测试完成")

if __name__ == "__main__":
    test_keyboard_monitor()
