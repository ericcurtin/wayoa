//! NSApplication delegate and event loop integration

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use log::{debug, error, info};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSMenu, NSMenuItem,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};

use crate::server::{ServerState, WaylandServer};

/// Wayoa application wrapper
pub struct WayoaApp {
    /// Main thread marker
    mtm: MainThreadMarker,
    /// NSApplication instance
    app: Retained<NSApplication>,
    /// Wayland server
    server: RefCell<WaylandServer>,
    /// Server state
    state: Rc<RefCell<ServerState>>,
}

impl WayoaApp {
    /// Create a new Wayoa application
    pub fn new() -> anyhow::Result<Self> {
        info!("Initializing Wayoa application");

        // Ensure we're on the main thread
        let mtm = MainThreadMarker::new()
            .ok_or_else(|| anyhow::anyhow!("Must be called from the main thread"))?;

        // Get the shared NSApplication
        let app = NSApplication::sharedApplication(mtm);

        // Set activation policy to regular (shows in dock)
        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

        // Create and set the app delegate
        let delegate = WayoaAppDelegate::new(mtm);
        let delegate_obj: &ProtocolObject<dyn NSApplicationDelegate> =
            ProtocolObject::from_ref(&*delegate);
        app.setDelegate(Some(delegate_obj));

        // Set up the menu bar
        Self::setup_menu_bar(mtm, &app);

        // Create Wayland server
        let mut server = WaylandServer::new()?;

        // Set WAYLAND_DISPLAY environment variable
        let socket_name = server.socket_name().to_string();
        std::env::set_var("WAYLAND_DISPLAY", &socket_name);
        info!("WAYLAND_DISPLAY={}", socket_name);

        // Register protocol globals
        server.register_globals();

        // Create server state
        let mut state = ServerState::new();
        state.set_main_thread_marker(mtm);

        // Create a default output
        let _output_id = state.compositor.outputs.create_output(
            "default".to_string(),
            "Wayoa".to_string(),
            "Virtual Display".to_string(),
        );

        debug!("Wayoa application initialized");

        Ok(Self {
            mtm,
            app,
            server: RefCell::new(server),
            state: Rc::new(RefCell::new(state)),
        })
    }

    /// Set up the application menu bar
    fn setup_menu_bar(mtm: MainThreadMarker, app: &NSApplication) {
        unsafe {
            // Create main menu
            let main_menu = NSMenu::new(mtm);

            // Application menu
            let app_menu_item = NSMenuItem::new(mtm);
            let app_menu = NSMenu::new(mtm);

            // Quit menu item
            let quit_title = NSString::from_str("Quit Wayoa");
            let quit_key = NSString::from_str("q");
            let quit_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &quit_title,
                Some(objc2::sel!(terminate:)),
                &quit_key,
            );
            app_menu.addItem(&quit_item);

            app_menu_item.setSubmenu(Some(&app_menu));
            main_menu.addItem(&app_menu_item);

            app.setMainMenu(Some(&main_menu));
        }
    }

    /// Run the application event loop
    pub fn run(&self) {
        info!("Starting Wayoa event loop");
        info!(
            "Wayland clients can connect to: {}",
            self.server.borrow().socket_name()
        );

        // Activate the application
        #[allow(deprecated)]
        self.app.activateIgnoringOtherApps(true);

        // We'll use a manual run loop to integrate Wayland dispatch
        // This is more portable than NSTimer for this use case
        loop {
            // Process pending NSApplication events with a small timeout
            let event = self.app.nextEventMatchingMask_untilDate_inMode_dequeue(
                objc2_app_kit::NSEventMask::Any,
                None, // Don't wait for events
                objc2_foundation::ns_string!("kCFRunLoopDefaultMode"),
                true,
            );

            if let Some(event) = event {
                self.app.sendEvent(&event);
            }

            // Dispatch Wayland events
            if let Err(e) = self.dispatch_wayland() {
                error!("Wayland dispatch error: {}", e);
            }

            // Small sleep to avoid busy-waiting when idle
            std::thread::sleep(Duration::from_millis(1));

            // Check if we should stop
            if !self.app.isRunning() {
                break;
            }
        }
    }

    /// Dispatch pending Wayland events
    fn dispatch_wayland(&self) -> anyhow::Result<()> {
        let mut server = self.server.borrow_mut();
        let mut state = self.state.borrow_mut();
        server.dispatch(&mut state)
    }

    /// Stop the application
    pub fn stop(&self) {
        self.app.stop(None);
    }

    /// Get the main thread marker
    pub fn main_thread_marker(&self) -> MainThreadMarker {
        self.mtm
    }
}

/// Application delegate ivars
struct WayoaAppDelegateIvars {
    // Add any instance variables here
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "WayoaAppDelegate"]
    #[ivars = WayoaAppDelegateIvars]
    struct WayoaAppDelegate;

    unsafe impl NSObjectProtocol for WayoaAppDelegate {}

    unsafe impl NSApplicationDelegate for WayoaAppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn application_did_finish_launching(&self, _notification: &NSNotification) {
            info!("Application did finish launching");
        }

        #[unsafe(method(applicationWillTerminate:))]
        fn application_will_terminate(&self, _notification: &NSNotification) {
            info!("Application will terminate");
        }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        fn application_should_terminate_after_last_window_closed(
            &self,
            _app: &NSApplication,
        ) -> bool {
            // Don't quit when all windows are closed (compositor stays running)
            false
        }
    }
);

impl WayoaAppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(WayoaAppDelegateIvars {});
        let this: Option<Retained<Self>> = unsafe { msg_send![super(this), init] };
        this.expect("init failed")
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests require running on the main thread with a display
    // They are disabled by default as they require a GUI environment
}
