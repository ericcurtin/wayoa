//! Backend module
//!
//! This module contains platform-specific backends:
//! - Cocoa backend for macOS (NSWindow, Metal rendering)
//! - Event loop integration with calloop

#[cfg(target_os = "macos")]
pub mod cocoa;
pub mod event_loop;

pub use event_loop::EventLoop;
