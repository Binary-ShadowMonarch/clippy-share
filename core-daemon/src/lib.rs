use platform_adapters::{ClipData, ClipboardAdapter};

use net::{Net, NetEvent};
use tokio::sync::mpsc;

mod policy;

use policy::PolicyEngine;

pub struct CoreDaemon {
    policy: PolicyEngine,
}

impl CoreDaemon {
    pub fn new() -> Self {
        Self {
            policy: PolicyEngine::from_env(),
        }
    }

    pub async fn run(&self) {
        log::info!("Core daemon start");

        let node_name = std::env::var("CLIPPY_NODE_NAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "clippy-node".to_string());

        let mut net = match Net::start(node_name).await {
            Ok(net) => net,
            Err(err) => {
                log::error!("Failed to start network layer: {}", err);
                return;
            }
        };

        let (tx, rx) = crossbeam_channel::unbounded();
        let adapter: Box<dyn ClipboardAdapter> = platform_adapters::create_adapter();
        adapter.start(tx);

        let (local_clip_tx, mut local_clip_rx) = mpsc::channel::<ClipData>(64);
        std::thread::spawn(move || {
            while let Ok(item) = rx.recv() {
                if local_clip_tx.blocking_send(item).is_err() {
                    break;
                }
            }
        });

        log::info!("Daemon routing loop started");
        let mut suppress_next_outbound: Option<String> = None;

        loop {
            tokio::select! {
                local_item = local_clip_rx.recv() => {
                    match local_item {
                        Some(ClipData::Text(text)) => {
                            if suppress_next_outbound.as_ref() == Some(&text) {
                                suppress_next_outbound = None;
                                log::debug!("Skipping rebroadcast of remotely applied clipboard text");
                                continue;
                            }

                            let target_peer_ids = self.policy.outbound_targets();
                            if let Err(err) = net.broadcast_text(text.clone(), target_peer_ids).await {
                                log::error!("Failed to broadcast clipboard text: {}", err);
                            }
                        }
                        Some(ClipData::Raw { .. }) => {
                            log::debug!("Ignoring non-text clipboard payload for MVP transport");
                        }
                        None => {
                            log::warn!("Clipboard channel closed");
                            break;
                        }
                    }
                }
                net_event = net.next_event() => {
                    match net_event {
                        Some(NetEvent::Started { local_peer_id }) => {
                            log::info!("Network started with local peer {}", local_peer_id);
                        }
                        Some(NetEvent::Listening { address }) => {
                            log::info!("Listening on {}", address);
                        }
                        Some(NetEvent::PeerDiscovered { peer_id }) => {
                            log::info!("Peer discovered: {}", peer_id);
                        }
                        Some(NetEvent::PeerExpired { peer_id }) => {
                            log::info!("Peer expired: {}", peer_id);
                        }
                        Some(NetEvent::Published { bytes }) => {
                            log::debug!("Published clipboard update ({bytes} bytes)");
                        }
                        Some(NetEvent::ClipboardReceived { from_peer, message }) => {
                            if from_peer == net.local_peer_id() {
                                continue;
                            }

                            if !self.policy.should_apply_message(net.local_peer_id(), &message.target_peer_ids) {
                                log::debug!("Ignoring clipboard message because this peer is not targeted");
                                continue;
                            }

                            match adapter.set_text(&message.text) {
                                Ok(()) => {
                                    suppress_next_outbound = Some(message.text);
                                }
                                Err(err) => {
                                    log::error!("Failed to apply remote clipboard text: {}", err);
                                }
                            }
                        }
                        Some(NetEvent::Error { message }) => {
                            log::warn!("Network warning: {}", message);
                        }
                        None => {
                            log::error!("Network event channel closed");
                            break;
                        }
                    }
                }
            }
        }
    }
}
