use crate::monitor_keyboard::{self, KeyEvent, KeyState};
use crate::monitor_mouse::{self, MouseEvent, MouseEventKind};
use crate::monitor_screen::{self, ScreenEvent, ScreenEventKind};
use crate::operator_keyboard::{KeyboardController, SystemCommand};
use crate::operator_mouse::MouseController;
use enigo::{Button, Direction, Enigo, Key, Settings};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

struct KeyboardMonitorState {
    events: Mutex<Vec<KeyEvent>>,
    #[allow(dead_code)]
    handle: Option<monitor_keyboard::MonitorHandle>,
}

struct MouseMonitorState {
    events: Mutex<Vec<MouseEvent>>,
    #[allow(dead_code)]
    handle: Option<monitor_mouse::MonitorHandle>,
}

static KEYBOARD_STATE: OnceLock<Result<KeyboardMonitorState, String>> = OnceLock::new();
static MOUSE_STATE: OnceLock<Result<MouseMonitorState, String>> = OnceLock::new();

fn ensure_keyboard_monitor_started() -> Result<&'static KeyboardMonitorState, JsonRpcError> {
    KEYBOARD_STATE.get_or_init(|| {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            use std::sync::Arc;
            
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = Arc::clone(&events);
            
            match monitor_keyboard::start_monitor(move |evt| {
                if let Ok(mut guard) = events_clone.lock() {
                    guard.push(evt);
                }
            }) {
                Ok(handle) => {
                    let final_events = Arc::try_unwrap(events)
                        .unwrap_or_else(|arc| Mutex::new(arc.lock().unwrap().clone()));
                    Ok(KeyboardMonitorState {
                        events: final_events,
                        handle: Some(handle),
                    })
                }
                Err(e) => Err(e.to_string()),
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err("keyboard monitor unsupported on this platform".to_string())
        }
    });

    match KEYBOARD_STATE.get().expect("state initialized") {
        Ok(state) => Ok(state),
        Err(msg) => Err(JsonRpcError {
            code: -32002,
            message: msg.clone(),
            data: None,
        }),
    }
}

fn ensure_mouse_monitor_started() -> Result<&'static MouseMonitorState, JsonRpcError> {
    MOUSE_STATE.get_or_init(|| {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            use std::sync::Arc;
            
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = Arc::clone(&events);
            
            match monitor_mouse::start_monitor(move |evt| {
                if let Ok(mut guard) = events_clone.lock() {
                    guard.push(evt);
                }
            }) {
                Ok(handle) => {
                    let final_events = Arc::try_unwrap(events)
                        .unwrap_or_else(|arc| Mutex::new(arc.lock().unwrap().clone()));
                    Ok(MouseMonitorState {
                        events: final_events,
                        handle: Some(handle),
                    })
                }
                Err(e) => Err(e.to_string()),
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err("mouse monitor unsupported on this platform".to_string())
        }
    });

    match MOUSE_STATE.get().expect("state initialized") {
        Ok(state) => Ok(state),
        Err(msg) => Err(JsonRpcError {
            code: -32003,
            message: msg.clone(),
            data: None,
        }),
    }
}

fn screen_event_to_json(evt: &ScreenEvent) -> Value {
    let kind = match evt.kind {
        ScreenEventKind::GeometryChanged { width, height, scale } => json!({
            "type": "geometry_changed",
            "width": width,
            "height": height,
            "scale": scale
        }),
        ScreenEventKind::DisplayAdded => json!({ "type": "display_added" }),
        ScreenEventKind::DisplayRemoved => json!({ "type": "display_removed" }),
        ScreenEventKind::FrameCaptured { width, height, format } => json!({
            "type": "frame_captured",
            "width": width,
            "height": height,
            "format": format,
        }),
    };

    json!({
        "timestamp_micros": evt.timestamp_micros,
        "kind": kind,
    })
}

fn keyboard_event_to_json(evt: &KeyEvent) -> Value {
    let code = match &evt.code {
        monitor_keyboard::KeyCode::Char(c) => json!({ "type": "char", "value": c }),
        monitor_keyboard::KeyCode::Named(name) => json!({ "type": "named", "value": name }),
        monitor_keyboard::KeyCode::ScanCode(code) => json!({ "type": "scancode", "value": code }),
    };
    let state = match evt.state {
        KeyState::Press => "press",
        KeyState::Release => "release",
        KeyState::Repeat => "repeat",
    };

    json!({
        "timestamp_micros": evt.timestamp_micros,
        "code": code,
        "state": state,
    })
}

