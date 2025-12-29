//! NSView with Metal layer for rendering

use log::debug;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, msg_send_id, AllocAnyThread, DeclaredClass, MainThreadOnly};
use objc2_app_kit::NSView;
use objc2_foundation::{CGRect, CGSize, MainThreadMarker, NSObject, NSObjectProtocol};
use objc2_quartz_core::CAMetalLayer;

use crate::compositor::SurfaceId;

/// A view with a Metal layer for rendering Wayland surface content
pub struct MetalView {
    /// The underlying NSView
    view: Retained<WayoaView>,
    /// Associated surface ID
    surface_id: SurfaceId,
}

impl MetalView {
    /// Create a new Metal view
    pub fn new(mtm: MainThreadMarker, surface_id: SurfaceId, frame: CGRect) -> anyhow::Result<Self> {
        let view = WayoaView::new(mtm, surface_id, frame)?;

        debug!(
            "Created Metal view for surface {:?}, size {}x{}",
            surface_id, frame.size.width, frame.size.height
        );

        Ok(Self { view, surface_id })
    }

    /// Get the underlying NSView
    pub fn ns_view(&self) -> &NSView {
        // WayoaView is a subclass of NSView
        unsafe { &*(self.view.as_ref() as *const WayoaView as *const NSView) }
    }

    /// Get the surface ID
    pub fn surface_id(&self) -> SurfaceId {
        self.surface_id
    }

    /// Get the Metal layer
    pub fn metal_layer(&self) -> Option<Retained<CAMetalLayer>> {
        self.view.metal_layer()
    }

    /// Set the view frame
    pub fn set_frame(&self, frame: CGRect) {
        unsafe {
            self.view.setFrame(frame);
        }
    }

    /// Get the view frame
    pub fn frame(&self) -> CGRect {
        self.view.frame()
    }

    /// Set the drawable size for the Metal layer
    pub fn set_drawable_size(&self, width: u32, height: u32) {
        if let Some(layer) = self.metal_layer() {
            let size = CGSize::new(width as f64, height as f64);
            layer.setDrawableSize(size);
        }
    }

    /// Request a redraw
    pub fn set_needs_display(&self) {
        unsafe {
            let _: () = msg_send![&*self.view, setNeedsDisplay: true];
        }
    }
}

/// View ivars
struct WayoaViewIvars {
    surface_id: SurfaceId,
    metal_layer: Option<Retained<CAMetalLayer>>,
}

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "WayoaView"]
    #[ivars = WayoaViewIvars]
    struct WayoaView;

    unsafe impl NSObjectProtocol for WayoaView {}

    // Make this view layer-backed and return our Metal layer
    unsafe impl WayoaView {
        #[unsafe(method(wantsLayer))]
        fn wants_layer(&self) -> bool {
            true
        }

        #[unsafe(method(wantsUpdateLayer))]
        fn wants_update_layer(&self) -> bool {
            true
        }

        #[unsafe(method(isOpaque))]
        fn is_opaque(&self) -> bool {
            true
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        #[unsafe(method(updateLayer))]
        fn update_layer(&self) {
            // Called when the view needs to update its layer content
            // The actual rendering is handled by the Metal renderer
            debug!("Update layer for surface {:?}", self.ivars().surface_id);
        }
    }
);

impl DeclaredClass for WayoaView {
    type Ivars = WayoaViewIvars;
}

impl WayoaView {
    fn new(mtm: MainThreadMarker, surface_id: SurfaceId, frame: CGRect) -> anyhow::Result<Retained<Self>> {
        let this = mtm.alloc();

        // Create Metal layer
        let metal_layer = unsafe { CAMetalLayer::new() };
        metal_layer.setContentsScale(2.0); // For Retina displays
        metal_layer.setDrawableSize(CGSize::new(
            frame.size.width * 2.0,
            frame.size.height * 2.0,
        ));

        // Initialize the view
        let this: Retained<Self> = unsafe {
            let this: Retained<Self> = msg_send_id![super(this), initWithFrame: frame];
            this
        };

        // Set up ivars
        *this.ivars().surface_id.get_mut() = surface_id;
        *this.ivars().metal_layer.get_mut() = Some(metal_layer.clone());

        // Set the layer
        unsafe {
            let _: () = msg_send![&*this, setLayer: &*metal_layer];
            let _: () = msg_send![&*this, setWantsLayer: true];
        }

        Ok(this)
    }

    fn metal_layer(&self) -> Option<Retained<CAMetalLayer>> {
        self.ivars().metal_layer.clone()
    }
}

// Interior mutability helpers for ivars
impl WayoaViewIvars {
    fn get_mut(&mut self) -> &mut Self {
        self
    }
}

impl std::ops::Deref for WayoaViewIvars {
    type Target = SurfaceId;
    fn deref(&self) -> &Self::Target {
        &self.surface_id
    }
}

#[cfg(test)]
mod tests {
    // Note: View tests require a display environment
}
