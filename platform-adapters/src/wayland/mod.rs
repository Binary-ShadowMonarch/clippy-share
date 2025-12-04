use crate::{ClipData, ClipboardAdapter};
use crossbeam_channel::Sender;

pub mod portal;
pub mod wlroots;

pub struct WaylandAdapter {
    inner: Box<dyn ClipboardAdapter>,
}

impl WaylandAdapter {
    pub fn new() -> Self {
        // Detect which Wayland compositor/protocol to use
        let inner: Box<dyn ClipboardAdapter> = if Self::is_gnome_wayland() {
            println!("🔍 Detected GNOME Wayland session");
            println!("⚠️  Portal adapter not yet implemented, falling back to wlroots");
            // TODO: Once portal is implemented, use:
            // Box::new(portal::PortalAdapter::new())
            Box::new(wlroots::WlrootsAdapter::new())
        } else {
            println!("🔍 Detected wlroots-compatible Wayland session");
            Box::new(wlroots::WlrootsAdapter::new())
        };

        WaylandAdapter { inner }
    }

    /// Detect if running under GNOME Wayland
    /// GNOME doesn't support wlr-data-control protocol
    fn is_gnome_wayland() -> bool {
        // Check for GNOME session indicators
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_lowercase().contains("gnome") {
                return true;
            }
        }

        if let Ok(session) = std::env::var("GDMSESSION") {
            if session.to_lowercase().contains("gnome") {
                return true;
            }
        }

        if let Ok(session) = std::env::var("DESKTOP_SESSION") {
            if session.to_lowercase().contains("gnome") {
                return true;
            }
        }

        false
    }
}

impl ClipboardAdapter for WaylandAdapter {
    fn start(&self, tx: Sender<ClipData>) {
        self.inner.start(tx);
    }
}
