use super::jsonrpc::JsonRpcError;
use crate::operator::keyboard::{KeyboardController, SystemCommand};
use enigo::{Direction, Enigo, Key, Settings};
use serde_json::{json, Value};

pub fn handle_type_text(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_system_command(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_key_control(arguments: &Value) -> Result<Value, JsonRpcError> {
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
