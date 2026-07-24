#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::State;

mod identity;
mod network;
mod storage;

use identity::{generate_new_identity, restore_identity};
use network::{run_network_node, NetworkHandle};
use storage::{load_app_data, save_app_data, AppData, Channel, Friend, Group, Message};

pub struct AppState {
    pub data: Mutex<AppData>,
    pub network: NetworkHandle,
}

#[tauri::command]
fn get_current_data(state: State<AppState>) -> Result<AppData, String> {
    let data = state.data.lock().map_err(|e| e.to_string())?;
    Ok(data.clone())
}

#[tauri::command]
fn create_account(display_name: String, state: State<AppState>) -> Result<AppData, String> {
    let new_identity = generate_new_identity(display_name)?;

    let mut data = state.data.lock().map_err(|e| e.to_string())?;
    data.identity = Some(new_identity);

    save_app_data(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn recover_account(
    display_name: String,
    user_id: String,
    seed_phrase: String,
    state: State<AppState>,
) -> Result<AppData, String> {
    let restored_identity = restore_identity(display_name, user_id, seed_phrase)?;

    let mut data = state.data.lock().map_err(|e| e.to_string())?;
    data.identity = Some(restored_identity);

    save_app_data(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn add_friend(
    friend_id: String,
    display_name: String,
    state: State<AppState>,
) -> Result<AppData, String> {
    let clean_id = friend_id.trim();
    if !clean_id.starts_with("zd1") && clean_id.len() < 10 {
        return Err("Invalid Friend ID format. Must start with 'zd1'".to_string());
    }

    let mut data = state.data.lock().map_err(|e| e.to_string())?;

    if let Some(ref my_id) = data.identity {
        if my_id.user_id == clean_id {
            return Err("You cannot add your own ID as a friend.".to_string());
        }
    }

    if data.friends.iter().any(|f| f.user_id == clean_id) {
        return Err("This user is already in your address book.".to_string());
    }

    let friend_name = if display_name.trim().is_empty() {
        format!("User_{}", &clean_id[..8.min(clean_id.len())])
    } else {
        display_name.trim().to_string()
    };

    data.friends.push(Friend {
        user_id: clean_id.to_string(),
        display_name: friend_name,
        public_key_hex: String::new(),
        status: "accepted".to_string(),
    });

    save_app_data(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn create_group(name: String, state: State<AppState>) -> Result<AppData, String> {
    let group_name = name.trim();
    if group_name.is_empty() {
        return Err("Group name cannot be empty.".to_string());
    }

    let mut data = state.data.lock().map_err(|e| e.to_string())?;

    let owner_id = data
        .identity
        .as_ref()
        .map(|i| i.user_id.clone())
        .ok_or_else(|| "No active identity found.".to_string())?;

    let random_suffix: String = (0..8)
        .map(|_| format!("{:x}", rand::random::<u8>() % 16))
        .collect();
    let group_id = format!("zd-grp-{}", random_suffix);

    let default_channel = Channel {
        id: format!("{}-general", group_id),
        name: "general".to_string(),
        category: "TEXT CHANNELS".to_string(),
    };

    let new_group = Group {
        id: group_id,
        name: group_name.to_string(),
        owner_id,
        channels: vec![default_channel],
    };

    data.groups.push(new_group);
    save_app_data(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn join_group(group_id: String, state: State<AppState>) -> Result<AppData, String> {
    let clean_id = group_id.trim();
    if clean_id.is_empty() {
        return Err("Group ID link cannot be empty.".to_string());
    }

    let mut data = state.data.lock().map_err(|e| e.to_string())?;

    if data.groups.iter().any(|g| g.id == clean_id) {
        return Err("You are already a member of this group.".to_string());
    }

    let joined_group = Group {
        id: clean_id.to_string(),
        name: format!("Group {}", &clean_id[..8.min(clean_id.len())]),
        owner_id: "external_owner".to_string(),
        channels: vec![Channel {
            id: format!("{}-general", clean_id),
            name: "general".to_string(),
            category: "TEXT CHANNELS".to_string(),
        }],
    };

    data.groups.push(joined_group);
    save_app_data(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn create_channel(
    group_id: String,
    channel_name: String,
    category: String,
    state: State<AppState>,
) -> Result<AppData, String> {
    let mut data = state.data.lock().map_err(|e| e.to_string())?;

    let current_user_id = data
        .identity
        .as_ref()
        .map(|i| i.user_id.clone())
        .ok_or_else(|| "No active identity found.".to_string())?;

    let group = data
        .groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| "Group chat not found.".to_string())?;

    if group.owner_id != current_user_id {
        return Err("Permission Denied: Only the group owner can create channels.".to_string());
    }

    let clean_channel_name = channel_name.trim().to_lowercase().replace(' ', "-");
    let channel_cat = if category.trim().is_empty() {
        "TEXT CHANNELS".to_string()
    } else {
        category.trim().to_uppercase()
    };

    let channel_id = format!("{}-{}", group.id, clean_channel_name);

    if group.channels.iter().any(|c| c.name == clean_channel_name) {
        return Err("A channel with this name already exists in this group.".to_string());
    }

    group.channels.push(Channel {
        id: channel_id,
        name: clean_channel_name,
        category: channel_cat,
    });

    save_app_data(&data)?;
    Ok(data.clone())
}

#[tauri::command]
fn send_message(
    target_id: String,
    channel_id: Option<String>,
    content: String,
    state: State<AppState>,
) -> Result<AppData, String> {
    if content.trim().is_empty() {
        return Err("Message content cannot be empty.".to_string());
    }

    let mut data = state.data.lock().map_err(|e| e.to_string())?;

    let (sender_id, sender_name) = match &data.identity {
        Some(id) => (id.user_id.clone(), id.display_name.clone()),
        None => return Err("No active user session.".to_string()),
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let random_msg_id: String = (0..8)
        .map(|_| format!("{:x}", rand::random::<u8>() % 16))
        .collect();

    let new_message = Message {
        id: format!("msg-{}", random_msg_id),
        sender_id,
        sender_name,
        target_id: target_id.clone(),
        channel_id,
        content: content.clone(),
        timestamp,
    };

    data.messages.push(new_message);
    save_app_data(&data)?;

    let _ = state.network.broadcast(&target_id, &content);

    Ok(data.clone())
}

fn main() {
    let initial_data = load_app_data();
    let (network_handle, rx) = NetworkHandle::new();

    tauri::async_runtime::spawn(run_network_node(rx));

    tauri::Builder::default()
        .manage(AppState {
            data: Mutex::new(initial_data),
            network: network_handle,
        })
        .invoke_handler(tauri::generate_handler![
            get_current_data,
            create_account,
            recover_account,
            add_friend,
            create_group,
            join_group,
            create_channel,
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("Error running Zero-Day Chat backend application");
}
