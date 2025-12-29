//! Metal render pipeline setup

use log::{debug, info};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLDevice, MTLFunction, MTLLibrary, MTLPixelFormat, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState, MTLVertexDescriptor,
};

use super::MetalDevice;

/// Vertex data for rendering quads
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
}

/// Metal render pipeline
pub struct RenderPipeline {
    /// Pipeline state object
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    /// Vertex function
    _vertex_function: Retained<ProtocolObject<dyn MTLFunction>>,
    /// Fragment function
    _fragment_function: Retained<ProtocolObject<dyn MTLFunction>>,
}

impl RenderPipeline {
    /// Create a new render pipeline
    pub fn new(device: &MetalDevice) -> anyhow::Result<Self> {
        info!("Creating Metal render pipeline");

        // Create shader library from source
        let shader_source = include_str!("../shaders/blit.metal");
        let source = NSString::from_str(shader_source);

        let library = unsafe {
            device
                .raw()
                .newLibraryWithSource_options_error(&source, None)
        }
        .map_err(|e| anyhow::anyhow!("Failed to compile shaders: {:?}", e))?;

        // Get shader functions
        let vertex_name = NSString::from_str("vertex_main");
        let vertex_function = library
            .newFunctionWithName(&vertex_name)
            .ok_or_else(|| anyhow::anyhow!("Failed to find vertex function"))?;

        let fragment_name = NSString::from_str("fragment_main");
        let fragment_function = library
            .newFunctionWithName(&fragment_name)
            .ok_or_else(|| anyhow::anyhow!("Failed to find fragment function"))?;

        // Create pipeline descriptor
        let pipeline_descriptor = MTLRenderPipelineDescriptor::new();
        pipeline_descriptor.setVertexFunction(Some(&vertex_function));
        pipeline_descriptor.setFragmentFunction(Some(&fragment_function));

        // Set up color attachment
        unsafe {
            let color_attachments = pipeline_descriptor.colorAttachments();
            let attachment = color_attachments.objectAtIndexedSubscript(0);
            attachment.setPixelFormat(MTLPixelFormat::BGRA8Unorm);

            // Enable blending for alpha
            attachment.setBlendingEnabled(true);
            attachment.setSourceRGBBlendFactor(objc2_metal::MTLBlendFactor::SourceAlpha);
            attachment.setDestinationRGBBlendFactor(
                objc2_metal::MTLBlendFactor::OneMinusSourceAlpha,
            );
            attachment.setSourceAlphaBlendFactor(objc2_metal::MTLBlendFactor::One);
            attachment.setDestinationAlphaBlendFactor(
                objc2_metal::MTLBlendFactor::OneMinusSourceAlpha,
            );
        }

        // Create pipeline state
        let pipeline_state = unsafe {
            device
                .raw()
                .newRenderPipelineStateWithDescriptor_error(&pipeline_descriptor)
        }
        .map_err(|e| anyhow::anyhow!("Failed to create pipeline state: {:?}", e))?;

        debug!("Render pipeline created successfully");

        Ok(Self {
            pipeline_state,
            _vertex_function: vertex_function,
            _fragment_function: fragment_function,
        })
    }

    /// Get the pipeline state object
    pub fn state(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.pipeline_state
    }

    /// Create vertex data for a full-screen quad
    pub fn create_quad_vertices(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> [Vertex; 6] {
        // Convert from pixel coordinates to normalized device coordinates
        let left = (x / viewport_width) * 2.0 - 1.0;
        let right = ((x + width) / viewport_width) * 2.0 - 1.0;
        let top = 1.0 - (y / viewport_height) * 2.0;
        let bottom = 1.0 - ((y + height) / viewport_height) * 2.0;

        [
            // First triangle
            Vertex {
                position: [left, top],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                position: [right, top],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                position: [left, bottom],
                tex_coord: [0.0, 1.0],
            },
            // Second triangle
            Vertex {
                position: [right, top],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                position: [right, bottom],
                tex_coord: [1.0, 1.0],
            },
            Vertex {
                position: [left, bottom],
                tex_coord: [0.0, 1.0],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad_vertices() {
        let vertices = RenderPipeline::create_quad_vertices(0.0, 0.0, 100.0, 100.0, 200.0, 200.0);
        assert_eq!(vertices.len(), 6);

        // Check that the first vertex is top-left
        assert_eq!(vertices[0].position, [-1.0, 1.0]);
        assert_eq!(vertices[0].tex_coord, [0.0, 0.0]);
    }
}
