use crossbeam_channel::Sender;

/// GTK clipboard fallback stub
pub struct GtkBackend;

pub fn is_supported() -> bool {
    // Optionally detect GTK availability; keep false for now.
    false
}

impl GtkBackend {
    pub fn new() -> Self {
        GtkBackend
    }

    pub fn start(&self, _tx: Sender<crate::ClipData>) {
        // Stub: real implementation might poll GTK clipboard or use gtk APIs.
        println!("[gtk-clipboard] start() stub - not implemented yet");
    }
}
