//! wlr-layer-shell protocol implementation
//!
//! Implements layer shell surfaces for panels, overlays, etc.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use log::debug;

use crate::compositor::{OutputId, SurfaceId};

/// Unique identifier for layer surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerSurfaceId(pub u64);

impl LayerSurfaceId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        LayerSurfaceId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Layer shell layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layer {
    /// Background layer (below everything)
    Background,
    /// Bottom layer (below windows)
    #[default]
    Bottom,
    /// Top layer (above windows)
    Top,
    /// Overlay layer (above everything)
    Overlay,
}

impl Layer {
    /// Create from protocol value
    pub fn from_protocol(value: u32) -> Option<Self> {
        match value {
            0 => Some(Layer::Background),
            1 => Some(Layer::Bottom),
            2 => Some(Layer::Top),
            3 => Some(Layer::Overlay),
            _ => None,
        }
    }

    /// Convert to protocol value
    pub fn to_protocol(&self) -> u32 {
        match self {
            Layer::Background => 0,
            Layer::Bottom => 1,
            Layer::Top => 2,
            Layer::Overlay => 3,
        }
    }
}

// Edge anchoring for layer surfaces
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Anchor: u32 {
        const TOP = 1;
        const BOTTOM = 2;
        const LEFT = 4;
        const RIGHT = 8;
    }
}

/// Keyboard interactivity mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyboardInteractivity {
    /// No keyboard focus
    #[default]
    None,
    /// Exclusive keyboard focus (locks keyboard)
    Exclusive,
    /// On-demand keyboard focus
    OnDemand,
}

impl KeyboardInteractivity {
    /// Create from protocol value
    pub fn from_protocol(value: u32) -> Option<Self> {
        match value {
            0 => Some(KeyboardInteractivity::None),
            1 => Some(KeyboardInteractivity::Exclusive),
            2 => Some(KeyboardInteractivity::OnDemand),
            _ => None,
        }
    }
}

/// A layer shell surface
#[derive(Debug)]
pub struct LayerSurface {
    /// Unique identifier
    pub id: LayerSurfaceId,
    /// Associated wl_surface
    pub surface_id: SurfaceId,
    /// Target output (None = current output)
    pub output: Option<OutputId>,
    /// Layer
    pub layer: Layer,
    /// Namespace (application identifier)
    pub namespace: String,
    /// Size (0 = use anchor constraints)
    pub size: (u32, u32),
    /// Anchor edges
    pub anchor: Anchor,
    /// Exclusive zone (pixels to reserve)
    pub exclusive_zone: i32,
    /// Margin from edges
    pub margin: (i32, i32, i32, i32), // top, right, bottom, left
    /// Keyboard interactivity
    pub keyboard_interactivity: KeyboardInteractivity,
    /// Configured state
    pub configured: bool,
    /// Configure serial
    pub configure_serial: u32,
}

impl LayerSurface {
    /// Create a new layer surface
    pub fn new(
        surface_id: SurfaceId,
        output: Option<OutputId>,
        layer: Layer,
        namespace: String,
    ) -> Self {
        Self {
            id: LayerSurfaceId::new(),
            surface_id,
            output,
            layer,
            namespace,
            size: (0, 0),
            anchor: Anchor::empty(),
            exclusive_zone: 0,
            margin: (0, 0, 0, 0),
            keyboard_interactivity: KeyboardInteractivity::None,
            configured: false,
            configure_serial: 0,
        }
    }

    /// Set size
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    /// Set anchor
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    /// Set exclusive zone
    pub fn set_exclusive_zone(&mut self, zone: i32) {
        self.exclusive_zone = zone;
    }

    /// Set margin
    pub fn set_margin(&mut self, top: i32, right: i32, bottom: i32, left: i32) {
        self.margin = (top, right, bottom, left);
    }

    /// Set layer
    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    /// Set keyboard interactivity
    pub fn set_keyboard_interactivity(&mut self, mode: KeyboardInteractivity) {
        self.keyboard_interactivity = mode;
    }

