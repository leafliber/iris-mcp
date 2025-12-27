//! Cross-platform screen monitoring design (stubs per platform).
//! Goals: detect display topology/geometry changes or periodic frame capture events.
//! Current state: per-platform stubs returning NotImplemented but compiling everywhere.

use serde::Serialize;
use std::fmt;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ScreenEventKind {
    GeometryChanged { width: u32, height: u32, scale: f32 },
    DisplayAdded,
    DisplayRemoved,
    FrameCaptured { 
        width: u32, 
        height: u32, 
        format: FrameFormat,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_data: Option<Vec<u8>>,
    },
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
        let cg_image: CGImage = main.image()?;

        let width = cg_image.width() as u32;
        let height = cg_image.height() as u32;

        // 将 CGImage 转换为 PNG
        let image_data = cgimage_to_png(&cg_image, width, height)?;

        Some(ScreenEvent {
            kind: ScreenEventKind::FrameCaptured {
                width,
                height,
                format: FrameFormat::Bgra8,
                image_data: Some(image_data),
            },
            timestamp_micros: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_micros())
                .unwrap_or(0),
        })
    }

    /// 将 CGImage 转换为 PNG 字节
    fn cgimage_to_png(cg_image: &CGImage, width: u32, height: u32) -> Option<Vec<u8>> {
        use image::{ImageBuffer, RgbaImage, ImageFormat};
        use std::io::Cursor;
        use std::os::raw::c_void;
        use core_graphics::color_space::CGColorSpace;
        use core_graphics::context::CGContext;
        use core_graphics::geometry::CGRect;

        // 创建位图上下文来提取像素数据
        let bytes_per_pixel = 4;
        let bytes_per_row = bytes_per_pixel * width as usize;
        let buffer_size = bytes_per_row * height as usize;
        let mut buffer: Vec<u8> = vec![0; buffer_size];

        // 使用 Core Graphics 创建位图上下文并绘制图像
        let color_space = CGColorSpace::create_device_rgb();
        let bitmap_info = core_graphics::base::kCGImageAlphaPremultipliedLast 
            | core_graphics::base::kCGBitmapByteOrder32Big;

        let context = CGContext::create_bitmap_context(
            Some(buffer.as_mut_ptr() as *mut c_void),
            width as usize,
            height as usize,
            8,
            bytes_per_row,
            &color_space,
            bitmap_info,
        );

        let rect = CGRect::new(
            &core_graphics::geometry::CGPoint::new(0.0, 0.0),
            &core_graphics::geometry::CGSize::new(width as f64, height as f64),
        );

        context.draw_image(rect, cg_image);

        // 现在 buffer 包含 RGBA 数据，转换为 PNG
        let rgba_image: RgbaImage = ImageBuffer::from_raw(width, height, buffer)?;

        // 编码为 PNG
        let mut png_data = Vec::new();
        let mut cursor = Cursor::new(&mut png_data);
        rgba_image.write_to(&mut cursor, ImageFormat::Png).ok()?;
        
        Some(png_data)
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
