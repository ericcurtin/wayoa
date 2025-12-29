//! wl_output protocol implementation
//!
//! Implements display/monitor enumeration and configuration.

use log::debug;

use crate::compositor::{Output, OutputId, OutputMode};

/// Handler for wl_output protocol
pub struct WlOutputHandler;

impl WlOutputHandler {
    /// Create a new output handler
    pub fn new() -> Self {
        Self
    }

    /// Send output geometry event
    pub fn send_geometry(&self, output: &Output) -> OutputGeometryEvent {
        OutputGeometryEvent {
            x: output.x,
            y: output.y,
            physical_width: output.physical_width as i32,
            physical_height: output.physical_height as i32,
            subpixel: output.subpixel.to_wayland() as i32,
            make: output.make.clone(),
            model: output.model.clone(),
            transform: output.transform.to_wayland() as i32,
        }
    }

    /// Send output mode event
    pub fn send_mode(&self, mode: &OutputMode) -> OutputModeEvent {
        let mut flags = 0u32;
        if mode.current {
            flags |= 1; // WL_OUTPUT_MODE_CURRENT
        }
        if mode.preferred {
            flags |= 2; // WL_OUTPUT_MODE_PREFERRED
        }

        OutputModeEvent {
            flags,
            width: mode.width as i32,
            height: mode.height as i32,
            refresh: mode.refresh as i32,
        }
    }

    /// Send output scale event
    pub fn send_scale(&self, output: &Output) -> i32 {
        output.scale.round() as i32
    }

    /// Send output name event (wl_output version 4+)
    pub fn send_name(&self, output: &Output) -> String {
        output.name.clone()
    }

    /// Send output description event (wl_output version 4+)
    pub fn send_description(&self, output: &Output) -> String {
        format!("{} {}", output.make, output.model)
    }

    /// Handle release request
    pub fn release(&self, _output_id: OutputId) {
        debug!("Output released");
    }
}

impl Default for WlOutputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Output geometry event data
#[derive(Debug, Clone)]
pub struct OutputGeometryEvent {
    pub x: i32,
    pub y: i32,
    pub physical_width: i32,
    pub physical_height: i32,
    pub subpixel: i32,
    pub make: String,
    pub model: String,
    pub transform: i32,
}

/// Output mode event data
#[derive(Debug, Clone)]
pub struct OutputModeEvent {
    pub flags: u32,
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

/// Enumerate outputs from the system
#[cfg(target_os = "macos")]
pub fn enumerate_outputs() -> Vec<Output> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    let mut outputs = Vec::new();

    // This requires running on the main thread
    if let Some(mtm) = MainThreadMarker::new() {
        let screens = NSScreen::screens(mtm);
        for (i, screen) in screens.iter().enumerate() {
            let frame = screen.frame();
            let visible_frame = screen.visibleFrame();

            let mut output = Output::new(format!("screen-{}", i));
            output.make = "Apple".to_string();
            output.model = format!("Display {}", i);
            output.x = frame.origin.x as i32;
            output.y = frame.origin.y as i32;

            // Get backing scale factor for Retina displays
            output.scale = screen.backingScaleFactor();

            // Add current mode
            output.add_mode(OutputMode {
                width: frame.size.width as u32,
                height: frame.size.height as u32,
                refresh: 60000, // Assume 60Hz
                current: true,
                preferred: true,
            });

            outputs.push(output);
        }
    }

    outputs
}

#[cfg(not(target_os = "macos"))]
pub fn enumerate_outputs() -> Vec<Output> {
    // Return a dummy output for non-macOS platforms
    let mut output = Output::new("dummy-0".to_string());
    output.make = "Virtual".to_string();
    output.model = "Display".to_string();
    output.add_mode(OutputMode {
        width: 1920,
        height: 1080,
        refresh: 60000,
        current: true,
        preferred: true,
    });
    vec![output]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_handler() {
        let handler = WlOutputHandler::new();
        let mut output = Output::new("test".to_string());
        output.make = "Test".to_string();
        output.model = "Monitor".to_string();
        output.scale = 2.0;

        let geometry = handler.send_geometry(&output);
        assert_eq!(geometry.make, "Test");

        assert_eq!(handler.send_scale(&output), 2);
        assert_eq!(handler.send_description(&output), "Test Monitor");
    }

    #[test]
    fn test_mode_event() {
        let handler = WlOutputHandler::new();
        let mode = OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60000,
            current: true,
            preferred: true,
        };

        let event = handler.send_mode(&mode);
        assert_eq!(event.width, 1920);
        assert_eq!(event.height, 1080);
        assert_eq!(event.flags, 3); // CURRENT | PREFERRED
    }
}
