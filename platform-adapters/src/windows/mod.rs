use crate::{ClipData, ClipboardAdapter};
use crossbeam_channel::Sender;

#[derive(Debug)]
pub struct WindowsAdapter;

impl WindowsAdapter {
    pub fn new() -> Self {
        WindowsAdapter
    }
}

impl ClipboardAdapter for WindowsAdapter {
    fn start(&self, _tx: Sender<ClipData>) {
        eprintln!("❌ Windows clipboard adapter not yet implemented.");
        eprintln!("💡 This is needed for Windows desktop support.");
        eprintln!("📝 TODO: Implement using clipboard-win or Windows API directly.");
    }
}
