use crate::{ClipData, ClipboardAdapter, ClipboardError};
use crossbeam_channel::Sender;

#[derive(Debug)]
pub struct X11Adapter;

impl X11Adapter {
    pub fn new() -> Self {
        X11Adapter
    }
}

impl ClipboardAdapter for X11Adapter {
    fn start(&self, _tx: Sender<ClipData>) {
        eprintln!("❌ X11 clipboard adapter not yet implemented.");
        eprintln!("💡 This is needed for X11 desktop support.");
        eprintln!("📝 TODO: Implement using x11-clipboard or similar library.");
    }

    fn set_text(&self, _text: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::new(
            "X11 clipboard apply is not implemented yet",
        ))
    }
}