fn mouse_event_to_json(evt: &MouseEvent) -> Value {
    let kind = match evt.kind {
        MouseEventKind::Move { x, y } => json!({ "type": "move", "x": x, "y": y }),
        MouseEventKind::Button { button, state } => {
            let button = match button {
                monitor_mouse::MouseButton::Left => "left".to_string(),
                monitor_mouse::MouseButton::Middle => "middle".to_string(),
                monitor_mouse::MouseButton::Right => "right".to_string(),
                monitor_mouse::MouseButton::Other(v) => format!("other_{}", v),
            };
            let state = match state {
                monitor_mouse::ButtonState::Press => "press",
                monitor_mouse::ButtonState::Release => "release",
            };
            json!({ "type": "button", "button": button, "state": state })
        }
        MouseEventKind::Scroll { delta_x, delta_y } => json!({
            "type": "scroll",
            "delta_x": delta_x,
            "delta_y": delta_y,
        }),
    };

    json!({
        "timestamp_micros": evt.timestamp_micros,
        "kind": kind,
    })
}

fn parse_button(s: &str) -> Result<Button, JsonRpcError> {
    match s {
        "right" => Ok(Button::Right),
        "middle" => Ok(Button::Middle),
        "left" => Ok(Button::Left),
        _ => Err(JsonRpcError {
            code: -32602,
            message: format!("Invalid button: {}", s),
            data: None,
        }),
    }
}

fn handle_initialize(_params: Option<Value>) -> Value {
    // 自动启动键盘和鼠标监控（后台积累事件）
    let _ = ensure_keyboard_monitor_started();
    let _ = ensure_mouse_monitor_started();
    
    // 注意：屏幕监控不在这里启动，因为它是按需截图的
    
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "iris-mcp",
            "version": "0.1.0"
        }
    })
}

fn handle_list_tools(_params: Option<Value>) -> Value {
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
            json!({
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
            }),
            json!({
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
            }),
            json!({
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
            }),
            json!({
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
            }),
            json!({
                "name": "monitor_screen_events",
                "description": "截取当前屏幕画面（每次调用返回一帧新的屏幕截图）",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }),
            json!({
                "name": "monitor_keyboard_events",
                "description": "获取已积累的键盘监控事件（服务器启动时自动开始监控）",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "cursor": {
                            "type": "integer",
                            "description": "从该游标开始读取事件，默认0"
                        }
                    },
                    "required": []
                }
            }),
            json!({
                "name": "monitor_mouse_events",
                "description": "获取已积累的鼠标监控事件（服务器启动时自动开始监控）",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "cursor": {
                            "type": "integer",
                            "description": "从该游标开始读取事件，默认0"
                        }
                    },
                    "required": []
                }
            })
        ]
    })
}

