//! Surface management
//!
//! This module handles Wayland surface tracking, damage regions,
//! and buffer attachment.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceId(pub u64);

impl SurfaceId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        SurfaceId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// A damage region on a surface
#[derive(Debug, Clone, Copy)]
pub struct DamageRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Buffer information attached to a surface
#[derive(Debug, Clone)]
pub struct BufferInfo {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Stride (bytes per row)
    pub stride: u32,
    /// Pixel format
    pub format: u32,
    /// Offset into the shared memory pool
    pub offset: u32,
}

/// Pending state for a surface (not yet committed)
#[derive(Debug, Default)]
pub struct SurfacePendingState {
    /// Pending buffer attachment
    pub buffer: Option<BufferInfo>,
    /// Accumulated damage regions
    pub damage: Vec<DamageRect>,
    /// Buffer transform
    pub transform: i32,
    /// Buffer scale factor
    pub scale: i32,
    /// Frame callbacks to be fired
    pub frame_callbacks: Vec<u32>,
}

/// A Wayland surface
#[derive(Debug)]
pub struct Surface {
    /// Unique identifier
    pub id: SurfaceId,
    /// Current buffer info
    pub buffer: Option<BufferInfo>,
    /// Current damage regions
    pub damage: Vec<DamageRect>,
    /// Buffer transform
    pub transform: i32,
    /// Buffer scale factor (default 1)
    pub scale: i32,
    /// Pending state (not yet committed)
    pub pending: SurfacePendingState,
    /// Role-specific data (e.g., xdg_surface role)
    pub role: SurfaceRole,
    /// Parent surface (for subsurfaces)
    pub parent: Option<SurfaceId>,
    /// Child subsurfaces
    pub children: Vec<SurfaceId>,
}

/// Surface role determines how the surface is used
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SurfaceRole {
    /// No role assigned yet
    #[default]
    None,
    /// XDG toplevel window
    XdgToplevel,
    /// XDG popup
    XdgPopup,
    /// Subsurface
    Subsurface,
    /// Cursor surface
    Cursor,
    /// Layer shell surface
    LayerSurface,
}

impl Surface {
    /// Create a new surface
    pub fn new() -> Self {
        Self {
            id: SurfaceId::new(),
            buffer: None,
            damage: Vec::new(),
            transform: 0,
            scale: 1,
            pending: SurfacePendingState::default(),
            role: SurfaceRole::None,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Attach a buffer to the pending state
    pub fn attach(&mut self, buffer: Option<BufferInfo>) {
        self.pending.buffer = buffer;
    }

    /// Add damage to the pending state
    pub fn damage(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.pending.damage.push(DamageRect {
            x,
            y,
            width,
            height,
        });
    }

    /// Add a frame callback
    pub fn frame(&mut self, callback_id: u32) {
        self.pending.frame_callbacks.push(callback_id);
    }

    /// Set the buffer scale
    pub fn set_scale(&mut self, scale: i32) {
        self.pending.scale = scale;
    }

    /// Set the buffer transform
    pub fn set_transform(&mut self, transform: i32) {
        self.pending.transform = transform;
    }

    /// Commit pending state to current state
    pub fn commit(&mut self) {
        if self.pending.buffer.is_some() || self.buffer.is_none() {
            self.buffer = self.pending.buffer.take();
        }

        if !self.pending.damage.is_empty() {
            self.damage = std::mem::take(&mut self.pending.damage);
        }

        if self.pending.scale != 0 {
            self.scale = self.pending.scale;
            self.pending.scale = 0;
        }

        if self.pending.transform != 0 {
            self.transform = self.pending.transform;
            self.pending.transform = 0;
        }

        // Frame callbacks are handled separately by the caller
    }

    /// Set the surface role
    pub fn set_role(&mut self, role: SurfaceRole) -> Result<(), &'static str> {
        if self.role != SurfaceRole::None && self.role != role {
            return Err("Surface already has a different role");
        }
        self.role = role;
        Ok(())
    }
}

impl Default for Surface {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager for all surfaces
#[derive(Debug)]
pub struct SurfaceManager {
    surfaces: HashMap<SurfaceId, Surface>,
}

impl SurfaceManager {
    /// Create a new surface manager
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
        }
    }

    /// Create a new surface and return its ID
    pub fn create_surface(&mut self) -> SurfaceId {
        let surface = Surface::new();
        let id = surface.id;
        self.surfaces.insert(id, surface);
        id
    }

    /// Get a surface by ID
    pub fn get(&self, id: SurfaceId) -> Option<&Surface> {
        self.surfaces.get(&id)
    }

    /// Get a mutable surface by ID
    pub fn get_mut(&mut self, id: SurfaceId) -> Option<&mut Surface> {
        self.surfaces.get_mut(&id)
    }

    /// Remove a surface
    pub fn remove(&mut self, id: SurfaceId) -> Option<Surface> {
        self.surfaces.remove(&id)
    }

    /// Get all surfaces
    pub fn iter(&self) -> impl Iterator<Item = (&SurfaceId, &Surface)> {
        self.surfaces.iter()
    }

    /// Get count of surfaces
    pub fn len(&self) -> usize {
        self.surfaces.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }
}

impl Default for SurfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_id_unique() {
        let id1 = SurfaceId::new();
        let id2 = SurfaceId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_surface_new() {
        let surface = Surface::new();
        assert!(surface.buffer.is_none());
        assert_eq!(surface.scale, 1);
        assert_eq!(surface.role, SurfaceRole::None);
    }

    #[test]
    fn test_surface_damage() {
        let mut surface = Surface::new();
        surface.damage(0, 0, 100, 100);
        assert_eq!(surface.pending.damage.len(), 1);
        surface.commit();
        assert_eq!(surface.damage.len(), 1);
    }

    #[test]
    fn test_surface_manager() {
        let mut manager = SurfaceManager::new();
        let id = manager.create_surface();
        assert!(manager.get(id).is_some());
        manager.remove(id);
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn test_surface_role() {
        let mut surface = Surface::new();
        assert!(surface.set_role(SurfaceRole::XdgToplevel).is_ok());
        assert!(surface.set_role(SurfaceRole::XdgToplevel).is_ok()); // Same role is OK
        assert!(surface.set_role(SurfaceRole::XdgPopup).is_err()); // Different role fails
    }
}
