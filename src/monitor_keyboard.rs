//! Cross-platform keyboard monitoring design.
//! - macOS: plan to use CGEventTap (kCGHIDEventTap) to capture key down/up.
//! - Windows: plan to use WH_KEYBOARD_LL with SetWindowsHookExW.
//! - Linux: plan to use evdev (preferred) or X11 grabs; Wayland often blocks global hooks.
//! Currently returns NotImplemented per platform but compiles on all targets with per-OS stubs.

use std::fmt;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum KeyState {
    Press,
    Release,
    Repeat,
}

#[derive(Debug, Clone, Serialize)]
pub enum KeyCode {
    Char(char),
    Named(&'static str), // e.g., "enter", "shift", "ctrl", function keys, arrows
    ScanCode(u16),
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub state: KeyState,
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
            MonitorError::UnsupportedPlatform(p) => write!(f, "keyboard monitor unsupported on {}", p),
            MonitorError::NotImplemented(msg) => write!(f, "keyboard monitor not implemented: {}", msg),
            MonitorError::Io(msg) => write!(f, "keyboard monitor io error: {}", msg),
        }
    }
}

impl std::error::Error for MonitorError {}

pub struct MonitorHandle;

pub fn start_monitor<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(KeyEvent) + Send + 'static,
{
    platform::start(on_event)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(KeyEvent) + Send + 'static,
    {
        // Placeholder: actual implementation would spawn a CGEventTap runloop and map events.
        Err(MonitorError::NotImplemented("macOS: implement CGEventTap for key events"))
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(KeyEvent) + Send + 'static,
    {
        // Placeholder: actual implementation would install WH_KEYBOARD_LL with SetWindowsHookExW.
        Err(MonitorError::NotImplemented("Windows: implement WH_KEYBOARD_LL hook"))
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(KeyEvent) + Send + 'static,
    {
        // Placeholder: actual implementation would read from evdev (preferred) or X11 grabs.
        Err(MonitorError::NotImplemented(
            "Linux: implement evdev or X11; Wayland likely unsupported",
        ))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(KeyEvent) + Send + 'static,
    {
        Err(MonitorError::UnsupportedPlatform(std::env::consts::OS))
    }
}
