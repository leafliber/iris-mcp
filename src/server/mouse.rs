use super::jsonrpc::JsonRpcError;
use crate::operator::mouse::MouseController;
use enigo::{Button, Direction, Enigo, Settings};
use serde_json::{json, Value};

pub fn parse_button(s: &str) -> Result<Button, JsonRpcError> {
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

pub fn handle_mouse_move(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_mouse_click(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_mouse_double_click(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_mouse_scroll(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_mouse_get_position(_arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_mouse_drag(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_mouse_button_control(arguments: &Value) -> Result<Value, JsonRpcError> {
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

pub fn handle_mouse_move_path(arguments: &Value) -> Result<Value, JsonRpcError> {
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
