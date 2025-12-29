//! Wayland protocol implementations
//!
//! This module contains implementations of various Wayland protocols:
//! - wl_compositor, wl_surface
//! - xdg_shell (xdg_wm_base, xdg_surface, xdg_toplevel, xdg_popup)
//! - wl_seat, wl_keyboard, wl_pointer, wl_touch
//! - wl_shm, wl_buffer
//! - wl_output
//! - wl_data_device (clipboard/drag-and-drop)
//! - wlr-layer-shell
//! - wlr-screencopy

pub mod compositor;
pub mod data_device;
pub mod layer_shell;
pub mod output;
pub mod screencopy;
pub mod seat;
pub mod shell;
pub mod shm;

pub use compositor::WlCompositorHandler;
pub use data_device::DataDeviceHandler;
pub use layer_shell::LayerShellHandler;
pub use output::WlOutputHandler;
pub use screencopy::ScreencopyHandler;
pub use seat::WlSeatHandler;
pub use shell::XdgShellHandler;
pub use shm::WlShmHandler;
