//! 跨平台键盘监控实现（使用 rdev 事件驱动）
//! 基于操作系统原生事件机制，零 CPU 占用

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use serde::Serialize;
use rdev::{listen, Event, EventType, Key};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum KeyEventType {
    Press,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyEvent {
    pub key: String,
    pub event_type: KeyEventType,
    pub timestamp_micros: u128,
}

#[derive(Debug)]
pub enum MonitorError {
    UnsupportedPlatform(&'static str),
    PermissionDenied(&'static str),
    NotImplemented(&'static str),
    Io(String),
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorError::UnsupportedPlatform(p) => write!(f, "keyboard monitor unsupported on {}", p),
            MonitorError::PermissionDenied(msg) => write!(f, "keyboard monitor permission denied: {}", msg),
            MonitorError::NotImplemented(msg) => write!(f, "keyboard monitor not implemented: {}", msg),
            MonitorError::Io(msg) => write!(f, "keyboard monitor io error: {}", msg),
        }
    }
}

impl std::error::Error for MonitorError {}

pub struct MonitorHandle {
    _thread: Option<thread::JoinHandle<()>>,
}

pub fn start_monitor<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(KeyEvent) + Send + Sync + 'static,
{
    let handle = thread::Builder::new()
        .name("keyboard-monitor".to_string())
        .spawn(move || {
            // rdev 使用事件驱动，只在有键盘事件时才触发回调
            if let Err(error) = listen(move |event: Event| {
                if let Some(key_event) = process_keyboard_event(event) {
                    on_event(key_event);
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

fn process_keyboard_event(event: Event) -> Option<KeyEvent> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    
    match event.event_type {
        EventType::KeyPress(key) => Some(KeyEvent {
            key: key_to_string(key),
            event_type: KeyEventType::Press,
            timestamp_micros: timestamp,
        }),
        EventType::KeyRelease(key) => Some(KeyEvent {
            key: key_to_string(key),
            event_type: KeyEventType::Release,
            timestamp_micros: timestamp,
        }),
        _ => None, // 忽略非键盘事件
    }
}

fn key_to_string(key: Key) -> String {
    match key {
        Key::Num0 => "0".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        Key::KeyA => "A".to_string(),
        Key::KeyB => "B".to_string(),
        Key::KeyC => "C".to_string(),
        Key::KeyD => "D".to_string(),
        Key::KeyE => "E".to_string(),
        Key::KeyF => "F".to_string(),
        Key::KeyG => "G".to_string(),
        Key::KeyH => "H".to_string(),
        Key::KeyI => "I".to_string(),
        Key::KeyJ => "J".to_string(),
        Key::KeyK => "K".to_string(),
        Key::KeyL => "L".to_string(),
        Key::KeyM => "M".to_string(),
        Key::KeyN => "N".to_string(),
        Key::KeyO => "O".to_string(),
        Key::KeyP => "P".to_string(),
        Key::KeyQ => "Q".to_string(),
        Key::KeyR => "R".to_string(),
        Key::KeyS => "S".to_string(),
        Key::KeyT => "T".to_string(),
        Key::KeyU => "U".to_string(),
        Key::KeyV => "V".to_string(),
        Key::KeyW => "W".to_string(),
        Key::KeyX => "X".to_string(),
        Key::KeyY => "Y".to_string(),
        Key::KeyZ => "Z".to_string(),
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        Key::Escape => "Escape".to_string(),
        Key::Space => "Space".to_string(),
        Key::ControlLeft => "LeftControl".to_string(),
        Key::ControlRight => "RightControl".to_string(),
        Key::ShiftLeft => "LeftShift".to_string(),
        Key::ShiftRight => "RightShift".to_string(),
        Key::Alt => "Alt".to_string(),
        Key::AltGr => "AltGr".to_string(),
        Key::MetaLeft => "LeftMeta".to_string(),
        Key::MetaRight => "RightMeta".to_string(),
        Key::Return => "Enter".to_string(),
        Key::UpArrow => "Up".to_string(),
        Key::DownArrow => "Down".to_string(),
        Key::LeftArrow => "Left".to_string(),
        Key::RightArrow => "Right".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::CapsLock => "CapsLock".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::KpMinus => "NumpadSubtract".to_string(),
        Key::KpPlus => "NumpadAdd".to_string(),
        Key::KpDivide => "NumpadDivide".to_string(),
        Key::KpMultiply => "NumpadMultiply".to_string(),
        Key::BackQuote => "Grave".to_string(),
        Key::Minus => "Minus".to_string(),
        Key::Equal => "Equal".to_string(),
        Key::LeftBracket => "LeftBracket".to_string(),
        Key::RightBracket => "RightBracket".to_string(),
        Key::BackSlash => "BackSlash".to_string(),
        Key::SemiColon => "Semicolon".to_string(),
        Key::Quote => "Apostrophe".to_string(),
        Key::Comma => "Comma".to_string(),
        Key::Dot => "Dot".to_string(),
        Key::Slash => "Slash".to_string(),
        _ => format!("{:?}", key),
    }
}
