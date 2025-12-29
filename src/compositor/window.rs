//! Window management
//!
//! This module maps Wayland toplevels to native macOS windows.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::compositor::SurfaceId;

/// Unique identifier for windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

impl WindowId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        WindowId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Window state flags
#[derive(Debug, Clone, Default)]
pub struct WindowState {
    /// Window is maximized
    pub maximized: bool,
    /// Window is fullscreen
    pub fullscreen: bool,
    /// Window is minimized/iconified
    pub minimized: bool,
    /// Window is focused
    pub focused: bool,
    /// Window is activated (has keyboard focus)
    pub activated: bool,
    /// Window is resizing
    pub resizing: bool,
    /// Window is being moved
    pub moving: bool,
}

/// Window geometry
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowGeometry {
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// A native window representing a Wayland toplevel
#[derive(Debug)]
pub struct Window {
    /// Unique identifier
    pub id: WindowId,
    /// Associated surface
    pub surface_id: SurfaceId,
    /// Window title
    pub title: Option<String>,
    /// Application ID (app_id)
    pub app_id: Option<String>,
    /// Is maximized
    pub maximized: bool,
    /// Is fullscreen
    pub fullscreen: bool,
    /// Window geometry
    pub geometry: WindowGeometry,
    /// Minimum size (0 = no minimum)
    pub min_size: (u32, u32),
    /// Maximum size (0 = no maximum)
    pub max_size: (u32, u32),
    /// Current window state
    pub state: WindowState,
    /// Parent window (for transient windows)
    pub parent: Option<WindowId>,
    /// Native window handle (platform-specific)
    #[cfg(target_os = "macos")]
    pub native_handle: Option<crate::backend::cocoa::window::NativeWindowHandle>,
    #[cfg(not(target_os = "macos"))]
    pub native_handle: Option<()>,
}

impl Window {
    /// Create a new window
    pub fn new(surface_id: SurfaceId) -> Self {
        Self {
            id: WindowId::new(),
            surface_id,
            title: None,
            app_id: None,
            maximized: false,
            fullscreen: false,
            geometry: WindowGeometry::default(),
            min_size: (0, 0),
            max_size: (0, 0),
            state: WindowState::default(),
            parent: None,
            native_handle: None,
        }
    }

    /// Set the window title
    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    /// Set the application ID
    pub fn set_app_id(&mut self, app_id: String) {
        self.app_id = Some(app_id);
    }

    /// Set window geometry
    pub fn set_geometry(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.geometry = WindowGeometry {
            x,
            y,
            width,
            height,
        };
    }

    /// Set minimum size
    pub fn set_min_size(&mut self, width: u32, height: u32) {
        self.min_size = (width, height);
    }

    /// Set maximum size
    pub fn set_max_size(&mut self, width: u32, height: u32) {
        self.max_size = (width, height);
    }

    /// Set maximized state
    pub fn set_maximized(&mut self, maximized: bool) {
        self.state.maximized = maximized;
    }

    /// Set fullscreen state
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.state.fullscreen = fullscreen;
    }

    /// Set minimized state
    pub fn set_minimized(&mut self, minimized: bool) {
        self.state.minimized = minimized;
    }

    /// Set focused state
    pub fn set_focused(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    /// Set activated state
    pub fn set_activated(&mut self, activated: bool) {
        self.state.activated = activated;
    }
}

/// Manager for all windows
#[derive(Debug)]
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    /// Map from surface ID to window ID
    surface_to_window: HashMap<SurfaceId, WindowId>,
    /// Currently focused window
    focused_window: Option<WindowId>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            surface_to_window: HashMap::new(),
            focused_window: None,
        }
    }

    /// Create a new window for a surface
    pub fn create_window(&mut self, surface_id: SurfaceId) -> WindowId {
        let window = Window::new(surface_id);
        let id = window.id;
        self.surface_to_window.insert(surface_id, id);
        self.windows.insert(id, window);
        id
    }

    /// Get a window by ID
    pub fn get(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    /// Get a mutable window by ID
    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    /// Get a window by surface ID
    pub fn get_by_surface(&self, surface_id: SurfaceId) -> Option<&Window> {
        self.surface_to_window
            .get(&surface_id)
            .and_then(|id| self.windows.get(id))
    }

    /// Get a mutable window by surface ID
    pub fn get_by_surface_mut(&mut self, surface_id: SurfaceId) -> Option<&mut Window> {
        self.surface_to_window
            .get(&surface_id)
            .copied()
            .and_then(move |id| self.windows.get_mut(&id))
    }

    /// Get the window ID for a surface
    pub fn window_for_surface(&self, surface_id: SurfaceId) -> Option<WindowId> {
        self.surface_to_window.get(&surface_id).copied()
    }

    /// Remove a window
    pub fn remove(&mut self, id: WindowId) -> Option<Window> {
        if let Some(window) = self.windows.remove(&id) {
            self.surface_to_window.remove(&window.surface_id);
            if self.focused_window == Some(id) {
                self.focused_window = None;
            }
            Some(window)
        } else {
            None
        }
    }

    /// Set the focused window
    pub fn set_focused(&mut self, id: Option<WindowId>) {
        // Unfocus previous window
        if let Some(prev_id) = self.focused_window {
            if let Some(window) = self.windows.get_mut(&prev_id) {
                window.set_focused(false);
                window.set_activated(false);
            }
        }

        self.focused_window = id;

        // Focus new window
        if let Some(new_id) = id {
            if let Some(window) = self.windows.get_mut(&new_id) {
                window.set_focused(true);
                window.set_activated(true);
            }
        }
    }

    /// Get the currently focused window
    pub fn focused(&self) -> Option<&Window> {
        self.focused_window.and_then(|id| self.windows.get(&id))
    }

    /// Get all windows
    pub fn iter(&self) -> impl Iterator<Item = (&WindowId, &Window)> {
        self.windows.iter()
    }

    /// Get count of windows
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_id_unique() {
        let id1 = WindowId::new();
        let id2 = WindowId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_window_new() {
        let surface_id = SurfaceId(1);
        let window = Window::new(surface_id);
        assert_eq!(window.surface_id, surface_id);
        assert!(window.title.is_none());
    }

    #[test]
    fn test_window_manager() {
        let mut manager = WindowManager::new();
        let surface_id = SurfaceId(1);
        let id = manager.create_window(surface_id);
        assert!(manager.get(id).is_some());
        assert!(manager.get_by_surface(surface_id).is_some());
        manager.remove(id);
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn test_window_focus() {
        let mut manager = WindowManager::new();
        let id1 = manager.create_window(SurfaceId(1));
        let id2 = manager.create_window(SurfaceId(2));

        manager.set_focused(Some(id1));
        assert!(manager.get(id1).unwrap().state.focused);
        assert!(!manager.get(id2).unwrap().state.focused);

        manager.set_focused(Some(id2));
        assert!(!manager.get(id1).unwrap().state.focused);
        assert!(manager.get(id2).unwrap().state.focused);
    }
}
