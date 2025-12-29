//! Wayland global registry implementations
//!
//! Implements GlobalDispatch for advertising globals to clients.

use log::debug;
use wayland_protocols::xdg::shell::server::xdg_wm_base;
use wayland_server::protocol::{wl_compositor, wl_output, wl_seat, wl_shm};
use wayland_server::{Client, DataInit, Dispatch, GlobalDispatch, New, Resource};

use super::dispatch::{OutputData, SeatData};
use super::ServerState;

// ============================================================================
// wl_compositor global
// ============================================================================

impl GlobalDispatch<wl_compositor::WlCompositor, ()> for ServerState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<wl_compositor::WlCompositor>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        debug!("Client bound wl_compositor");
        data_init.init(resource, ());
    }
}

// ============================================================================
// wl_shm global
// ============================================================================

impl GlobalDispatch<wl_shm::WlShm, ()> for ServerState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<wl_shm::WlShm>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        debug!("Client bound wl_shm");
        let shm = data_init.init(resource, ());

        // Send supported formats
        shm.format(wl_shm::Format::Argb8888);
        shm.format(wl_shm::Format::Xrgb8888);
    }
}

// ============================================================================
// wl_seat global
// ============================================================================

impl GlobalDispatch<wl_seat::WlSeat, ()> for ServerState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<wl_seat::WlSeat>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        debug!("Client bound wl_seat");

        let capabilities = wl_seat::Capability::Pointer | wl_seat::Capability::Keyboard;

        let seat = data_init.init(resource, SeatData { capabilities });

        // Send capabilities
        seat.capabilities(capabilities);

        // Send name if version >= 2
        if seat.version() >= 2 {
            seat.name("seat0".to_string());
        }
    }
}

// ============================================================================
// wl_output global
// ============================================================================

impl GlobalDispatch<wl_output::WlOutput, ()> for ServerState {
    fn bind(
        state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<wl_output::WlOutput>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        debug!("Client bound wl_output");

        // Create a default output if we don't have one
        let output_id = if state.compositor.outputs.is_empty() {
            state.compositor.outputs.create_output(
                "default".to_string(),
                "Wayoa".to_string(),
                "Virtual Display".to_string(),
            )
        } else {
            state
                .compositor
                .outputs
                .iter()
                .next()
                .map(|(id, _)| *id)
                .unwrap()
        };

        let output = data_init.init(resource, OutputData { output_id });

        // Get output info
        if let Some(out) = state.compositor.outputs.get(output_id) {
            // Send geometry
            output.geometry(
                0, // x
                0, // y
                out.physical_width as i32,
                out.physical_height as i32,
                wl_output::Subpixel::Unknown,
                out.make.clone(),
                out.model.clone(),
                wl_output::Transform::Normal,
            );

            // Send mode
            if let Some(mode) = out.current_mode() {
                output.mode(
                    wl_output::Mode::Current | wl_output::Mode::Preferred,
                    mode.width as i32,
                    mode.height as i32,
                    mode.refresh as i32,
                );
            } else {
                // Default mode
                output.mode(
                    wl_output::Mode::Current | wl_output::Mode::Preferred,
                    1920,
                    1080,
                    60000,
                );
            }

            // Send scale if version >= 2
            if output.version() >= 2 {
                output.scale(out.scale);
            }

            // Send name if version >= 4
            if output.version() >= 4 {
                output.name(out.name.clone());
                output.description(format!("{} {}", out.make, out.model));
            }

            // Send done if version >= 2
            if output.version() >= 2 {
                output.done();
            }
        }
    }
}

// ============================================================================
// xdg_wm_base global
// ============================================================================

impl GlobalDispatch<xdg_wm_base::XdgWmBase, ()> for ServerState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<xdg_wm_base::XdgWmBase>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        debug!("Client bound xdg_wm_base");
        data_init.init(resource, ());
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &xdg_wm_base::XdgWmBase,
        request: xdg_wm_base::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            xdg_wm_base::Request::CreatePositioner { id } => {
                debug!("Creating xdg_positioner");
                data_init.init(id, PositionerData::default());
            }
            xdg_wm_base::Request::GetXdgSurface { id, surface } => {
                debug!("Creating xdg_surface");
                let surface_id = *surface.data::<crate::compositor::SurfaceId>().unwrap();
                data_init.init(
                    id,
                    XdgSurfaceData {
                        surface_id,
                        configured: false,
                    },
                );
            }
            xdg_wm_base::Request::Pong { serial } => {
                debug!("Received pong for serial {}", serial);
            }
            xdg_wm_base::Request::Destroy => {
                debug!("xdg_wm_base destroy");
            }
            _ => {}
        }
    }
}

// ============================================================================
// xdg_positioner
// ============================================================================

use wayland_protocols::xdg::shell::server::xdg_positioner;

