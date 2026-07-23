mod identity;
mod network;
mod storage;

use identity::{generate_identity, IdentityInfo};

#[tauri::command]
fn create_identity(username: String) -> Result<IdentityInfo, String> {
    let id = generate_identity(username)?;
    if let Ok(json) = serde_json::to_string(&id) {
        let _ = storage::save_local_data(&json);
    }
    Ok(id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![create_identity])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
