use platform_adapters::{ClipboardAdapter, ClipData};
use crypto_engine::CryptoEngine;
use net::Net;
use tokio::task;

pub struct CoreDaemon {
    crypto: CryptoEngine,
    net: Net,
}

impl CoreDaemon {
    pub fn new() -> Self {
        Self {
            crypto: CryptoEngine::new(),
            net: Net,
        }
    }
pub async fn run(&self) {
        eprintln!("[CoreDaemon] Core daemon start");

        // Start networking
        task::spawn(Net::start());
        eprintln!("[CoreDaemon] Network task spawned");

        // Start clipboard listener (stub: X11)
        let (tx, rx) = crossbeam_channel::unbounded();
        eprintln!("[CoreDaemon] Created clipboard channel");
        let adapter: Box<dyn ClipboardAdapter> = platform_adapters::create_adapter();
        eprintln!("[CoreDaemon] Created clipboard adapter");
        adapter.start(tx);
        eprintln!("[CoreDaemon] Adapter started, entering receive loop");

        loop {
            if let Ok(item) = rx.recv() {
                eprintln!("[CoreDaemon] ✅ RECEIVED clipboard data: {:?}", item);

                let bytes = match item {
                    ClipData::Text(s) => {
                        eprintln!("[CoreDaemon] Processing text clipboard, length: {}", s.len());
                        s.into_bytes()
                    },
                    _ => {
                        eprintln!("[CoreDaemon] Processing non-text clipboard data");
                        vec![]
                    }
                };

                eprintln!("[CoreDaemon] Encrypting clipboard data");
                let encrypted = self.crypto.encrypt(&bytes);
                eprintln!("[CoreDaemon] Encrypted data length: {}", encrypted.len());
                eprintln!("[CoreDaemon] Broadcasting encrypted data");
                self.net.broadcast(encrypted).await;
                eprintln!("[CoreDaemon] Broadcast complete");
            } else {
                eprintln!("[CoreDaemon] Channel receive error or closed");
            }
        }
    }
}