    /// Calculate the geometry based on output and anchoring
    pub fn calculate_geometry(
        &self,
        output_width: u32,
        output_height: u32,
    ) -> (i32, i32, u32, u32) {
        let (mut width, mut height) = self.size;
        let (margin_top, margin_right, margin_bottom, margin_left) = self.margin;

        // If anchored to opposite edges and size is 0, stretch to fill
        if self.anchor.contains(Anchor::LEFT | Anchor::RIGHT) && width == 0 {
            width = output_width - (margin_left + margin_right) as u32;
        }
        if self.anchor.contains(Anchor::TOP | Anchor::BOTTOM) && height == 0 {
            height = output_height - (margin_top + margin_bottom) as u32;
        }

        // Calculate position based on anchoring
        let x = if self.anchor.contains(Anchor::LEFT) {
            margin_left
        } else if self.anchor.contains(Anchor::RIGHT) {
            output_width as i32 - width as i32 - margin_right
        } else {
            (output_width as i32 - width as i32) / 2
        };

        let y = if self.anchor.contains(Anchor::TOP) {
            margin_top
        } else if self.anchor.contains(Anchor::BOTTOM) {
            output_height as i32 - height as i32 - margin_bottom
        } else {
            (output_height as i32 - height as i32) / 2
        };

        (x, y, width, height)
    }
}

/// Handler for wlr-layer-shell protocol
pub struct LayerShellHandler {
    surfaces: HashMap<LayerSurfaceId, LayerSurface>,
    /// Map from surface ID to layer surface ID
    surface_to_layer: HashMap<SurfaceId, LayerSurfaceId>,
}

impl LayerShellHandler {
    /// Create a new layer shell handler
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
            surface_to_layer: HashMap::new(),
        }
    }

    /// Create a layer surface
    pub fn get_layer_surface(
        &mut self,
        surface_id: SurfaceId,
        output: Option<OutputId>,
        layer: Layer,
        namespace: String,
    ) -> LayerSurfaceId {
        let layer_surface = LayerSurface::new(surface_id, output, layer, namespace);
        let id = layer_surface.id;
        self.surface_to_layer.insert(surface_id, id);
        self.surfaces.insert(id, layer_surface);
        debug!(
            "Created layer surface {:?} for surface {:?}",
            id, surface_id
        );
        id
    }

    /// Get a layer surface
    pub fn get(&self, id: LayerSurfaceId) -> Option<&LayerSurface> {
        self.surfaces.get(&id)
    }

    /// Get a mutable layer surface
    pub fn get_mut(&mut self, id: LayerSurfaceId) -> Option<&mut LayerSurface> {
        self.surfaces.get_mut(&id)
    }

    /// Get layer surface by wl_surface
    pub fn get_by_surface(&self, surface_id: SurfaceId) -> Option<&LayerSurface> {
        self.surface_to_layer
            .get(&surface_id)
            .and_then(|id| self.surfaces.get(id))
    }

    /// Destroy a layer surface
    pub fn destroy(&mut self, id: LayerSurfaceId) {
        if let Some(surface) = self.surfaces.remove(&id) {
            self.surface_to_layer.remove(&surface.surface_id);
            debug!("Destroyed layer surface {:?}", id);
        }
    }

    /// Get all layer surfaces on a specific layer
    pub fn surfaces_on_layer(&self, layer: Layer) -> impl Iterator<Item = &LayerSurface> {
        self.surfaces.values().filter(move |s| s.layer == layer)
    }

    /// Get count of layer surfaces
    pub fn len(&self) -> usize {
        self.surfaces.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }
}

impl Default for LayerShellHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer() {
        assert_eq!(Layer::from_protocol(0), Some(Layer::Background));
        assert_eq!(Layer::Top.to_protocol(), 2);
    }

    #[test]
    fn test_layer_surface_geometry() {
        let mut surface = LayerSurface::new(SurfaceId(1), None, Layer::Top, "test".to_string());

        // Panel at top, full width, 50px height
        surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        surface.set_size(0, 50);

        let (x, y, w, h) = surface.calculate_geometry(1920, 1080);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert_eq!(w, 1920);
        assert_eq!(h, 50);
    }

    #[test]
    fn test_layer_shell_handler() {
        let mut handler = LayerShellHandler::new();

        let id = handler.get_layer_surface(SurfaceId(1), None, Layer::Top, "panel".to_string());

        assert!(handler.get(id).is_some());
        assert!(handler.get_by_surface(SurfaceId(1)).is_some());

        handler.destroy(id);
        assert!(handler.get(id).is_none());
    }
}
