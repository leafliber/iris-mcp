#![allow(unused_imports, unused_variables, unused_mut)]

mod mouse;
mod keyboard;

use enigo::{Enigo, Settings};
use mouse::MouseController;
use keyboard::{KeyboardController, SystemCommand};
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Enigo instances
    let enigo_mouse = Enigo::new(&Settings::default())?;
    let enigo_keyboard = Enigo::new(&Settings::default())?;

    let mut mouse = MouseController::new(enigo_mouse);
    let mut keyboard = KeyboardController::new(enigo_keyboard);

    println!("Starting Mouse Demo in 3 seconds...");
    thread::sleep(Duration::from_secs(3));

    // Mouse Demo
    let (x, y) = mouse.mouse_get_position()?;
    println!("Current Mouse Position: ({}, {})", x, y);

    println!("Moving mouse...");
    mouse.mouse_move(500, 500)?;
    
    println!("Clicking...");
    mouse.mouse_click(500, 500, enigo::Button::Left)?;

    // Keyboard Demo
    println!("Starting Keyboard Demo...");
    // keyboard.type_text("Hello from Rust Enigo!")?;
    
    // System Command Demo
    // keyboard.system_command(SystemCommand::Copy)?;

    println!("Done!");
    Ok(())
}
