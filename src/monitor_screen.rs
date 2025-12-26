#![allow(dead_code)]

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

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use core_graphics::display::CGDisplay;
    use core_graphics::image::CGImage;
    use std::sync::Arc;

    pub fn start<F>(_on_event: F) -> Result<MonitorHandle, MonitorError>
    where
        F: Fn(ScreenEvent) + Send + Sync + 'static,
    {
        // Capture a screenshot of the main display immediately before spawning thread
        let on_event = Arc::new(_on_event);
        
        // Immediately capture one frame before returning
        if let Some(event) = capture_main_display_frame() {
            on_event(event);
        }
        
        // Spawn a background thread for future captures (if needed)
        let handle = thread::Builder::new()
            .name("screen-monitor-macos".to_string())
            .spawn({
                let _on_event = Arc::clone(&on_event);
                move || {
                    // Background thread can periodically capture or wait for changes
                    // For now, just complete immediately since we already captured once
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            })
            .map_err(|e| MonitorError::Io(e.to_string()))?;

        Ok(MonitorHandle { thread: Some(handle) })
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
}
