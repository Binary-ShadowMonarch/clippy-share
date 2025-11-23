use crate::{ClipData, ClipboardAdapter};
use crossbeam_channel::Sender;

pub struct WindowsAdapter;

impl WindowsAdapter {
    pub fn new() -> Self {
        WindowsAdapter
    }
}

impl ClipboardAdapter for WindowsAdapter {
    fn start(&self, _tx: Sender<ClipData>) {
        // Stub: real implementation should use clipboard-win crate or Win32 API.
        println!("[windows] start() stub - not implemented yet");
    }
}