/// Positioner data for popup placement
#[derive(Debug, Default)]
pub struct PositionerData {
    pub width: i32,
    pub height: i32,
    pub anchor_rect: (i32, i32, i32, i32),
    pub anchor: u32,
    pub gravity: u32,
    pub constraint_adjustment: u32,
    pub offset: (i32, i32),
}

impl Dispatch<xdg_positioner::XdgPositioner, PositionerData> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &xdg_positioner::XdgPositioner,
        request: xdg_positioner::Request,
        _data: &PositionerData,
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        // Note: In a real implementation, we'd need interior mutability for data
        // For now, just log the requests
        match request {
            xdg_positioner::Request::SetSize { width, height } => {
                debug!("Positioner set size {}x{}", width, height);
            }
            xdg_positioner::Request::SetAnchorRect {
                x,
                y,
                width,
                height,
            } => {
                debug!(
                    "Positioner set anchor rect ({}, {}, {}, {})",
                    x, y, width, height
                );
            }
            xdg_positioner::Request::SetAnchor { anchor } => {
                debug!("Positioner set anchor {:?}", anchor);
            }
            xdg_positioner::Request::SetGravity { gravity } => {
                debug!("Positioner set gravity {:?}", gravity);
            }
            xdg_positioner::Request::SetConstraintAdjustment {
                constraint_adjustment,
            } => {
                debug!(
                    "Positioner set constraint adjustment {:?}",
                    constraint_adjustment
                );
            }
            xdg_positioner::Request::SetOffset { x, y } => {
                debug!("Positioner set offset ({}, {})", x, y);
            }
            xdg_positioner::Request::Destroy => {
                debug!("Positioner destroy");
            }
            _ => {}
        }
    }
}

// ============================================================================
// xdg_surface
// ============================================================================

use wayland_protocols::xdg::shell::server::xdg_surface;

/// XDG surface data
pub struct XdgSurfaceData {
    pub surface_id: crate::compositor::SurfaceId,
    pub configured: bool,
}

impl Dispatch<xdg_surface::XdgSurface, XdgSurfaceData> for ServerState {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &xdg_surface::XdgSurface,
        request: xdg_surface::Request,
        data: &XdgSurfaceData,
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            xdg_surface::Request::GetToplevel { id } => {
                debug!("Creating xdg_toplevel for surface {:?}", data.surface_id);

                // Set surface role
                if let Some(surface) = state.compositor.surfaces.get_mut(data.surface_id) {
                    let _ = surface.set_role(crate::compositor::SurfaceRole::XdgToplevel);
                }

                // Create window
                let window_id = state.compositor.windows.create_window(data.surface_id);

                let toplevel = data_init.init(
                    id,
                    ToplevelData {
                        surface_id: data.surface_id,
                        window_id,
                    },
                );

                // Send initial configure
                toplevel.configure(640, 480, vec![]);

                // Send xdg_surface configure
                let serial = state.compositor.next_serial();
                resource.configure(serial);
            }
            xdg_surface::Request::GetPopup {
                id,
                parent: _,
                positioner: _,
            } => {
                debug!("Creating xdg_popup for surface {:?}", data.surface_id);

                // Set surface role
                if let Some(surface) = state.compositor.surfaces.get_mut(data.surface_id) {
                    let _ = surface.set_role(crate::compositor::SurfaceRole::XdgPopup);
                }

                let popup = data_init.init(
                    id,
                    PopupData {
                        surface_id: data.surface_id,
                    },
                );

                // Send configure
                popup.configure(0, 0, 200, 200);

                let serial = state.compositor.next_serial();
                resource.configure(serial);
            }
            xdg_surface::Request::SetWindowGeometry {
                x,
                y,
                width,
                height,
            } => {
                debug!("Set window geometry ({}, {}, {}, {})", x, y, width, height);
            }
            xdg_surface::Request::AckConfigure { serial } => {
                debug!("Ack configure {}", serial);
            }
            xdg_surface::Request::Destroy => {
                debug!("xdg_surface destroy");
            }
            _ => {}
        }
    }
}

// ============================================================================
// xdg_toplevel
// ============================================================================

use wayland_protocols::xdg::shell::server::xdg_toplevel;

/// Toplevel window data
pub struct ToplevelData {
    pub surface_id: crate::compositor::SurfaceId,
    pub window_id: crate::compositor::WindowId,
}

