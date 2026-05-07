use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity};
use libp2p::mdns;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{PeerId, Swarm};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;

const CLIPBOARD_TOPIC: &str = "clippy-share/clipboard/v1";
const COMMAND_CHANNEL_SIZE: usize = 64;
const EVENT_CHANNEL_SIZE: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardMessage {
    pub text: String,
    pub source_device: String,
    #[serde(default)]
    pub target_peer_ids: Vec<String>,
    pub sent_at_ms: u128,
}

impl ClipboardMessage {
    pub fn new(text: String, source_device: String, target_peer_ids: Vec<String>) -> Self {
        let sent_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        Self {
            text,
            source_device,
            target_peer_ids,
            sent_at_ms,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NetEvent {
    Started { local_peer_id: String },
    Listening { address: String },
    PeerDiscovered { peer_id: String },
    PeerExpired { peer_id: String },
    ClipboardReceived {
        from_peer: String,
        message: ClipboardMessage,
    },
    Published { bytes: usize },
    Error { message: String },
}

#[derive(Debug)]
enum NetCommand {
    Broadcast(ClipboardMessage),
    Shutdown,
}

#[derive(Debug, Error)]
pub enum NetError {
    #[error("network setup failed: {0}")]
    Setup(String),
    #[error("network command queue is closed")]
    CommandChannelClosed,
}

#[derive(NetworkBehaviour)]
struct ClippyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Clone)]
pub struct NetHandle {
    command_tx: mpsc::Sender<NetCommand>,
    local_peer_id: String,
    node_name: String,
}

impl NetHandle {
    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    pub async fn broadcast_text(
        &self,
        text: String,
        target_peer_ids: Vec<String>,
    ) -> Result<(), NetError> {
        self.broadcast(ClipboardMessage::new(
            text,
            self.node_name.clone(),
            target_peer_ids,
        ))
        .await
    }

    pub async fn broadcast(&self, message: ClipboardMessage) -> Result<(), NetError> {
        self.command_tx
            .send(NetCommand::Broadcast(message))
            .await
            .map_err(|_| NetError::CommandChannelClosed)
    }

    pub async fn shutdown(&self) -> Result<(), NetError> {
        self.command_tx
            .send(NetCommand::Shutdown)
            .await
            .map_err(|_| NetError::CommandChannelClosed)
    }
}

pub struct Net {
    handle: NetHandle,
    event_rx: mpsc::Receiver<NetEvent>,
}

impl Net {
    pub async fn start(node_name: impl Into<String>) -> Result<Self, NetError> {
        let node_name = node_name.into();
        let topic = IdentTopic::new(CLIPBOARD_TOPIC);

        let identity = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(identity.public());

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(2))
            .validation_mode(gossipsub::ValidationMode::Permissive)
            .build()
            .map_err(|err| NetError::Setup(format!("invalid gossipsub config: {err}")))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(identity.clone()),
            gossipsub_config,
        )
        .map_err(|err| NetError::Setup(format!("gossipsub init failed: {err}")))?;

        gossipsub
            .subscribe(&topic)
            .map_err(|err| NetError::Setup(format!("topic subscribe failed: {err}")))?;

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
            .map_err(|err| NetError::Setup(format!("mDNS init failed: {err}")))?;

