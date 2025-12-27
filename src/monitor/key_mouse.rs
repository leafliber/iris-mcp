//! 跨平台键盘和鼠标监控实现（使用 rdev 事件驱动）
//! 基于操作系统原生事件机制，零 CPU 占用
//! 
//! 启动时自动开始监控，将事件存储在 FIFO 队列中。
//! MCP 协议调用时返回存储的事件并清空队列。

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::env;
use std::fs;
use std::path::PathBuf;
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
// 配置常量
// ============================================================

/// 最大存储的键盘事件数量
const MAX_KEYBOARD_EVENTS: usize = 100;

/// 最大存储的鼠标事件数量
const MAX_MOUSE_EVENTS: usize = 200;

/// 鼠标移动采样默认间隔（微秒）。
const DEFAULT_MOUSE_MOVE_INTERVAL_MICROS: u128 = 2_000; // 2ms

// ============================================================
// 事件存储
// ============================================================

struct EventStorage {
    keyboard_events: Arc<Mutex<VecDeque<KeyEvent>>>,
    mouse_events: Arc<Mutex<VecDeque<MouseEvent>>>,
}

impl EventStorage {
    fn new() -> Self {
        EventStorage {
            keyboard_events: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_KEYBOARD_EVENTS))),
            mouse_events: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_MOUSE_EVENTS))),
        }
    }
    
    /// 添加键盘事件，超过容量时移除最旧的事件
    fn push_keyboard_event(&self, event: KeyEvent) {
        let mut queue = self.keyboard_events.lock().unwrap();
        if queue.len() >= MAX_KEYBOARD_EVENTS {
            queue.pop_front();
        }
        queue.push_back(event);
    }
    
    /// 添加鼠标事件，超过容量时移除最旧的事件
    fn push_mouse_event(&self, event: MouseEvent) {
        let mut queue = self.mouse_events.lock().unwrap();
        if queue.len() >= MAX_MOUSE_EVENTS {
            queue.pop_front();
        }
        queue.push_back(event);
    }
    
    /// 获取所有键盘事件并清空队列
    fn take_keyboard_events(&self) -> Vec<KeyEvent> {
        let mut queue = self.keyboard_events.lock().unwrap();
        let events: Vec<KeyEvent> = queue.drain(..).collect();
        events
    }
    
    /// 获取所有鼠标事件并清空队列
    fn take_mouse_events(&self) -> Vec<MouseEvent> {
        let mut queue = self.mouse_events.lock().unwrap();
        let events: Vec<MouseEvent> = queue.drain(..).collect();
        events
    }
}

// ============================================================
// 统一监听器实现
// ============================================================

struct UnifiedMonitor {
    storage: Arc<EventStorage>,
    #[allow(dead_code)]
    last_mouse_move_micros: Arc<Mutex<u128>>,
    started: Arc<AtomicBool>,
    event_count: Arc<AtomicU64>,
}

static GLOBAL_MONITOR: OnceLock<UnifiedMonitor> = OnceLock::new();

