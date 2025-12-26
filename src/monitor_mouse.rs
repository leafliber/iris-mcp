//! Cross-platform mouse monitoring design (stubs per platform).
//! - macOS: plan to use CGEventTap (kCGHIDEventTap) for mouse moved/scroll/button.
//! - Windows: plan to use SetWindowsHookExW with WH_MOUSE_LL.
//! - Linux: plan to use evdev (preferred) or X11 pointer grabs; Wayland often disallows global hooks.
//! Currently returns NotImplemented per platform but compiles on all targets.

use std::fmt;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MouseEventKind {
    Move { x: i32, y: i32 },
    Button { button: MouseButton, state: ButtonState },
    Scroll { delta_x: i32, delta_y: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ButtonState {
    Press,
    Release,
}

#[derive(Debug, Clone, Serialize)]
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

pub struct MonitorHandle;

pub fn start_monitor<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(MouseEvent) + Send + 'static,
{
    platform::start(on_event)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(MouseEvent) + Send + 'static,
    {
        Err(MonitorError::NotImplemented(
            "macOS: implement CGEventTap for mouse events",
        ))
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(MouseEvent) + Send + 'static,
    {
        Err(MonitorError::NotImplemented(
            "Windows: implement WH_MOUSE_LL hook",
        ))
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(MouseEvent) + Send + 'static,
    {
        Err(MonitorError::NotImplemented(
            "Linux: implement evdev or X11 pointer grabs; Wayland likely unsupported",
        ))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(MouseEvent) + Send + 'static,
    {
        Err(MonitorError::UnsupportedPlatform(std::env::consts::OS))
    }
}
