//! Input seat coordination

use log::debug;

use super::{Keyboard, Pointer};
use crate::compositor::SurfaceId;

/// Input seat that coordinates keyboard and pointer
#[derive(Debug)]
pub struct Seat {
    /// Seat name
    name: String,
    /// Keyboard device
    keyboard: Keyboard,
    /// Pointer device
    pointer: Pointer,
    /// Capabilities
    capabilities: SeatCapabilities,
}

/// Seat capabilities
#[derive(Debug, Clone, Copy, Default)]
pub struct SeatCapabilities {
    pub keyboard: bool,
    pub pointer: bool,
    pub touch: bool,
}

impl SeatCapabilities {
    /// Convert to Wayland capability flags
    pub fn to_wayland(&self) -> u32 {
        let mut flags = 0u32;
        if self.pointer {
            flags |= 1;
        }
        if self.keyboard {
            flags |= 2;
        }
        if self.touch {
            flags |= 4;
        }
        flags
    }
}

impl Seat {
    /// Create a new seat
    pub fn new() -> Self {
        Self {
            name: "seat0".to_string(),
            keyboard: Keyboard::new(),
            pointer: Pointer::new(),
            capabilities: SeatCapabilities {
                keyboard: true,
                pointer: true,
                touch: false,
            },
        }
    }

    /// Create a seat with a specific name
    pub fn with_name(name: String) -> Self {
        Self {
            name,
            ..Self::new()
        }
    }

    /// Get the seat name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get seat capabilities
    pub fn capabilities(&self) -> SeatCapabilities {
        self.capabilities
    }

    /// Set seat capabilities
    pub fn set_capabilities(&mut self, capabilities: SeatCapabilities) {
        self.capabilities = capabilities;
    }

    /// Get keyboard reference
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    /// Get mutable keyboard reference
    pub fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    /// Get pointer reference
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    /// Get mutable pointer reference
    pub fn pointer_mut(&mut self) -> &mut Pointer {
        &mut self.pointer
    }

    /// Focus a surface for both keyboard and pointer
    pub fn focus_surface(&mut self, surface: Option<SurfaceId>, x: f64, y: f64) {
        self.keyboard.set_focus(surface);
        self.pointer.set_focus(surface, x, y);
        debug!("Focused surface {:?} at ({}, {})", surface, x, y);
    }

    /// Get the keyboard-focused surface
    pub fn keyboard_focus(&self) -> Option<SurfaceId> {
        self.keyboard.focus()
    }

    /// Get the pointer-focused surface
    pub fn pointer_focus(&self) -> Option<SurfaceId> {
        self.pointer.focus()
    }
}

impl Default for Seat {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seat_new() {
        let seat = Seat::new();
        assert_eq!(seat.name(), "seat0");
        assert!(seat.capabilities().keyboard);
        assert!(seat.capabilities().pointer);
    }

    #[test]
    fn test_seat_capabilities() {
        let caps = SeatCapabilities {
            keyboard: true,
            pointer: true,
            touch: false,
        };
        assert_eq!(caps.to_wayland(), 3); // pointer (1) + keyboard (2)
    }

    #[test]
    fn test_focus_surface() {
        let mut seat = Seat::new();
        let surface = SurfaceId(1);

        seat.focus_surface(Some(surface), 100.0, 50.0);

        assert_eq!(seat.keyboard_focus(), Some(surface));
        assert_eq!(seat.pointer_focus(), Some(surface));
    }

    #[test]
    fn test_keyboard_access() {
        let mut seat = Seat::new();

        seat.keyboard_mut().key_press(30);
        assert!(seat.keyboard().pressed_keys().contains(&30));
    }

    #[test]
    fn test_pointer_access() {
        let mut seat = Seat::new();

        seat.pointer_mut().button_press(0x110);
        assert!(seat.pointer().has_button_pressed());
    }
}
