mod identity;
mod network;
mod storage;

use identity::IdentityState;
use network::NetworkState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(IdentityState(Mutex::new(None)))
        .manage(NetworkState::new())
        .invoke_handler(tauri::generate_handler![
            identity::generate_identity,
            identity::import_identity,
            identity::get_current_identity,
            network::join_channel,
            network::send_peer_message,
            network::get_connected_peers,
            storage::save_message,
            storage::load_messages,
            storage::clear_storage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
