//! Global compositor state
//!
//! This module contains the central compositor state that coordinates
//! all subsystems including surfaces, windows, input, and outputs.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::compositor::{OutputManager, SurfaceManager, WindowManager};
use crate::input::Seat;

/// Unique identifier for clients
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);

impl ClientId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        ClientId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// The global compositor state
///
/// This struct holds all the state needed to run the compositor,
/// including surfaces, windows, inputs, and outputs.
pub struct CompositorState {
    /// Surface manager - tracks all wl_surfaces
    pub surfaces: SurfaceManager,

    /// Window manager - maps toplevels to native windows
    pub windows: WindowManager,

    /// Output manager - tracks displays/monitors
    pub outputs: OutputManager,

    /// Input seat - manages keyboard, pointer, touch
    pub seat: Seat,

    /// Connected clients
    clients: HashMap<ClientId, ClientData>,

    /// Serial counter for Wayland events
    serial: AtomicU64,
}

/// Per-client data
#[derive(Debug)]
pub struct ClientData {
    pub id: ClientId,
    // Additional client-specific data can be added here
}

impl CompositorState {
    /// Create a new compositor state
    pub fn new() -> Self {
        Self {
            surfaces: SurfaceManager::new(),
            windows: WindowManager::new(),
            outputs: OutputManager::new(),
            seat: Seat::new(),
            clients: HashMap::new(),
            serial: AtomicU64::new(1),
        }
    }

    /// Get the next serial number for Wayland events
    pub fn next_serial(&self) -> u32 {
        self.serial.fetch_add(1, Ordering::Relaxed) as u32
    }

    /// Register a new client
    pub fn add_client(&mut self) -> ClientId {
        let id = ClientId::new();
        self.clients.insert(id, ClientData { id });
        id
    }

    /// Remove a client and clean up its resources
    pub fn remove_client(&mut self, id: ClientId) {
        self.clients.remove(&id);
        // TODO: Clean up surfaces and windows owned by this client
    }

    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

impl Default for CompositorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_id_unique() {
        let id1 = ClientId::new();
        let id2 = ClientId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_compositor_state_new() {
        let state = CompositorState::new();
        assert_eq!(state.client_count(), 0);
    }

    #[test]
    fn test_add_remove_client() {
        let mut state = CompositorState::new();
        let id = state.add_client();
        assert_eq!(state.client_count(), 1);
        state.remove_client(id);
        assert_eq!(state.client_count(), 0);
    }

    #[test]
    fn test_serial_increments() {
        let state = CompositorState::new();
        let s1 = state.next_serial();
        let s2 = state.next_serial();
        assert!(s2 > s1);
    }
}
