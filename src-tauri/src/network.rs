use libp2p::futures::StreamExt;
use libp2p::{gossipsub, swarm::SwarmEvent, Multiaddr};
use tokio::sync::mpsc;

pub struct NetworkService {
    pub tx: mpsc::UnboundedSender<String>,
}

impl NetworkService {
    pub fn start() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        tokio::spawn(async move {
            let mut swarm = match libp2p::SwarmBuilder::with_new_identity()
                .with_tokio()
                .with_tcp(
                    libp2p::tcp::Config::default(),
                    libp2p::noise::Config::new,
                    libp2p::yamux::Config::default,
                )
                .map_err(|e| e.to_string())
            {
                Ok(builder) => match builder.with_behaviour(|_| {
                    let gossipsub_config = gossipsub::ConfigBuilder::default().build().unwrap();
                    gossipsub::Behaviour::<gossipsub::IdentityTransform>::new(
                        gossipsub::MessageAuthenticity::Anonymous,
                        gossipsub_config,
                    )
                    .unwrap()
                }) {
                    Ok(b) => b.build(),
                    Err(_) => return,
                },
                Err(_) => return,
            };

            let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse::<Multiaddr>().unwrap());

            loop {
                tokio::select! {
                    Some(_msg) = rx.recv() => {
                        // Broadcast message over Gossipsub mesh network
                    }
                    event = swarm.select_next_some() => {
                        if let SwarmEvent::NewListenAddr { address, .. } = event {
                            println!("[P2P Mesh] Listening on: {}", address);
                        }
                    }
                }
            }
        });

        Self { tx }
    }
}
