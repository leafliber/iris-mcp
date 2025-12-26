#!/usr/bin/env python3
"""测试键盘监控功能"""

import json
import subprocess
import sys
import time
import threading

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
        
        # 调用 monitor_keyboard_events - 使用线程异步调用
        print("📝 启动键盘监控...")
        print("   请在接下来的3秒内按几个键...")
        print()
        
        monitor_response = [None]
        
        def send_monitor_request():
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
            
            # 读取响应
            response_line = process.stdout.readline()
            if response_line:
                monitor_response[0] = json.loads(response_line)
        
        # 在后台线程发送请求
        thread = threading.Thread(target=send_monitor_request)
        thread.daemon = True
        thread.start()
        
        # 等待3秒让用户按键
        print("⏳ 监控中... (3秒)")
        for i in range(3, 0, -1):
            print(f"   {i}...", end='\r', flush=True)
            time.sleep(1)
        print()
        
        # 等待响应
        thread.join(timeout=2)
        
        if not monitor_response[0]:
            print("⚠️  未收到监控响应 (可能超时)")
            print("   提示：键盘监控会立即返回当前已捕获的事件")
            return
        
        monitor_result = monitor_response[0]
        
        if 'result' in monitor_result:
            content = monitor_result['result'].get('content', [])
            if content:
                # MCP 返回两个 content 项：第一个是文本描述，第二个是 JSON 数据
                # 找到 type 为 "json" 的项
                json_content = None
                for item in content:
                    if item.get('type') == 'json':
                        json_content = item.get('json')
                        break
                
                if not json_content:
                    # 如果没有 json 类型，尝试解析 text
                    text_content = content[0].get('text', '')
                    print(f"ℹ️  服务器响应: {text_content}")
                    if len(content) > 1:
                        print(f"   收到 {len(content)} 个内容项")
                    return
                
                result_data = json_content
                
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
                    
                    print()
                    print("✅ 键盘监控工作正常！")
                else:
                    print("⚠️  未捕获到键盘事件")
                    print()
                    print("💡 可能的原因:")
                    print("   1. 监控期间没有按键")
                    print("   2. 需要授予辅助功能权限")
                    print()
                    print("📋 授予权限的步骤:")
                    print("   1. 打开 系统设置 > 隐私与安全性 > 辅助功能")
                    print("   2. 点击 + 按钮")
                    print("   3. 添加运行此脚本的应用 (终端/iTerm/VS Code)")
                    print("   4. 确保开关已启用")
                    print("   5. 重启应用并重新运行测试")
            else:
                print("❌ 响应内容为空")
        elif 'error' in monitor_result:
            error = monitor_result['error']
            print(f"❌ 错误: [{error['code']}] {error['message']}")
            
            if error['code'] == -32002:
                print()
                print("💡 这是预期的错误 - 键盘监控需要辅助功能权限")
                print()
                print("📋 授予权限的步骤:")
                print("   1. 打开 系统设置 > 隐私与安全性 > 辅助功能")
                print("   2. 点击 + 按钮")
                print("   3. 添加运行此脚本的应用 (终端/iTerm/VS Code)")
                print("   4. 确保开关已启用")
                print("   5. 重启应用并重新运行测试")
        else:
            print(f"❌ 未预期的响应: {monitor_result}")
    
    except KeyboardInterrupt:
        print("\n\n⚠️  测试被用户中断")
    except Exception as e:
        print(f"❌ 测试失败: {e}")
        import traceback
        traceback.print_exc()
    
    finally:
        try:
            process.stdin.close()
        except:
            pass
        process.terminate()
        process.wait(timeout=1)
        print()
        print("=" * 60)
        print("测试完成")

if __name__ == "__main__":
    test_keyboard_monitor()
