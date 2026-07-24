use bip39::{Language, Mnemonic};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentityKeys {
    pub user_id: String,
    pub display_name: String,
    pub seed_phrase: String,
    pub public_key_hex: String,
    pub private_key_hex: String,
}

pub fn generate_new_identity(display_name: String) -> Result<IdentityKeys, String> {
    let mnemonic = Mnemonic::generate_in(Language::English, 24)
        .map_err(|e| format!("Failed to generate 24-word seed phrase: {}", e))?;

    let seed_phrase = mnemonic.to_string();
    derive_identity_from_seed(display_name, &seed_phrase)
}

pub fn restore_identity(
    display_name: String,
    user_id: String,
    seed_phrase: String,
) -> Result<IdentityKeys, String> {
    let clean_phrase = seed_phrase.trim();
    let mnemonic: Mnemonic = clean_phrase
        .parse()
        .map_err(|_| "Invalid BIP-39 seed phrase. Please check your 24 words.".to_string())?;

    let derived = derive_identity_from_seed(display_name, &mnemonic.to_string())?;

    let clean_user_id = user_id.trim();
    if !clean_user_id.is_empty() && clean_user_id != derived.user_id {
        return Err("Account ID does not match the key derived from this seed phrase.".to_string());
    }

    Ok(derived)
}

pub fn derive_identity_from_seed(display_name: String, seed_phrase: &str) -> Result<IdentityKeys, String> {
    let mnemonic: Mnemonic = seed_phrase
        .parse()
        .map_err(|_| "Failed to parse seed phrase.".to_string())?;

    let seed = mnemonic.to_seed("");
    let seed_bytes = seed.as_bytes();
    let secret_bytes: [u8; 32] = seed_bytes[0..32]
        .try_into()
        .map_err(|_| "Failed to derive secret key from seed.".to_string())?;

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key: VerifyingKey = signing_key.verifying_key();

    let pub_key_hex = hex::encode(verifying_key.as_bytes());
    let priv_key_hex = hex::encode(signing_key.to_bytes());

    let user_id = format!("zd1{}", &pub_key_hex[..32]);

    Ok(IdentityKeys {
        user_id,
        display_name: display_name.trim().to_string(),
        seed_phrase: seed_phrase.to_string(),
        public_key_hex: pub_key_hex,
        private_key_hex: priv_key_hex,
    })
}
