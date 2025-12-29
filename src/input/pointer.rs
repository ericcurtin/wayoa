//! Pointer (mouse/trackpad) handling

use log::debug;

use crate::compositor::SurfaceId;

/// Pointer state
#[derive(Debug)]
pub struct Pointer {
    /// Currently focused surface
    focus: Option<SurfaceId>,
    /// Position in focused surface coordinates
    position: (f64, f64),
    /// Currently pressed buttons
    pressed_buttons: Vec<u32>,
    /// Cursor surface (for software cursor)
    cursor_surface: Option<SurfaceId>,
    /// Cursor hotspot
    cursor_hotspot: (i32, i32),
    /// Grab state
    grab: Option<PointerGrab>,
}

/// Pointer grab state
#[derive(Debug, Clone)]
pub struct PointerGrab {
    /// Surface that has the grab
    pub surface: SurfaceId,
    /// Serial that initiated the grab
    pub serial: u32,
    /// Type of grab
    pub grab_type: GrabType,
}

/// Type of pointer grab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabType {
    /// Button press grab
    Button,
    /// Popup grab
    Popup,
    /// Move operation
    Move,
    /// Resize operation
    Resize(ResizeEdge),
}

/// Resize edge for resize grab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    None,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Pointer {
    /// Create a new pointer
    pub fn new() -> Self {
        Self {
            focus: None,
            position: (0.0, 0.0),
            pressed_buttons: Vec::new(),
            cursor_surface: None,
            cursor_hotspot: (0, 0),
            grab: None,
        }
    }

    /// Set pointer focus to a surface
    pub fn set_focus(&mut self, surface: Option<SurfaceId>, x: f64, y: f64) -> PointerFocusChange {
        let old_focus = self.focus;
        let _old_position = self.position;

        self.focus = surface;
        self.position = (x, y);

        PointerFocusChange {
            old_focus,
            new_focus: surface,
            x,
            y,
        }
    }

    /// Get the currently focused surface
    pub fn focus(&self) -> Option<SurfaceId> {
        self.focus
    }

    /// Update pointer position
    pub fn motion(&mut self, x: f64, y: f64) {
        self.position = (x, y);
    }

    /// Get current position
    pub fn position(&self) -> (f64, f64) {
        self.position
    }

    /// Handle a button press
    pub fn button_press(&mut self, button: u32) -> bool {
        if !self.pressed_buttons.contains(&button) {
            self.pressed_buttons.push(button);
            debug!("Button pressed: {}", button);
            true
        } else {
            false
        }
    }

    /// Handle a button release
    pub fn button_release(&mut self, button: u32) -> bool {
        if let Some(idx) = self.pressed_buttons.iter().position(|&b| b == button) {
            self.pressed_buttons.remove(idx);
            debug!("Button released: {}", button);
            true
        } else {
            false
        }
    }

    /// Get currently pressed buttons
    pub fn pressed_buttons(&self) -> &[u32] {
        &self.pressed_buttons
    }

    /// Check if any button is pressed
    pub fn has_button_pressed(&self) -> bool {
        !self.pressed_buttons.is_empty()
    }

    /// Set the cursor surface
    pub fn set_cursor(&mut self, surface: Option<SurfaceId>, hotspot_x: i32, hotspot_y: i32) {
        self.cursor_surface = surface;
        self.cursor_hotspot = (hotspot_x, hotspot_y);
    }

    /// Get the cursor surface
    pub fn cursor(&self) -> Option<SurfaceId> {
        self.cursor_surface
    }

    /// Get cursor hotspot
    pub fn cursor_hotspot(&self) -> (i32, i32) {
        self.cursor_hotspot
    }

    /// Start a grab
    pub fn start_grab(&mut self, surface: SurfaceId, serial: u32, grab_type: GrabType) {
        self.grab = Some(PointerGrab {
            surface,
            serial,
            grab_type,
        });
    }

    /// End the current grab
    pub fn end_grab(&mut self) {
        self.grab = None;
    }

    /// Get the current grab
    pub fn grab(&self) -> Option<&PointerGrab> {
        self.grab.as_ref()
    }

    /// Check if there's an active grab
    pub fn has_grab(&self) -> bool {
        self.grab.is_some()
    }
}

impl Default for Pointer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a focus change operation
#[derive(Debug)]
pub struct PointerFocusChange {
    /// Previously focused surface
    pub old_focus: Option<SurfaceId>,
    /// Newly focused surface
    pub new_focus: Option<SurfaceId>,
    /// X coordinate in new surface
    pub x: f64,
    /// Y coordinate in new surface
    pub y: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_new() {
        let pointer = Pointer::new();
        assert!(pointer.focus().is_none());
        assert!(pointer.pressed_buttons().is_empty());
    }

    #[test]
    fn test_button_press_release() {
        let mut pointer = Pointer::new();

        // Press left button
        assert!(pointer.button_press(0x110));
        assert!(pointer.pressed_buttons().contains(&0x110));
        assert!(pointer.has_button_pressed());

        // Release
        assert!(pointer.button_release(0x110));
        assert!(!pointer.has_button_pressed());
    }

    #[test]
    fn test_focus_change() {
        let mut pointer = Pointer::new();

        let surface1 = SurfaceId(1);
        let surface2 = SurfaceId(2);

        // Set initial focus
        let change = pointer.set_focus(Some(surface1), 100.0, 50.0);
        assert!(change.old_focus.is_none());
        assert_eq!(change.new_focus, Some(surface1));

        // Check position
        assert_eq!(pointer.position(), (100.0, 50.0));

        // Change focus
        let change = pointer.set_focus(Some(surface2), 200.0, 100.0);
        assert_eq!(change.old_focus, Some(surface1));
        assert_eq!(change.new_focus, Some(surface2));
    }

    #[test]
    fn test_cursor() {
        let mut pointer = Pointer::new();

        let cursor_surface = SurfaceId(100);
        pointer.set_cursor(Some(cursor_surface), 10, 5);

        assert_eq!(pointer.cursor(), Some(cursor_surface));
        assert_eq!(pointer.cursor_hotspot(), (10, 5));
    }

    #[test]
    fn test_grab() {
        let mut pointer = Pointer::new();

        let surface = SurfaceId(1);
        pointer.start_grab(surface, 1, GrabType::Move);

        assert!(pointer.has_grab());
        assert_eq!(pointer.grab().unwrap().grab_type, GrabType::Move);

        pointer.end_grab();
        assert!(!pointer.has_grab());
    }
}
