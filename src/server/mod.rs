pub mod jsonrpc;
pub mod keyboard;
pub mod monitor;
pub mod mouse;
pub mod tools_list;

use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::monitor::key_mouse;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn sanitize_id(id: Option<Value>) -> Value {
    match id {
        Some(v) if !v.is_null() => v,
        _ => json!(0),
    }
}

fn default_initialize_request() -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(0)),
        method: "initialize".to_string(),
        params: None,
    }
}

fn handle_initialize(_params: Option<Value>) -> Value {
    // 启动键盘和鼠标事件监控系统
    key_mouse::initialize();
    
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
    tools_list::get_tools_list()
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
        // 鼠标操作
        "mouse_move" => mouse::handle_mouse_move(arguments),
        "mouse_click" => mouse::handle_mouse_click(arguments),
        "mouse_double_click" => mouse::handle_mouse_double_click(arguments),
        "mouse_scroll" => mouse::handle_mouse_scroll(arguments),
        "mouse_get_position" => mouse::handle_mouse_get_position(arguments),
        "mouse_drag" => mouse::handle_mouse_drag(arguments),
        "mouse_button_control" => mouse::handle_mouse_button_control(arguments),
        "mouse_move_path" => mouse::handle_mouse_move_path(arguments),
        
        // 键盘操作
        "type_text" => keyboard::handle_type_text(arguments),
        "system_command" => keyboard::handle_system_command(arguments),
        "key_control" => keyboard::handle_key_control(arguments),
        
        // 监控操作
        "monitor_screen_events" => monitor::handle_monitor_screen_events(arguments),
        "monitor_keyboard_events" => monitor::handle_monitor_keyboard_events(arguments),
        "monitor_mouse_events" => monitor::handle_monitor_mouse_events(arguments),
        
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

        // 一些客户端在握手时发送空对象 {}，在此兼容为 initialize 请求
        let parsed_req = if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&line) {
            if map.is_empty() {
                Ok(default_initialize_request())
            } else {
                serde_json::from_value::<JsonRpcRequest>(Value::Object(map))
            }
        } else {
            serde_json::from_str::<JsonRpcRequest>(&line)
        };

        match parsed_req {
            Ok(request) => {
                let id = sanitize_id(request.id.clone());
                let response = handle_request(request);
                // Ensure id is always string/number to satisfy strict clients
                let response = JsonRpcResponse {
                    id: Some(id),
                    ..response
                };
                let response_json = serde_json::to_string(&response)?;
                eprintln!("Sending: {}", response_json);
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                // Some clients reject `null` ids; use 0 to conform to string/number schema.
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(json!(0)),
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
