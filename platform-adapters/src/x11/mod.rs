
use crossbeam_channel::Sender;
use crate::{ClipData, ClipboardAdapter};

#[derive(Debug)]
pub struct X11Adapter;

impl X11Adapter {
    pub fn new() -> Self {
        X11Adapter
    }
}

impl ClipboardAdapter for X11Adapter {
    fn start(&self, _tx: Sender<ClipData>) {
        eprintln!("X11 clipboard adapter not implemented yet.");
    }
}
