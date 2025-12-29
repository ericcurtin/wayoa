//! Surface composition with Metal

use std::ptr::NonNull;

use log::debug;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLCommandBuffer, MTLCommandEncoder, MTLDrawable, MTLLoadAction, MTLRenderCommandEncoder,
    MTLRenderPassDescriptor, MTLStoreAction,
};
use objc2_quartz_core::CAMetalDrawable;

use super::{MetalDevice, RenderPipeline, TextureManager};
use crate::compositor::SurfaceId;

/// Metal surface compositor
pub struct MetalCompositor {
    /// Clear color (RGBA)
    clear_color: [f64; 4],
}

impl MetalCompositor {
    /// Create a new compositor
    pub fn new(_device: &MetalDevice) -> Self {
        Self {
            clear_color: [0.0, 0.0, 0.0, 1.0], // Black background
        }
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear_color = [r, g, b, a];
    }

    /// Begin a render pass to a drawable
    pub fn begin_render_pass(
        &self,
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        drawable: &ProtocolObject<dyn CAMetalDrawable>,
    ) -> Option<Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>> {
        let render_pass = MTLRenderPassDescriptor::new();

        unsafe {
            let color_attachments = render_pass.colorAttachments();
            let attachment = color_attachments.objectAtIndexedSubscript(0);

            let texture = drawable.texture();
            attachment.setTexture(Some(&texture));
            attachment.setLoadAction(MTLLoadAction::Clear);
            attachment.setStoreAction(MTLStoreAction::Store);
            attachment.setClearColor(objc2_metal::MTLClearColor {
                red: self.clear_color[0],
                green: self.clear_color[1],
                blue: self.clear_color[2],
                alpha: self.clear_color[3],
            });
        }

        command_buffer.renderCommandEncoderWithDescriptor(&render_pass)
    }

    /// Render a surface to the current render pass
    #[allow(clippy::too_many_arguments)]
    pub fn render_surface(
        &self,
        encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
        pipeline: &RenderPipeline,
        textures: &TextureManager,
        surface_id: SurfaceId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        let texture = match textures.get(surface_id) {
            Some(t) => t,
            None => {
                debug!("No texture for surface {:?}", surface_id);
                return;
            }
        };

        // Set pipeline state
        encoder.setRenderPipelineState(pipeline.state());

        // Create vertex data
        let vertices = RenderPipeline::create_quad_vertices(
            x,
            y,
            width,
            height,
            viewport_width,
            viewport_height,
        );

        // Set vertex buffer
        let bytes_ptr = NonNull::new(vertices.as_ptr() as *mut std::ffi::c_void)
            .expect("vertices pointer should not be null");
        unsafe {
            encoder.setVertexBytes_length_atIndex(bytes_ptr, std::mem::size_of_val(&vertices), 0);
        }

        // Set texture
        unsafe {
            encoder.setFragmentTexture_atIndex(Some(texture), 0);
        }

        // Draw
        unsafe {
            encoder.drawPrimitives_vertexStart_vertexCount(
                objc2_metal::MTLPrimitiveType::Triangle,
                0,
                6,
            );
        }
    }

    /// End the render pass and present
    pub fn end_render_pass(
        &self,
        encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        drawable: &ProtocolObject<dyn CAMetalDrawable>,
    ) {
        encoder.endEncoding();
        // Cast CAMetalDrawable to MTLDrawable (CAMetalDrawable conforms to MTLDrawable)
        let mtl_drawable: &ProtocolObject<dyn MTLDrawable> =
            unsafe { &*(drawable as *const _ as *const ProtocolObject<dyn MTLDrawable>) };
        command_buffer.presentDrawable(mtl_drawable);
        command_buffer.commit();
    }

    /// Composite all surfaces for a window
    #[allow(clippy::too_many_arguments)]
    pub fn composite_window(
        &self,
        device: &MetalDevice,
        pipeline: &RenderPipeline,
        textures: &TextureManager,
        drawable: &ProtocolObject<dyn CAMetalDrawable>,
        surfaces: &[(SurfaceId, f32, f32, f32, f32)], // (id, x, y, width, height)
        viewport_width: f32,
        viewport_height: f32,
    ) {
        let command_buffer = match device.new_command_buffer() {
            Some(cb) => cb,
            None => {
                debug!("Failed to create command buffer");
                return;
            }
        };

        let encoder = match self.begin_render_pass(&command_buffer, drawable) {
            Some(e) => e,
            None => {
                debug!("Failed to create render encoder");
                return;
            }
        };

        // Render each surface
        for (surface_id, x, y, width, height) in surfaces {
            self.render_surface(
                &encoder,
                pipeline,
                textures,
                *surface_id,
                *x,
                *y,
                *width,
                *height,
                viewport_width,
                viewport_height,
            );
        }

        self.end_render_pass(&encoder, &command_buffer, drawable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_color() {
        let device = MetalDevice::new();
        if device.is_err() {
            // Skip test if Metal is not available
            return;
        }

        let mut compositor = MetalCompositor::new(&device.unwrap());
        compositor.set_clear_color(1.0, 0.0, 0.0, 1.0);
        assert_eq!(compositor.clear_color, [1.0, 0.0, 0.0, 1.0]);
    }
}
