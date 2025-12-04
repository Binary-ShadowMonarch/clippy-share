
use crossbeam_channel::Sender;
use crate::{ClipData, ClipboardAdapter};

#[derive(Debug)]
pub struct AndroidAdapter;

impl AndroidAdapter {
    pub fn new() -> Self {
        AndroidAdapter
    }
}

impl ClipboardAdapter for AndroidAdapter {
    fn start(&self, _tx: Sender<ClipData>) {
        eprintln!("Android clipboard adapter not implemented yet.");
    }
}
