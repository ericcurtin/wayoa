//! Input handling module
//!
//! This module provides keyboard, pointer, and seat management.

pub mod keyboard;
pub mod pointer;
pub mod seat;

pub use keyboard::Keyboard;
pub use pointer::Pointer;
pub use seat::Seat;
