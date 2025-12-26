//! 跨平台鼠标监控实现（使用 rdev 事件驱动）
//! 基于操作系统原生事件机制，零 CPU 占用

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use serde::Serialize;
use rdev::{listen, Button, Event, EventType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ButtonState {
    Press,
    Release,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MouseEventKind {
    Move { x: i32, y: i32 },
    Button { button: MouseButton, state: ButtonState },
    Scroll { delta_x: i32, delta_y: i32 },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub timestamp_micros: u128,
}

#[derive(Debug)]
pub enum MonitorError {
    UnsupportedPlatform(&'static str),
    NotImplemented(&'static str),
    Io(String),
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorError::UnsupportedPlatform(p) => write!(f, "mouse monitor unsupported on {}", p),
            MonitorError::NotImplemented(msg) => write!(f, "mouse monitor not implemented: {}", msg),
            MonitorError::Io(msg) => write!(f, "mouse monitor io error: {}", msg),
        }
    }
}

impl std::error::Error for MonitorError {}

pub struct MonitorHandle {
    _thread: Option<thread::JoinHandle<()>>,
}

pub fn start_monitor<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(MouseEvent) + Send + Sync + 'static,
{
    let handle = thread::Builder::new()
        .name("mouse-monitor".to_string())
        .spawn(move || {
            // rdev 使用事件驱动，只在有鼠标事件时才触发回调
            if let Err(error) = listen(move |event: Event| {
                if let Some(mouse_event) = process_mouse_event(event) {
                    on_event(mouse_event);
                }
            }) {
                eprintln!("rdev listen error: {:?}", error);
            }
        })
        .map_err(|e| MonitorError::Io(e.to_string()))?;
    
    Ok(MonitorHandle {
        _thread: Some(handle),
    })
}

fn process_mouse_event(event: Event) -> Option<MouseEvent> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    
    match event.event_type {
        EventType::MouseMove { x, y } => Some(MouseEvent {
            kind: MouseEventKind::Move {
                x: x as i32,
                y: y as i32,
            },
            timestamp_micros: timestamp,
        }),
        EventType::ButtonPress(button) => Some(MouseEvent {
            kind: MouseEventKind::Button {
                button: map_button(button),
                state: ButtonState::Press,
            },
            timestamp_micros: timestamp,
        }),
        EventType::ButtonRelease(button) => Some(MouseEvent {
            kind: MouseEventKind::Button {
                button: map_button(button),
                state: ButtonState::Release,
            },
            timestamp_micros: timestamp,
        }),
        EventType::Wheel { delta_x, delta_y } => Some(MouseEvent {
            kind: MouseEventKind::Scroll {
                delta_x: delta_x as i32,
                delta_y: delta_y as i32,
            },
            timestamp_micros: timestamp,
        }),
        _ => None, // 忽略非鼠标事件
    }
}

fn map_button(button: Button) -> MouseButton {
    match button {
        Button::Left => MouseButton::Left,
        Button::Right => MouseButton::Right,
        Button::Middle => MouseButton::Middle,
        Button::Unknown(n) => MouseButton::Other(n),
    }
}