fn handle_call_tool(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing params".to_string(),
        data: None,
    })?;

    let name = params["name"].as_str().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing tool name".to_string(),
        data: None,
    })?;

    let arguments = &params["arguments"];

    match name {
        "mouse_move" => {
            let x = arguments["x"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing x".to_string(),
                data: None,
            })? as i32;
            let y = arguments["y"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing y".to_string(),
                data: None,
            })? as i32;

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut mouse = MouseController::new(enigo);
            mouse.mouse_move(x, y).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to move mouse: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("鼠标已移动到 ({}, {})", x, y)
                }]
            }))
        }
        "mouse_click" => {
            let x = arguments["x"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing x".to_string(),
                data: None,
            })? as i32;
            let y = arguments["y"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing y".to_string(),
                data: None,
            })? as i32;
            let btn_str = arguments["button"].as_str().unwrap_or("left");
            let button = parse_button(btn_str)?;

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut mouse = MouseController::new(enigo);
            mouse.mouse_click(x, y, button).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to click: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("在 ({}, {}) 点击了 {} 键", x, y, btn_str)
                }]
            }))
        }
        "mouse_double_click" => {
            let x = arguments["x"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing x".to_string(),
                data: None,
            })? as i32;
            let y = arguments["y"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing y".to_string(),
                data: None,
            })? as i32;
            let btn_str = arguments["button"].as_str().unwrap_or("left");
            let button = parse_button(btn_str)?;

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut mouse = MouseController::new(enigo);
            mouse.mouse_double_click(x, y, button).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to double click: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("在 ({}, {}) 双击了 {} 键", x, y, btn_str)
                }]
            }))
        }
        "mouse_scroll" => {
            let lines_x = arguments["lines_x"].as_i64().unwrap_or(0) as i32;
            let lines_y = arguments["lines_y"].as_i64().unwrap_or(0) as i32;

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut mouse = MouseController::new(enigo);
            mouse.mouse_scroll(lines_x, lines_y).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to scroll: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("滚动 ({}, {})", lines_x, lines_y)
                }]
            }))
        }
        "mouse_get_position" => {
            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mouse = MouseController::new(enigo);
            let (x, y) = mouse.mouse_get_position().map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to get position: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("当前鼠标位置: ({}, {})", x, y)
                }]
            }))
        }
        "type_text" => {
            let text = arguments["text"].as_str().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing text".to_string(),
                data: None,
            })?;

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut keyboard = KeyboardController::new(enigo);
            keyboard.type_text(text).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to type: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("已输入文本: {}", text)
                }]
            }))
        }
        "system_command" => {
            let cmd_str = arguments["command"].as_str().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing command".to_string(),
                data: None,
            })?;
            
            let command = match cmd_str {
                "copy" => SystemCommand::Copy,
                "paste" => SystemCommand::Paste,
                "cut" => SystemCommand::Cut,
                "undo" => SystemCommand::Undo,
                "save" => SystemCommand::Save,
                "select_all" => SystemCommand::SelectAll,
                _ => return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown command: {}", cmd_str),
                    data: None,
                }),
            };

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut keyboard = KeyboardController::new(enigo);
            keyboard.system_command(command).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to execute command: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("已执行命令: {}", cmd_str)
                }]
            }))
        }
        "mouse_drag" => {
            let target_x = arguments["target_x"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing target_x".to_string(),
                data: None,
            })? as i32;
            let target_y = arguments["target_y"].as_i64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing target_y".to_string(),
                data: None,
            })? as i32;
            let button_str = arguments["button"].as_str().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing button".to_string(),
                data: None,
            })?;
            let button = parse_button(button_str)?;

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut mouse = MouseController::new(enigo);
            mouse.mouse_drag(target_x, target_y, button).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to drag: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("已拖拽鼠标到 ({}, {}) 使用{}键", target_x, target_y, button_str)
                }]
            }))
        }
        "mouse_button_control" => {
            let button_str = arguments["button"].as_str().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing button".to_string(),
                data: None,
            })?;
            let button = parse_button(button_str)?;
            let direction_str = arguments["direction"].as_str().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing direction".to_string(),
                data: None,
            })?;
            let direction = match direction_str {
                "press" => Direction::Press,
                "release" => Direction::Release,
                "click" => Direction::Click,
                _ => return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Invalid direction: {}", direction_str),
                    data: None,
                }),
            };

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut mouse = MouseController::new(enigo);
            mouse.mouse_button_control(button, direction).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to control button: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("已执行鼠标{}键{}", button_str, direction_str)
                }]
            }))
        }
        "mouse_move_path" => {
            let points_array = arguments["points"].as_array().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing or invalid points".to_string(),
                data: None,
            })?;
            let speed_ms = arguments["speed_ms"].as_u64().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing speed_ms".to_string(),
                data: None,
            })?;

            let mut points = Vec::new();
            for point in points_array {
                let x = point["x"].as_i64().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Invalid point x coordinate".to_string(),
                    data: None,
                })? as i32;
                let y = point["y"].as_i64().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Invalid point y coordinate".to_string(),
                    data: None,
                })? as i32;
                points.push((x, y));
            }

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut mouse = MouseController::new(enigo);
            mouse.mouse_move_path(&points, speed_ms).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to move path: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("已沿路径移动鼠标，共{}个点", points.len())
                }]
            }))
        }
        "key_control" => {
            let key_str = arguments["key"].as_str().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing key".to_string(),
                data: None,
            })?;
            let direction_str = arguments["direction"].as_str().ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing direction".to_string(),
                data: None,
            })?;
            
            let direction = match direction_str {
                "press" => Direction::Press,
                "release" => Direction::Release,
                "click" => Direction::Click,
                _ => return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Invalid direction: {}", direction_str),
                    data: None,
                }),
            };

            let key = match key_str.to_lowercase().as_str() {
                "return" | "enter" => Key::Return,
                "shift" => Key::Shift,
                "control" | "ctrl" => Key::Control,
                "alt" | "option" => Key::Alt,
                "meta" | "command" | "cmd" => Key::Meta,
                "space" => Key::Space,
                "tab" => Key::Tab,
                "escape" | "esc" => Key::Escape,
                "backspace" => Key::Backspace,
                "delete" => Key::Delete,
                "up" | "uparrow" => Key::UpArrow,
                "down" | "downarrow" => Key::DownArrow,
                "left" | "leftarrow" => Key::LeftArrow,
                "right" | "rightarrow" => Key::RightArrow,
                s if s.len() == 1 => Key::Unicode(s.chars().next().unwrap()),
                _ => return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown key: {}", key_str),
                    data: None,
                }),
            };

            let enigo = Enigo::new(&Settings::default()).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to initialize: {}", e),
                data: None,
            })?;
            let mut keyboard = KeyboardController::new(enigo);
            keyboard.key_control(key, direction).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to control key: {}", e),
                data: None,
            })?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("已执行按键{}操作: {}", key_str, direction_str)
                }]
            }))
        }
        "monitor_screen_events" => {
            // 屏幕监控：每次调用时截取一帧新的屏幕
            let event = monitor_screen::capture_frame().map_err(|e| JsonRpcError {
                code: -32001,
                message: e.to_string(),
                data: None,
            })?;

            let event_json = screen_event_to_json(&event);

            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": "已捕获当前屏幕帧"
                    },
                    {
                        "type": "json",
                        "json": {
                            "event": event_json
                        }
                    }
                ]
            }))
        }
        "monitor_keyboard_events" => {
            let cursor = arguments["cursor"].as_u64().unwrap_or(0) as usize;
            let state = ensure_keyboard_monitor_started()?;
            let events_guard = state.events.lock().map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to lock events: {}", e),
                data: None,
            })?;

            let total = events_guard.len();
            let slice = if cursor >= total {
                &[][..]
            } else {
                &events_guard[cursor..]
            };

            let events_json: Vec<Value> = slice.iter().map(keyboard_event_to_json).collect();
            let next_cursor = total;

            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("返回{}条键盘事件，next_cursor={} (total={})", events_json.len(), next_cursor, total)
                    },
                    {
                        "type": "json",
                        "json": {
                            "events": events_json,
                            "next_cursor": next_cursor
                        }
                    }
                ]
            }))
        }
        "monitor_mouse_events" => {
            let cursor = arguments["cursor"].as_u64().unwrap_or(0) as usize;
            let state = ensure_mouse_monitor_started()?;
            let events_guard = state.events.lock().map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to lock events: {}", e),
                data: None,
            })?;

            let total = events_guard.len();
            let slice = if cursor >= total {
                &[][..]
            } else {
                &events_guard[cursor..]
            };

            let events_json: Vec<Value> = slice.iter().map(mouse_event_to_json).collect();
            let next_cursor = total;

            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("返回{}条鼠标事件，next_cursor={} (total={})", events_json.len(), next_cursor, total)
                    },
                    {
                        "type": "json",
                        "json": {
                            "events": events_json,
                            "next_cursor": next_cursor
                        }
                    }
                ]
            }))
        }
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Unknown tool: {}", name),
            data: None,
        }),
    }
}

fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: format!("Invalid JSON-RPC version: {}. Expected 2.0", request.jsonrpc),
                data: None,
            }),
        };
    }

    let result = match request.method.as_str() {
        "initialize" => Ok(handle_initialize(request.params)),
        "initialized" => Ok(json!({})),
        "tools/list" => Ok(handle_list_tools(request.params)),
        "tools/call" => handle_call_tool(request.params),
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(res) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(res),
            error: None,
        },
        Err(err) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(err),
        },
    }
}

pub fn run_server() -> io::Result<()> {
    eprintln!("Iris MCP Server 启动中...");
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        eprintln!("Received: {}", line);

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = handle_request(request);
                let response_json = serde_json::to_string(&response)?;
                eprintln!("Sending: {}", response_json);
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}
