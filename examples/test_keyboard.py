#!/usr/bin/env python3
"""
测试键盘监控功能（rdev 事件驱动实现）

这个脚本测试基于 rdev 的键盘监控：
- 使用操作系统原生事件机制（零 CPU 占用）
- 从服务器启动开始自动累积事件
- 支持增量读取（使用 cursor）
"""

import json
import subprocess
import sys
import time

def test_keyboard_monitor():
    """测试键盘监控"""
    print("🎹 测试键盘监控功能 (rdev 事件驱动)")
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
        
        print("📝 键盘监控已自动启动（从初始化开始累积事件）")
        print("   请在接下来的5秒内按几个键...")
        print()
        
        # 倒计时让用户按键
        for i in range(5, 0, -1):
            print(f"   ⏳ {i}秒...", end='\r', flush=True)
            time.sleep(1)
        print()
        
        # 获取累积的键盘事件
        print("\n📥 获取累积的键盘事件...")
        monitor_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "monitor_keyboard_events",
                "arguments": {"cursor": 0}
            }
        }
        
        process.stdin.write(json.dumps(monitor_request) + '\n')
        process.stdin.flush()
        
        response_line = process.stdout.readline()
        if not response_line:
            print("⚠️  未收到响应")
            return
        
        monitor_result = json.loads(response_line)
        
        if 'result' in monitor_result:
            content = monitor_result['result'].get('content', [])
            
            # 找到 JSON 数据
            json_content = None
            for item in content:
                if item.get('type') == 'json':
                    json_content = item.get('json')
                    break
            
            if not json_content:
                print(f"⚠️  未找到事件数据")
                return
            
            events = json_content.get('events', [])
            next_cursor = json_content.get('next_cursor', 0)
            
            print(f"✅ 成功获取键盘事件!")
            print(f"   事件总数: {len(events)}")
            print(f"   next_cursor: {next_cursor}")
            print()
            
            if events:
                print(f"📋 捕获的键盘事件 (最多显示 15 个):")
                print()
                for i, event in enumerate(events[:15], 1):
                    key = event.get('key', 'unknown')
                    event_type = event.get('event_type', 'unknown')
                    timestamp = event.get('timestamp_micros', 0)
                    
                    # 格式化时间戳（显示相对时间）
                    if i == 1:
                        time_str = "0ms"
                        base_time = timestamp
                    else:
                        delta_ms = (timestamp - base_time) // 1000
                        time_str = f"+{delta_ms}ms"
                    
                    # 格式化事件类型
                    type_icon = "↓" if event_type == "press" else "↑"
                    
                    print(f"   {i:2d}. {type_icon} {key:20s} @ {time_str}")
                
                if len(events) > 15:
                    print(f"   ... 还有 {len(events) - 15} 个事件")
                
                print()
                print("✅ 键盘监控工作正常！")
                print()
                print("💡 特性说明:")
                print("   - 使用 rdev 事件驱动（零 CPU 占用）")
                print("   - 基于操作系统原生事件机制")
                print("   - 从服务器启动自动累积事件")
                print("   - 支持增量读取（使用 cursor 参数）")
                
                # 测试增量读取
                if len(events) > 5:
                    print()
                    print("🔄 测试增量读取...")
                    print("   再按几个键...")
                    time.sleep(2)
                    
                    # 使用 next_cursor 获取新事件
                    incremental_request = {
                        "jsonrpc": "2.0",
                        "id": 3,
                        "method": "tools/call",
                        "params": {
                            "name": "monitor_keyboard_events",
                            "arguments": {"cursor": next_cursor}
                        }
                    }
                    
                    process.stdin.write(json.dumps(incremental_request) + '\n')
                    process.stdin.flush()
                    
                    response_line = process.stdout.readline()
                    if response_line:
                        incremental_result = json.loads(response_line)
                        if 'result' in incremental_result:
                            inc_content = incremental_result['result']['content']
                            inc_json = None
                            for item in inc_content:
                                if item.get('type') == 'json':
                                    inc_json = item.get('json')
                                    break
                            
                            if inc_json:
                                new_events = inc_json.get('events', [])
                                print(f"   ✅ 增量读取到 {len(new_events)} 个新事件")
            else:
                print("⚠️  未捕获到键盘事件")
                print()
                print("💡 可能的原因:")
                print("   1. 监控期间没有按键")
                print("   2. 需要授予辅助功能权限")
                print()
                print("📋 授予权限的步骤 (macOS):")
                print("   1. 打开 系统设置 > 隐私与安全性 > 辅助功能")
                print("   2. 点击 + 按钮")
                print("   3. 添加运行此脚本的应用 (终端/iTerm/VS Code)")
                print("   4. 确保开关已启用")
                print("   5. 重启应用并重新运行测试")
        
        elif 'error' in monitor_result:
            error = monitor_result['error']
            print(f"❌ 错误: [{error['code']}] {error['message']}")
            print()
            print("💡 常见错误解决方案:")
            print()
            print("📋 macOS 授予辅助功能权限:")
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
