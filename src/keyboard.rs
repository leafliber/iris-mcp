#![allow(dead_code)]
use enigo::{
    Direction, Enigo, Key, Keyboard,
};

pub struct KeyboardController {
    enigo: Enigo,
}

impl KeyboardController {
    pub fn new(enigo: Enigo) -> Self {
        Self { enigo }
    }

    /// Type text
    pub fn type_text(&mut self, text: &str) -> Result<(), enigo::InputError> {
        self.enigo.text(text)
    }

    /// Advanced key press/release control
    pub fn key_control(&mut self, key: Key, direction: Direction) -> Result<(), enigo::InputError> {
        self.enigo.key(key, direction)
    }

    /// Common shortcuts (copy, paste, undo, save, etc.)
    /// This is a helper to execute common commands
    pub fn system_command(&mut self, command: SystemCommand) -> Result<(), enigo::InputError> {
        // Determine the modifier key based on OS (usually Ctrl or Cmd)
        // Enigo doesn't expose OS detection directly in a cross-platform way for modifiers usually, 
        // but we can assume a default or let the user configure it.
        // For simplicity, we'll use Control for now, or Command on Mac if we could detect it.
        // Since we are on macOS (from environment info), we should use Meta (Command).
        
        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        self.enigo.key(modifier, Direction::Press)?;
        
        let key = match command {
            SystemCommand::Copy => Key::Unicode('c'),
            SystemCommand::Paste => Key::Unicode('v'),
            SystemCommand::Cut => Key::Unicode('x'),
            SystemCommand::Undo => Key::Unicode('z'),
            SystemCommand::Save => Key::Unicode('s'),
            SystemCommand::SelectAll => Key::Unicode('a'),
        };

        self.enigo.key(key, Direction::Click)?;
        self.enigo.key(modifier, Direction::Release)?;
        Ok(())
    }
}

pub enum SystemCommand {
    Copy,
    Paste,
    Cut,
    Undo,
    Save,
    SelectAll,
}
