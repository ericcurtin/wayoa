//! Wayland protocol dispatch implementations
//!
//! Implements the Dispatch trait for each Wayland protocol object.

use log::{debug, warn};
use wayland_server::protocol::{
    wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_region, wl_seat,
    wl_shm, wl_shm_pool, wl_surface,
};
use wayland_server::{Client, DataInit, Dispatch, Resource};

use crate::compositor::{SurfaceId, SurfaceRole};

use super::ServerState;

// ============================================================================
// wl_compositor
// ============================================================================

impl Dispatch<wl_compositor::WlCompositor, ()> for ServerState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &wl_compositor::WlCompositor,
        request: wl_compositor::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                let surface_id = state.compositor.surfaces.create_surface();
                debug!("Created wl_surface {:?}", surface_id);
                data_init.init(id, surface_id);
            }
            wl_compositor::Request::CreateRegion { id } => {
                debug!("Created wl_region");
                data_init.init(id, ());
            }
            _ => {}
        }
    }
}

// ============================================================================
// wl_surface
// ============================================================================

impl Dispatch<wl_surface::WlSurface, SurfaceId> for ServerState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &wl_surface::WlSurface,
        request: wl_surface::Request,
        surface_id: &SurfaceId,
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        let Some(surface) = state.compositor.surfaces.get_mut(*surface_id) else {
            warn!("Surface {:?} not found", surface_id);
            return;
        };

        match request {
            wl_surface::Request::Attach { buffer, x, y } => {
                debug!("Surface {:?} attach buffer at ({}, {})", surface_id, x, y);
                if buffer.is_some() {
                    // Get buffer info from our shm handler if available
                    // For now, just mark that we have a buffer attached
                    surface.attach(Some(crate::compositor::surface::BufferInfo {
                        width: 0, // Will be filled in from shm buffer
                        height: 0,
                        stride: 0,
                        format: 0,
                        offset: 0,
                    }));
                } else {
                    surface.attach(None);
                }
            }
            wl_surface::Request::Damage {
                x,
                y,
                width,
                height,
            } => {
                debug!(
                    "Surface {:?} damage ({}, {}, {}, {})",
                    surface_id, x, y, width, height
                );
                surface.damage(x, y, width, height);
            }
            wl_surface::Request::DamageBuffer {
                x,
                y,
                width,
                height,
            } => {
                debug!(
                    "Surface {:?} damage_buffer ({}, {}, {}, {})",
                    surface_id, x, y, width, height
                );
                surface.damage(x, y, width, height);
            }
            wl_surface::Request::Frame { callback } => {
                debug!("Surface {:?} frame callback", surface_id);
                let cb: wl_callback::WlCallback = data_init.init(callback, ());
                surface.frame(cb.id().protocol_id());
            }
            wl_surface::Request::SetOpaqueRegion { region: _ } => {
                debug!("Surface {:?} set opaque region", surface_id);
            }
            wl_surface::Request::SetInputRegion { region: _ } => {
                debug!("Surface {:?} set input region", surface_id);
            }
            wl_surface::Request::Commit => {
                debug!("Surface {:?} commit", surface_id);

                // Get the frame callbacks before committing
                let _frame_callbacks: Vec<u32> =
                    surface.pending.frame_callbacks.drain(..).collect();

                // Commit the surface state
                surface.commit();

                // Check if this surface is a toplevel and needs a native window
                #[cfg(target_os = "macos")]
                {
                    let surface = state.compositor.surfaces.get(*surface_id).unwrap();
                    if surface.role == SurfaceRole::XdgToplevel {
                        // Find the window for this surface
                        if let Some(window_id) =
                            state.compositor.windows.window_for_surface(*surface_id)
                        {
                            // Create native window if it doesn't exist
                            if !state.native_windows.contains_key(&window_id) {
                                if let Some(mtm) = state.mtm {
                                    let (width, height) = surface
                                        .buffer
                                        .as_ref()
                                        .map(|b| (b.width.max(640), b.height.max(480)))
                                        .unwrap_or((640, 480));

                                    match crate::backend::cocoa::window::WayoaWindow::new(
                                        mtm,
                                        window_id,
                                        width,
                                        height,
                                        "Wayland Window",
                                    ) {
                                        Ok(window) => {
                                            window.show();
                                            state.native_windows.insert(window_id, window);
                                            debug!("Created native window for {:?}", window_id);
                                        }
                                        Err(e) => {
                                            warn!("Failed to create native window: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Fire frame callbacks
                // In a full implementation, this would be done after rendering
                // For now, we'll just mark them as done
            }
            wl_surface::Request::SetBufferTransform { transform } => {
                debug!("Surface {:?} set transform {:?}", surface_id, transform);
                // Convert WEnum to raw value
                let transform_val = match transform {
                    wayland_server::WEnum::Value(v) => v as i32,
                    wayland_server::WEnum::Unknown(v) => v as i32,
                };
                surface.set_transform(transform_val);
            }
            wl_surface::Request::SetBufferScale { scale } => {
                debug!("Surface {:?} set scale {}", surface_id, scale);
                surface.set_scale(scale);
            }
            wl_surface::Request::Offset { x, y } => {
                debug!("Surface {:?} offset ({}, {})", surface_id, x, y);
            }
            wl_surface::Request::Destroy => {
                debug!("Surface {:?} destroy", surface_id);
                state.compositor.surfaces.remove(*surface_id);
            }
            _ => {}
        }
    }

    fn destroyed(
        state: &mut Self,
        _client: wayland_server::backend::ClientId,
        _resource: &wl_surface::WlSurface,
        data: &SurfaceId,
    ) {
        debug!("Surface {:?} destroyed", data);
        state.compositor.surfaces.remove(*data);
    }
}

// ============================================================================
// wl_region
// ============================================================================

impl Dispatch<wl_region::WlRegion, ()> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &wl_region::WlRegion,
        request: wl_region::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_region::Request::Add {
                x,
                y,
                width,
                height,
            } => {
                debug!("Region add ({}, {}, {}, {})", x, y, width, height);
            }
            wl_region::Request::Subtract {
                x,
                y,
                width,
                height,
            } => {
                debug!("Region subtract ({}, {}, {}, {})", x, y, width, height);
            }
            wl_region::Request::Destroy => {
                debug!("Region destroy");
            }
            _ => {}
        }
    }
}

// ============================================================================
// wl_callback
// ============================================================================

impl Dispatch<wl_callback::WlCallback, ()> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &wl_callback::WlCallback,
        _request: wl_callback::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        // wl_callback has no requests
    }
}

// ============================================================================
// wl_shm
// ============================================================================

impl Dispatch<wl_shm::WlShm, ()> for ServerState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &wl_shm::WlShm,
        request: wl_shm::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        if let wl_shm::Request::CreatePool { id, fd, size } = request {
            use std::os::unix::io::AsRawFd;
            debug!("Creating shm pool, size {}", size);
            let pool_id = state.shm.create_pool(fd.as_raw_fd(), size as usize);
            data_init.init(id, pool_id);
        }
    }
}

