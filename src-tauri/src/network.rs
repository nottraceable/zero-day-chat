use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tauri::{AppHandle, Emitter};
use crate::storage::{AppData, Friend, Message, FriendRequest};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum NetworkPacket {
    FriendRequestPacket { request: FriendRequest },
    FriendAcceptPacket { friend: Friend, target_id: String },
    ChatMessagePacket { message: Message },
}

pub struct NetworkService {
    pub tx: mpsc::UnboundedSender<NetworkPacket>,
}

impl NetworkService {
    pub fn start(
        app_handle: AppHandle,
        app_state: Arc<std::sync::Mutex<AppData>>,
        storage: crate::storage::StorageManager,
    ) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<NetworkPacket>();

        tauri::async_runtime::spawn(async move {
            // Processing loop for incoming / outgoing network traffic
            while let Some(packet) = rx.recv().await {
                let mut data = app_state.lock().unwrap();
                let my_id = data.identity.as_ref().map(|i| i.user_id.clone()).unwrap_or_default();

                match packet {
                    NetworkPacket::FriendRequestPacket { request } => {
                        if request.target_id == my_id || request.target_id == "*" {
                            if !data.pending_requests.iter().any(|r| r.id == request.id) {
                                data.pending_requests.push(request);
                                storage.save(&data).ok();
                                let _ = app_handle.emit("app-data-updated", data.clone());
                            }
                        }
                    }
                    NetworkPacket::FriendAcceptPacket { friend, target_id } => {
                        if target_id == my_id {
                            if !data.friends.iter().any(|f| f.user_id == friend.user_id) {
                                data.friends.push(friend);
                                storage.save(&data).ok();
                                let _ = app_handle.emit("app-data-updated", data.clone());
                            }
                        }
                    }
                    NetworkPacket::ChatMessagePacket { message } => {
                        if message.target_id == my_id || data.friends.iter().any(|f| f.user_id == message.sender_id) {
                            if !data.messages.iter().any(|m| m.id == message.id) {
                                data.messages.push(message);
                                storage.save(&data).ok();
                                let _ = app_handle.emit("app-data-updated", data.clone());
                            }
                        }
                    }
                }
            }
        });

        Self { tx }
    }
}
