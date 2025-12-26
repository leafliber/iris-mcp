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
    F: Fn(KeyEvent) + Send + Sync + 'static,
{
    platform::start(on_event)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use core_graphics::event::{
        CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, 
        CGEventTapPlacement, CGEventType, EventField, CallbackResult,
    };
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn start<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(KeyEvent) + Send + Sync + 'static,
    {
        let callback = Arc::new(on_event);
        let callback_clone = callback.clone();

        std::thread::Builder::new()
            .name("keyboard-monitor-macos".to_string())
            .spawn(move || {
                let event_mask = vec![
                    CGEventType::KeyDown,
                    CGEventType::KeyUp,
                    CGEventType::FlagsChanged,
                ];

                let tap_result = CGEventTap::new(
                    CGEventTapLocation::HID,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::ListenOnly,
                    event_mask,
                    move |_proxy, event_type, event| {
                        if let Some(key_event) = map_cg_event_to_key_event(event_type, &event) {
                            callback_clone(key_event);
                        }
                        CallbackResult::Keep
                    },
                );

                match tap_result {
                    Ok(tap) => {
                        let loop_source = tap
                            .mach_port()
                            .create_runloop_source(0)
                            .expect("Failed to create runloop source");

                        let current_loop = CFRunLoop::get_current();
                        unsafe {
                            current_loop.add_source(&loop_source, kCFRunLoopCommonModes);
                        }

                        tap.enable();
                        CFRunLoop::run_current();
                    }
                    Err(()) => {
                        eprintln!("Failed to create CGEventTap - check accessibility permissions");
                    }
                }
            })
            .map_err(|e| MonitorError::Io(e.to_string()))?;

        Ok(MonitorHandle)
    }

    fn map_cg_event_to_key_event(event_type: CGEventType, event: &CGEvent) -> Option<KeyEvent> {
        let timestamp_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros())
            .unwrap_or(0);

        let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
        
        match event_type {
            CGEventType::KeyDown => {
                let code = map_keycode_to_key_code(keycode);
                Some(KeyEvent {
                    code,
                    state: KeyState::Press,
                    timestamp_micros,
                })
            }
            CGEventType::KeyUp => {
                let code = map_keycode_to_key_code(keycode);
                Some(KeyEvent {
                    code,
                    state: KeyState::Release,
                    timestamp_micros,
                })
            }
            CGEventType::FlagsChanged => {
                // Modifier keys (shift, ctrl, cmd, etc.) are reported via FlagsChanged
                let code = map_keycode_to_key_code(keycode);
                let flags = event.get_flags();
                // Check if the modifier is pressed or released based on flags
                let state = if is_modifier_pressed(keycode, flags) {
                    KeyState::Press
                } else {
                    KeyState::Release
                };
                Some(KeyEvent {
                    code,
                    state,
                    timestamp_micros,
                })
            }
            _ => None,
        }
    }

    fn map_keycode_to_key_code(keycode: u16) -> KeyCode {
        // Map common macOS keycodes to named keys or characters
        match keycode {
            // Letters (A-Z)
            0x00 => KeyCode::Char('a'),
            0x01 => KeyCode::Char('s'),
            0x02 => KeyCode::Char('d'),
            0x03 => KeyCode::Char('f'),
            0x04 => KeyCode::Char('h'),
            0x05 => KeyCode::Char('g'),
            0x06 => KeyCode::Char('z'),
            0x07 => KeyCode::Char('x'),
            0x08 => KeyCode::Char('c'),
            0x09 => KeyCode::Char('v'),
            0x0B => KeyCode::Char('b'),
            0x0C => KeyCode::Char('q'),
            0x0D => KeyCode::Char('w'),
            0x0E => KeyCode::Char('e'),
            0x0F => KeyCode::Char('r'),
            0x10 => KeyCode::Char('y'),
            0x11 => KeyCode::Char('t'),
            0x12 => KeyCode::Char('1'),
            0x13 => KeyCode::Char('2'),
            0x14 => KeyCode::Char('3'),
            0x15 => KeyCode::Char('4'),
            0x16 => KeyCode::Char('6'),
            0x17 => KeyCode::Char('5'),
            0x18 => KeyCode::Char('='),
            0x19 => KeyCode::Char('9'),
            0x1A => KeyCode::Char('7'),
            0x1B => KeyCode::Char('-'),
            0x1C => KeyCode::Char('8'),
            0x1D => KeyCode::Char('0'),
            0x1E => KeyCode::Char(']'),
            0x1F => KeyCode::Char('o'),
            0x20 => KeyCode::Char('u'),
            0x21 => KeyCode::Char('['),
            0x22 => KeyCode::Char('i'),
            0x23 => KeyCode::Char('p'),
            0x25 => KeyCode::Char('l'),
            0x26 => KeyCode::Char('j'),
            0x27 => KeyCode::Char('\''),
            0x28 => KeyCode::Char('k'),
            0x29 => KeyCode::Char(';'),
            0x2A => KeyCode::Char('\\'),
            0x2B => KeyCode::Char(','),
            0x2C => KeyCode::Char('/'),
            0x2D => KeyCode::Char('n'),
            0x2E => KeyCode::Char('m'),
            0x2F => KeyCode::Char('.'),
            0x32 => KeyCode::Char('`'),
            
            // Special keys
            0x24 => KeyCode::Named("enter"),
            0x4C => KeyCode::Named("enter"), // numpad enter
            0x30 => KeyCode::Named("tab"),
            0x31 => KeyCode::Named("space"),
            0x33 => KeyCode::Named("backspace"),
            0x35 => KeyCode::Named("escape"),
            0x37 => KeyCode::Named("meta"), // Command/Cmd
            0x38 => KeyCode::Named("shift"), // Left Shift
            0x3C => KeyCode::Named("shift"), // Right Shift
            0x3B => KeyCode::Named("control"), // Left Control
            0x3E => KeyCode::Named("control"), // Right Control
            0x3A => KeyCode::Named("alt"), // Left Alt/Option
            0x3D => KeyCode::Named("alt"), // Right Alt/Option
            0x39 => KeyCode::Named("capslock"),
            
            // Function keys
            0x7A => KeyCode::Named("f1"),
            0x78 => KeyCode::Named("f2"),
            0x63 => KeyCode::Named("f3"),
            0x76 => KeyCode::Named("f4"),
            0x60 => KeyCode::Named("f5"),
            0x61 => KeyCode::Named("f6"),
            0x62 => KeyCode::Named("f7"),
            0x64 => KeyCode::Named("f8"),
            0x65 => KeyCode::Named("f9"),
            0x6D => KeyCode::Named("f10"),
            0x67 => KeyCode::Named("f11"),
            0x6F => KeyCode::Named("f12"),
            
            // Arrow keys
            0x7E => KeyCode::Named("up"),
            0x7D => KeyCode::Named("down"),
            0x7B => KeyCode::Named("left"),
            0x7C => KeyCode::Named("right"),
            
            // Navigation keys
            0x73 => KeyCode::Named("home"),
            0x77 => KeyCode::Named("end"),
            0x74 => KeyCode::Named("pageup"),
            0x79 => KeyCode::Named("pagedown"),
            0x75 => KeyCode::Named("delete"), // Forward delete
            
            _ => KeyCode::ScanCode(keycode),
        }
    }

    fn is_modifier_pressed(keycode: u16, flags: core_graphics::event::CGEventFlags) -> bool {
        use core_graphics::event::CGEventFlags;
        
        match keycode {
            0x37 => flags.contains(CGEventFlags::CGEventFlagCommand), // Command
            0x38 | 0x3C => flags.contains(CGEventFlags::CGEventFlagShift), // Shift
            0x3B | 0x3E => flags.contains(CGEventFlags::CGEventFlagControl), // Control
            0x3A | 0x3D => flags.contains(CGEventFlags::CGEventFlagAlternate), // Alt/Option
            0x39 => flags.contains(CGEventFlags::CGEventFlagAlphaShift), // Caps Lock
            _ => false,
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(KeyEvent) + Send + Sync + 'static,
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
        F: Fn(KeyEvent) + Send + Sync + 'static,
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
