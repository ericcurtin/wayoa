//! wl_seat protocol implementation
//!
//! Implements input device handling (keyboard, pointer, touch).

use crate::compositor::SurfaceId;

/// Seat capabilities
#[derive(Debug, Clone, Copy, Default)]
pub struct SeatCapabilities {
    pub pointer: bool,
    pub keyboard: bool,
    pub touch: bool,
}

impl SeatCapabilities {
    /// Convert to Wayland capability bitmask
    pub fn to_wayland(&self) -> u32 {
        let mut caps = 0u32;
        if self.pointer {
            caps |= 1; // WL_SEAT_CAPABILITY_POINTER
        }
        if self.keyboard {
            caps |= 2; // WL_SEAT_CAPABILITY_KEYBOARD
        }
        if self.touch {
            caps |= 4; // WL_SEAT_CAPABILITY_TOUCH
        }
        caps
    }
}

/// Handler for wl_seat protocol
pub struct WlSeatHandler {
    /// Seat capabilities
    capabilities: SeatCapabilities,
    /// Seat name
    name: String,
}

impl WlSeatHandler {
    /// Create a new seat handler
    pub fn new() -> Self {
        Self {
            capabilities: SeatCapabilities {
                pointer: true,
                keyboard: true,
                touch: false,
            },
            name: "default".to_string(),
        }
    }

    /// Get the seat capabilities
    pub fn capabilities(&self) -> SeatCapabilities {
        self.capabilities
    }

    /// Get the seat name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set capabilities
    pub fn set_capabilities(&mut self, caps: SeatCapabilities) {
        self.capabilities = caps;
    }
}

impl Default for WlSeatHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard state
#[derive(Debug, Default)]
pub struct KeyboardState {
    /// Currently focused surface
    pub focus: Option<SurfaceId>,
    /// Currently pressed keys (keycodes)
    pub pressed_keys: Vec<u32>,
    /// Modifier state
    pub modifiers: ModifierState,
}

/// Keyboard modifier state
#[derive(Debug, Default, Clone, Copy)]
pub struct ModifierState {
    pub depressed: u32,
    pub latched: u32,
    pub locked: u32,
    pub group: u32,
}

/// Pointer state
#[derive(Debug, Default)]
pub struct PointerState {
    /// Currently focused surface
    pub focus: Option<SurfaceId>,
    /// Position in focused surface coordinates
    pub x: f64,
    pub y: f64,
    /// Currently pressed buttons
    pub pressed_buttons: Vec<u32>,
}

/// Touch state
#[derive(Debug, Default)]
pub struct TouchState {
    /// Active touch points
    pub points: Vec<TouchPoint>,
}

/// A touch point
#[derive(Debug, Clone)]
pub struct TouchPoint {
    /// Touch point ID
    pub id: i32,
    /// Surface being touched
    pub surface: SurfaceId,
    /// Position in surface coordinates
    pub x: f64,
    pub y: f64,
}

/// Keyboard events to send to clients
#[derive(Debug)]
pub enum KeyboardEvent {
    /// Keyboard focus entered a surface
    Enter {
        surface: SurfaceId,
        pressed_keys: Vec<u32>,
    },
    /// Keyboard focus left a surface
    Leave { surface: SurfaceId },
    /// Key press or release
    Key {
        time: u32,
        key: u32,
        state: KeyState,
    },
    /// Modifier state changed
    Modifiers {
        depressed: u32,
        latched: u32,
        locked: u32,
        group: u32,
    },
}

/// Key state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Released = 0,
    Pressed = 1,
}

/// Pointer events to send to clients
#[derive(Debug)]
pub enum PointerEvent {
    /// Pointer entered a surface
    Enter { surface: SurfaceId, x: f64, y: f64 },
    /// Pointer left a surface
    Leave { surface: SurfaceId },
    /// Pointer motion
    Motion { time: u32, x: f64, y: f64 },
    /// Button press or release
    Button {
        time: u32,
        button: u32,
        state: ButtonState,
    },
    /// Axis (scroll) event
    Axis {
        time: u32,
        axis: AxisType,
        value: f64,
    },
    /// Frame delimiter
    Frame,
}

/// Button state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}

/// Axis type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisType {
    VerticalScroll = 0,
    HorizontalScroll = 1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seat_capabilities() {
        let caps = SeatCapabilities {
            pointer: true,
            keyboard: true,
            touch: false,
        };
        assert_eq!(caps.to_wayland(), 3); // pointer (1) + keyboard (2)
    }

    #[test]
    fn test_seat_handler() {
        let handler = WlSeatHandler::new();
        assert!(handler.capabilities().pointer);
        assert!(handler.capabilities().keyboard);
        assert_eq!(handler.name(), "default");
    }
}
