//! Keyboard handling and XKB integration

use log::debug;

use crate::compositor::SurfaceId;

/// Keyboard state and XKB integration
#[derive(Debug)]
pub struct Keyboard {
    /// Currently focused surface
    focus: Option<SurfaceId>,
    /// Currently pressed keys (keycodes)
    pressed_keys: Vec<u32>,
    /// Modifier state
    modifiers: ModifierState,
    /// Repeat rate (characters per second)
    repeat_rate: u32,
    /// Repeat delay (milliseconds)
    repeat_delay: u32,
    /// Keymap string (XKB format)
    keymap: Option<String>,
}

/// Keyboard modifier state
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    /// Depressed modifiers (currently held down)
    pub depressed: u32,
    /// Latched modifiers (sticky, cleared on next key)
    pub latched: u32,
    /// Locked modifiers (toggled, like caps lock)
    pub locked: u32,
    /// Keyboard group/layout
    pub group: u32,
}

impl Keyboard {
    /// Create a new keyboard
    pub fn new() -> Self {
        Self {
            focus: None,
            pressed_keys: Vec::new(),
            modifiers: ModifierState::default(),
            repeat_rate: 25,
            repeat_delay: 600,
            keymap: None,
        }
    }

    /// Set keyboard focus to a surface
    pub fn set_focus(&mut self, surface: Option<SurfaceId>) -> KeyboardFocusChange {
        let old_focus = self.focus;
        self.focus = surface;

        KeyboardFocusChange {
            old_focus,
            new_focus: surface,
            pressed_keys: self.pressed_keys.clone(),
        }
    }

    /// Get the currently focused surface
    pub fn focus(&self) -> Option<SurfaceId> {
        self.focus
    }

    /// Handle a key press
    pub fn key_press(&mut self, keycode: u32) -> bool {
        if !self.pressed_keys.contains(&keycode) {
            self.pressed_keys.push(keycode);
            debug!("Key pressed: {}", keycode);
            true
        } else {
            false // Key already pressed (repeat)
        }
    }

    /// Handle a key release
    pub fn key_release(&mut self, keycode: u32) -> bool {
        if let Some(idx) = self.pressed_keys.iter().position(|&k| k == keycode) {
            self.pressed_keys.remove(idx);
            debug!("Key released: {}", keycode);
            true
        } else {
            false
        }
    }

    /// Update modifier state
    pub fn update_modifiers(&mut self, modifiers: ModifierState) {
        self.modifiers = modifiers;
    }

    /// Get current modifier state
    pub fn modifiers(&self) -> ModifierState {
        self.modifiers
    }

    /// Get currently pressed keys
    pub fn pressed_keys(&self) -> &[u32] {
        &self.pressed_keys
    }

    /// Set repeat rate
    pub fn set_repeat_rate(&mut self, rate: u32) {
        self.repeat_rate = rate;
    }

    /// Set repeat delay
    pub fn set_repeat_delay(&mut self, delay: u32) {
        self.repeat_delay = delay;
    }

    /// Get repeat info
    pub fn repeat_info(&self) -> (u32, u32) {
        (self.repeat_rate, self.repeat_delay)
    }

    /// Set the keymap
    pub fn set_keymap(&mut self, keymap: String) {
        self.keymap = Some(keymap);
    }

    /// Get the keymap
    pub fn keymap(&self) -> Option<&str> {
        self.keymap.as_deref()
    }

    /// Create a default XKB keymap string
    pub fn default_keymap() -> String {
        // This is a minimal XKB keymap for US keyboard layout
        // In a full implementation, this would use xkbcommon to generate the keymap
        String::from(
            r#"xkb_keymap {
    xkb_keycodes "evdev+aliases(qwerty)" { };
    xkb_types "complete" { };
    xkb_compat "complete" { };
    xkb_symbols "pc+us+inet(evdev)" { };
    xkb_geometry "pc(pc105)" { };
};"#,
        )
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a focus change operation
#[derive(Debug)]
pub struct KeyboardFocusChange {
    /// Previously focused surface
    pub old_focus: Option<SurfaceId>,
    /// Newly focused surface
    pub new_focus: Option<SurfaceId>,
    /// Keys that were pressed when focus changed
    pub pressed_keys: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_new() {
        let keyboard = Keyboard::new();
        assert!(keyboard.focus().is_none());
        assert!(keyboard.pressed_keys().is_empty());
    }

    #[test]
    fn test_key_press_release() {
        let mut keyboard = Keyboard::new();

        // Press a key
        assert!(keyboard.key_press(30)); // 'A' key
        assert!(keyboard.pressed_keys().contains(&30));

        // Press same key again should return false (already pressed)
        assert!(!keyboard.key_press(30));

        // Release the key
        assert!(keyboard.key_release(30));
        assert!(!keyboard.pressed_keys().contains(&30));

        // Release again should return false
        assert!(!keyboard.key_release(30));
    }

    #[test]
    fn test_focus_change() {
        let mut keyboard = Keyboard::new();

        let surface1 = SurfaceId(1);
        let surface2 = SurfaceId(2);

        // Set initial focus
        let change = keyboard.set_focus(Some(surface1));
        assert!(change.old_focus.is_none());
        assert_eq!(change.new_focus, Some(surface1));

        // Change focus
        let change = keyboard.set_focus(Some(surface2));
        assert_eq!(change.old_focus, Some(surface1));
        assert_eq!(change.new_focus, Some(surface2));
    }

    #[test]
    fn test_modifiers() {
        let mut keyboard = Keyboard::new();

        let mods = ModifierState {
            depressed: 1,
            latched: 0,
            locked: 2,
            group: 0,
        };

        keyboard.update_modifiers(mods);
        assert_eq!(keyboard.modifiers().depressed, 1);
        assert_eq!(keyboard.modifiers().locked, 2);
    }

    #[test]
    fn test_repeat_info() {
        let mut keyboard = Keyboard::new();
        keyboard.set_repeat_rate(30);
        keyboard.set_repeat_delay(500);

        let (rate, delay) = keyboard.repeat_info();
        assert_eq!(rate, 30);
        assert_eq!(delay, 500);
    }
}
