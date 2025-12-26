#![cfg(target_os = "macos")]

use iris_mcp::monitor_screen::{capture_frame, FrameFormat, ScreenEventKind};

#[test]
fn macos_capture_frame_works() {
    // 测试按需截图功能
    let evt = capture_frame()
        .expect("capture_frame should succeed on macOS");
    
    match evt.kind {
        ScreenEventKind::FrameCaptured { width, height, format } => {
            assert!(width > 0 && height > 0, "captured dimensions should be positive");
            assert_eq!(format, FrameFormat::Bgra8);
        }
        _ => panic!("unexpected event kind"),
    }
}
