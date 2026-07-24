use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;

use crate::storage::{AppData, Friend, FriendRequest, Message, StorageManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPacket {
    FriendRequestPacket {
        request: FriendRequest,
    },
    FriendAcceptPacket {
        friend: Friend,
        target_id: String,
    },
    ChatMessagePacket {
        message: Message,
    },
    SyncStatePacket {
        data: AppData,
    },
}

pub struct NetworkService {
    pub tx: mpsc::UnboundedSender<NetworkPacket>,
}

impl NetworkService {
    pub fn start(
        app_handle: AppHandle,
        state: Arc<Mutex<AppData>>,
        storage: StorageManager,
    ) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<NetworkPacket>();

        tauri::async_runtime::spawn(async move {
            while let Some(packet) = rx.recv().await {
                if let Ok(mut data) = state.lock() {
                    match &packet {
                        NetworkPacket::FriendRequestPacket { request } => {
                            let exists = data
                                .pending_requests
                                .iter()
                                .any(|r| r.sender_id == request.sender_id);
                            if !exists {
                                data.pending_requests.push(request.clone());
                            }
                        }
                        NetworkPacket::FriendAcceptPacket { friend, .. } => {
                            if !data.friends.iter().any(|f| f.user_id == friend.user_id) {
                                data.friends.push(friend.clone());
                            }
                        }
                        NetworkPacket::ChatMessagePacket { message } => {
                            if !data.messages.iter().any(|m| m.id == message.id) {
                                data.messages.push(message.clone());
                            }
                        }
                        NetworkPacket::SyncStatePacket { data: new_data } => {
                            *data = new_data.clone();
                        }
                    }

                    // Save state changes to disk
                    let _ = storage.save(&data);

                    // Notify Tauri v1 frontend via events
                    let _ = app_handle.emit_all("p2p_event", &packet);
                    let _ = app_handle.emit_all("app-data-updated", data.clone());
                }
            }
        });

        NetworkService { tx }
    }
}
