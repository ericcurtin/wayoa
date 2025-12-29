//! NSEvent handling and translation to Wayland events

use crate::protocol::seat::{AxisType, ButtonState, KeyState, KeyboardEvent, PointerEvent};

/// Translates macOS NSEvent to Wayland input events
pub struct InputTranslator;

impl InputTranslator {
    /// Create a new input translator
    pub fn new() -> Self {
        Self
    }

    /// Translate a key code from macOS to Linux/evdev keycode
    pub fn translate_keycode(&self, macos_keycode: u16) -> u32 {
        // macOS virtual key codes to Linux evdev keycodes
        // This is a simplified mapping - a full implementation would have all keys
        match macos_keycode {
            0x00 => 30,  // A
            0x01 => 31,  // S
            0x02 => 32,  // D
            0x03 => 33,  // F
            0x04 => 35,  // H
            0x05 => 34,  // G
            0x06 => 44,  // Z
            0x07 => 45,  // X
            0x08 => 46,  // C
            0x09 => 47,  // V
            0x0B => 48,  // B
            0x0C => 16,  // Q
            0x0D => 17,  // W
            0x0E => 18,  // E
            0x0F => 19,  // R
            0x10 => 21,  // Y
            0x11 => 20,  // T
            0x12 => 2,   // 1
            0x13 => 3,   // 2
            0x14 => 4,   // 3
            0x15 => 5,   // 4
            0x16 => 7,   // 6
            0x17 => 6,   // 5
            0x18 => 13,  // =
            0x19 => 10,  // 9
            0x1A => 8,   // 7
            0x1B => 12,  // -
            0x1C => 9,   // 8
            0x1D => 11,  // 0
            0x1E => 27,  // ]
            0x1F => 24,  // O
            0x20 => 22,  // U
            0x21 => 26,  // [
            0x22 => 23,  // I
            0x23 => 25,  // P
            0x24 => 28,  // Return
            0x25 => 38,  // L
            0x26 => 36,  // J
            0x27 => 40,  // '
            0x28 => 37,  // K
            0x29 => 39,  // ;
            0x2A => 43,  // \
            0x2B => 51,  // ,
            0x2C => 53,  // /
            0x2D => 49,  // N
            0x2E => 50,  // M
            0x2F => 52,  // .
            0x30 => 15,  // Tab
            0x31 => 57,  // Space
            0x32 => 41,  // `
            0x33 => 14,  // Backspace
            0x35 => 1,   // Escape
            0x37 => 125, // Left Command
            0x38 => 42,  // Left Shift
            0x39 => 58,  // Caps Lock
            0x3A => 56,  // Left Alt/Option
            0x3B => 29,  // Left Control
            0x3C => 54,  // Right Shift
            0x3D => 100, // Right Alt/Option
            0x3E => 97,  // Right Control
            0x40 => 126, // F17
            0x4F => 127, // F18
            0x50 => 128, // F19
            0x5A => 129, // F20
            0x60 => 63,  // F5
            0x61 => 64,  // F6
            0x62 => 65,  // F7
            0x63 => 61,  // F3
            0x64 => 66,  // F8
            0x65 => 67,  // F9
            0x67 => 87,  // F11
            0x69 => 183, // F13
            0x6A => 184, // F16
            0x6B => 185, // F14
            0x6D => 68,  // F10
            0x6F => 88,  // F12
            0x71 => 186, // F15
            0x72 => 110, // Insert (Help on Mac)
            0x73 => 102, // Home
            0x74 => 104, // Page Up
            0x75 => 111, // Delete
            0x76 => 62,  // F4
            0x77 => 107, // End
            0x78 => 60,  // F2
            0x79 => 109, // Page Down
            0x7A => 59,  // F1
            0x7B => 105, // Left Arrow
            0x7C => 106, // Right Arrow
            0x7D => 108, // Down Arrow
            0x7E => 103, // Up Arrow
            _ => 0,      // Unknown key
        }
    }

    /// Translate mouse button from macOS to Linux button code
    pub fn translate_button(&self, macos_button: i32) -> u32 {
        // macOS button numbers to Linux evdev button codes
        match macos_button {
            0 => 0x110, // BTN_LEFT
            1 => 0x111, // BTN_RIGHT
            2 => 0x112, // BTN_MIDDLE
            3 => 0x113, // BTN_SIDE
            4 => 0x114, // BTN_EXTRA
            _ => 0x110, // Default to left
        }
    }

