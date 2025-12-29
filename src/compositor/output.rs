//! Output/display management
//!
//! This module tracks monitors/displays and maps them to wl_output.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for outputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputId(pub u64);

impl OutputId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        OutputId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Output transform (rotation/flip)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputTransform {
    #[default]
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

impl OutputTransform {
    /// Convert to Wayland wl_output::transform value
    pub fn to_wayland(&self) -> u32 {
        match self {
            OutputTransform::Normal => 0,
            OutputTransform::Rotate90 => 1,
            OutputTransform::Rotate180 => 2,
            OutputTransform::Rotate270 => 3,
            OutputTransform::Flipped => 4,
            OutputTransform::Flipped90 => 5,
            OutputTransform::Flipped180 => 6,
            OutputTransform::Flipped270 => 7,
        }
    }
}

/// Output subpixel layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Subpixel {
    #[default]
    Unknown,
    None,
    HorizontalRgb,
    HorizontalBgr,
    VerticalRgb,
    VerticalBgr,
}

impl Subpixel {
    /// Convert to Wayland wl_output::subpixel value
    pub fn to_wayland(&self) -> u32 {
        match self {
            Subpixel::Unknown => 0,
            Subpixel::None => 1,
            Subpixel::HorizontalRgb => 2,
            Subpixel::HorizontalBgr => 3,
            Subpixel::VerticalRgb => 4,
            Subpixel::VerticalBgr => 5,
        }
    }
}

/// An output mode (resolution + refresh rate)
#[derive(Debug, Clone, Copy)]
pub struct OutputMode {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Refresh rate in mHz (e.g., 60000 for 60Hz)
    pub refresh: u32,
    /// Is this the current mode?
    pub current: bool,
    /// Is this the preferred mode?
    pub preferred: bool,
}

/// A display output (monitor)
#[derive(Debug)]
pub struct Output {
    /// Unique identifier
    pub id: OutputId,
    /// Output name (e.g., "eDP-1")
    pub name: String,
    /// Manufacturer name
    pub make: String,
    /// Model name
    pub model: String,
    /// Serial number
    pub serial: String,
    /// Position in global coordinate space
    pub x: i32,
    pub y: i32,
    /// Physical size in millimeters
    pub physical_width: u32,
    pub physical_height: u32,
    /// Transform applied to output
    pub transform: OutputTransform,
    /// Subpixel layout
    pub subpixel: Subpixel,
    /// Available modes
    pub modes: Vec<OutputMode>,
    /// Current mode index
    pub current_mode: Option<usize>,
    /// Scale factor
    pub scale: f64,
}

impl Output {
    /// Create a new output
    pub fn new(name: String) -> Self {
        Self {
            id: OutputId::new(),
            name,
            make: String::new(),
            model: String::new(),
            serial: String::new(),
            x: 0,
            y: 0,
            physical_width: 0,
            physical_height: 0,
            transform: OutputTransform::Normal,
            subpixel: Subpixel::Unknown,
            modes: Vec::new(),
            current_mode: None,
            scale: 1.0,
        }
    }

    /// Get the current mode
    pub fn current_mode(&self) -> Option<&OutputMode> {
        self.current_mode.and_then(|i| self.modes.get(i))
    }

    /// Get current width
    pub fn width(&self) -> u32 {
        self.current_mode().map(|m| m.width).unwrap_or(0)
    }

    /// Get current height
    pub fn height(&self) -> u32 {
        self.current_mode().map(|m| m.height).unwrap_or(0)
    }

    /// Add a mode
    pub fn add_mode(&mut self, mode: OutputMode) {
        let is_current = mode.current;
        self.modes.push(mode);
        if is_current {
            self.current_mode = Some(self.modes.len() - 1);
        }
    }
}

/// Manager for all outputs
#[derive(Debug)]
pub struct OutputManager {
    outputs: HashMap<OutputId, Output>,
    /// Primary output
    primary: Option<OutputId>,
}

impl OutputManager {
    /// Create a new output manager
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
            primary: None,
        }
    }

    /// Add an output
    pub fn add(&mut self, output: Output) -> OutputId {
        let id = output.id;
        let is_first = self.outputs.is_empty();
        self.outputs.insert(id, output);
        if is_first {
            self.primary = Some(id);
        }
        id
    }

    /// Get an output by ID
    pub fn get(&self, id: OutputId) -> Option<&Output> {
        self.outputs.get(&id)
    }

    /// Get a mutable output by ID
    pub fn get_mut(&mut self, id: OutputId) -> Option<&mut Output> {
        self.outputs.get_mut(&id)
    }

    /// Remove an output
    pub fn remove(&mut self, id: OutputId) -> Option<Output> {
        let output = self.outputs.remove(&id);
        if self.primary == Some(id) {
            self.primary = self.outputs.keys().next().copied();
        }
        output
    }

    /// Get the primary output
    pub fn primary(&self) -> Option<&Output> {
        self.primary.and_then(|id| self.outputs.get(&id))
    }

    /// Set the primary output
    pub fn set_primary(&mut self, id: OutputId) {
        if self.outputs.contains_key(&id) {
            self.primary = Some(id);
        }
    }

    /// Get all outputs
    pub fn iter(&self) -> impl Iterator<Item = (&OutputId, &Output)> {
        self.outputs.iter()
    }

    /// Get count of outputs
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_id_unique() {
        let id1 = OutputId::new();
        let id2 = OutputId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_output_new() {
        let output = Output::new("test".to_string());
        assert_eq!(output.name, "test");
        assert!(output.modes.is_empty());
    }

    #[test]
    fn test_output_mode() {
        let mut output = Output::new("test".to_string());
        output.add_mode(OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60000,
            current: true,
            preferred: true,
        });
        assert_eq!(output.width(), 1920);
        assert_eq!(output.height(), 1080);
    }

    #[test]
    fn test_output_manager() {
        let mut manager = OutputManager::new();
        let output = Output::new("test".to_string());
        let id = manager.add(output);
        assert!(manager.get(id).is_some());
        assert!(manager.primary().is_some());
        manager.remove(id);
        assert!(manager.get(id).is_none());
    }
}
