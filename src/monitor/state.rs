use crate::server::jsonrpc::JsonRpcError;
use crate::monitor::key_mouse::{self, KeyEvent, MouseEvent};
use std::sync::{Arc, Mutex, OnceLock};

pub struct KeyboardMonitorState {
    pub events: Arc<Mutex<Vec<KeyEvent>>>,
    #[allow(dead_code)]
    handle: Option<key_mouse::MonitorHandle>,
}

pub struct MouseMonitorState {
    pub events: Arc<Mutex<Vec<MouseEvent>>>,
    #[allow(dead_code)]
    handle: Option<key_mouse::MonitorHandle>,
}

static KEYBOARD_STATE: OnceLock<Result<KeyboardMonitorState, String>> = OnceLock::new();
static MOUSE_STATE: OnceLock<Result<MouseMonitorState, String>> = OnceLock::new();

pub fn ensure_keyboard_monitor_started() -> Result<&'static KeyboardMonitorState, JsonRpcError> {
    KEYBOARD_STATE.get_or_init(|| {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            use std::sync::Arc;
            
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = Arc::clone(&events);
            
            match key_mouse::start_keyboard_monitor(move |evt| {
                if let Ok(mut guard) = events_clone.lock() {
                    guard.push(evt);
                }
            }) {
                Ok(handle) => {
                    Ok(KeyboardMonitorState {
                        events: events,
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

pub fn ensure_mouse_monitor_started() -> Result<&'static MouseMonitorState, JsonRpcError> {
    MOUSE_STATE.get_or_init(|| {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            use std::sync::Arc;
            
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = Arc::clone(&events);
            
            match key_mouse::start_mouse_monitor(move |evt| {
                if let Ok(mut guard) = events_clone.lock() {
                    guard.push(evt);
                }
            }) {
                Ok(handle) => {
                    Ok(MouseMonitorState {
                        events: events,
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
