use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentityData {
    pub display_name: String,
    pub user_id: String,
    pub seed_phrase: String,
    pub public_key_hex: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Friend {
    pub user_id: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FriendRequest {
    pub id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub target_id: String,
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
    pub members: Vec<String>,
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
    pub identity: Option<IdentityData>,
    pub friends: Vec<Friend>,
    pub pending_requests: Vec<FriendRequest>,
    pub groups: Vec<Group>,
    pub messages: Vec<Message>,
    pub bootstrap_nodes: Vec<String>,
}

pub struct StorageManager {
    file_path: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zero-day-chat");
        fs::create_dir_all(&path).ok();
        path.push("data.json");
        Self { file_path: path }
    }

    pub fn load(&self) -> AppData {
        if let Ok(content) = fs::read_to_string(&self.file_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppData::default()
        }
    }

    pub fn save(&self, data: &AppData) -> Result<(), String> {
        let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        fs::write(&self.file_path, json).map_err(|e| e.to_string())
    }
}
