//! Metal device setup

use log::{debug, info};
use objc2::rc::Retained;
use objc2_metal::{MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice};

/// Metal device wrapper
pub struct MetalDevice {
    /// The Metal device
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    /// Command queue
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
}

use objc2::runtime::ProtocolObject;

impl MetalDevice {
    /// Create a new Metal device
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating Metal device");

        // Get the system default Metal device
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| anyhow::anyhow!("Failed to create Metal device"))?;

        debug!("Metal device: {:?}", device.name());

        // Create command queue
        let command_queue = device
            .newCommandQueue()
            .ok_or_else(|| anyhow::anyhow!("Failed to create command queue"))?;

        Ok(Self {
            device,
            command_queue,
        })
    }

    /// Get the raw Metal device
    pub fn raw(&self) -> &ProtocolObject<dyn MTLDevice> {
        &self.device
    }

    /// Get the command queue
    pub fn command_queue(&self) -> &ProtocolObject<dyn MTLCommandQueue> {
        &self.command_queue
    }

    /// Create a new command buffer
    pub fn new_command_buffer(
        &self,
    ) -> Option<Retained<ProtocolObject<dyn objc2_metal::MTLCommandBuffer>>> {
        self.command_queue.commandBuffer()
    }

    /// Get device name
    pub fn name(&self) -> String {
        self.device.name().to_string()
    }

    /// Check if device supports a feature
    pub fn supports_family(&self, _family: u32) -> bool {
        // Simplified - would check MTLGPUFamily
        true
    }
}

#[cfg(test)]
mod tests {
    // Note: Metal device tests require macOS with GPU
}
