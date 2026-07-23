use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkMessage {
    pub id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub payload: String,
    pub timestamp: u64,
}

pub struct NetworkState {
    pub active_channel: Mutex<Option<String>>,
    pub connected_peers: Mutex<HashSet<String>>,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            active_channel: Mutex::new(None),
            connected_peers: Mutex::new(HashSet::new()),
        }
    }
}

#[tauri::command]
pub fn join_channel(channel_id: String, state: State<'_, NetworkState>) -> Result<bool, String> {
    let mut channel_lock = state.active_channel.lock().map_err(|e| e.to_string())?;
    *channel_lock = Some(channel_id);
    Ok(true)
}

#[tauri::command]
pub fn send_peer_message(
    channel_id: String,
    content: String,
    sender_id: String,
) -> Result<NetworkMessage, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let message = NetworkMessage {
        id: format!("msg_{}", timestamp),
        channel_id,
        sender_id,
        payload: content,
        timestamp,
    };

    Ok(message)
}

#[tauri::command]
pub fn get_connected_peers(state: State<'_, NetworkState>) -> Result<Vec<String>, String> {
    let peers = state.connected_peers.lock().map_err(|e| e.to_string())?;
    Ok(peers.iter().cloned().collect())
}
