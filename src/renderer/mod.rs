//! Rendering module
//!
//! This module handles rendering using Metal on macOS.
//! It includes texture management, shader pipelines, and surface composition.

#[cfg(target_os = "macos")]
pub mod metal;

// Re-export Metal renderer on macOS
#[cfg(target_os = "macos")]
pub use metal::MetalRenderer;

// Stub for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub struct MetalRenderer;

#[cfg(not(target_os = "macos"))]
impl MetalRenderer {
    pub fn new() -> anyhow::Result<Self> {
        anyhow::bail!("Metal renderer is only available on macOS")
    }
}
