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
        println!("Core daemon start");

        // Start networking
        task::spawn(Net::start());

        // Start clipboard listener (stub: X11)
        let (tx, rx) = crossbeam_channel::unbounded();
        let adapter: Box<dyn ClipboardAdapter> = platform_adapters::create_adapter();
        adapter.start(tx);

        loop {
            if let Ok(item) = rx.recv() {
                println!("Core got clipboard: {:?}", item);

                let bytes = match item {
                    ClipData::Text(s) => s.into_bytes(),
                    _ => vec![]
                };

                let encrypted = self.crypto.encrypt(&bytes);
                self.net.broadcast(encrypted).await;
            }
        }
    }
}
