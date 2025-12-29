//! wl_shm protocol implementation
//!
//! Implements shared memory buffer management for software rendering.

use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicU64, Ordering};

use log::debug;

/// Unique identifier for shm pools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShmPoolId(pub u64);

impl ShmPoolId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        ShmPoolId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Unique identifier for shm buffers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShmBufferId(pub u64);

impl ShmBufferId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        ShmBufferId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmFormat {
    /// 32-bit ARGB (A in high byte)
    Argb8888,
    /// 32-bit XRGB (X in high byte, alpha ignored)
    Xrgb8888,
    /// Other format with raw value
    Other(u32),
}

impl ShmFormat {
    /// Create from Wayland format value
    pub fn from_wayland(format: u32) -> Self {
        match format {
            0 => ShmFormat::Argb8888,
            1 => ShmFormat::Xrgb8888,
            other => ShmFormat::Other(other),
        }
    }

    /// Convert to Wayland format value
    pub fn to_wayland(&self) -> u32 {
        match self {
            ShmFormat::Argb8888 => 0,
            ShmFormat::Xrgb8888 => 1,
            ShmFormat::Other(v) => *v,
        }
    }

    /// Get bytes per pixel
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            ShmFormat::Argb8888 | ShmFormat::Xrgb8888 => 4,
            ShmFormat::Other(_) => 4, // Assume 4 for unknown formats
        }
    }
}

/// A shared memory pool
#[derive(Debug)]
pub struct ShmPool {
    /// Unique identifier
    pub id: ShmPoolId,
    /// File descriptor for the shared memory
    pub fd: RawFd,
    /// Size of the pool in bytes
    pub size: usize,
    /// Memory-mapped data (when mapped)
    #[cfg(target_os = "macos")]
    pub data: Option<memmap2::Mmap>,
    #[cfg(not(target_os = "macos"))]
    pub data: Option<()>,
}

impl ShmPool {
    /// Create a new shm pool
    pub fn new(fd: RawFd, size: usize) -> Self {
        Self {
            id: ShmPoolId::new(),
            fd,
            size,
            data: None,
        }
    }

    /// Resize the pool
    pub fn resize(&mut self, new_size: usize) {
        if new_size > self.size {
            self.size = new_size;
            // Re-map will happen on next access
            self.data = None;
        }
    }
}

/// A buffer created from an shm pool
#[derive(Debug, Clone)]
pub struct ShmBuffer {
    /// Unique identifier
    pub id: ShmBufferId,
    /// Parent pool
    pub pool_id: ShmPoolId,
    /// Offset into the pool
    pub offset: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Stride (bytes per row)
    pub stride: u32,
    /// Pixel format
    pub format: ShmFormat,
}

impl ShmBuffer {
    /// Create a new buffer
    pub fn new(
        pool_id: ShmPoolId,
        offset: u32,
        width: u32,
        height: u32,
        stride: u32,
        format: ShmFormat,
    ) -> Self {
        Self {
            id: ShmBufferId::new(),
            pool_id,
            offset,
            width,
            height,
            stride,
            format,
        }
    }

    /// Get the size of the buffer data in bytes
    pub fn data_size(&self) -> usize {
        (self.stride * self.height) as usize
    }
}

/// Handler for wl_shm protocol
pub struct WlShmHandler {
    pools: HashMap<ShmPoolId, ShmPool>,
    buffers: HashMap<ShmBufferId, ShmBuffer>,
}

impl WlShmHandler {
    /// Create a new shm handler
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            buffers: HashMap::new(),
        }
    }

    /// Get supported formats
    pub fn formats(&self) -> Vec<ShmFormat> {
        vec![ShmFormat::Argb8888, ShmFormat::Xrgb8888]
    }

    /// Create a new shm pool
    pub fn create_pool(&mut self, fd: RawFd, size: usize) -> ShmPoolId {
        let pool = ShmPool::new(fd, size);
        let id = pool.id;
        self.pools.insert(id, pool);
        debug!("Created shm pool {:?}, size {}", id, size);
        id
    }

    /// Resize a pool
    pub fn resize_pool(&mut self, pool_id: ShmPoolId, new_size: usize) -> Result<(), ShmError> {
        let pool = self.pools.get_mut(&pool_id).ok_or(ShmError::InvalidPool)?;
        pool.resize(new_size);
        debug!("Resized shm pool {:?} to {}", pool_id, new_size);
        Ok(())
    }

    /// Destroy a pool
    pub fn destroy_pool(&mut self, pool_id: ShmPoolId) {
        self.pools.remove(&pool_id);
        debug!("Destroyed shm pool {:?}", pool_id);
    }

    /// Create a buffer from a pool
    pub fn create_buffer(
        &mut self,
        pool_id: ShmPoolId,
        offset: u32,
        width: u32,
        height: u32,
        stride: u32,
        format: u32,
    ) -> Result<ShmBufferId, ShmError> {
        // Validate pool exists
        let pool = self.pools.get(&pool_id).ok_or(ShmError::InvalidPool)?;

        let format = ShmFormat::from_wayland(format);

        // Validate buffer fits in pool
        let buffer_end = offset as usize + (stride * height) as usize;
        if buffer_end > pool.size {
            return Err(ShmError::BufferTooLarge);
        }

        // Validate stride
        let min_stride = width * format.bytes_per_pixel();
        if stride < min_stride {
            return Err(ShmError::InvalidStride);
        }

        let buffer = ShmBuffer::new(pool_id, offset, width, height, stride, format);
        let id = buffer.id;
        self.buffers.insert(id, buffer);

        debug!(
            "Created shm buffer {:?}, {}x{}, format {:?}",
            id, width, height, format
        );

        Ok(id)
    }

    /// Destroy a buffer
    pub fn destroy_buffer(&mut self, buffer_id: ShmBufferId) {
        self.buffers.remove(&buffer_id);
        debug!("Destroyed shm buffer {:?}", buffer_id);
    }

    /// Get a buffer by ID
    pub fn get_buffer(&self, id: ShmBufferId) -> Option<&ShmBuffer> {
        self.buffers.get(&id)
    }

    /// Get a pool by ID
    pub fn get_pool(&self, id: ShmPoolId) -> Option<&ShmPool> {
        self.pools.get(&id)
    }
}

impl Default for WlShmHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// SHM errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ShmError {
    #[error("Invalid pool")]
    InvalidPool,
    #[error("Buffer too large for pool")]
    BufferTooLarge,
    #[error("Invalid stride")]
    InvalidStride,
    #[error("Invalid format")]
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_format() {
        assert_eq!(ShmFormat::from_wayland(0), ShmFormat::Argb8888);
        assert_eq!(ShmFormat::Argb8888.to_wayland(), 0);
        assert_eq!(ShmFormat::Argb8888.bytes_per_pixel(), 4);
    }

    #[test]
    fn test_shm_buffer() {
        let buffer = ShmBuffer::new(ShmPoolId(1), 0, 100, 100, 400, ShmFormat::Argb8888);
        assert_eq!(buffer.data_size(), 40000);
    }

    #[test]
    fn test_shm_handler() {
        let mut handler = WlShmHandler::new();
        let formats = handler.formats();
        assert!(formats.contains(&ShmFormat::Argb8888));

        // Create pool with a fake fd (-1 for testing)
        let pool_id = handler.create_pool(-1, 40000);
        assert!(handler.get_pool(pool_id).is_some());

        // Create buffer
        let buffer_id = handler.create_buffer(pool_id, 0, 100, 100, 400, 0).unwrap();
        assert!(handler.get_buffer(buffer_id).is_some());
    }
}
