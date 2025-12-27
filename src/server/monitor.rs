use super::jsonrpc::JsonRpcError;
use crate::monitor::key_mouse::{self, KeyEvent, KeyEventType, MouseEvent, MouseEventKind, MouseButton, ButtonState};
use crate::monitor::screen::{self, ScreenEvent, ScreenEventKind};
use serde_json::{json, Value};

pub fn screen_event_to_json(evt: &ScreenEvent) -> Value {
    let kind = match &evt.kind {
        ScreenEventKind::GeometryChanged { width, height, scale } => json!({
            "type": "geometry_changed",
            "width": width,
            "height": height,
            "scale": scale
        }),
        ScreenEventKind::DisplayAdded => json!({ "type": "display_added" }),
        ScreenEventKind::DisplayRemoved => json!({ "type": "display_removed" }),
        ScreenEventKind::FrameCaptured { width, height, format, image_data } => {
            let mut result = json!({
                "type": "frame_captured",
                "width": width,
                "height": height,
                "format": format,
            });
            if let Some(data) = image_data {
                result["has_image_data"] = json!(true);
                result["image_size_bytes"] = json!(data.len());
            }
            result
        },
    };

    json!({
        "timestamp_micros": evt.timestamp_micros,
        "kind": kind,
    })
}

pub fn keyboard_event_to_json(evt: &KeyEvent) -> Value {
    let event_type = match evt.event_type {
        KeyEventType::Press => "press",
        KeyEventType::Release => "release",
    };

    json!({
        "timestamp_micros": evt.timestamp_micros,
        "key": evt.key,
        "event_type": event_type,
    })
}

pub fn mouse_event_to_json(evt: &MouseEvent) -> Value {
    let kind = match evt.kind {
        MouseEventKind::Move { x, y } => json!({ "type": "move", "x": x, "y": y }),
        MouseEventKind::Button { button, state } => {
            let button = match button {
                MouseButton::Left => "left".to_string(),
                MouseButton::Middle => "middle".to_string(),
                MouseButton::Right => "right".to_string(),
                MouseButton::Other(v) => format!("other_{}", v),
            };
            let state = match state {
                ButtonState::Press => "press",
                ButtonState::Release => "release",
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

pub fn handle_monitor_screen_events(_arguments: &Value) -> Result<Value, JsonRpcError> {
    let event = screen::capture_frame().map_err(|e| JsonRpcError {
        code: -32001,
        message: e.to_string(),
        data: None,
    })?;

    // 提取图像数据
    let (width, height, image_data) = match &event.kind {
        ScreenEventKind::FrameCaptured { width, height, image_data, .. } => {
            (*width, *height, image_data.clone())
        }
        _ => {
            return Err(JsonRpcError {
                code: -32001,
                message: "Unexpected event type".to_string(),
                data: None,
            });
        }
    };

    let event_json = screen_event_to_json(&event);

    match image_data {
        Some(data) => {
            // 使用 base64 编码图像数据
            use base64::{Engine as _, engine::general_purpose};
            let base64_data = general_purpose::STANDARD.encode(&data);
            
            Ok(json!({
                "content": [
                    {
                        "type": "image",
                        "data": base64_data,
                        "mimeType": "image/png"
                    },
                    {
                        "type": "text",
                        "text": format!("已捕获屏幕截图\n尺寸: {}x{}\n大小: {} bytes", 
                            width, height, data.len())
                    }
                ]
            }))
        }
        None => {
            // 如果没有图像数据，返回事件信息
            let event_text = serde_json::to_string_pretty(&event_json)
                .unwrap_or_else(|_| event_json.to_string());
            
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("屏幕事件信息\n尺寸: {}x{}\n\n详情：\n{}", 
                            width, height, event_text)
                    }
                ]
            }))
        }
    }
}

pub fn handle_monitor_keyboard_events(_arguments: &Value) -> Result<Value, JsonRpcError> {
    // 获取所有键盘事件并清空存储
    let events = key_mouse::take_keyboard_events();
    
    let events_json: Vec<Value> = events.iter().map(keyboard_event_to_json).collect();
    let total = events.len();

    let result = json!({
        "events": events_json,
        "total": total
    });
    let result_text = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| result.to_string());

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("返回{}条键盘事件（已清空存储）\n\n事件数据：\n{}", 
                    total, result_text)
            }
        ]
    }))
}

pub fn handle_monitor_mouse_events(_arguments: &Value) -> Result<Value, JsonRpcError> {
    // 获取所有鼠标事件并清空存储
    let events = key_mouse::take_mouse_events();
    
    let events_json: Vec<Value> = events.iter().map(mouse_event_to_json).collect();
    let total = events.len();

    let result = json!({
        "events": events_json,
        "total": total
    });
    let result_text = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| result.to_string());

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("返回{}条鼠标事件（已清空存储）\n\n事件数据：\n{}", 
                    total, result_text)
            }
        ]
    }))
}
