//! Wayoa - A Wayland compositor for macOS
//!
//! Wayoa is a full-featured Wayland compositor using Metal for rendering
//! and Cocoa for windowing. Each Wayland toplevel surface maps to a native
//! macOS window.
//!
//! # Architecture
//!
//! - **Protocol Layer**: Implements Wayland protocols using wayland-server-rs
//! - **Compositor Core**: Manages surfaces, windows, and input routing
//! - **Cocoa Backend**: NSApplication event loop, NSWindow per toplevel
//! - **Metal Renderer**: GPU-accelerated surface composition
//!
//! # Example
//!
//! ```no_run
//! use wayoa::compositor::CompositorState;
//!
//! // The compositor is typically run via the main binary
//! // See src/main.rs for the entry point
//! ```

pub mod backend;
pub mod compositor;
pub mod input;
pub mod protocol;
pub mod renderer;
