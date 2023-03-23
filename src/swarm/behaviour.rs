use crate::{constants::*, swarm};
use libp2p::{gossipsub, identify, identity, mdns, swarm::NetworkBehaviour, PeerId};
use std::error::Error;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourEvent")]
pub struct Behaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub mdns: mdns::async_io::Behaviour,
}

impl Behaviour {
    pub async fn new(
        local_key: identity::Keypair,
        local_peer_id: PeerId,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub::ConfigBuilder::default().build()?,
            )?,
            identify: identify::Behaviour::new(identify::Config::new(
                swarm::protocol_version(),
                local_key.public(),
            )),
            mdns: mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id)?,
        })
    }
}

pub enum BehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Identify(identify::Event),
}

impl From<gossipsub::Event> for BehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        Self::Gossipsub(event)
    }
}

impl From<mdns::Event> for BehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
    }
}

impl From<identify::Event> for BehaviourEvent {
    fn from(event: identify::Event) -> Self {
        Self::Identify(event)
    }
}
