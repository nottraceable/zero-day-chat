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
                    // Check local identity
                    let my_user_id = data.identity.as_ref().map(|i| i.user_id.clone());

                    if let Some(my_id) = my_user_id {
                        match &packet {
                            NetworkPacket::FriendRequestPacket { request } => {
                                // ONLY add to pending requests if it was sent TO us by someone else
                                if request.target_id == my_id && request.sender_id != my_id {
                                    let exists = data
                                        .pending_requests
                                        .iter()
                                        .any(|r| r.sender_id == request.sender_id);
                                    if !exists {
                                        data.pending_requests.push(request.clone());
                                    }
                                }
                            }
                            NetworkPacket::FriendAcceptPacket { friend, target_id } => {
                                // ONLY accept if WE were the target of the acceptance
                                if target_id == &my_id {
                                    if !data.friends.iter().any(|f| f.user_id == friend.user_id) {
                                        data.friends.push(friend.clone());
                                    }
                                }
                            }
                            NetworkPacket::ChatMessagePacket { message } => {
                                // Store messages intended for us or sent by us
                                if message.target_id == my_id || message.sender_id == my_id {
                                    if !data.messages.iter().any(|m| m.id == message.id) {
                                        data.messages.push(message.clone());
                                    }
                                }
                            }
                            NetworkPacket::SyncStatePacket { data: new_data } => {
                                *data = new_data.clone();
                            }
                        }

                        // Save state changes to disk
                        let _ = storage.save(&data);

                        // Notify Tauri frontend
                        let _ = app_handle.emit_all("p2p_event", &packet);
                        let _ = app_handle.emit_all("app-data-updated", data.clone());
                    }
                }
            }
        });

        NetworkService { tx }
    }
}
