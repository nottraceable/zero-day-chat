use bip39::{Mnemonic, Language};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IdentityInfo {
    pub username: String,
    pub seed_phrase: String,
    pub public_key: String,
}

pub fn generate_identity(username: String) -> Result<IdentityInfo, String> {
    let mut rng = OsRng;
    let mnemonic = Mnemonic::generate_in_with(&mut rng, Language::English, 24)
        .map_err(|e| e.to_string())?;
    
    let seed_phrase = mnemonic.to_string();
    let pub_key_mock = format!("0x{}...", &hex::encode(&seed_phrase.as_bytes()[0..8]));

    Ok(IdentityInfo {
        username,
        seed_phrase,
        public_key: pub_key_mock,
    })
}