    /// Create a key event
    pub fn key_event(&self, keycode: u16, pressed: bool, time: u32) -> KeyboardEvent {
        let key = self.translate_keycode(keycode);
        let state = if pressed {
            KeyState::Pressed
        } else {
            KeyState::Released
        };

        KeyboardEvent::Key { time, key, state }
    }

    /// Create a modifier event
    pub fn modifier_event(
        &self,
        depressed: u32,
        latched: u32,
        locked: u32,
        group: u32,
    ) -> KeyboardEvent {
        KeyboardEvent::Modifiers {
            depressed,
            latched,
            locked,
            group,
        }
    }

    /// Create a pointer motion event
    pub fn motion_event(&self, x: f64, y: f64, time: u32) -> PointerEvent {
        PointerEvent::Motion { time, x, y }
    }

    /// Create a pointer button event
    pub fn button_event(&self, button: i32, pressed: bool, time: u32) -> PointerEvent {
        let button = self.translate_button(button);
        let state = if pressed {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        };

        PointerEvent::Button {
            time,
            button,
            state,
        }
    }

    /// Create a scroll/axis event
    pub fn scroll_event(&self, dx: f64, dy: f64, time: u32) -> Vec<PointerEvent> {
        let mut events = Vec::new();

        if dy.abs() > 0.0 {
            events.push(PointerEvent::Axis {
                time,
                axis: AxisType::VerticalScroll,
                value: dy,
            });
        }

        if dx.abs() > 0.0 {
            events.push(PointerEvent::Axis {
                time,
                axis: AxisType::HorizontalScroll,
                value: dx,
            });
        }

        events.push(PointerEvent::Frame);
        events
    }

    /// Translate macOS modifier flags to XKB modifier mask
    pub fn translate_modifiers(&self, macos_flags: u64) -> (u32, u32, u32, u32) {
        // macOS NSEventModifierFlags to XKB modifier state
        let mut depressed = 0u32;
        let latched = 0u32;
        let group = 0u32;

        // Shift
        if macos_flags & (1 << 17) != 0 {
            depressed |= 1; // MOD_SHIFT
        }
        // Control
        if macos_flags & (1 << 18) != 0 {
            depressed |= 4; // MOD_CTRL
        }
        // Alt/Option
        if macos_flags & (1 << 19) != 0 {
            depressed |= 8; // MOD_ALT
        }
        // Command (map to Super/Logo)
        if macos_flags & (1 << 20) != 0 {
            depressed |= 64; // MOD_LOGO
        }
        // Caps Lock
        let locked = if macos_flags & (1 << 16) != 0 {
            2 // MOD_CAPS
        } else {
            0
        };

        (depressed, latched, locked, group)
    }
}

impl Default for InputTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_translation() {
        let translator = InputTranslator::new();

        // Test some common keys
        assert_eq!(translator.translate_keycode(0x00), 30); // A
        assert_eq!(translator.translate_keycode(0x24), 28); // Return
        assert_eq!(translator.translate_keycode(0x31), 57); // Space
        assert_eq!(translator.translate_keycode(0x35), 1); // Escape
    }

    #[test]
    fn test_button_translation() {
        let translator = InputTranslator::new();

        assert_eq!(translator.translate_button(0), 0x110); // Left
        assert_eq!(translator.translate_button(1), 0x111); // Right
        assert_eq!(translator.translate_button(2), 0x112); // Middle
    }

    #[test]
    fn test_key_event() {
        let translator = InputTranslator::new();
        let event = translator.key_event(0x00, true, 1000);

        match event {
            KeyboardEvent::Key { key, state, time } => {
                assert_eq!(key, 30); // A
                assert_eq!(state, KeyState::Pressed);
                assert_eq!(time, 1000);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_scroll_event() {
        let translator = InputTranslator::new();
        let events = translator.scroll_event(0.0, 10.0, 1000);

        assert_eq!(events.len(), 2); // Vertical scroll + frame
        match &events[0] {
            PointerEvent::Axis { axis, value, .. } => {
                assert_eq!(*axis, AxisType::VerticalScroll);
                assert_eq!(*value, 10.0);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_modifier_translation() {
        let translator = InputTranslator::new();

        // Shift pressed
        let (dep, _, _, _) = translator.translate_modifiers(1 << 17);
        assert_eq!(dep & 1, 1);

        // Command pressed
        let (dep, _, _, _) = translator.translate_modifiers(1 << 20);
        assert_eq!(dep & 64, 64);

        // Caps lock
        let (_, _, locked, _) = translator.translate_modifiers(1 << 16);
        assert_eq!(locked, 2);
    }
}
