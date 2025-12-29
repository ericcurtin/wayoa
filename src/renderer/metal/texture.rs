//! Metal texture management

use std::collections::HashMap;
use std::ptr::NonNull;

use log::debug;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLTexture, MTLTextureDescriptor, MTLTextureUsage};

use crate::compositor::SurfaceId;
use crate::protocol::shm::ShmFormat;

use super::MetalDevice;

/// Texture manager for surface content
pub struct TextureManager {
    /// Cached textures by surface ID
    textures: HashMap<SurfaceId, TextureEntry>,
}

/// A cached texture entry
struct TextureEntry {
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    width: u32,
    height: u32,
    format: ShmFormat,
}

impl TextureManager {
    /// Create a new texture manager
    pub fn new(_device: &MetalDevice) -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// Create or update a texture from pixel data
    #[allow(clippy::too_many_arguments)]
    pub fn upload_texture(
        &mut self,
        device: &MetalDevice,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
        stride: u32,
        format: ShmFormat,
        data: &[u8],
    ) -> anyhow::Result<()> {
        // Check if we can reuse existing texture
        let needs_new_texture = match self.textures.get(&surface_id) {
            Some(entry) => entry.width != width || entry.height != height || entry.format != format,
            None => true,
        };

        let texture = if needs_new_texture {
            // Create new texture
            let descriptor = MTLTextureDescriptor::new();
            unsafe {
                descriptor.setWidth(width as usize);
                descriptor.setHeight(height as usize);
            }
            descriptor.setPixelFormat(Self::format_to_metal(format));
            descriptor.setUsage(MTLTextureUsage::ShaderRead);

            let texture = device
                .raw()
                .newTextureWithDescriptor(&descriptor)
                .ok_or_else(|| anyhow::anyhow!("Failed to create texture"))?;

            debug!(
                "Created new texture for surface {:?}, {}x{}, format {:?}",
                surface_id, width, height, format
            );

            texture
        } else {
            self.textures.get(&surface_id).unwrap().texture.clone()
        };

        // Upload pixel data
        let region = objc2_metal::MTLRegion {
            origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
            size: objc2_metal::MTLSize {
                width: width as usize,
                height: height as usize,
                depth: 1,
            },
        };

        // Upload pixel data
        let bytes_ptr = NonNull::new(data.as_ptr() as *mut std::ffi::c_void)
            .expect("data pointer should not be null");
        unsafe {
            texture.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                region,
                0,
                bytes_ptr,
                stride as usize,
            );
        }

        // Store texture
        self.textures.insert(
            surface_id,
            TextureEntry {
                texture,
                width,
                height,
                format,
            },
        );

        Ok(())
    }

    /// Get a texture for a surface
    pub fn get(&self, surface_id: SurfaceId) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.textures.get(&surface_id).map(|e| e.texture.as_ref())
    }

    /// Remove a texture
    pub fn remove(&mut self, surface_id: SurfaceId) {
        self.textures.remove(&surface_id);
    }

    /// Convert SHM format to Metal pixel format
    fn format_to_metal(format: ShmFormat) -> MTLPixelFormat {
        match format {
            ShmFormat::Argb8888 => MTLPixelFormat::BGRA8Unorm,
            ShmFormat::Xrgb8888 => MTLPixelFormat::BGRA8Unorm,
            ShmFormat::Other(_) => MTLPixelFormat::BGRA8Unorm, // Default
        }
    }

    /// Get the number of cached textures
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_conversion() {
        assert_eq!(
            TextureManager::format_to_metal(ShmFormat::Argb8888),
            MTLPixelFormat::BGRA8Unorm
        );
    }
}
