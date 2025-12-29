//! Simple Wayland test client
//!
//! This client connects to the compositor, creates a window, and displays it.
//! Run with: cargo run --example test_client

use std::os::unix::io::AsFd;

use wayland_client::{
    protocol::{wl_buffer, wl_compositor, wl_registry, wl_seat, wl_shm, wl_shm_pool, wl_surface},
    Connection, Dispatch, EventQueue, QueueHandle, WEnum,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

fn main() -> anyhow::Result<()> {
    println!("Connecting to Wayland compositor...");

    // Connect to the Wayland display
    let conn = Connection::connect_to_env()?;
    println!("Connected!");

    // Create event queue
    let mut event_queue: EventQueue<AppState> = conn.new_event_queue();
    let qh = event_queue.handle();

    // Get the registry
    let display = conn.display();
    display.get_registry(&qh, ());

    // Create app state
    let mut state = AppState {
        running: true,
        compositor: None,
        shm: None,
        seat: None,
        xdg_wm_base: None,
        surface: None,
        xdg_surface: None,
        xdg_toplevel: None,
        buffer: None,
        configured: false,
    };

    // Roundtrip to get globals
    println!("Getting globals...");
    event_queue.roundtrip(&mut state)?;

    // Create surface
    if let Some(compositor) = &state.compositor {
        println!("Creating surface...");
        let surface = compositor.create_surface(&qh, ());
        state.surface = Some(surface);
    } else {
        anyhow::bail!("No wl_compositor available");
    }

    // Create xdg_surface
    if let (Some(xdg_wm_base), Some(surface)) = (&state.xdg_wm_base, &state.surface) {
        println!("Creating xdg_surface...");
        let xdg_surface = xdg_wm_base.get_xdg_surface(surface, &qh, ());
        state.xdg_surface = Some(xdg_surface);
    } else {
        anyhow::bail!("No xdg_wm_base available");
    }

    // Create xdg_toplevel
    if let Some(xdg_surface) = &state.xdg_surface {
        println!("Creating xdg_toplevel...");
        let xdg_toplevel = xdg_surface.get_toplevel(&qh, ());
        xdg_toplevel.set_title("Wayoa Test Client".to_string());
        xdg_toplevel.set_app_id("wayoa.test.client".to_string());
        state.xdg_toplevel = Some(xdg_toplevel);
    }

    // Commit to trigger configure
    if let Some(surface) = &state.surface {
        surface.commit();
    }

    // Wait for configure
    println!("Waiting for configure...");
    while !state.configured {
        event_queue.blocking_dispatch(&mut state)?;
    }

    // Create a buffer
    if let Some(shm) = &state.shm {
        println!("Creating buffer...");
        let width = 640i32;
        let height = 480i32;
        let stride = width * 4;
        let size = stride * height;

        // Create shared memory
        let file = tempfile::tempfile()?;
        file.set_len(size as u64)?;

        // Map and fill with color
        let mut mmap = unsafe { memmap2::MmapMut::map_mut(&file)? };
        for y in 0..height {
            for x in 0..width {
                let offset = ((y * stride) + (x * 4)) as usize;
                // ARGB format - fill with blue
                mmap[offset] = 0xFF; // B
                mmap[offset + 1] = 0x00; // G
                mmap[offset + 2] = 0x00; // R
                mmap[offset + 3] = 0xFF; // A
            }
        }

        // Create shm pool
        let pool = shm.create_pool(file.as_fd(), size, &qh, ());

        // Create buffer
        let buffer =
            pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, &qh, ());
        state.buffer = Some(buffer);

        // Attach buffer to surface
        if let (Some(surface), Some(buffer)) = (&state.surface, &state.buffer) {
            surface.attach(Some(buffer), 0, 0);
            surface.damage_buffer(0, 0, width, height);
            surface.commit();
        }
    }

    println!("Window created! Running event loop...");
    println!("(Press Ctrl+C to exit)");

    // Run event loop
    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
    }

    println!("Done!");
    Ok(())
}

struct AppState {
    running: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    seat: Option<wl_seat::WlSeat>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    surface: Option<wl_surface::WlSurface>,
    xdg_surface: Option<xdg_surface::XdgSurface>,
    xdg_toplevel: Option<xdg_toplevel::XdgToplevel>,
    buffer: Option<wl_buffer::WlBuffer>,
    configured: bool,
}

// Registry handler
impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("  Global: {} v{}", interface, version);
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version.min(6),
                        qh,
                        (),
                    );
                    state.compositor = Some(compositor);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, version.min(1), qh, ());
                    state.shm = Some(shm);
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(9), qh, ());
                    state.seat = Some(seat);
                }
                "xdg_wm_base" => {
                    let xdg_wm_base =
                        registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, version.min(6), qh, ());
                    state.xdg_wm_base = Some(xdg_wm_base);
                }
                _ => {}
            }
        }
    }
}

// Compositor handler
impl Dispatch<wl_compositor::WlCompositor, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

// Surface handler
impl Dispatch<wl_surface::WlSurface, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

// SHM handler
impl Dispatch<wl_shm::WlShm, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_shm::Event::Format { format } = event {
            println!("  SHM format: {:?}", format);
        }
    }
}

// SHM pool handler
impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

// Buffer handler
impl Dispatch<wl_buffer::WlBuffer, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            println!("Buffer released");
        }
    }
}

// Seat handler
impl Dispatch<wl_seat::WlSeat, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_seat::Event::Capabilities { capabilities } => {
                println!("  Seat capabilities: {:?}", capabilities);
            }
            wl_seat::Event::Name { name } => {
                println!("  Seat name: {}", name);
            }
            _ => {}
        }
    }
}

// XDG WM Base handler
impl Dispatch<xdg_wm_base::XdgWmBase, ()> for AppState {
    fn event(
        _state: &mut Self,
        proxy: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            println!("Received ping, sending pong");
            proxy.pong(serial);
        }
    }
}

// XDG Surface handler
impl Dispatch<xdg_surface::XdgSurface, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            println!("XDG surface configured (serial {})", serial);
            proxy.ack_configure(serial);
            state.configured = true;
        }
    }
}

// XDG Toplevel handler
impl Dispatch<xdg_toplevel::XdgToplevel, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Configure {
                width,
                height,
                states,
            } => {
                println!(
                    "Toplevel configure: {}x{}, states: {:?}",
                    width, height, states
                );
            }
            xdg_toplevel::Event::Close => {
                println!("Close requested");
                state.running = false;
            }
            _ => {}
        }
    }
}