// ============================================================================
// wl_shm_pool
// ============================================================================

impl Dispatch<wl_shm_pool::WlShmPool, crate::protocol::shm::ShmPoolId> for ServerState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &wl_shm_pool::WlShmPool,
        request: wl_shm_pool::Request,
        pool_id: &crate::protocol::shm::ShmPoolId,
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_shm_pool::Request::CreateBuffer {
                id,
                offset,
                width,
                height,
                stride,
                format,
            } => {
                debug!(
                    "Creating buffer {}x{} from pool {:?}",
                    width, height, pool_id
                );
                match state.shm.create_buffer(
                    *pool_id,
                    offset as u32,
                    width as u32,
                    height as u32,
                    stride as u32,
                    format.into(),
                ) {
                    Ok(buffer_id) => {
                        data_init.init(id, buffer_id);
                    }
                    Err(e) => {
                        warn!("Failed to create buffer: {}", e);
                    }
                }
            }
            wl_shm_pool::Request::Resize { size } => {
                debug!("Resizing pool {:?} to {}", pool_id, size);
                let _ = state.shm.resize_pool(*pool_id, size as usize);
            }
            wl_shm_pool::Request::Destroy => {
                debug!("Destroying pool {:?}", pool_id);
                state.shm.destroy_pool(*pool_id);
            }
            _ => {}
        }
    }
}

// ============================================================================
// wl_buffer
// ============================================================================

impl Dispatch<wl_buffer::WlBuffer, crate::protocol::shm::ShmBufferId> for ServerState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &wl_buffer::WlBuffer,
        request: wl_buffer::Request,
        buffer_id: &crate::protocol::shm::ShmBufferId,
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        if let wl_buffer::Request::Destroy = request {
            debug!("Destroying buffer {:?}", buffer_id);
            state.shm.destroy_buffer(*buffer_id);
        }
    }
}

// ============================================================================
// wl_seat
// ============================================================================

/// Seat user data - tracks capabilities
pub struct SeatData {
    pub capabilities: wl_seat::Capability,
}

impl Dispatch<wl_seat::WlSeat, SeatData> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &wl_seat::WlSeat,
        request: wl_seat::Request,
        _data: &SeatData,
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_seat::Request::GetPointer { id } => {
                debug!("Creating pointer");
                data_init.init(id, ());
            }
            wl_seat::Request::GetKeyboard { id } => {
                debug!("Creating keyboard");
                data_init.init(id, ());
            }
            wl_seat::Request::GetTouch { id: _ } => {
                debug!("Creating touch");
                // Touch not implemented, but we'll create the object
            }
            wl_seat::Request::Release => {
                debug!("Seat release");
            }
            _ => {}
        }
    }
}

// ============================================================================
// wl_pointer
// ============================================================================

impl Dispatch<wl_pointer::WlPointer, ()> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &wl_pointer::WlPointer,
        request: wl_pointer::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_pointer::Request::SetCursor {
                serial: _,
                surface: _,
                hotspot_x,
                hotspot_y,
            } => {
                debug!("Set cursor at ({}, {})", hotspot_x, hotspot_y);
            }
            wl_pointer::Request::Release => {
                debug!("Pointer release");
            }
            _ => {}
        }
    }
}

// ============================================================================
// wl_keyboard
// ============================================================================

impl Dispatch<wl_keyboard::WlKeyboard, ()> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &wl_keyboard::WlKeyboard,
        request: wl_keyboard::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        if let wl_keyboard::Request::Release = request {
            debug!("Keyboard release");
        }
    }
}

// ============================================================================
// wl_output
// ============================================================================

/// Output user data
pub struct OutputData {
    pub output_id: crate::compositor::OutputId,
}

impl Dispatch<wl_output::WlOutput, OutputData> for ServerState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &wl_output::WlOutput,
        request: wl_output::Request,
        _data: &OutputData,
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        if let wl_output::Request::Release = request {
            debug!("Output release");
        }
    }
}
