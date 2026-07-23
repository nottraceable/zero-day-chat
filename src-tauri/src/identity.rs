use bip39::{Mnemonic, MnemonicType, Language};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Identity {
    pub peer_id: String,
    pub public_key: String,
    pub mnemonic: Option<String>,
}

pub struct IdentityState(pub Mutex<Option<Identity>>);

#[tauri::command]
pub fn generate_identity(state: State<'_, IdentityState>) -> Result<Identity, String> {
    let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
    let phrase = mnemonic.phrase().to_string();

    let entropy = mnemonic.entropy();
    let peer_id = format!("peer_{}", hex::encode(&entropy[..4]));
    let public_key = format!("pk_{}", hex::encode(&entropy[4..12]));

    let identity = Identity {
        peer_id,
        public_key,
        mnemonic: Some(phrase),
    };

    let mut lock = state.0.lock().map_err(|e| e.to_string())?;
    *lock = Some(identity.clone());

    Ok(identity)
}

#[tauri::command]
pub fn import_identity(phrase: String, state: State<'_, IdentityState>) -> Result<Identity, String> {
    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)
        .map_err(|_| "Invalid seed phrase format".to_string())?;

    let entropy = mnemonic.entropy();
    let peer_id = format!("peer_{}", hex::encode(&entropy[..4]));
    let public_key = format!("pk_{}", hex::encode(&entropy[4..12]));

    let identity = Identity {
        peer_id,
        public_key,
        mnemonic: None,
    };

    let mut lock = state.0.lock().map_err(|e| e.to_string())?;
    *lock = Some(identity.clone());

    Ok(identity)
}

#[tauri::command]
pub fn get_current_identity(state: State<'_, IdentityState>) -> Result<Option<Identity>, String> {
    let lock = state.0.lock().map_err(|e| e.to_string())?;
    Ok((*lock).clone())
}
