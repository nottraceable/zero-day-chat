use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredMessage {
    pub id: String,
    pub channel_id: String,
    pub sender: String,
    pub text: String,
    pub timestamp: u64,
}

fn get_storage_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to retrieve app data path: {}", e))?;
    
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| format!("Failed to create storage directory: {}", e))?;
    }
    
    path.push("messages.json");
    Ok(path)
}

#[tauri::command]
pub fn save_message(app: AppHandle, message: StoredMessage) -> Result<(), String> {
    let path = get_storage_path(&app)?;
    let mut messages = load_messages(app.clone(), message.channel_id.clone()).unwrap_or_default();
    
    messages.push(message);

    let data = serde_json::to_string_pretty(&messages)
        .map_err(|e| format!("Failed to serialize message payload: {}", e))?;

    fs::write(path, data).map_err(|e| format!("Disk write failure: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn load_messages(app: AppHandle, channel_id: String) -> Result<Vec<StoredMessage>, String> {
    let path = get_storage_path(&app)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("Disk read error: {}", e))?;
    let all_messages: Vec<StoredMessage> = serde_json::from_str(&raw).unwrap_or_default();

    let filtered = all_messages
        .into_iter()
        .filter(|m| m.channel_id == channel_id)
        .collect();

    Ok(filtered)
}

#[tauri::command]
pub fn clear_storage(app: AppHandle) -> Result<(), String> {
    let path = get_storage_path(&app)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Removal failed: {}", e))?;
    }
    Ok(())
}
