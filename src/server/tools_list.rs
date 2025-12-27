use serde_json::{json, Value};

pub fn get_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "mouse_move",
                "description": "移动鼠标到指定坐标",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "X 坐标" },
                        "y": { "type": "integer", "description": "Y 坐标" }
                    },
                    "required": ["x", "y"]
                }
            },
            {
                "name": "mouse_click",
                "description": "在指定坐标点击鼠标按钮",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "X 坐标" },
                        "y": { "type": "integer", "description": "Y 坐标" },
                        "button": { "type": "string", "enum": ["left", "right", "middle"], "description": "鼠标按钮" }
                    },
                    "required": ["x", "y", "button"]
                }
            },
            {
                "name": "mouse_double_click",
                "description": "在指定坐标双击鼠标按钮",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "X 坐标" },
                        "y": { "type": "integer", "description": "Y 坐标" },
                        "button": { "type": "string", "enum": ["left", "right", "middle"], "description": "鼠标按钮" }
                    },
                    "required": ["x", "y", "button"]
                }
            },
            {
                "name": "mouse_scroll",
                "description": "滚动鼠标滚轮",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "lines_x": { "type": "integer", "description": "水平滚动行数" },
                        "lines_y": { "type": "integer", "description": "垂直滚动行数" }
                    },
                    "required": ["lines_x", "lines_y"]
                }
            },
            {
                "name": "mouse_get_position",
                "description": "获取当前鼠标位置",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "type_text",
                "description": "使用键盘输入文本",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "要输入的文本" }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "system_command",
                "description": "执行系统命令快捷键(复制、粘贴等)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "enum": ["copy", "paste", "cut", "undo", "save", "select_all"],
                            "description": "要执行的命令"
                        }
                    },
                    "required": ["command"]
                }
            },
            {
                "name": "mouse_drag",
                "description": "拖拽鼠标从当前位置到目标位置",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target_x": {
                            "type": "integer",
                            "description": "目标X坐标"
                        },
                        "target_y": {
                            "type": "integer",
                            "description": "目标Y坐标"
                        },
                        "button": {
                            "type": "string",
                            "enum": ["left", "middle", "right"],
                            "description": "鼠标按钮"
                        }
                    },
                    "required": ["target_x", "target_y", "button"]
                }
            },
            {
                "name": "mouse_button_control",
                "description": "控制鼠标按钮按下或释放",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "button": {
                            "type": "string",
                            "enum": ["left", "middle", "right"],
                            "description": "鼠标按钮"
                        },
                        "direction": {
                            "type": "string",
                            "enum": ["press", "release", "click"],
                            "description": "操作方向：press按下/release释放/click点击"
                        }
                    },
                    "required": ["button", "direction"]
                }
            },
            {
                "name": "mouse_move_path",
                "description": "按指定路径移动鼠标",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "points": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "x": {"type": "integer"},
                                    "y": {"type": "integer"}
                                },
                                "required": ["x", "y"]
                            },
                            "description": "路径点数组"
                        },
                        "speed_ms": {
                            "type": "integer",
                            "description": "每个点之间的延迟毫秒数"
                        }
                    },
                    "required": ["points", "speed_ms"]
                }
            },
            {
                "name": "key_control",
                "description": "控制键盘按键按下或释放",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "按键名称，如：a, b, return, shift, control, alt等"
                        },
                        "direction": {
                            "type": "string",
                            "enum": ["press", "release", "click"],
                            "description": "操作方向：press按下/release释放/click点击"
                        }
                    },
                    "required": ["key", "direction"]
                }
            },
            {
                "name": "monitor_screen_events",
                "description": "截取当前屏幕画面，返回 PNG 格式的图像（每次调用返回一帧新的屏幕截图）",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "reason": { "type": "string", "description": "调用原因，便于审计" }
                    },
                    "required": ["reason"]
                }
            },
            {
                "name": "monitor_keyboard_events",
                "description": "获取已积累的键盘监控事件（服务器启动时自动开始监控）",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "reason": { "type": "string", "description": "调用原因，便于审计" },
                        "cursor": {
                            "type": "integer",
                            "description": "从该游标开始读取事件，默认0"
                        }
                    },
                    "required": ["reason"]
                }
            },
            {
                "name": "monitor_mouse_events",
                "description": "获取已积累的鼠标监控事件（服务器启动时自动开始监控）",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "reason": { "type": "string", "description": "调用原因，便于审计" },
                        "cursor": {
                            "type": "integer",
                            "description": "从该游标开始读取事件，默认0"
                        }
                    },
                    "required": ["reason"]
                }
            }
        ]
    })
}