impl UnifiedMonitor {
    /// 获取或初始化全局监听器
    fn global() -> &'static Self {
        GLOBAL_MONITOR.get_or_init(|| {
            let storage = Arc::new(EventStorage::new());
            let last_mouse_move_micros = Arc::new(Mutex::new(0u128));
            let started = Arc::new(AtomicBool::new(false));
            let event_count = Arc::new(AtomicU64::new(0));
            
            let pid = std::process::id();
            eprintln!("[monitor_key_mouse][PID:{}] Initializing event monitor...", pid);
            
            // 尝试获取全局锁
            if !try_acquire_lock() {
                eprintln!("[monitor_key_mouse][PID:{}] Another process is already monitoring. This process will not start a listener.", pid);
                // 不启动监听器，但返回有效的结构
                return UnifiedMonitor {
                    storage,
                    last_mouse_move_micros,
                    started, // 保持 false
                    event_count,
                };
            }
            
            let storage_clone = storage.clone();
            let last_mouse_move_micros_clone = last_mouse_move_micros.clone();
            let started_clone = started.clone();
            let event_count_clone = event_count.clone();
            
            // 启动统一的事件监听线程
            thread::Builder::new()
                .name("key-mouse-monitor".to_string())
                .spawn(move || {
                    eprintln!("[monitor_key_mouse][PID:{}] Starting rdev listen...", pid);
                    started_clone.store(true, Ordering::SeqCst);
                    
                    if let Err(error) = listen(move |event: Event| {
                        event_count_clone.fetch_add(1, Ordering::Relaxed);
                        Self::handle_event(
                            event,
                            storage_clone.clone(),
                            last_mouse_move_micros_clone.clone(),
                        );
                    }) {
                        eprintln!("[monitor_key_mouse][PID:{}] rdev listen error: {:?}", pid, error);
                        started_clone.store(false, Ordering::SeqCst);
                        release_lock();
                    }
                })
                .expect("Failed to start key-mouse monitor thread");
            
            // 等待一小段时间确保线程启动
            thread::sleep(std::time::Duration::from_millis(50));
            eprintln!("[monitor_key_mouse][PID:{}] Monitor initialization complete", pid);
            
            UnifiedMonitor {
                storage,
                last_mouse_move_micros,
                started,
                event_count,
            }
        })
    }
    
    /// 处理并存储事件
    fn handle_event(
        event: Event,
        storage: Arc<EventStorage>,
        last_mouse_move_micros: Arc<Mutex<u128>>,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros())
            .unwrap_or(0);
        
        match event.event_type {
            // 键盘事件
            EventType::KeyPress(key) => {
                storage.push_keyboard_event(KeyEvent {
                    key: key_to_string(key),
                    event_type: KeyEventType::Press,
                    timestamp_micros: timestamp,
                });
            }
            EventType::KeyRelease(key) => {
                storage.push_keyboard_event(KeyEvent {
                    key: key_to_string(key),
                    event_type: KeyEventType::Release,
                    timestamp_micros: timestamp,
                });
            }
            
            // 鼠标事件
            EventType::MouseMove { x, y } => {
                // 节流：仅在距离上次记录超过采样间隔时保存
                let mut last = last_mouse_move_micros.lock().unwrap();
                if timestamp.saturating_sub(*last) < mouse_move_interval_micros() {
                    return;
                }
                *last = timestamp;

                storage.push_mouse_event(MouseEvent {
                    kind: MouseEventKind::Move {
                        x: x as i32,
                        y: y as i32,
                    },
                    timestamp_micros: timestamp,
                });
            }
            EventType::ButtonPress(button) => {
                storage.push_mouse_event(MouseEvent {
                    kind: MouseEventKind::Button {
                        button: map_button(button),
                        state: ButtonState::Press,
                    },
                    timestamp_micros: timestamp,
                });
            }
            EventType::ButtonRelease(button) => {
                storage.push_mouse_event(MouseEvent {
                    kind: MouseEventKind::Button {
                        button: map_button(button),
                        state: ButtonState::Release,
                    },
                    timestamp_micros: timestamp,
                });
            }
            EventType::Wheel { delta_x, delta_y } => {
                storage.push_mouse_event(MouseEvent {
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

// ============================================================
// 公共 API
// ============================================================

/// 初始化监控系统（自动启动）
pub fn initialize() {
    // 触发全局监听器初始化
    let _ = UnifiedMonitor::global();
}

/// 获取所有键盘事件并清空存储
pub fn take_keyboard_events() -> Vec<KeyEvent> {
    let monitor = UnifiedMonitor::global();
    let events = monitor.storage.take_keyboard_events();
    let total_events = monitor.event_count.load(Ordering::Relaxed);
    let started = monitor.started.load(Ordering::SeqCst);
    eprintln!("[monitor_key_mouse][PID:{}] take_keyboard_events: returning {} events, started={}, total_processed={}", 
        std::process::id(), events.len(), started, total_events);
    events
}

/// 获取所有鼠标事件并清空存储
pub fn take_mouse_events() -> Vec<MouseEvent> {
    let monitor = UnifiedMonitor::global();
    let events = monitor.storage.take_mouse_events();
    let total_events = monitor.event_count.load(Ordering::Relaxed);
    let started = monitor.started.load(Ordering::SeqCst);
    eprintln!("[monitor_key_mouse][PID:{}] take_mouse_events: returning {} events, started={}, total_processed={}", 
        std::process::id(), events.len(), started, total_events);
    events
}

// ============================================================
// 兼容性 API（保持向后兼容）
// ============================================================

/// 监控句柄
pub struct MonitorHandle {
    // 占位符，实际监听由统一监听器处理
}

/// 启动键盘监控（已废弃，系统自动启动）
#[deprecated(note = "Monitor is now automatically started. Use take_keyboard_events() instead.")]
pub fn start_keyboard_monitor<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(KeyEvent) + Send + Sync + 'static,
{
    initialize();
    Ok(MonitorHandle {})
}

/// 启动鼠标监控（已废弃，系统自动启动）
#[deprecated(note = "Monitor is now automatically started. Use take_mouse_events() instead.")]
pub fn start_mouse_monitor<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(MouseEvent) + Send + Sync + 'static,
{
    initialize();
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

/// 获取鼠标移动采样间隔（微秒）。
/// 优先读取环境变量 IRIS_MOUSE_MOVE_INTERVAL_US，值需为正整数。
fn mouse_move_interval_micros() -> u128 {
    static INTERVAL: OnceLock<u128> = OnceLock::new();
    *INTERVAL.get_or_init(|| {
        env::var("IRIS_MOUSE_MOVE_INTERVAL_US")
            .ok()
            .and_then(|v| v.parse::<u128>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_MOUSE_MOVE_INTERVAL_MICROS)
    })
}

/// 获取监听器锁文件路径
fn get_lock_file_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("iris-mcp-monitor.lock");
    path
}

/// 尝试获取监听器锁，返回是否成功
fn try_acquire_lock() -> bool {
    let lock_path = get_lock_file_path();
    let pid = std::process::id();
    
    // 检查锁文件是否存在
    if lock_path.exists() {
        // 读取锁文件中的 PID
        if let Ok(content) = fs::read_to_string(&lock_path) {
            if let Ok(locked_pid) = content.trim().parse::<u32>() {
                // 检查该进程是否还活着（简单检查：如果是自己的 PID 就认为已锁定）
                if locked_pid == pid {
                    return true; // 已经是自己持有锁
                }
                eprintln!("[monitor_key_mouse][PID:{}] Lock file exists with PID:{}", pid, locked_pid);
                return false;
            }
        }
    }
    
    // 尝试创建锁文件
    match fs::write(&lock_path, pid.to_string()) {
        Ok(_) => {
            eprintln!("[monitor_key_mouse][PID:{}] Acquired lock at {:?}", pid, lock_path);
            true
        }
        Err(e) => {
            eprintln!("[monitor_key_mouse][PID:{}] Failed to acquire lock: {}", pid, e);
            false
        }
    }
}

/// 释放监听器锁
fn release_lock() {
    let lock_path = get_lock_file_path();
    let _ = fs::remove_file(&lock_path);
}
