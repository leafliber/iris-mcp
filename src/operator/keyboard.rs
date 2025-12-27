use enigo::{Direction, Key, Keyboard};

/// Keyboard operations wrapper; generic over any `Keyboard` impl so we can mock in tests.
pub struct KeyboardController<K: Keyboard + Send> {
    keyboard: K,
}

impl<K: Keyboard + Send> KeyboardController<K> {
    pub fn new(keyboard: K) -> Self {
        Self { keyboard }
    }

    /// Type text
    pub fn type_text(&mut self, text: &str) -> Result<(), enigo::InputError> {
        self.keyboard.text(text)
    }

    /// Advanced key press/release control
    pub fn key_control(&mut self, key: Key, direction: Direction) -> Result<(), enigo::InputError> {
        self.keyboard.key(key, direction)
    }

    /// Common shortcuts (copy, paste, undo, save, etc.)
    pub fn system_command(&mut self, command: SystemCommand) -> Result<(), enigo::InputError> {
        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        self.keyboard.key(modifier, Direction::Press)?;
        
        let key = match command {
            SystemCommand::Copy => Key::Unicode('c'),
            SystemCommand::Paste => Key::Unicode('v'),
            SystemCommand::Cut => Key::Unicode('x'),
            SystemCommand::Undo => Key::Unicode('z'),
            SystemCommand::Save => Key::Unicode('s'),
            SystemCommand::SelectAll => Key::Unicode('a'),
        };

        self.keyboard.key(key, Direction::Click)?;
        self.keyboard.key(modifier, Direction::Release)?;
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
