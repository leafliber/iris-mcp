#![cfg(target_os = "macos")]

use iris_mcp::monitor_screen::{start_monitor, FrameFormat, ScreenEventKind};
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn macos_emits_geometry_once() {
    let (tx, rx) = mpsc::channel();
    let _handle = start_monitor(move |evt| {
        let _ = tx.send(evt);
    })
    .expect("start_monitor should succeed on macOS");

    let evt = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("no screen event received");
    match evt.kind {
        ScreenEventKind::FrameCaptured { width, height, format } => {
            assert!(width > 0 && height > 0, "captured dimensions should be positive");
            assert_eq!(format, FrameFormat::Bgra8);
        }
        _ => panic!("unexpected event kind"),
    }
}
