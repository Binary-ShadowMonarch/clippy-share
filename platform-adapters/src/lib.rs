use crossbeam_channel::Sender;

pub mod android;
pub mod wayland;
pub mod windows;
pub mod x11;

#[derive(Debug, Clone)]
pub enum ClipData {
    Text(String),
    Raw { mime_type: String, bytes: Vec<u8> },
}

pub trait ClipboardAdapter: Send + Sync {
    fn start(&self, tx: Sender<ClipData>);
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

    #[cfg(target_os = "linux")]
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
        return Box::new(android::AndroidAdapter::new()); // harmless stub fallback
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "android")))]
    {
        compile_error!("Unsupported target for platform-adapters::create_adapter");
    }
}