impl Dispatch<xdg_toplevel::XdgToplevel, ToplevelData> for ServerState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &xdg_toplevel::XdgToplevel,
        request: xdg_toplevel::Request,
        data: &ToplevelData,
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            xdg_toplevel::Request::SetParent { parent: _ } => {
                debug!("Toplevel {:?} set parent", data.window_id);
            }
            xdg_toplevel::Request::SetTitle { title } => {
                debug!("Toplevel {:?} set title: {}", data.window_id, title);
                if let Some(window) = state.compositor.windows.get_mut(data.window_id) {
                    window.title = Some(title.clone());
                }
                #[cfg(target_os = "macos")]
                if let Some(native_window) = state.native_windows.get(&data.window_id) {
                    native_window.set_title(&title);
                }
            }
            xdg_toplevel::Request::SetAppId { app_id } => {
                debug!("Toplevel {:?} set app_id: {}", data.window_id, app_id);
                if let Some(window) = state.compositor.windows.get_mut(data.window_id) {
                    window.app_id = Some(app_id);
                }
            }
            xdg_toplevel::Request::ShowWindowMenu {
                seat: _,
                serial: _,
                x,
                y,
            } => {
                debug!(
                    "Toplevel {:?} show window menu at ({}, {})",
                    data.window_id, x, y
                );
            }
            xdg_toplevel::Request::Move { seat: _, serial: _ } => {
                debug!("Toplevel {:?} move", data.window_id);
            }
            xdg_toplevel::Request::Resize {
                seat: _,
                serial: _,
                edges,
            } => {
                debug!("Toplevel {:?} resize {:?}", data.window_id, edges);
            }
            xdg_toplevel::Request::SetMaxSize { width, height } => {
                debug!(
                    "Toplevel {:?} set max size {}x{}",
                    data.window_id, width, height
                );
            }
            xdg_toplevel::Request::SetMinSize { width, height } => {
                debug!(
                    "Toplevel {:?} set min size {}x{}",
                    data.window_id, width, height
                );
            }
            xdg_toplevel::Request::SetMaximized => {
                debug!("Toplevel {:?} set maximized", data.window_id);
                if let Some(window) = state.compositor.windows.get_mut(data.window_id) {
                    window.maximized = true;
                }
                #[cfg(target_os = "macos")]
                if let Some(native_window) = state.native_windows.get(&data.window_id) {
                    native_window.set_maximized(true);
                }
            }
            xdg_toplevel::Request::UnsetMaximized => {
                debug!("Toplevel {:?} unset maximized", data.window_id);
                if let Some(window) = state.compositor.windows.get_mut(data.window_id) {
                    window.maximized = false;
                }
                #[cfg(target_os = "macos")]
                if let Some(native_window) = state.native_windows.get(&data.window_id) {
                    native_window.set_maximized(false);
                }
            }
            xdg_toplevel::Request::SetFullscreen { output: _ } => {
                debug!("Toplevel {:?} set fullscreen", data.window_id);
                if let Some(window) = state.compositor.windows.get_mut(data.window_id) {
                    window.fullscreen = true;
                }
                #[cfg(target_os = "macos")]
                if let Some(native_window) = state.native_windows.get(&data.window_id) {
                    native_window.set_fullscreen(true);
                }
            }
            xdg_toplevel::Request::UnsetFullscreen => {
                debug!("Toplevel {:?} unset fullscreen", data.window_id);
                if let Some(window) = state.compositor.windows.get_mut(data.window_id) {
                    window.fullscreen = false;
                }
                #[cfg(target_os = "macos")]
                if let Some(native_window) = state.native_windows.get(&data.window_id) {
                    native_window.set_fullscreen(false);
                }
            }
            xdg_toplevel::Request::SetMinimized => {
                debug!("Toplevel {:?} set minimized", data.window_id);
                #[cfg(target_os = "macos")]
                if let Some(native_window) = state.native_windows.get(&data.window_id) {
                    native_window.minimize();
                }
            }
            xdg_toplevel::Request::Destroy => {
                debug!("Toplevel {:?} destroy", data.window_id);

                // Remove native window
                #[cfg(target_os = "macos")]
                {
                    if let Some(native_window) = state.native_windows.remove(&data.window_id) {
                        native_window.close();
                    }
                }

                // Remove window from compositor
                state.compositor.windows.remove(data.window_id);
            }
            _ => {}
        }
    }
}

// ============================================================================
// xdg_popup
// ============================================================================

use wayland_protocols::xdg::shell::server::xdg_popup;

/// Popup data
pub struct PopupData {
    pub surface_id: crate::compositor::SurfaceId,
}

impl Dispatch<xdg_popup::XdgPopup, PopupData> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &xdg_popup::XdgPopup,
        request: xdg_popup::Request,
        data: &PopupData,
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            xdg_popup::Request::Grab { seat: _, serial: _ } => {
                debug!("Popup {:?} grab", data.surface_id);
            }
            xdg_popup::Request::Reposition {
                positioner: _,
                token: _,
            } => {
                debug!("Popup {:?} reposition", data.surface_id);
            }
            xdg_popup::Request::Destroy => {
                debug!("Popup {:?} destroy", data.surface_id);
            }
            _ => {}
        }
    }
}
