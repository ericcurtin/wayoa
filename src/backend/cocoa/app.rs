//! NSApplication delegate and event loop integration

use log::{debug, info};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSMenu, NSMenuItem,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};

use crate::compositor::CompositorState;

/// Wayoa application wrapper
pub struct WayoaApp {
    /// Main thread marker
    mtm: MainThreadMarker,
    /// NSApplication instance
    app: Retained<NSApplication>,
    /// Compositor state
    _state: CompositorState,
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

        let state = CompositorState::new();

        debug!("Wayoa application initialized");

        Ok(Self {
            mtm,
            app,
            _state: state,
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
                None,
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

        // Activate the application
        #[allow(deprecated)]
        self.app.activateIgnoringOtherApps(true);

        // Run the event loop
        self.app.run();
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
