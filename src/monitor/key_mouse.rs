//! 跨平台键盘和鼠标监控实现（使用 rdev 事件驱动）
//! 基于操作系统原生事件机制，零 CPU 占用
//! 
//! rdev 的 listen() 函数是全局的，一次只能运行一个监听器。
//! 这个模块提供统一的事件分发机制。

use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use rdev::{listen, Event, EventType};
use serde::Serialize;

// ============================================================
// 键盘事件类型定义
// ============================================================

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

// ============================================================
// 鼠标事件类型定义
// ============================================================

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

// ============================================================
// 错误类型定义
// ============================================================

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
            MonitorError::UnsupportedPlatform(p) => write!(f, "monitor unsupported on {}", p),
            MonitorError::PermissionDenied(msg) => write!(f, "monitor permission denied: {}", msg),
            MonitorError::NotImplemented(msg) => write!(f, "monitor not implemented: {}", msg),
            MonitorError::Io(msg) => write!(f, "monitor io error: {}", msg),
        }
    }
}

impl std::error::Error for MonitorError {}

// ============================================================
// 监控句柄
// ============================================================

pub struct MonitorHandle {
    // 占位符，实际监听由统一监听器处理
}

// ============================================================
// 统一监听器实现
// ============================================================

type KeyboardCallback = Arc<Mutex<Option<Box<dyn Fn(KeyEvent) + Send + Sync + 'static>>>>;
type MouseCallback = Arc<Mutex<Option<Box<dyn Fn(MouseEvent) + Send + Sync + 'static>>>>;

struct UnifiedMonitor {
    keyboard_callback: KeyboardCallback,
    mouse_callback: MouseCallback,
}

static GLOBAL_MONITOR: OnceLock<UnifiedMonitor> = OnceLock::new();

impl UnifiedMonitor {
    /// 获取或初始化全局监听器
    fn global() -> &'static Self {
        GLOBAL_MONITOR.get_or_init(|| {
            let keyboard_callback = Arc::new(Mutex::new(None));
            let mouse_callback = Arc::new(Mutex::new(None));
            
            // 启动统一的事件监听线程
            thread::Builder::new()
                .name("key-mouse-monitor".to_string())
                .spawn(move || {
                    if let Err(error) = listen(move |event: Event| {
                        // 从全局实例获取最新的回调
                        if let Some(monitor) = GLOBAL_MONITOR.get() {
                            Self::dispatch_event(
                                event,
                                monitor.keyboard_callback.clone(),
                                monitor.mouse_callback.clone(),
                            );
                        }
                    }) {
                        eprintln!("[monitor_key_mouse] rdev listen error: {:?}", error);
                    }
                })
                .expect("Failed to start key-mouse monitor thread");
            
            UnifiedMonitor {
                keyboard_callback,
                mouse_callback,
            }
        })
    }
    
    /// 设置键盘事件回调
    fn set_keyboard_callback<F>(&self, callback: F)
    where
        F: Fn(KeyEvent) + Send + Sync + 'static,
    {
        let mut guard = self.keyboard_callback.lock().unwrap();
        *guard = Some(Box::new(callback));
    }
    
    /// 设置鼠标事件回调
    fn set_mouse_callback<F>(&self, callback: F)
    where
        F: Fn(MouseEvent) + Send + Sync + 'static,
    {
        let mut guard = self.mouse_callback.lock().unwrap();
        *guard = Some(Box::new(callback));
    }
    
    /// 分发事件到相应的回调
    fn dispatch_event(
        event: Event,
        keyboard_callback: KeyboardCallback,
        mouse_callback: MouseCallback,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros())
            .unwrap_or(0);
        
        match event.event_type {
            // 键盘事件
            EventType::KeyPress(key) => {
                if let Some(callback) = keyboard_callback.lock().unwrap().as_ref() {
                    callback(KeyEvent {
                        key: key_to_string(key),
                        event_type: KeyEventType::Press,
                        timestamp_micros: timestamp,
                    });
                }
            }
            EventType::KeyRelease(key) => {
                if let Some(callback) = keyboard_callback.lock().unwrap().as_ref() {
                    callback(KeyEvent {
                        key: key_to_string(key),
                        event_type: KeyEventType::Release,
                        timestamp_micros: timestamp,
                    });
                }
            }
            
            // 鼠标事件
            EventType::MouseMove { x, y } => {
                if let Some(callback) = mouse_callback.lock().unwrap().as_ref() {
                    callback(MouseEvent {
                        kind: MouseEventKind::Move {
                            x: x as i32,
                            y: y as i32,
                        },
                        timestamp_micros: timestamp,
                    });
                }
            }
            EventType::ButtonPress(button) => {
                if let Some(callback) = mouse_callback.lock().unwrap().as_ref() {
                    callback(MouseEvent {
                        kind: MouseEventKind::Button {
                            button: map_button(button),
                            state: ButtonState::Press,
                        },
                        timestamp_micros: timestamp,
                    });
                }
            }
            EventType::ButtonRelease(button) => {
                if let Some(callback) = mouse_callback.lock().unwrap().as_ref() {
                    callback(MouseEvent {
                        kind: MouseEventKind::Button {
                            button: map_button(button),
                            state: ButtonState::Release,
                        },
                        timestamp_micros: timestamp,
                    });
                }
            }
            EventType::Wheel { delta_x, delta_y } => {
                if let Some(callback) = mouse_callback.lock().unwrap().as_ref() {
                    callback(MouseEvent {
                        kind: MouseEventKind::Scroll {
                            delta_x: delta_x as i32,
                            delta_y: delta_y as i32,
                        },
                        timestamp_micros: timestamp,
                    });
                }
            }
        }
    }
}

// ============================================================
// 公共 API
// ============================================================

/// 启动键盘监控
pub fn start_keyboard_monitor<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(KeyEvent) + Send + Sync + 'static,
{
    let monitor = UnifiedMonitor::global();
    monitor.set_keyboard_callback(on_event);
    Ok(MonitorHandle {})
}

/// 启动鼠标监控
pub fn start_mouse_monitor<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(MouseEvent) + Send + Sync + 'static,
{
    let monitor = UnifiedMonitor::global();
    monitor.set_mouse_callback(on_event);
    Ok(MonitorHandle {})
}

// ============================================================
// 辅助函数
// ============================================================

fn key_to_string(key: rdev::Key) -> String {
    use rdev::Key;
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

fn map_button(button: rdev::Button) -> MouseButton {
    use rdev::Button;
    match button {
        Button::Left => MouseButton::Left,
        Button::Right => MouseButton::Right,
        Button::Middle => MouseButton::Middle,
        Button::Unknown(n) => MouseButton::Other(n),
    }
}
