use enigo::{Direction, InputError, Key, Keyboard};
use iris_mcp::operator_keyboard::{KeyboardController, SystemCommand};
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
struct MockKeyboard {
    events: Arc<Mutex<Vec<Event>>>,
    texts: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug, PartialEq)]
enum Event {
    Key(Key, Direction),
    Raw(u16, Direction),
}

impl MockKeyboard {
    fn with_shared_state() -> (Self, Arc<Mutex<Vec<Event>>>, Arc<Mutex<Vec<String>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let texts = Arc::new(Mutex::new(Vec::new()));
        let kb = MockKeyboard {
            events: Arc::clone(&events),
            texts: Arc::clone(&texts),
        };
        (kb, events, texts)
    }
}

impl Keyboard for MockKeyboard {
    fn key(&mut self, key: Key, direction: Direction) -> Result<(), InputError> {
        self.events.lock().unwrap().push(Event::Key(key, direction));
        Ok(())
    }

    fn text(&mut self, text: &str) -> Result<(), InputError> {
        self.texts.lock().unwrap().push(text.to_string());
        Ok(())
    }

    fn fast_text(&mut self, text: &str) -> Result<Option<()>, InputError> {
        self.texts.lock().unwrap().push(text.to_string());
        Ok(Some(()))
    }

    fn raw(&mut self, code: u16, direction: Direction) -> Result<(), InputError> {
        self.events.lock().unwrap().push(Event::Raw(code, direction));
        Ok(())
    }
}

#[test]
fn type_text_invokes_text() {
    let (keyboard, _events, texts) = MockKeyboard::with_shared_state();
    let mut controller = KeyboardController::new(keyboard);

    controller.type_text("hello").unwrap();

    assert_eq!(texts.lock().unwrap().as_slice(), ["hello".to_string()]);
}

#[test]
fn key_control_invokes_key() {
    let (keyboard, events, _texts) = MockKeyboard::with_shared_state();
    let mut controller = KeyboardController::new(keyboard);

    controller
        .key_control(Key::Unicode('a'), Direction::Press)
        .unwrap();

    let guard = events.lock().unwrap();
    assert_eq!(guard.len(), 1);
    assert!(matches!(guard[0], Event::Key(Key::Unicode('a'), Direction::Press)));
}

#[test]
fn system_command_emits_modifier_and_action() {
    let (keyboard, events, _texts) = MockKeyboard::with_shared_state();
    let mut controller = KeyboardController::new(keyboard);

    controller.system_command(SystemCommand::Copy).unwrap();

    #[cfg(target_os = "macos")]
    let expected_modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let expected_modifier = Key::Control;

    let guard = events.lock().unwrap();
    assert_eq!(guard.len(), 3);
    assert!(matches!(guard[0], Event::Key(k, Direction::Press) if k == expected_modifier));
    assert!(matches!(guard[1], Event::Key(Key::Unicode('c'), Direction::Click)));
    assert!(matches!(guard[2], Event::Key(k, Direction::Release) if k == expected_modifier));
}
