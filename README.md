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

## Usage

When running Wayoa, it creates a Wayland socket that clients can connect to:

```bash
# Set the Wayland display
export WAYLAND_DISPLAY=wayland-0

# Run a Wayland client (e.g., from a cross-compiled Linux environment)
./my-wayland-app
```

