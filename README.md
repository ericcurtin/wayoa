# Wayoa

A Wayland compositor for macOS, using Metal for rendering and Cocoa for windowing.

## Overview

Wayoa is a full-featured Wayland compositor where each Wayland toplevel surface maps to a native macOS window. This allows running Linux/Wayland applications on macOS with native window management.

## Features

- **Native macOS Windows**: Each Wayland toplevel becomes an NSWindow
- **Metal Rendering**: GPU-accelerated surface composition using Apple's Metal API
- **Full Wayland Protocol Support**:
  - Core: wl_compositor, wl_surface, wl_shm, wl_output
  - XDG Shell: xdg_wm_base, xdg_surface, xdg_toplevel, xdg_popup
  - Input: wl_seat, wl_keyboard, wl_pointer
  - Extensions: wlr-layer-shell, wlr-screencopy
- **XKB Keyboard Support**: Full keyboard mapping with XKB integration
- **HiDPI Support**: Retina display aware with proper scaling

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                      Wayland Clients                             │
└──────────────────────────┬───────────────────────────────────────┘
                           │ Unix Socket (Wayland Protocol)
┌──────────────────────────▼───────────────────────────────────────┐
│                         WAYOA                                    │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Protocol Layer (wayland-server-rs)                        │  │
│  │  • wl_display, wl_registry, wl_compositor                  │  │
│  │  • xdg_shell, xdg_surface, xdg_toplevel, xdg_popup         │  │
│  │  • wl_seat, wl_keyboard, wl_pointer                        │  │
│  │  • wl_shm, wl_buffer                                       │  │
│  │  • wl_output, wl_data_device                               │  │
│  │  • wlr-layer-shell, wlr-screencopy                         │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Compositor Core                                           │  │
│  │  • Surface Manager (tracks all surfaces, damage, commits)  │  │
│  │  • Window Manager (maps toplevels to NSWindows)            │  │
│  │  • Input Router (keyboard focus, pointer grab)             │  │
│  │  • Seat (manages input devices)                            │  │
│  │  • Output Manager (monitors → wl_output)                   │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Cocoa Backend                                             │  │
│  │  • NSApplication event loop integration                    │  │
│  │  • WayoaWindow (NSWindow subclass per toplevel)            │  │
│  │  • WayoaView (NSView with CAMetalLayer)                    │  │
│  │  • Input event translation (NSEvent → Wayland events)      │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Metal Renderer                                            │  │
│  │  • Texture upload from wl_shm buffers                      │  │
│  │  • Surface composition (subsurfaces, popups)               │  │
│  │  • Damage tracking for efficient redraw                    │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
wayoa/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point, NSApplication setup
│   ├── lib.rs                  # Public API
│   ├── compositor/
│   │   ├── mod.rs
│   │   ├── state.rs            # Global compositor state
│   │   ├── surface.rs          # Surface management
│   │   ├── window.rs           # Window/toplevel management
│   │   └── output.rs           # Output/display management
│   ├── protocol/
│   │   ├── mod.rs
│   │   ├── compositor.rs       # wl_compositor, wl_surface
│   │   ├── shell.rs            # xdg_shell implementation
│   │   ├── seat.rs             # wl_seat, keyboard, pointer
│   │   ├── shm.rs              # wl_shm, buffer management
│   │   ├── output.rs           # wl_output
│   │   ├── data_device.rs      # Clipboard/DnD
│   │   ├── layer_shell.rs      # wlr-layer-shell
│   │   └── screencopy.rs       # wlr-screencopy
│   ├── backend/
│   │   ├── mod.rs
│   │   ├── cocoa/
│   │   │   ├── mod.rs
│   │   │   ├── app.rs          # NSApplication delegate
│   │   │   ├── window.rs       # NSWindow wrapper
│   │   │   ├── view.rs         # NSView with Metal layer
│   │   │   └── input.rs        # NSEvent handling
│   │   └── event_loop.rs       # Integration with calloop
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── metal/
│   │   │   ├── mod.rs
│   │   │   ├── device.rs       # MTLDevice setup
│   │   │   ├── pipeline.rs     # Render pipelines
│   │   │   ├── texture.rs      # Buffer → MTLTexture
│   │   │   └── compositor.rs   # Surface composition
│   │   └── shaders/
│   │       ├── blit.metal      # Basic texture blit
│   │       └── composite.metal # Alpha blending
│   └── input/
│       ├── mod.rs
│       ├── keyboard.rs         # Keymap, XKB integration
│       ├── pointer.rs          # Mouse/trackpad
│       └── seat.rs             # Input device coordination
└── protocols/                  # Wayland protocol XML (reference)
```

## Building

### Requirements

- macOS 10.15 or later
- Rust 1.70 or later
- Xcode Command Line Tools (for Metal compiler)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

## Usage

When running Wayoa, it creates a Wayland socket that clients can connect to:

```bash
# Set the Wayland display
export WAYLAND_DISPLAY=wayland-0

# Run a Wayland client (e.g., from a cross-compiled Linux environment)
./my-wayland-app
```

## Development Status

This project is in early development. Current status:

- [x] Project structure and build system
- [x] Compositor core (surfaces, windows, outputs)
- [x] Protocol handlers (compositor, shell, seat, shm, etc.)
- [x] Cocoa backend (NSApplication, NSWindow, NSView)
- [x] Metal renderer foundation
- [x] Input handling and translation
- [ ] Wayland server socket integration
- [ ] Full protocol event dispatch
- [ ] Clipboard integration with macOS pasteboard
- [ ] Cursor rendering
- [ ] Multi-monitor support

## License

Apache License 2.0
