//! Wayland server implementation
//!
//! This module sets up the Wayland display server, registers globals,
//! and dispatches protocol events to the compositor.

mod dispatch;
mod globals;

use std::os::unix::io::AsFd;
use std::sync::{Arc, Mutex};

use calloop::generic::Generic;
use calloop::{Interest, LoopHandle, Mode, PostAction};
use log::{debug, error, info};
use wayland_server::{Display, ListeningSocket};

use crate::compositor::CompositorState;
use crate::protocol::WlShmHandler;

pub use dispatch::*;
pub use globals::*;

/// The Wayland server state
///
/// This holds the compositor state and protocol handlers,
/// wrapped in Arc<Mutex<>> for safe sharing with the Wayland dispatch.
pub struct WaylandServer {
    /// The Wayland display
    display: Display<ServerState>,
    /// Listening socket for client connections
    socket: ListeningSocket,
    /// Socket name for WAYLAND_DISPLAY
    socket_name: String,
}

/// State passed to Wayland dispatch handlers
pub struct ServerState {
    /// Compositor state (surfaces, windows, outputs, seat)
    pub compositor: CompositorState,
    /// SHM handler
    pub shm: WlShmHandler,
    /// Main thread marker (for creating native windows)
    #[cfg(target_os = "macos")]
    pub mtm: Option<objc2_foundation::MainThreadMarker>,
    /// Native windows
    #[cfg(target_os = "macos")]
    pub native_windows: std::collections::HashMap<
        crate::compositor::WindowId,
        crate::backend::cocoa::window::WayoaWindow,
    >,
}

impl ServerState {
    /// Create a new server state
    pub fn new() -> Self {
        Self {
            compositor: CompositorState::new(),
            shm: WlShmHandler::new(),
            #[cfg(target_os = "macos")]
            mtm: None,
            #[cfg(target_os = "macos")]
            native_windows: std::collections::HashMap::new(),
        }
    }

    /// Set the main thread marker (must be called from main thread)
    #[cfg(target_os = "macos")]
    pub fn set_main_thread_marker(&mut self, mtm: objc2_foundation::MainThreadMarker) {
        self.mtm = Some(mtm);
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl WaylandServer {
    /// Create a new Wayland server
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating Wayland display server");

        // Create the Wayland display
        let display: Display<ServerState> = Display::new()?;

        // Create a listening socket
        let socket = ListeningSocket::bind_auto("wayland", 0..33)?;
        let socket_name = socket
            .socket_name()
            .and_then(|n| n.to_str().map(String::from))
            .unwrap_or_else(|| "wayland-0".to_string());

        info!("Wayland socket: {}", socket_name);

        Ok(Self {
            display,
            socket,
            socket_name,
        })
    }

    /// Get the socket name (for WAYLAND_DISPLAY)
    pub fn socket_name(&self) -> &str {
        &self.socket_name
    }

    /// Get a handle to the display for registering globals
    pub fn display_handle(&self) -> wayland_server::DisplayHandle {
        self.display.handle()
    }

    /// Register all protocol globals
    pub fn register_globals(&mut self) {
        let dh = self.display.handle();

        // Register wl_compositor (version 6)
        dh.create_global::<ServerState, wayland_server::protocol::wl_compositor::WlCompositor, _>(
            6,
            (),
        );

        // Register wl_shm (version 1)
        dh.create_global::<ServerState, wayland_server::protocol::wl_shm::WlShm, _>(1, ());

        // Register wl_seat (version 9)
        dh.create_global::<ServerState, wayland_server::protocol::wl_seat::WlSeat, _>(9, ());

        // Register wl_output (version 4)
        dh.create_global::<ServerState, wayland_server::protocol::wl_output::WlOutput, _>(4, ());

        // Register xdg_wm_base (version 6)
        dh.create_global::<ServerState, wayland_protocols::xdg::shell::server::xdg_wm_base::XdgWmBase, _>(6, ());

        info!("Registered Wayland globals: wl_compositor, wl_shm, wl_seat, wl_output, xdg_wm_base");
    }

    /// Insert the Wayland event sources into a calloop event loop
    pub fn insert_into_loop(
        mut self,
        handle: LoopHandle<'static, Arc<Mutex<ServerState>>>,
        _state: Arc<Mutex<ServerState>>,
    ) -> anyhow::Result<()> {
        // Insert the listening socket
        handle.insert_source(
            Generic::new(
                self.socket.as_fd().try_clone_to_owned()?,
                Interest::READ,
                Mode::Level,
            ),
            {
                let socket = self.socket;
                let mut display_handle = self.display.handle();
                move |_, _, state| {
                    // Accept new client connections
                    if let Some(stream) = socket.accept()? {
                        debug!("New Wayland client connected");
                        let mut state_guard = state.lock().unwrap();
                        if let Err(e) = display_handle.insert_client(stream, Arc::new(())) {
                            error!("Failed to insert client: {}", e);
                        } else {
                            state_guard.compositor.add_client();
                        }
                    }
                    Ok(PostAction::Continue)
                }
            },
        )?;

        // Insert the display's event source
        handle.insert_source(
            Generic::new(
                self.display.backend().poll_fd().try_clone_to_owned()?,
                Interest::READ,
                Mode::Level,
            ),
            {
                let mut display = self.display;
                move |_, _, state| {
                    let mut state_guard = state.lock().unwrap();
                    display.dispatch_clients(&mut *state_guard)?;
                    display.flush_clients()?;
                    Ok(PostAction::Continue)
                }
            },
        )?;

        Ok(())
    }

    /// Dispatch pending events (for use without calloop)
    pub fn dispatch(&mut self, state: &mut ServerState) -> anyhow::Result<()> {
        // Accept any new connections
        while let Some(stream) = self.socket.accept()? {
            debug!("New Wayland client connected");
            if let Err(e) = self.display.handle().insert_client(stream, Arc::new(())) {
                error!("Failed to insert client: {}", e);
            } else {
                state.compositor.add_client();
            }
        }

        // Dispatch to clients
        self.display.dispatch_clients(state)?;
        self.display.flush_clients()?;

        Ok(())
    }
}
