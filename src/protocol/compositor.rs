//! wl_compositor protocol implementation
//!
//! The compositor global is responsible for creating surfaces and regions.

use log::debug;

use crate::compositor::{CompositorState, SurfaceId};

/// Handler for wl_compositor protocol
pub struct WlCompositorHandler;

impl WlCompositorHandler {
    /// Create a new compositor handler
    pub fn new() -> Self {
        Self
    }

    /// Create a new surface
    pub fn create_surface(&self, state: &mut CompositorState) -> SurfaceId {
        let id = state.surfaces.create_surface();
        debug!("Created surface {:?}", id);
        id
    }

    /// Create a new region (for input/opaque regions)
    pub fn create_region(&self) -> Region {
        Region::new()
    }
}

impl Default for WlCompositorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// A region for defining input or opaque areas
#[derive(Debug, Clone, Default)]
pub struct Region {
    /// List of rectangles that make up the region
    rects: Vec<RegionRect>,
}

/// A rectangle operation in a region
#[derive(Debug, Clone)]
struct RegionRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    add: bool, // true = add, false = subtract
}

impl Region {
    /// Create a new empty region
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }

    /// Add a rectangle to the region
    pub fn add(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rects.push(RegionRect {
            x,
            y,
            width,
            height,
            add: true,
        });
    }

    /// Subtract a rectangle from the region
    pub fn subtract(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rects.push(RegionRect {
            x,
            y,
            width,
            height,
            add: false,
        });
    }

    /// Check if a point is inside the region
    pub fn contains(&self, px: i32, py: i32) -> bool {
        let mut inside = false;
        for rect in &self.rects {
            let in_rect = px >= rect.x
                && px < rect.x + rect.width
                && py >= rect.y
                && py < rect.y + rect.height;
            if rect.add && in_rect {
                inside = true;
            } else if !rect.add && in_rect {
                inside = false;
            }
        }
        inside
    }

    /// Check if the region is empty
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_add() {
        let mut region = Region::new();
        region.add(0, 0, 100, 100);
        assert!(region.contains(50, 50));
        assert!(!region.contains(150, 150));
    }

    #[test]
    fn test_region_subtract() {
        let mut region = Region::new();
        region.add(0, 0, 100, 100);
        region.subtract(25, 25, 50, 50);
        assert!(region.contains(10, 10));
        assert!(!region.contains(50, 50));
    }

    #[test]
    fn test_create_surface() {
        let handler = WlCompositorHandler::new();
        let mut state = CompositorState::new();
        let id = handler.create_surface(&mut state);
        assert!(state.surfaces.get(id).is_some());
    }
}
