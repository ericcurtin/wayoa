//! Compositor core module
//!
//! This module contains the core compositor logic including:
//! - Global compositor state management
//! - Surface management and damage tracking
//! - Window/toplevel management
//! - Output/display management

pub mod output;
pub mod state;
pub mod surface;
pub mod window;

pub use output::{Output, OutputId, OutputManager, OutputMode};
pub use state::CompositorState;
pub use surface::{Surface, SurfaceId, SurfaceManager, SurfaceRole};
pub use window::{Window, WindowId, WindowManager};
