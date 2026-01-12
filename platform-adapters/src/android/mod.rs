use crate::{ClipData, ClipboardAdapter};
use crossbeam_channel::Sender;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Global sender that will be set when the daemon starts
pub static CLIP_SENDER: Lazy<Mutex<Option<Sender<ClipData>>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug)]
pub struct AndroidAdapter;

impl AndroidAdapter {
    pub fn new() -> Self {
        AndroidAdapter
    }
}

impl ClipboardAdapter for AndroidAdapter {
    fn start(&self, tx: Sender<ClipData>) {
        log::info!("🤖 Android clipboard adapter initialized");
        log::info!("📋 Waiting for clipboard events from Kotlin...");

        // Store the sender globally so JNI can access it
        let mut guard = CLIP_SENDER.lock().expect("CLIP_SENDER mutex poisoned");
        *guard = Some(tx);
        log::info!("CLIP_SENDER has been set");

        log::info!("✅ Android adapter ready to receive clipboard data");
    }
}
