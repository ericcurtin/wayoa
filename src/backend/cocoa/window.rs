//! NSWindow wrapper for Wayland toplevels

use log::debug;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, AllocAnyThread, DeclaredClass, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    CGPoint, CGRect, CGSize, MainThreadMarker, NSNotification, NSObject, NSObjectProtocol,
    NSString,
};

use crate::compositor::WindowId;

/// Native window handle
#[derive(Debug)]
pub struct NativeWindowHandle {
    /// The underlying NSWindow
    window: Retained<NSWindow>,
    /// Window ID
    window_id: WindowId,
}

impl NativeWindowHandle {
    /// Get the window ID
    pub fn id(&self) -> WindowId {
        self.window_id
    }

    /// Get the NSWindow reference
    pub fn ns_window(&self) -> &NSWindow {
        &self.window
    }
}

/// Wayoa native window
pub struct WayoaWindow {
    /// Main thread marker
    mtm: MainThreadMarker,
    /// The underlying NSWindow
    window: Retained<NSWindow>,
    /// Window ID
    window_id: WindowId,
}

impl WayoaWindow {
    /// Create a new native window
    pub fn new(
        mtm: MainThreadMarker,
        window_id: WindowId,
        width: u32,
        height: u32,
        title: &str,
    ) -> anyhow::Result<Self> {
        let frame = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(width as f64, height as f64));

        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Miniaturizable
            | NSWindowStyleMask::Resizable;

        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc(),
                frame,
                style,
                NSBackingStoreType::NSBackingStoreBuffered,
                false,
            )
        };

        // Set title
        let ns_title = NSString::from_str(title);
        window.setTitle(&ns_title);

        // Center on screen
        window.center();

        // Create and set delegate
        let delegate = WayoaWindowDelegate::new(mtm, window_id);
        let delegate_obj: &ProtocolObject<dyn NSWindowDelegate> =
            ProtocolObject::from_ref(delegate.as_ref());
        window.setDelegate(Some(delegate_obj));

        debug!(
            "Created native window {:?}, {}x{}, title: {}",
            window_id, width, height, title
        );

        Ok(Self {
            mtm,
            window,
            window_id,
        })
    }

    /// Show the window
    pub fn show(&self) {
        unsafe {
            self.window.makeKeyAndOrderFront(None);
        }
    }

    /// Hide the window
    pub fn hide(&self) {
        self.window.orderOut(None);
    }

    /// Close the window
    pub fn close(&self) {
        self.window.close();
    }

    /// Set the window title
    pub fn set_title(&self, title: &str) {
        let ns_title = NSString::from_str(title);
        self.window.setTitle(&ns_title);
    }

    /// Set the window size
    pub fn set_size(&self, width: u32, height: u32) {
        let size = CGSize::new(width as f64, height as f64);
        unsafe {
            self.window.setContentSize(size);
        }
    }

    /// Set the window position
    pub fn set_position(&self, x: i32, y: i32) {
        let point = CGPoint::new(x as f64, y as f64);
        self.window.setFrameTopLeftPoint(point);
    }

    /// Get the window size
    pub fn size(&self) -> (u32, u32) {
        let frame = self.window.frame();
        (frame.size.width as u32, frame.size.height as u32)
    }

    /// Get the content size (excluding title bar)
    pub fn content_size(&self) -> (u32, u32) {
        let content_rect = self.window.contentRectForFrameRect(self.window.frame());
        (content_rect.size.width as u32, content_rect.size.height as u32)
    }

    /// Get the window ID
    pub fn id(&self) -> WindowId {
        self.window_id
    }

    /// Get a native handle
    pub fn native_handle(&self) -> NativeWindowHandle {
        NativeWindowHandle {
            window: self.window.clone(),
            window_id: self.window_id,
        }
    }

    /// Set fullscreen state
    pub fn set_fullscreen(&self, fullscreen: bool) {
        let is_fullscreen = self.window.styleMask().contains(NSWindowStyleMask::FullScreen);
        if fullscreen != is_fullscreen {
            unsafe {
                self.window.toggleFullScreen(None);
            }
        }
    }

    /// Set maximized state
    pub fn set_maximized(&self, maximized: bool) {
        let is_zoomed = self.window.isZoomed();
        if maximized != is_zoomed {
            unsafe {
                self.window.zoom(None);
            }
        }
    }

    /// Minimize the window
    pub fn minimize(&self) {
        unsafe {
            self.window.miniaturize(None);
        }
    }

    /// Restore from minimized
    pub fn restore(&self) {
        unsafe {
            self.window.deminiaturize(None);
        }
    }

    /// Check if window is key (focused)
    pub fn is_key(&self) -> bool {
        self.window.isKeyWindow()
    }

    /// Make window key (focused)
    pub fn make_key(&self) {
        self.window.makeKeyWindow();
    }
}

