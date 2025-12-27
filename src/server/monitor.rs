use super::jsonrpc::JsonRpcError;
use crate::monitor::key_mouse::{KeyEvent, KeyEventType, MouseEvent, MouseEventKind, MouseButton, ButtonState};
use crate::monitor::screen::{self, ScreenEvent, ScreenEventKind};
use crate::monitor::state as monitor_state;
use serde_json::{json, Value};

pub fn screen_event_to_json(evt: &ScreenEvent) -> Value {
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

pub fn handle_monitor_keyboard_events(arguments: &Value) -> Result<Value, JsonRpcError> {
    let cursor = arguments["cursor"].as_u64().unwrap_or(0) as usize;
    let state = monitor_state::ensure_keyboard_monitor_started()?;
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

pub fn handle_monitor_mouse_events(arguments: &Value) -> Result<Value, JsonRpcError> {
    let cursor = arguments["cursor"].as_u64().unwrap_or(0) as usize;
    let state = monitor_state::ensure_mouse_monitor_started()?;
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
