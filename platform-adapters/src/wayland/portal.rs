use crate::{ClipData, ClipboardAdapter, ClipboardError};
use crossbeam_channel::Sender;

/// TODO: Implement XDG Desktop Portal clipboard listener for GNOME support
///
/// This adapter should use the XDG Desktop Portal API to listen for clipboard
/// changes on GNOME Wayland sessions, which don't support wlr-data-control.
///
/// Recommended implementation approach:
/// 1. Use the `ashpd` crate for XDG Portal communication
/// 2. Request clipboard access via org.freedesktop.portal.Clipboard
/// 3. Listen for SelectionOwnerChanged signals
/// 4. Read clipboard contents when changes are detected
///
/// References:
/// - ashpd: https://github.com/bilelmoussaoui/ashpd
/// - XDG Desktop Portal spec: https://flatpak.github.io/xdg-desktop-portal/
/// - Clipboard portal: https://flatpak.github.io/xdg-desktop-portal/#gdbus-org.freedesktop.portal.Clipboard
///
/// Note: Portal-based clipboard access may require user permission grants.

pub struct PortalAdapter;

impl PortalAdapter {
    pub fn new() -> Self {
        PortalAdapter
    }
}

impl ClipboardAdapter for PortalAdapter {
    fn start(&self, _tx: Sender<ClipData>) {
        eprintln!("❌ XDG Desktop Portal clipboard adapter not yet implemented.");
        eprintln!("💡 This is needed for GNOME Wayland support.");
        eprintln!("📝 TODO: Implement using ashpd crate or similar Portal API library.");
    }

    fn set_text(&self, _text: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::new(
            "Portal clipboard apply is not implemented yet",
        ))
    }
}
