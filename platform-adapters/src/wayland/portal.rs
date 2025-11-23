use crossbeam_channel::Sender;

/// xdg-desktop-portal backend stub
pub struct PortalBackend;

pub fn is_supported() -> bool {
    // Real detection should check DBus for org.freedesktop.portal.Desktop.
    false
}

impl PortalBackend {
    pub fn new() -> Self {
        PortalBackend
    }

    pub fn start(&self, _tx: Sender<crate::ClipData>) {
        // Stub: real implementation will subscribe to portal clipboard events via DBus.
        println!("[portal] start() stub - not implemented yet");
    }
}
