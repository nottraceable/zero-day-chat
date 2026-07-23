use bip39::{Mnemonic, Language};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IdentityInfo {
    pub username: String,
    pub seed_phrase: String,
    pub public_key: String,
}

pub fn generate_identity(username: String) -> Result<IdentityInfo, String> {
    let mut entropy = [0u8; 32];
    OsRng.fill_bytes(&mut entropy);
    
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| e.to_string())?;

    let seed_phrase = mnemonic.to_string();
    let pub_key_mock = format!("0x{}", hex::encode(&entropy[0..8]));

    Ok(IdentityInfo {
        username,
        seed_phrase,
        public_key: pub_key_mock,
    })
}