        let behaviour = ClippyBehaviour { gossipsub, mdns };

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity)
            .with_tokio()
            .with_quic()
            .with_behaviour(move |_| behaviour)
            .map_err(|err| NetError::Setup(format!("swarm init failed: {err}")))?
            .build();

        swarm
            .listen_on(
                "/ip4/0.0.0.0/udp/0/quic-v1"
                    .parse()
                    .map_err(|err| NetError::Setup(format!("invalid IPv4 listen addr: {err}")))?,
            )
            .map_err(|err| NetError::Setup(format!("IPv4 listen failed: {err}")))?;

        swarm
            .listen_on(
                "/ip6/::/udp/0/quic-v1"
                    .parse()
                    .map_err(|err| NetError::Setup(format!("invalid IPv6 listen addr: {err}")))?,
            )
            .map_err(|err| NetError::Setup(format!("IPv6 listen failed: {err}")))?;

        let local_peer_id = swarm.local_peer_id().to_string();

        let (command_tx, command_rx) = mpsc::channel(COMMAND_CHANNEL_SIZE);
        let (event_tx, event_rx) = mpsc::channel(EVENT_CHANNEL_SIZE);

        let started_event_tx = event_tx.clone();
        let started_peer_id = local_peer_id.clone();
        tokio::spawn(async move {
            let _ = started_event_tx
                .send(NetEvent::Started {
                    local_peer_id: started_peer_id,
                })
                .await;
            run_swarm_loop(swarm, topic, command_rx, event_tx).await;
        });

        Ok(Self {
            handle: NetHandle {
                command_tx,
                local_peer_id,
                node_name,
            },
            event_rx,
        })
    }

    pub fn handle(&self) -> NetHandle {
        self.handle.clone()
    }

    pub fn local_peer_id(&self) -> &str {
        self.handle.local_peer_id()
    }

    pub async fn broadcast_text(
        &self,
        text: String,
        target_peer_ids: Vec<String>,
    ) -> Result<(), NetError> {
        self.handle.broadcast_text(text, target_peer_ids).await
    }

    pub async fn next_event(&mut self) -> Option<NetEvent> {
        self.event_rx.recv().await
    }
}

async fn run_swarm_loop(
    mut swarm: Swarm<ClippyBehaviour>,
    topic: IdentTopic,
    mut command_rx: mpsc::Receiver<NetCommand>,
    event_tx: mpsc::Sender<NetEvent>,
) {
    let mut known_peers: HashSet<PeerId> = HashSet::new();

    loop {
        tokio::select! {
            maybe_command = command_rx.recv() => {
                match maybe_command {
                    Some(NetCommand::Broadcast(message)) => {
                        match serde_json::to_vec(&message) {
                            Ok(payload) => {
                                if let Err(err) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), payload) {
                                    let _ = event_tx.send(NetEvent::Error {
                                        message: format!("publish failed: {err}")
                                    }).await;
                                } else {
                                    let _ = event_tx.send(NetEvent::Published {
                                        bytes: message.text.len()
                                    }).await;
                                }
                            }
                            Err(err) => {
                                let _ = event_tx.send(NetEvent::Error {
                                    message: format!("serialization failed: {err}")
                                }).await;
                            }
                        }
                    }
                    Some(NetCommand::Shutdown) | None => {
                        break;
                    }
                }
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        let _ = event_tx.send(NetEvent::Listening {
                            address: address.to_string()
                        }).await;
                    }
                    SwarmEvent::Behaviour(ClippyBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                        for (peer_id, _) in peers {
                            if known_peers.insert(peer_id) {
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                let _ = event_tx.send(NetEvent::PeerDiscovered {
                                    peer_id: peer_id.to_string()
                                }).await;
                            }
                        }
                    }
                    SwarmEvent::Behaviour(ClippyBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                        for (peer_id, _) in peers {
                            if known_peers.remove(&peer_id) {
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                let _ = event_tx.send(NetEvent::PeerExpired {
                                    peer_id: peer_id.to_string()
                                }).await;
                            }
                        }
                    }
                    SwarmEvent::Behaviour(ClippyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source,
                        message,
                        ..
                    })) => {
                        match serde_json::from_slice::<ClipboardMessage>(&message.data) {
                            Ok(parsed) => {
                                let _ = event_tx.send(NetEvent::ClipboardReceived {
                                    from_peer: propagation_source.to_string(),
                                    message: parsed,
                                }).await;
                            }
                            Err(err) => {
                                let _ = event_tx.send(NetEvent::Error {
                                    message: format!("failed to parse clipboard message: {err}")
                                }).await;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
