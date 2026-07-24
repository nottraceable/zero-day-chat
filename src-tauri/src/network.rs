use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkCommand {
    BroadcastMessage {
        topic: String,
        payload: String,
    },
    ConnectPeer {
        multiaddr: String,
    },
}

#[derive(Clone)]
pub struct NetworkHandle {
    pub sender: mpsc::UnboundedSender<NetworkCommand>,
}

impl NetworkHandle {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<NetworkCommand>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (NetworkHandle { sender: tx }, rx)
    }

    pub fn broadcast(&self, topic: &str, message: &str) -> Result<(), String> {
        self.sender
            .send(NetworkCommand::BroadcastMessage {
                topic: topic.to_string(),
                payload: message.to_string(),
            })
            .map_err(|e| format!("Failed to transmit network command: {}", e))
    }
}

pub async fn run_network_node(mut rx: mpsc::UnboundedReceiver<NetworkCommand>) {
    tokio::spawn(async move {
        println!("[Zero-Day Mesh] libp2p node listener initialized.");
        while let Some(cmd) = rx.recv().await {
            match cmd {
                NetworkCommand::BroadcastMessage { topic, payload } => {
                    println!("[Zero-Day Mesh] Broadcasting frame to topic '{}': {}", topic, payload);
                }
                NetworkCommand::ConnectPeer { multiaddr } => {
                    println!("[Zero-Day Mesh] Dialing peer address: {}", multiaddr);
                }
            }
        }
    });
}
