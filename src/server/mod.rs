pub mod jsonrpc;
pub mod keyboard;
pub mod monitor;
pub mod mouse;
pub mod tools_list;

use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::monitor::state as monitor_state;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn handle_initialize(_params: Option<Value>) -> Value {
    // 自动启动键盘和鼠标监控（后台积累事件）
    let _ = monitor_state::ensure_keyboard_monitor_started();
    let _ = monitor_state::ensure_mouse_monitor_started();
    
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
