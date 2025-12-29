//! xdg_shell protocol implementation
//!
//! Implements xdg_wm_base, xdg_surface, xdg_toplevel, and xdg_popup.

use log::debug;

use crate::compositor::{CompositorState, SurfaceId, SurfaceRole, WindowId};

/// Handler for xdg_shell protocol
pub struct XdgShellHandler;

impl XdgShellHandler {
    /// Create a new xdg_shell handler
    pub fn new() -> Self {
        Self
    }

    /// Handle xdg_wm_base::get_xdg_surface
    pub fn get_xdg_surface(
        &self,
        state: &mut CompositorState,
        surface_id: SurfaceId,
    ) -> Result<XdgSurface, XdgShellError> {
        // Check that surface exists and has no role
        let _surface = state
            .surfaces
            .get_mut(surface_id)
            .ok_or(XdgShellError::InvalidSurface)?;

        // The surface will get its role when get_toplevel or get_popup is called
        debug!("Created xdg_surface for {:?}", surface_id);

        Ok(XdgSurface {
            surface_id,
            configured: false,
            geometry: None,
        })
    }

    /// Handle xdg_surface::get_toplevel
    pub fn get_toplevel(
        &self,
        state: &mut CompositorState,
        xdg_surface: &mut XdgSurface,
    ) -> Result<WindowId, XdgShellError> {
        // Set the surface role to toplevel
        let surface = state
            .surfaces
            .get_mut(xdg_surface.surface_id)
            .ok_or(XdgShellError::InvalidSurface)?;

        surface
            .set_role(SurfaceRole::XdgToplevel)
            .map_err(|_| XdgShellError::RoleAlreadySet)?;

        // Create a window for this toplevel
        let window_id = state.windows.create_window(xdg_surface.surface_id);

        debug!(
            "Created xdg_toplevel {:?} for surface {:?}",
            window_id, xdg_surface.surface_id
        );

        Ok(window_id)
    }

    /// Handle xdg_surface::get_popup
    pub fn get_popup(
        &self,
        state: &mut CompositorState,
        xdg_surface: &mut XdgSurface,
        parent: SurfaceId,
        positioner: &XdgPositioner,
    ) -> Result<XdgPopup, XdgShellError> {
        // Set the surface role to popup
        let surface = state
            .surfaces
            .get_mut(xdg_surface.surface_id)
            .ok_or(XdgShellError::InvalidSurface)?;

        surface
            .set_role(SurfaceRole::XdgPopup)
            .map_err(|_| XdgShellError::RoleAlreadySet)?;

        surface.parent = Some(parent);

        let geometry = positioner.calculate_geometry();

        debug!(
            "Created xdg_popup for surface {:?}, parent {:?}",
            xdg_surface.surface_id, parent
        );

        Ok(XdgPopup {
            surface_id: xdg_surface.surface_id,
            parent,
            geometry,
        })
    }

    /// Handle xdg_wm_base::pong (response to ping)
    pub fn pong(&self, _serial: u32) {
        // Client responded to ping, they're alive
        debug!("Received pong");
    }
}

impl Default for XdgShellHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// XDG surface state
#[derive(Debug)]
pub struct XdgSurface {
    /// Associated wl_surface
    pub surface_id: SurfaceId,
    /// Has been configured
    pub configured: bool,
    /// Window geometry (set by client)
    pub geometry: Option<XdgGeometry>,
}

impl XdgSurface {
    /// Set the window geometry
    pub fn set_geometry(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.geometry = Some(XdgGeometry {
            x,
            y,
            width,
            height,
        });
    }

    /// Acknowledge a configure event
    pub fn ack_configure(&mut self, _serial: u32) {
        self.configured = true;
    }
}

/// Window geometry as set by the client
#[derive(Debug, Clone, Copy)]
pub struct XdgGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// XDG popup state
#[derive(Debug)]
pub struct XdgPopup {
    /// Associated wl_surface
    pub surface_id: SurfaceId,
    /// Parent surface
    pub parent: SurfaceId,
    /// Popup geometry relative to parent
    pub geometry: PopupGeometry,
}

