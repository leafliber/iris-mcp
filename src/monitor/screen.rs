//! Cross-platform screen monitoring design (stubs per platform).
//! Goals: detect display topology/geometry changes or periodic frame capture events.
//! Current state: per-platform stubs returning NotImplemented but compiling everywhere.

use serde::Serialize;
use std::fmt;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ScreenEventKind {
    GeometryChanged { width: u32, height: u32, scale: f32 },
    DisplayAdded,
    DisplayRemoved,
    FrameCaptured { width: u32, height: u32, format: FrameFormat },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FrameFormat {
    Rgba8,
    Bgra8,
    Nv12,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScreenEvent {
    pub kind: ScreenEventKind,
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
            MonitorError::UnsupportedPlatform(p) => write!(f, "screen monitor unsupported on {}", p),
            MonitorError::NotImplemented(msg) => write!(f, "screen monitor not implemented: {}", msg),
            MonitorError::Io(msg) => write!(f, "screen monitor io error: {}", msg),
        }
    }
}
 
impl std::error::Error for MonitorError {}

pub struct MonitorHandle {
    #[cfg(target_os = "macos")]
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for MonitorHandle {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

pub fn start_monitor<F>(on_event: F) -> Result<MonitorHandle, MonitorError>
where
    F: Fn(ScreenEvent) + Send + Sync + 'static,
{
    platform::start(on_event)
}

/// 按需捕获一帧屏幕截图（不启动持续监控）
pub fn capture_frame() -> Result<ScreenEvent, MonitorError> {
    platform::capture_frame()
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use core_graphics::display::CGDisplay;
    use core_graphics::image::CGImage;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(ScreenEvent) + Send + Sync + 'static,
    {
        // 屏幕监控不再持续运行，而是按需调用 capture_frame()
        // 这里保留接口但立即返回
        let handle = thread::Builder::new()
            .name("screen-monitor-macos".to_string())
            .spawn(move || {
                // 空线程，立即完成
            })
            .map_err(|e| MonitorError::Io(e.to_string()))?;

        Ok(MonitorHandle { thread: Some(handle) })
    }

    /// 按需捕获主显示器的一帧截图
    pub fn capture_frame() -> Result<ScreenEvent, MonitorError> {
        capture_main_display_frame()
            .ok_or_else(|| MonitorError::Io("Failed to capture screen frame".to_string()))
    }

    fn capture_main_display_frame() -> Option<ScreenEvent> {
        let main = CGDisplay::main();
        let image: CGImage = main.image()?;

        let width = image.width() as u32;
        let height = image.height() as u32;

        Some(ScreenEvent {
            kind: ScreenEventKind::FrameCaptured {
                width,
                height,
                format: FrameFormat::Bgra8,
            },
            timestamp_micros: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_micros())
                .unwrap_or(0),
        })
    }

}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(ScreenEvent) + Send + Sync + 'static,
    {
        Err(MonitorError::NotImplemented(
            "Windows: implement DXGI output duplication or display change notifications",
        ))
    }

    pub fn capture_frame() -> Result<ScreenEvent, MonitorError> {
        Err(MonitorError::NotImplemented(
            "Windows: implement screenshot capture",
        ))
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(ScreenEvent) + Send + 'static,
    {
        Err(MonitorError::NotImplemented(
            "Linux: implement DRM/GBM or X11 RandR events; Wayland likely restricted",
        ))
    }

    pub fn capture_frame() -> Result<ScreenEvent, MonitorError> {
        Err(MonitorError::NotImplemented(
            "Linux: implement screenshot capture",
        ))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod platform {
    use super::*;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(ScreenEvent) + Send + 'static,
    {
        Err(MonitorError::UnsupportedPlatform(std::env::consts::OS))
    }

    pub fn capture_frame() -> Result<ScreenEvent, MonitorError> {
        Err(MonitorError::UnsupportedPlatform(std::env::consts::OS))
    }
}
