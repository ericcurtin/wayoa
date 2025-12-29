//! Cocoa backend for macOS
//!
//! This module provides the macOS-specific implementation using:
//! - NSApplication for the application lifecycle
//! - NSWindow for native windows (one per Wayland toplevel)
//! - NSView with CAMetalLayer for Metal rendering
//! - NSEvent handling for input translation

pub mod app;
pub mod input;
pub mod view;
pub mod window;

pub use app::WayoaApp;
pub use input::InputTranslator;
pub use view::MetalView;
pub use window::{NativeWindowHandle, WayoaWindow};
