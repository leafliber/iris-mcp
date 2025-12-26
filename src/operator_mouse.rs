use enigo::{
    Button, Coordinate, Direction, Enigo, Mouse,
};
use std::{thread, time::Duration};

pub struct MouseController {
    enigo: Enigo,
}

impl MouseController {
    pub fn new(enigo: Enigo) -> Self {
        Self { enigo }
    }

    /// Click at coordinates with left/right/middle button
    pub fn mouse_click(&mut self, x: i32, y: i32, button: Button) -> Result<(), enigo::InputError> {
        self.enigo.move_mouse(x, y, Coordinate::Abs)?;
        self.enigo.button(button, Direction::Click)?;
        Ok(())
    }

    /// Double-click at coordinates
    pub fn mouse_double_click(&mut self, x: i32, y: i32, button: Button) -> Result<(), enigo::InputError> {
        self.enigo.move_mouse(x, y, Coordinate::Abs)?;
        self.enigo.button(button, Direction::Click)?;
        thread::sleep(Duration::from_millis(100));
        self.enigo.button(button, Direction::Click)?;
        Ok(())
    }

    /// Move cursor to position
    pub fn mouse_move(&mut self, x: i32, y: i32) -> Result<(), enigo::InputError> {
        self.enigo.move_mouse(x, y, Coordinate::Abs)
    }

    /// Get current cursor location
    pub fn mouse_get_position(&self) -> Result<(i32, i32), enigo::InputError> {
        self.enigo.location()
    }

    /// Scroll in any direction
    /// axis: horizontal or vertical (usually vertical is standard scroll)
    /// clicks: number of "clicks" to scroll. Positive is up/right, negative is down/left usually, but depends on OS.
    pub fn mouse_scroll(&mut self, lines_x: i32, lines_y: i32) -> Result<(), enigo::InputError> {
        if lines_x != 0 {
             self.enigo.scroll(lines_x, enigo::Axis::Horizontal)?;
        }
        if lines_y != 0 {
             self.enigo.scroll(lines_y, enigo::Axis::Vertical)?;
        }
        Ok(())
    }

    /// Drag from current position to target
    pub fn mouse_drag(&mut self, target_x: i32, target_y: i32, button: Button) -> Result<(), enigo::InputError> {
        // Press button
        self.enigo.button(button, Direction::Press)?;
        // Move to target
        self.enigo.move_mouse(target_x, target_y, Coordinate::Abs)?;
        // Release button
        self.enigo.button(button, Direction::Release)?;
        Ok(())
    }

    /// Press/release mouse buttons
    pub fn mouse_button_control(&mut self, button: Button, direction: Direction) -> Result<(), enigo::InputError> {
        self.enigo.button(button, direction)
    }

    /// Follow a smooth path with multiple points
    /// points: List of (x, y) tuples
    /// speed_ms: delay between points in milliseconds
    pub fn mouse_move_path(&mut self, points: &[(i32, i32)], speed_ms: u64) -> Result<(), enigo::InputError> {
        for &(x, y) in points {
            self.enigo.move_mouse(x, y, Coordinate::Abs)?;
            thread::sleep(Duration::from_millis(speed_ms));
        }
        Ok(())
    }
}
