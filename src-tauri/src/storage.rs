use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::identity::IdentityKeys;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Friend {
    pub user_id: String,
    pub display_name: String,
    pub public_key_hex: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub category: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub channels: Vec<Channel>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub target_id: String,
    pub channel_id: Option<String>,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppData {
    pub identity: Option<IdentityKeys>,
    pub friends: Vec<Friend>,
    pub groups: Vec<Group>,
    pub messages: Vec<Message>,
}

pub fn get_storage_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("zero-day-chat");
    fs::create_dir_all(&path).ok();
    path.push("app_data.json");
    path
}

pub fn load_app_data() -> AppData {
    let path = get_storage_path();
    if !path.exists() {
        return AppData::default();
    }

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return AppData::default(),
    };

    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return AppData::default();
    }

    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_app_data(data: &AppData) -> Result<(), String> {
    let path = get_storage_path();
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Serialization error: {}", e))?;

    let mut file = File::create(&path)
        .map_err(|e| format!("Failed to create storage file: {}", e))?;

    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write storage file: {}", e))?;

    Ok(())
}
