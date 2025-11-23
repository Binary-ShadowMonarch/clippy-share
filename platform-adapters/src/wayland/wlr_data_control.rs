use crossbeam_channel::Sender;

/// WLRoots (zwlr_data_control_manager_v1) backend stub
pub struct WlrBackend;

pub fn is_supported() -> bool {
    // Real detection should query the Wayland registry for the
    // zwlr_data_control_manager_v1 global. For now return false.
    false
}

impl WlrBackend {
    pub fn new() -> Self {
        WlrBackend
    }

    pub fn start(&self, _tx: Sender<crate::ClipData>) {
        // Stub: real implementation will listen to wlroots data-control offers.
        println!("[wlr-data-control] start() stub - not implemented yet");
    }
}
