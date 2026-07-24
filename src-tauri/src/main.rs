#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod identity;
mod storage;
mod network;

use identity::Identity;
use storage::{AppData, Channel, Friend, Group, Message, StorageManager, FriendRequest, IdentityData};
use network::{NetworkPacket, NetworkCommand};
use std::sync::{Arc, Mutex};
use tauri::{State, Manager};
use tokio::sync::mpsc;

impl From<Identity> for IdentityData {
    fn from(id: Identity) -> Self {
        IdentityData {
            display_name: id.display_name,
            user_id: id.user_id,
            seed_phrase: id.seed_phrase,
            public_key_hex: id.public_key_hex,
        }
    }
}

struct AppState {
    data: Arc<Mutex<AppData>>,
    storage: StorageManager,
    net_tx: mpsc::UnboundedSender<NetworkCommand>,
}

#[tauri::command]
fn get_current_data(state: State<'_, AppState>) -> Result<AppData, String> {
    let data = state.data.lock().unwrap();
    Ok(data.clone())
}

#[tauri::command]
fn create_account(display_name: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let identity = Identity::generate(display_name)?;
    data.identity = Some(identity.into());
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn recover_account(display_name: String, user_id: String, seed_phrase: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let identity = Identity::recover(display_name, user_id, seed_phrase)?;
    data.identity = Some(identity.into());
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn logout(state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    data.identity = None;
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn add_friend(friend_id: String, display_name: Option<String>, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();

    let clean_id = friend_id.trim().to_string();
    if clean_id.is_empty() {
        return Err("Friend User ID cannot be empty.".into());
    }

    if data.friends.iter().any(|f| f.user_id == clean_id) {
        return Err("Friend is already in your contacts.".into());
    }

    let name = display_name
        .unwrap_or_default()
        .trim()
        .to_string();

    let final_name = if name.is_empty() {
        format!("Peer-{}", &clean_id[..clean_id.len().min(8)])
    } else {
        name
    };

    data.friends.push(Friend {
        user_id: clean_id,
        display_name: final_name,
    });

    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn send_friend_request(target_id: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let data = state.data.lock().unwrap();
    let identity = data.identity.as_ref().ok_or("No active session.")?.clone();
    let clean_target = target_id.trim().to_string();

    if clean_target.is_empty() {
        return Err("Target User ID cannot be empty.".into());
    }
    if clean_target == identity.user_id {
        return Err("You cannot add yourself.".into());
    }

    let req = FriendRequest {
        id: format!("freq-{}", hex::encode(rand::random::<[u8; 6]>())),
        sender_id: identity.user_id,
        sender_name: identity.display_name,
        target_id: clean_target,
    };

    let _ = state.net_tx.send(NetworkCommand::SendPacket(NetworkPacket::FriendRequestPacket { request: req }));
    Ok(data.clone())
}

#[tauri::command]
fn accept_friend_request(request_id: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let identity = data.identity.as_ref().ok_or("No active session.")?.clone();

    let req_pos = data.pending_requests.iter().position(|r| r.id == request_id)
        .ok_or("Friend request not found.")?;

    let req = data.pending_requests.remove(req_pos);

    let friend = Friend {
        user_id: req.sender_id.clone(),
        display_name: req.sender_name.clone(),
    };

    if !data.friends.iter().any(|f| f.user_id == friend.user_id) {
        data.friends.push(friend.clone());
    }

    state.storage.save(&data)?;

    let _ = state.net_tx.send(NetworkCommand::SendPacket(NetworkPacket::FriendAcceptPacket {
        friend: Friend {
            user_id: identity.user_id,
            display_name: identity.display_name,
        },
        target_id: req.sender_id,
    }));

    Ok(data.clone())
}

#[tauri::command]
fn create_group(name: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let user_id = data.identity.as_ref().ok_or("No active session.")?.user_id.clone();

    let group_id = format!("zd-grp-{}", hex::encode(rand::random::<[u8; 6]>()));
    let default_channel = Channel {
        id: format!("ch-{}", hex::encode(rand::random::<[u8; 4]>())),
        name: "general".to_string(),
        category: "TEXT CHANNELS".to_string(),
    };

    let group = Group {
        id: group_id,
        name,
        owner_id: user_id.clone(),
        channels: vec![default_channel],
        members: vec![user_id],
    };

    data.groups.push(group);
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn join_group(group_id: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let user_id = data.identity.as_ref().ok_or("No active session.")?.user_id.clone();
    let clean_group_id = group_id.trim().to_string();

    if clean_group_id.is_empty() {
        return Err("Group ID link cannot be empty.".into());
    }

    if data.groups.iter().any(|g| g.id == clean_group_id) {
        return Err("You are already a member of this group.".into());
    }

    let default_channel = Channel {
        id: format!("ch-{}", hex::encode(rand::random::<[u8; 4]>())),
        name: "general".to_string(),
        category: "TEXT CHANNELS".to_string(),
    };

    let new_group = Group {
        id: clean_group_id,
        name: "Joined Mesh Group".to_string(),
        owner_id: "remote_owner".to_string(),
        channels: vec![default_channel],
        members: vec![user_id],
    };

    data.groups.push(new_group);
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn leave_group(group_id: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    data.groups.retain(|g| g.id != group_id);
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn delete_group(group_id: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let current_user_id = data.identity.as_ref().ok_or("No active session.")?.user_id.clone();

    if let Some(group) = data.groups.iter().find(|g| g.id == group_id) {
        if group.owner_id != current_user_id {
            return Err("Permission denied: Only the group owner can delete this group.".into());
        }
    }

    data.groups.retain(|g| g.id != group_id);
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn create_channel(group_id: String, channel_name: String, category: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let user_id = data.identity.as_ref().ok_or("No active session.")?.user_id.clone();

    let group = data.groups.iter_mut().find(|g| g.id == group_id)
        .ok_or("Group not found.")?;

    if group.owner_id != user_id {
        return Err("Only the group owner can create channels.".into());
    }

    let new_channel = Channel {
        id: format!("ch-{}", hex::encode(rand::random::<[u8; 4]>())),
        name: channel_name,
        category: if category.trim().is_empty() { "TEXT CHANNELS".to_string() } else { category },
    };

    group.channels.push(new_channel);
    state.storage.save(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn send_message(target_id: String, channel_id: Option<String>, content: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let identity = data.identity.as_ref().ok_or("No active session.")?.clone();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let msg = Message {
        id: format!("msg-{}", hex::encode(rand::random::<[u8; 6]>())),
        sender_id: identity.user_id.clone(),
        sender_name: identity.display_name.clone(),
        target_id: target_id.clone(),
        channel_id,
        content,
        timestamp,
    };

    data.messages.push(msg.clone());
    state.storage.save(&data)?;

    let _ = state.net_tx.send(NetworkCommand::SendPacket(NetworkPacket::ChatMessagePacket { message: msg }));
    Ok(data.clone())
}

#[tauri::command]
fn add_bootstrap_node(node_multiaddr: String, state: State<'_, AppState>) -> Result<AppData, String> {
    let mut data = state.data.lock().unwrap();
    let clean_addr = node_multiaddr.trim().to_string();

    if clean_addr.is_empty() {
        return Err("Bootstrap Multiaddress cannot be empty.".into());
    }

    if !data.bootstrap_nodes.contains(&clean_addr) {
        data.bootstrap_nodes.push(clean_addr);
        state.storage.save(&data)?;
    }

    Ok(data.clone())
}

fn main() {
    let storage = StorageManager::new();
    let initial_data = Arc::new(Mutex::new(storage.load()));

    tauri::Builder::default()
        .setup(move |app| {
            let net_service = network::NetworkService::start(
                app.handle().clone(),
                initial_data.clone(),
                StorageManager::new(),
            );

            app.manage(AppState {
                data: initial_data,
                storage,
                net_tx: net_service.tx,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_current_data,
            create_account,
            recover_account,
            logout,
            add_friend,
            send_friend_request,
            accept_friend_request,
            create_group,
            join_group,
            leave_group,
            delete_group,
            create_channel,
            send_message,
            add_bootstrap_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running zero-day-chat application");
}
