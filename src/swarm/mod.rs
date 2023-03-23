pub mod behaviour;

use crate::constants::*;
use behaviour::Behaviour;
use libp2p::{
    core::transport::upgrade::Version, gossipsub, identity, mplex, noise, tcp, PeerId, Swarm,
    Transport,
};
use std::{error::Error, time::Duration};

pub async fn init() -> Result<Swarm<Behaviour>, Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(Version::V1)
        .authenticate(noise::NoiseAuthenticated::xx(&local_key.clone()).unwrap())
        .multiplex(mplex::MplexConfig::new())
        .timeout(Duration::from_secs(20))
        .boxed();

    let mut behaviour = Behaviour::new(local_key, local_peer_id).await?;

    behaviour
        .gossipsub
        .subscribe(&gossipsub::IdentTopic::new(BLOCK_TOPIC))?;
    behaviour
        .gossipsub
        .subscribe(&gossipsub::IdentTopic::new(TRANSACTION_TOPIC))?;

    Ok(Swarm::with_async_std_executor(
        transport,
        behaviour,
        local_peer_id,
    ))
}

pub fn protocol_version() -> String {
    format!("gem/{}", CARGO_PKG_VERSION)
}
