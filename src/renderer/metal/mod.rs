//! Metal renderer for macOS
//!
//! This module provides GPU-accelerated rendering using Apple's Metal API.

pub mod compositor;
pub mod device;
pub mod pipeline;
pub mod texture;

pub use compositor::MetalCompositor;
pub use device::MetalDevice;
pub use pipeline::RenderPipeline;
pub use texture::TextureManager;

use log::info;

/// High-level Metal renderer
pub struct MetalRenderer {
    /// Metal device
    pub device: MetalDevice,
    /// Render pipeline
    pub pipeline: Option<RenderPipeline>,
    /// Texture manager
    pub textures: TextureManager,
    /// Surface compositor
    pub compositor: MetalCompositor,
}

impl MetalRenderer {
    /// Create a new Metal renderer
    pub fn new() -> anyhow::Result<Self> {
        info!("Initializing Metal renderer");

        let device = MetalDevice::new()?;
        let textures = TextureManager::new(&device);
        let compositor = MetalCompositor::new(&device);

        Ok(Self {
            device,
            pipeline: None,
            textures,
            compositor,
        })
    }

    /// Initialize the render pipeline (requires shaders to be loaded)
    pub fn init_pipeline(&mut self) -> anyhow::Result<()> {
        self.pipeline = Some(RenderPipeline::new(&self.device)?);
        Ok(())
    }

    /// Check if the renderer is ready
    pub fn is_ready(&self) -> bool {
        self.pipeline.is_some()
    }
}
