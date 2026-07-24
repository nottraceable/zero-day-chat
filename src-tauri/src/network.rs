use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{
    gossipsub, identity, mdns, noise, tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;

use crate::storage::{AppData, Friend, FriendRequest, Message, StorageManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPacket {
    FriendRequestPacket { request: FriendRequest },
    FriendAcceptPacket { friend: Friend, target_id: String },
    ChatMessagePacket { message: Message },
    SyncStatePacket { data: AppData },
}

#[derive(Debug)]
pub enum NetworkCommand {
    SendPacket(NetworkPacket),
    ConnectPeer(String),
}

#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct NetworkService {
    pub tx: mpsc::UnboundedSender<NetworkCommand>,
}

impl NetworkService {
    pub fn start(
        app_handle: AppHandle,
        state: Arc<Mutex<AppData>>,
        storage: StorageManager,
    ) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<NetworkCommand>();

        tauri::async_runtime::spawn(async move {
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(local_key.public());

            // Gossipsub setup
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .max_transmit_size(262144) // 256KB
                .build()
                .expect("Valid gossipsub config");

            let mut gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config,
            )
            .expect("Valid gossipsub behaviour");

            let topic = gossipsub::IdentTopic::new("zero-day-chat-global");
            let _ = gossipsub.subscribe(&topic);

            // mDNS setup for local network auto-discovery
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                local_peer_id,
            )
            .expect("Valid mDNS behaviour");

            let behaviour = AppBehaviour { gossipsub, mdns };

            let mut swarm = SwarmBuilder::with_existing_identity(local_key)
                .with_tokio()
                .with_tcp(
                    tcp::Config::default(),
                    noise::Config::new,
                    yamux::Config::default,
                )
                .expect("TCP transport build failed")
                .with_behaviour(|_| -> Result<AppBehaviour, Box<dyn Error + Send + Sync>> {
                    Ok(behaviour)
                })
                .expect("Behaviour init failed")
                .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
                .build();

            // Listen on all IPv4 interfaces
            let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap());

            loop {
                tokio::select! {
                    cmd = rx.recv() => {
                        match cmd {
                            Some(NetworkCommand::SendPacket(packet)) => {
                                // 1. Save state locally first
                                process_packet(&packet, &state, &storage, &app_handle);

                                // 2. Broadcast across P2P swarm
                                if let Ok(json_bytes) = serde_json::to_vec(&packet) {
                                    let _ = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json_bytes);
                                }
                            }
                            Some(NetworkCommand::ConnectPeer(addr_str)) => {
                                if let Ok(multiaddr) = addr_str.parse::<Multiaddr>() {
                                    let _ = swarm.dial(multiaddr);
                                }
                            }
                            None => break,
                        }
                    }
                    event = swarm.select_next_some() => {
                        match event {
                            SwarmEvent::NewListenAddr { address, .. } => {
                                let addr_msg = format!("{}/p2p/{}", address, local_peer_id);
                                let _ = app_handle.emit_all("p2p_listen_addr", addr_msg);
                            }
                            SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                                for (peer_id, _multiaddr) in list {
                                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                }
                            }
                            SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                                for (peer_id, _multiaddr) in list {
                                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                }
                            }
                            SwarmEvent::Behaviour(AppBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                                if let Ok(packet) = serde_json::from_slice::<NetworkPacket>(&message.data) {
                                    process_packet(&packet, &state, &storage, &app_handle);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        NetworkService { tx }
    }
}

fn process_packet(
    packet: &NetworkPacket,
    state: &Arc<Mutex<AppData>>,
    storage: &StorageManager,
    app_handle: &AppHandle,
) {
    if let Ok(mut data) = state.lock() {
        let my_user_id = data.identity.as_ref().map(|i| i.user_id.clone());

        if let Some(my_id) = my_user_id {
            let mut updated = false;

            match packet {
                NetworkPacket::FriendRequestPacket { request } => {
                    if request.target_id == my_id && request.sender_id != my_id {
                        let exists = data.pending_requests.iter().any(|r| r.sender_id == request.sender_id);
                        if !exists {
                            data.pending_requests.push(request.clone());
                            updated = true;
                        }
                    }
                }
                NetworkPacket::FriendAcceptPacket { friend, target_id } => {
                    if target_id == &my_id {
                        if !data.friends.iter().any(|f| f.user_id == friend.user_id) {
                            data.friends.push(friend.clone());
                            updated = true;
                        }
                    }
                }
                NetworkPacket::ChatMessagePacket { message } => {
                    let is_in_group = data.groups.iter().any(|g| g.id == message.target_id);
                    if message.target_id == my_id || message.sender_id == my_id || is_in_group {
                        if !data.messages.iter().any(|m| m.id == message.id) {
                            data.messages.push(message.clone());
                            updated = true;
                        }
                    }
                }
                NetworkPacket::SyncStatePacket { data: new_data } => {
                    *data = new_data.clone();
                    updated = true;
                }
            }

            if updated {
                let _ = storage.save(&data);
                let _ = app_handle.emit_all("p2p_event", packet);
                let _ = app_handle.emit_all("app-data-updated", data.clone());
            }
        }
    }
}	
