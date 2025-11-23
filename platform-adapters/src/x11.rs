use crate::{ClipData, ClipboardAdapter};
use crossbeam_channel::Sender;

pub struct X11Adapter;

impl X11Adapter {
    pub fn new() -> Self {
        X11Adapter
    }
}

impl ClipboardAdapter for X11Adapter {
    fn start(&self, _tx: Sender<ClipData>) {
        // Stub: replace with x11rb/x11-clipboard implementation later.
        println!("[x11] start() stub - not implemented yet");
    }
}
