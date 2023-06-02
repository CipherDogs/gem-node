pub mod behaviour;

use crate::constants::*;
use behaviour::Behaviour;
use libp2p::{
    core::transport::upgrade::Version, gossipsub, identity, noise, swarm, tcp, yamux, PeerId,
    Transport,
};
use std::{error::Error, time::Duration};

pub async fn init() -> Result<swarm::Swarm<Behaviour>, Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(Version::V1)
        .authenticate(noise::Config::new(&local_key.clone()).unwrap())
        .multiplex(yamux::Config::default())
        .timeout(Duration::from_secs(20))
        .boxed();

    let mut behaviour = Behaviour::new(local_key, local_peer_id).await?;

    behaviour
        .gossipsub
        .subscribe(&gossipsub::IdentTopic::new(BLOCK_TOPIC))?;
    behaviour
        .gossipsub
        .subscribe(&gossipsub::IdentTopic::new(TRANSACTION_TOPIC))?;

    Ok(swarm::SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build())
}

pub fn protocol_version() -> String {
    format!("gem/{}", CARGO_PKG_VERSION)
}
