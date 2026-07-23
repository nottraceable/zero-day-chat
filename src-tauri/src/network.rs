use libp2p::{identity, PeerId};
use tokio::sync::mpsc;

pub async fn init_p2p_node() -> Result<(PeerId, mpsc::Receiver<String>), Box<dyn std::error::Error>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    let (_tx, rx) = mpsc::channel(100);

    Ok((peer_id, rx))
}
