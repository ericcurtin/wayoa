//! wlr-screencopy protocol implementation
//!
//! Implements screen capture functionality.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use log::debug;

use crate::compositor::OutputId;
use crate::protocol::shm::ShmBufferId;

/// Unique identifier for screencopy frames
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreencopyFrameId(pub u64);

impl ScreencopyFrameId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        ScreencopyFrameId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

// Frame capture flags
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct FrameFlags: u32 {
        /// Y-axis is inverted (origin at bottom-left)
        const Y_INVERT = 1;
    }
}

/// A screencopy frame request
#[derive(Debug)]
pub struct ScreencopyFrame {
    /// Unique identifier
    pub id: ScreencopyFrameId,
    /// Target output
    pub output: OutputId,
    /// Capture region (None = full output)
    pub region: Option<CaptureRegion>,
    /// Whether to overlay the cursor
    pub overlay_cursor: bool,
    /// Buffer format info (sent to client)
    pub buffer_info: Option<BufferInfo>,
    /// Buffer to copy into
    pub buffer: Option<ShmBufferId>,
    /// Frame state
    pub state: FrameState,
}

/// Capture region
#[derive(Debug, Clone, Copy)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Buffer format information
#[derive(Debug, Clone)]
pub struct BufferInfo {
    /// Pixel format
    pub format: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Stride (bytes per row)
    pub stride: u32,
}

/// Frame capture state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameState {
    /// Waiting for buffer info to be sent
    #[default]
    Pending,
    /// Buffer info sent, waiting for client buffer
    Ready,
    /// Copying frame data
    Copying,
    /// Frame copied successfully
    Done,
    /// Frame capture failed
    Failed,
}

impl ScreencopyFrame {
    /// Create a new frame request
    pub fn new(output: OutputId, overlay_cursor: bool) -> Self {
        Self {
            id: ScreencopyFrameId::new(),
            output,
            region: None,
            overlay_cursor,
            buffer_info: None,
            buffer: None,
            state: FrameState::Pending,
        }
    }

    /// Set capture region
    pub fn set_region(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.region = Some(CaptureRegion {
            x,
            y,
            width,
            height,
        });
    }

    /// Set buffer info (called by compositor)
    pub fn set_buffer_info(&mut self, format: u32, width: u32, height: u32, stride: u32) {
        self.buffer_info = Some(BufferInfo {
            format,
            width,
            height,
            stride,
        });
        self.state = FrameState::Ready;
    }

    /// Copy frame to provided buffer
    pub fn copy(&mut self, buffer: ShmBufferId) {
        self.buffer = Some(buffer);
        self.state = FrameState::Copying;
    }

    /// Mark frame as done
    pub fn done(&mut self, flags: FrameFlags, tv_sec: u32, tv_nsec: u32) -> FrameDoneInfo {
        self.state = FrameState::Done;
        FrameDoneInfo {
            flags,
            tv_sec,
            tv_nsec,
        }
    }

    /// Mark frame as failed
    pub fn fail(&mut self) {
        self.state = FrameState::Failed;
    }
}

/// Frame done info
#[derive(Debug, Clone)]
pub struct FrameDoneInfo {
    pub flags: FrameFlags,
    pub tv_sec: u32,
    pub tv_nsec: u32,
}

/// Handler for wlr-screencopy protocol
pub struct ScreencopyHandler {
    frames: HashMap<ScreencopyFrameId, ScreencopyFrame>,
}

impl ScreencopyHandler {
    /// Create a new screencopy handler
    pub fn new() -> Self {
        Self {
            frames: HashMap::new(),
        }
    }

    /// Capture an output
    pub fn capture_output(&mut self, output: OutputId, overlay_cursor: bool) -> ScreencopyFrameId {
        let frame = ScreencopyFrame::new(output, overlay_cursor);
        let id = frame.id;
        self.frames.insert(id, frame);
        debug!("Created screencopy frame {:?} for output {:?}", id, output);
        id
    }

    /// Capture a region of an output
    pub fn capture_output_region(
        &mut self,
        output: OutputId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        overlay_cursor: bool,
    ) -> ScreencopyFrameId {
        let mut frame = ScreencopyFrame::new(output, overlay_cursor);
        frame.set_region(x, y, width, height);
        let id = frame.id;
        self.frames.insert(id, frame);
        debug!(
            "Created screencopy frame {:?} for output {:?}, region ({}, {}, {}, {})",
            id, output, x, y, width, height
        );
        id
    }

    /// Get a frame
    pub fn get(&self, id: ScreencopyFrameId) -> Option<&ScreencopyFrame> {
        self.frames.get(&id)
    }

    /// Get a mutable frame
    pub fn get_mut(&mut self, id: ScreencopyFrameId) -> Option<&mut ScreencopyFrame> {
        self.frames.get_mut(&id)
    }

    /// Destroy a frame
    pub fn destroy(&mut self, id: ScreencopyFrameId) {
        self.frames.remove(&id);
        debug!("Destroyed screencopy frame {:?}", id);
    }

    /// Get count of pending frames
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl Default for ScreencopyHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screencopy_frame() {
        let mut frame = ScreencopyFrame::new(OutputId(1), false);
        assert_eq!(frame.state, FrameState::Pending);

        frame.set_buffer_info(0, 1920, 1080, 7680);
        assert_eq!(frame.state, FrameState::Ready);

        frame.copy(ShmBufferId(1));
        assert_eq!(frame.state, FrameState::Copying);

        frame.done(FrameFlags::empty(), 0, 0);
        assert_eq!(frame.state, FrameState::Done);
    }

    #[test]
    fn test_screencopy_handler() {
        let mut handler = ScreencopyHandler::new();

        let id = handler.capture_output(OutputId(1), false);
        assert!(handler.get(id).is_some());

        handler.destroy(id);
        assert!(handler.get(id).is_none());
    }

    #[test]
    fn test_capture_region() {
        let mut handler = ScreencopyHandler::new();
        let id = handler.capture_output_region(OutputId(1), 0, 0, 100, 100, true);
        let frame = handler.get(id).unwrap();
        assert!(frame.region.is_some());
        assert!(frame.overlay_cursor);
    }
}