/// Popup geometry
#[derive(Debug, Clone, Copy, Default)]
pub struct PopupGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// XDG positioner for popup placement
#[derive(Debug, Default)]
pub struct XdgPositioner {
    /// Size of the popup
    pub size: (i32, i32),
    /// Anchor rectangle in parent surface coordinates
    pub anchor_rect: (i32, i32, i32, i32),
    /// Anchor edge
    pub anchor: Anchor,
    /// Gravity
    pub gravity: Gravity,
    /// Constraint adjustment
    pub constraint_adjustment: u32,
    /// Offset from calculated position
    pub offset: (i32, i32),
}

impl XdgPositioner {
    /// Create a new positioner
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the popup size
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.size = (width, height);
    }

    /// Set the anchor rectangle
    pub fn set_anchor_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.anchor_rect = (x, y, width, height);
    }

    /// Set the anchor edge
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    /// Set the gravity
    pub fn set_gravity(&mut self, gravity: Gravity) {
        self.gravity = gravity;
    }

    /// Set constraint adjustment
    pub fn set_constraint_adjustment(&mut self, adjustment: u32) {
        self.constraint_adjustment = adjustment;
    }

    /// Set offset
    pub fn set_offset(&mut self, x: i32, y: i32) {
        self.offset = (x, y);
    }

    /// Calculate the popup geometry
    pub fn calculate_geometry(&self) -> PopupGeometry {
        let (ax, ay, aw, ah) = self.anchor_rect;

        // Calculate anchor point based on anchor edge
        let (anchor_x, anchor_y) = match self.anchor {
            Anchor::None => (ax + aw / 2, ay + ah / 2),
            Anchor::Top => (ax + aw / 2, ay),
            Anchor::Bottom => (ax + aw / 2, ay + ah),
            Anchor::Left => (ax, ay + ah / 2),
            Anchor::Right => (ax + aw, ay + ah / 2),
            Anchor::TopLeft => (ax, ay),
            Anchor::TopRight => (ax + aw, ay),
            Anchor::BottomLeft => (ax, ay + ah),
            Anchor::BottomRight => (ax + aw, ay + ah),
        };

        // Apply gravity to position popup relative to anchor
        let (popup_w, popup_h) = self.size;
        let (mut x, mut y) = match self.gravity {
            Gravity::None => (anchor_x - popup_w / 2, anchor_y - popup_h / 2),
            Gravity::Top => (anchor_x - popup_w / 2, anchor_y - popup_h),
            Gravity::Bottom => (anchor_x - popup_w / 2, anchor_y),
            Gravity::Left => (anchor_x - popup_w, anchor_y - popup_h / 2),
            Gravity::Right => (anchor_x, anchor_y - popup_h / 2),
            Gravity::TopLeft => (anchor_x - popup_w, anchor_y - popup_h),
            Gravity::TopRight => (anchor_x, anchor_y - popup_h),
            Gravity::BottomLeft => (anchor_x - popup_w, anchor_y),
            Gravity::BottomRight => (anchor_x, anchor_y),
        };

        // Apply offset
        x += self.offset.0;
        y += self.offset.1;

        PopupGeometry {
            x,
            y,
            width: popup_w,
            height: popup_h,
        }
    }
}

/// Anchor edge for popup positioning
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Anchor {
    #[default]
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

/// Gravity for popup positioning
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Gravity {
    #[default]
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

/// XDG shell errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum XdgShellError {
    #[error("Invalid surface")]
    InvalidSurface,
    #[error("Surface already has a role")]
    RoleAlreadySet,
    #[error("Invalid positioner")]
    InvalidPositioner,
    #[error("Not configured")]
    NotConfigured,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdg_shell_handler() {
        let handler = XdgShellHandler::new();
        let mut state = CompositorState::new();

        let surface_id = state.surfaces.create_surface();
        let mut xdg_surface = handler.get_xdg_surface(&mut state, surface_id).unwrap();
        let window_id = handler.get_toplevel(&mut state, &mut xdg_surface).unwrap();

        assert!(state.windows.get(window_id).is_some());
    }

    #[test]
    fn test_positioner() {
        let mut positioner = XdgPositioner::new();
        positioner.set_size(200, 100);
        positioner.set_anchor_rect(0, 0, 100, 50);
        positioner.set_anchor(Anchor::BottomRight);
        positioner.set_gravity(Gravity::BottomRight);

        let geometry = positioner.calculate_geometry();
        assert_eq!(geometry.x, 100);
        assert_eq!(geometry.y, 50);
        assert_eq!(geometry.width, 200);
        assert_eq!(geometry.height, 100);
    }
}
