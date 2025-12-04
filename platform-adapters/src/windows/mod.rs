use crossbeam_channel::Sender;
use crate::{ClipData, ClipboardAdapter};

#[derive(Debug)]
pub struct WindowsAdapter;

impl WindowsAdapter {
    pub fn new() -> Self {
        WindowsAdapter
    }
}

impl ClipboardAdapter for WindowsAdapter {
    fn start(&self, _tx: Sender<ClipData>) {
        eprintln!("Windows clipboard adapter not implemented yet.");
    }
}

