use std::thread;
use std::time::Duration;

use iris_mcp::monitor::key_mouse;

fn main() {
    // Start the unified monitor (keyboard + mouse).
    key_mouse::initialize();
    println!("Started key/mouse monitor. Collect events for 5 seconds...\n");
    println!("Try moving the mouse or pressing keys now.\n");

    // Collect events for a short window.
    thread::sleep(Duration::from_secs(5));

    let keyboard = key_mouse::take_keyboard_events();
    let mouse = key_mouse::take_mouse_events();

    println!("Collected {} keyboard events and {} mouse events.\n", keyboard.len(), mouse.len());

    if !keyboard.is_empty() {
        println!("Keyboard events (JSON):");
        for evt in keyboard {
            match serde_json::to_string_pretty(&evt) {
                Ok(s) => println!("{}", s),
                Err(e) => println!("<serialize error: {}>", e),
            }
        }
        println!("\n");
    }

    if !mouse.is_empty() {
        println!("Mouse events (JSON):");
        for evt in mouse {
            match serde_json::to_string_pretty(&evt) {
                Ok(s) => println!("{}", s),
                Err(e) => println!("<serialize error: {}>", e),
            }
        }
    }

    println!("Done.\n");
}