/// Window delegate ivars
struct WayoaWindowDelegateIvars {
    window_id: WindowId,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "WayoaWindowDelegate"]
    #[ivars = WayoaWindowDelegateIvars]
    struct WayoaWindowDelegate;

    unsafe impl NSObjectProtocol for WayoaWindowDelegate {}

    unsafe impl NSWindowDelegate for WayoaWindowDelegate {
        #[unsafe(method(windowDidBecomeKey:))]
        fn window_did_become_key(&self, _notification: &NSNotification) {
            debug!("Window {:?} became key", self.ivars().window_id);
            // TODO: Send keyboard enter event to Wayland client
        }

        #[unsafe(method(windowDidResignKey:))]
        fn window_did_resign_key(&self, _notification: &NSNotification) {
            debug!("Window {:?} resigned key", self.ivars().window_id);
            // TODO: Send keyboard leave event to Wayland client
        }

        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            debug!("Window {:?} will close", self.ivars().window_id);
            // TODO: Send close request to Wayland client
        }

        #[unsafe(method(windowDidResize:))]
        fn window_did_resize(&self, _notification: &NSNotification) {
            debug!("Window {:?} did resize", self.ivars().window_id);
            // TODO: Send configure event to Wayland client
        }

        #[unsafe(method(windowDidMove:))]
        fn window_did_move(&self, _notification: &NSNotification) {
            debug!("Window {:?} did move", self.ivars().window_id);
        }

        #[unsafe(method(windowDidMiniaturize:))]
        fn window_did_miniaturize(&self, _notification: &NSNotification) {
            debug!("Window {:?} did miniaturize", self.ivars().window_id);
        }

        #[unsafe(method(windowDidDeminiaturize:))]
        fn window_did_deminiaturize(&self, _notification: &NSNotification) {
            debug!("Window {:?} did deminiaturize", self.ivars().window_id);
        }

        #[unsafe(method(windowDidEnterFullScreen:))]
        fn window_did_enter_full_screen(&self, _notification: &NSNotification) {
            debug!("Window {:?} entered full screen", self.ivars().window_id);
        }

        #[unsafe(method(windowDidExitFullScreen:))]
        fn window_did_exit_full_screen(&self, _notification: &NSNotification) {
            debug!("Window {:?} exited full screen", self.ivars().window_id);
        }
    }
);

impl DeclaredClass for WayoaWindowDelegate {
    type Ivars = WayoaWindowDelegateIvars;
}

impl WayoaWindowDelegate {
    fn new(mtm: MainThreadMarker, window_id: WindowId) -> Retained<Self> {
        let this = mtm.alloc();
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        this.ivars().window_id.set(window_id.0);
        this
    }
}

// WindowId needs interior mutability for initialization in the delegate
impl WayoaWindowDelegateIvars {
    fn new(window_id: WindowId) -> Self {
        Self { window_id }
    }
}

// Since we can't use Cell in ivars easily, we'll use a workaround
impl std::ops::Deref for WayoaWindowDelegateIvars {
    type Target = WindowId;
    fn deref(&self) -> &Self::Target {
        &self.window_id
    }
}

#[cfg(test)]
mod tests {
    // Note: Window tests require a display environment
}
