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
        log::info!("Core daemon start");

        // Start networking
        task::spawn(Net::start());
        log::info!("Network task spawned");

        // Start clipboard listener (stub: X11)
        let (tx, rx) = crossbeam_channel::unbounded();
        log::info!("Created clipboard channel");
        let adapter: Box<dyn ClipboardAdapter> = platform_adapters::create_adapter();
        log::info!("Created clipboard adapter");
        adapter.start(tx);
        log::info!("Adapter started, entering receive loop");

        loop {
            if let Ok(item) = rx.recv() {
                log::info!("✅ RECEIVED clipboard data: {:?}", item);

                let bytes = match item {
                    ClipData::Text(s) => {
                        log::debug!("Processing text clipboard, length: {}", s.len());
                        s.into_bytes()
                    },
                    _ => {
                        log::debug!("Processing non-text clipboard data");
                        vec![]
                    }
                };

                log::debug!("Encrypting clipboard data");
                let encrypted = self.crypto.encrypt(&bytes);
                log::debug!("Encrypted data length: {}", encrypted.len());
                log::debug!("Broadcasting encrypted data");
                self.net.broadcast(encrypted).await;
                log::debug!("Broadcast complete");
            } else {
                log::warn!("Channel receive error or closed");
            }
        }
    }
}
