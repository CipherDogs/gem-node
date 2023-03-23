use crate::{constants::*, swarm};
use async_std::io;
use async_trait::async_trait;
use libp2p::{
    core::upgrade::{read_length_prefixed, write_length_prefixed, ProtocolName},
    futures::prelude::*,
    gossipsub, identify, identity, mdns, request_response,
    swarm::NetworkBehaviour,
    PeerId,
};
use std::error::Error;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourEvent")]
pub struct Behaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub mdns: mdns::async_io::Behaviour,
    pub request_response: request_response::Behaviour<SyncCodec>,
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
            request_response: request_response::Behaviour::new(
                SyncCodec(),
                std::iter::once((SyncProtocol(), request_response::ProtocolSupport::Full)),
                Default::default(),
            ),
        })
    }
}

pub enum BehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Identify(identify::Event),
    RequestResponse(request_response::Event<SyncRequest, SyncResponse>),
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

impl From<request_response::Event<SyncRequest, SyncResponse>> for BehaviourEvent {
    fn from(event: request_response::Event<SyncRequest, SyncResponse>) -> Self {
        Self::RequestResponse(event)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRequest(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncResponse(pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct SyncProtocol();

impl ProtocolName for SyncProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/sync/1".as_bytes()
    }
}

#[derive(Clone)]
pub struct SyncCodec();

#[async_trait]
impl request_response::Codec for SyncCodec {
    type Protocol = SyncProtocol;
    type Request = SyncRequest;
    type Response = SyncResponse;

    async fn read_request<T>(&mut self, _: &SyncProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        Ok(SyncRequest(read_length_prefixed(io, 8).await?))
    }

    async fn read_response<T>(&mut self, _: &SyncProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        Ok(SyncResponse(
            read_length_prefixed(io, MAX_TRANSMIT_SIZE).await?,
        ))
    }

    async fn write_request<T>(
        &mut self,
        _: &SyncProtocol,
        io: &mut T,
        SyncRequest(data): SyncRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &SyncProtocol,
        io: &mut T,
        SyncResponse(data): SyncResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;
        Ok(())
    }
}
