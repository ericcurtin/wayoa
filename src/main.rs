//! Wayoa - A Wayland compositor for macOS
//!
//! This is the entry point that sets up the NSApplication event loop
//! and integrates the Wayland server.

use log::error;

#[cfg(target_os = "macos")]
mod macos_main {
    use log::info;
    use wayoa::backend::cocoa::app::WayoaApp;

    pub fn run() -> anyhow::Result<()> {
        info!("Starting Wayoa compositor");

        let app = WayoaApp::new()?;
        app.run();

        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
mod stub_main {
    use super::*;

    pub fn run() -> anyhow::Result<()> {
        error!("Wayoa only runs on macOS");
        anyhow::bail!("Wayoa requires macOS to run")
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    #[cfg(target_os = "macos")]
    {
        macos_main::run()
    }

    #[cfg(not(target_os = "macos"))]
    {
        stub_main::run()
    }
}
