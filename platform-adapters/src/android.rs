use crate::{ClipData, ClipboardAdapter};
use crossbeam_channel::Sender;

pub struct AndroidAdapter;

impl AndroidAdapter {
    pub fn new() -> Self {
        AndroidAdapter
    }
}

impl ClipboardAdapter for AndroidAdapter {
    fn start(&self, _tx: Sender<ClipData>) {
        // Stub: real implementation would bridge to Android/Flutter clipboard callbacks.
        println!("[android] start() stub - not implemented yet");
    }
}
