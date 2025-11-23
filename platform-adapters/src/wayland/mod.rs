mod gtk_clipboard;
mod portal;
mod wlr_data_control;

use crate::{ClipData, ClipboardAdapter};
use crossbeam_channel::Sender;

pub use gtk_clipboard::GtkBackend;
pub use portal::PortalBackend;
pub use wlr_data_control::WlrBackend;

pub struct WaylandAdapter {
    backend: Backend,
}

enum Backend {
    Wlr(WlrBackend),
    Portal(PortalBackend),
    Gtk(GtkBackend),
    Stub,
}

impl WaylandAdapter {
    pub fn new() -> Self {
        let backend = if wlr_data_control::is_supported() {
            Backend::Wlr(WlrBackend::new())
        } else if portal::is_supported() {
            Backend::Portal(PortalBackend::new())
        } else if gtk_clipboard::is_supported() {
            Backend::Gtk(GtkBackend::new())
        } else {
            eprintln!("No Wayland clipboard protocols available.");
            Backend::Stub
        };

        Self { backend }
    }
}

impl ClipboardAdapter for WaylandAdapter {
    fn start(&self, tx: Sender<ClipData>) {
        match &self.backend {
            Backend::Wlr(backend) => backend.start(tx),
            Backend::Portal(backend) => backend.start(tx),
            Backend::Gtk(backend) => backend.start(tx),
            Backend::Stub => {
                eprintln!("No clipboard backend available for Wayland (stub).");
            }
        }
    }
}
