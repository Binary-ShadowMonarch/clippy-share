use crossbeam_channel::Sender;
use std::fmt::{Display, Formatter};

pub mod android;

// Only compile wayland module on Linux (not Android)
#[cfg(all(unix, not(target_os = "android")))]
pub mod wayland;

// Only compile x11 module on Linux (not Android)
#[cfg(all(unix, not(target_os = "android")))]
pub mod x11;

#[cfg(windows)]
pub mod windows;

#[derive(Debug, Clone)]
pub enum ClipData {
    Text(String),
    Raw { mime_type: String, bytes: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct ClipboardError {
    message: String,
}

impl ClipboardError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ClipboardError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ClipboardError {}

pub trait ClipboardAdapter: Send + Sync {
    fn start(&self, tx: Sender<ClipData>);
    fn set_text(&self, text: &str) -> Result<(), ClipboardError>;
}

pub fn create_adapter() -> Box<dyn ClipboardAdapter> {
    #[cfg(target_os = "windows")]
    {
        return Box::new(windows::WindowsAdapter::new());
    }

    #[cfg(target_os = "android")]
    {
        return Box::new(android::AndroidAdapter::new());
    }

    #[cfg(all(target_os = "linux", not(target_os = "android")))]
    {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            println!("Detected Wayland session.");
            return Box::new(wayland::WaylandAdapter::new());
        }
        if std::env::var("DISPLAY").is_ok() {
            println!("Detected X11 session.");
            return Box::new(x11::X11Adapter::new());
        }
        eprintln!("No display server detected. Clipboard unavailable.");
        // Fallback to a stub adapter
        return Box::new(android::AndroidAdapter::new());
    }

    // Fallback for other platforms
    #[cfg(not(any(windows, target_os = "android", target_os = "linux")))]
    {
        eprintln!("Unsupported platform for clipboard monitoring");
        return Box::new(android::AndroidAdapter::new());
    }
}
