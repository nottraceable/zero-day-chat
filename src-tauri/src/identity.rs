use bip39::{Mnemonic, Language};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Identity {
    pub display_name: String,
    pub user_id: String,
    pub seed_phrase: String,
    pub public_key_hex: String,
}

impl Identity {
    pub fn generate(display_name: String) -> Result<Self, String> {
        let mut entropy = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| format!("Mnemonic generation failed: {}", e))?;
        
        let seed_phrase = mnemonic.to_string();
        let seed = mnemonic.to_seed("");
        
        let signing_key = SigningKey::from_bytes(&seed[0..32].try_into().unwrap());
        let public_key = signing_key.verifying_key();
        let public_key_hex = hex::encode(public_key.as_bytes());
        let user_id = format!("zd1{}", &public_key_hex[..16]);

        Ok(Self {
            display_name,
            user_id,
            seed_phrase,
            public_key_hex,
        })
    }

    pub fn recover(display_name: String, user_id: String, seed_phrase: String) -> Result<Self, String> {
        let mnemonic = Mnemonic::parse_in(Language::English, &seed_phrase)
            .map_err(|e| format!("Invalid seed phrase: {}", e))?;

        let seed = mnemonic.to_seed("");
        let signing_key = SigningKey::from_bytes(&seed[0..32].try_into().unwrap());
        let public_key = signing_key.verifying_key();
        let public_key_hex = hex::encode(public_key.as_bytes());
        
        let derived_user_id = format!("zd1{}", &public_key_hex[..16]);
        let final_user_id = if user_id.trim().is_empty() {
            derived_user_id
        } else {
            user_id
        };

        Ok(Self {
            display_name,
            user_id: final_user_id,
            seed_phrase,
            public_key_hex,
        })
    }
}
