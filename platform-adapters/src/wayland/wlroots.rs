use crate::{ClipData, ClipboardAdapter, ClipboardError};
use crossbeam_channel::Sender;
use std::thread;
use wayland_clipboard_listener::{WlClipboardPasteStream, WlListenType};

pub struct WlrootsAdapter;

impl WlrootsAdapter {
    pub fn new() -> Self {
        WlrootsAdapter
    }
}

impl ClipboardAdapter for WlrootsAdapter {
    fn start(&self, tx: Sender<ClipData>) {
        println!("🚀 Starting Wayland (wlroots) clipboard listener...");

        // Spawn a blocking thread since the clipboard listener uses blocking I/O
        thread::spawn(move || {
            match WlClipboardPasteStream::init(WlListenType::ListenOnCopy) {
                Ok(mut stream) => {
                    println!("✅ Wayland clipboard listener initialized successfully");
                    println!("📋 Listening for clipboard copy events...\n");

                    // Iterate over clipboard paste events
                    for result in stream.paste_stream() {
                        match result {
                            Ok(message) => {
                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                println!("📝 Clipboard Event Detected!");
                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                                // Log the full message for debugging
                                println!("🔍 Message: {:#?}", message);

                                // Extract mime type and data from the nested context
                                let mime_type = message.context.mime_type.clone();
                                let data = message.context.context;

                                println!("\n📌 MIME Type: {}", mime_type);

                                // Handle different mime types
                                let clip_data = if mime_type.starts_with("text/")
                                    || mime_type.contains("utf-8")
                                    || mime_type == "STRING"
                                    || mime_type == "UTF8_STRING"
                                {
                                    // Try to convert to text
                                    match String::from_utf8(data.clone()) {
                                        Ok(text) => {
                                            println!("📄 Content Type: Text");
                                            println!("📏 Length: {} characters", text.len());
                                            println!(
                                                "📝 Content:\n{}\n",
                                                if text.len() > 200 {
                                                    format!("{}... (truncated)", &text[..200])
                                                } else {
                                                    text.clone()
                                                }
                                            );
                                            ClipData::Text(text)
                                        }
                                        Err(_) => {
                                            println!(
                                                "⚠️  Failed to decode as UTF-8, treating as raw data"
                                            );
                                            println!("📦 Data size: {} bytes\n", data.len());
                                            ClipData::Raw {
                                                mime_type,
                                                bytes: data,
                                            }
                                        }
                                    }
                                } else {
                                    println!("📦 Content Type: Binary/Raw");
                                    println!("📏 Size: {} bytes\n", data.len());
                                    ClipData::Raw {
                                        mime_type,
                                        bytes: data,
                                    }
                                };

                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                                // Send to the daemon
                                if let Err(e) = tx.send(clip_data) {
                                    eprintln!("❌ Failed to send clipboard data: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Error reading clipboard: {:?}", e);
                                // Continue listening even after errors
                            }
                        }
                    }

                    println!("⚠️  Clipboard listener stream ended");
                }
                Err(e) => {
                    eprintln!(
                        "❌ Failed to initialize Wayland clipboard listener: {:?}",
                        e
                    );
                    eprintln!(
                        "💡 Make sure you're running in a Wayland session with wlr-data-control support"
                    );
                }
            }
        });
    }

    fn set_text(&self, text: &str) -> Result<(), ClipboardError> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| ClipboardError::new(format!("Failed to open clipboard handle: {err}")))?;

        clipboard
            .set_text(text.to_string())
            .map_err(|err| ClipboardError::new(format!("Failed to set Wayland clipboard text: {err}")))
    }
}
