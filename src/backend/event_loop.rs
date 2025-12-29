//! Event loop integration
//!
//! Integrates calloop with the platform-specific event loop.

use std::time::Duration;

use calloop::{EventLoop as CalLoop, LoopHandle, LoopSignal};
use log::{debug, error};

/// Wayoa event loop wrapper
pub struct EventLoop {
    /// Calloop event loop
    event_loop: CalLoop<'static, ()>,
    /// Loop signal for waking/stopping
    signal: LoopSignal,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> anyhow::Result<Self> {
        let event_loop = CalLoop::try_new()?;
        let signal = event_loop.get_signal();

        Ok(Self { event_loop, signal })
    }

    /// Get a handle to register event sources
    pub fn handle(&self) -> LoopHandle<'static, ()> {
        self.event_loop.handle()
    }

    /// Get the loop signal for waking
    pub fn signal(&self) -> LoopSignal {
        self.signal.clone()
    }

    /// Run one iteration of the event loop
    pub fn dispatch(&mut self, timeout: Option<Duration>) -> anyhow::Result<()> {
        self.event_loop.dispatch(timeout, &mut ())?;
        Ok(())
    }

    /// Run the event loop until stopped
    pub fn run(&mut self) -> anyhow::Result<()> {
        debug!("Starting event loop");

        loop {
            if let Err(e) = self.dispatch(None) {
                error!("Event loop error: {}", e);
                return Err(e);
            }
        }
    }

    /// Stop the event loop
    pub fn stop(&self) {
        self.signal.stop();
    }

    /// Wake the event loop from another thread
    pub fn wake(&self) {
        self.signal.wakeup();
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new().expect("Failed to create event loop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_new() {
        let event_loop = EventLoop::new();
        assert!(event_loop.is_ok());
    }

    #[test]
    fn test_event_loop_dispatch() {
        let mut event_loop = EventLoop::new().unwrap();
        // Dispatch with zero timeout should return immediately
        let result = event_loop.dispatch(Some(Duration::ZERO));
        assert!(result.is_ok());
    }
}
