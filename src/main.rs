use async_std::{
    stream,
    sync::{Arc, RwLock},
};
use base58::ToBase58;
use clap::Parser;
use gem_node::{
    constants::*,
    futures_handler::*,
    pow::miner,
    primitive::*,
    rpc::{Rpc, RpcHandler},
    state::State,
    swarm::{self, behaviour::BehaviourEvent},
    wallet,
};
use jsonrpc_http_server::{jsonrpc_core::IoHandler, ServerBuilder};
use libp2p::{
    futures::{select, FutureExt, StreamExt},
    gossipsub, identify, mdns, request_response,
    swarm::SwarmEvent,
};
use log::LevelFilter;
use std::{error::Error, time::Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from("./gem"))]
    directory: String,
    #[arg(long, value_enum, default_value_t = Network::Testnet)]
    network: Network,
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    rpc_address: String,
    #[arg(long, default_value_t = 31337)]
    rpc_port: u16,
    #[arg(long, default_value_t = false)]
    generate_keys: bool,
    #[arg(long, default_value_t = String::new())]
    import_secret_key: String,
    #[arg(long, default_value_t = false)]
    mining: bool,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialization of logging with the Info level
    env_logger::builder()
        .filter(Some("gem"), LevelFilter::Info)
        .filter(Some("gem_node"), LevelFilter::Info)
        .init();

    // Parsing command line arguments
    let args = Args::parse();

    // Initializing blockchain state
    let db_path = format!("{}/data", args.directory);
    let state = Arc::new(RwLock::new(State::new(&db_path, args.network)?));

    // Generating or importing keys
    let wallet_path = format!("{}/wallet.dat", args.directory);
    if args.generate_keys {
        let (secret_key, _) = wallet::generate();
        wallet::save(&wallet_path, secret_key)?;
        std::process::exit(0);
    } else if !args.import_secret_key.is_empty() {
        let (secret_key, _) = wallet::import(&args.import_secret_key)?;
        wallet::save(&wallet_path, secret_key)?;
        std::process::exit(0);
    }

    let (mut secret_key, mut public_key) = (EMPTY_SECRET_KEY, EMPTY_PUBLIC_KEY);

    // If the node mines blocks, then we load the wallet
    if args.mining {
        (secret_key, public_key) = wallet::load(&wallet_path)?;
        log::info!("Miner public key: {}", public_key.to_base58());
    } else {
        log::warn!("Block mining is disabled by default");
    }

    // Initializing libp2p Swarm
    let mut swarm = swarm::init().await?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut io = IoHandler::default();
    let rpc = RpcHandler::new(state.clone());
    io.extend_with(rpc.to_delegate());

    let rpc_addr = format!("{}:{}", args.rpc_address, args.rpc_port);
    let _server = ServerBuilder::new(io)
        .threads(1)
        .start_http(&rpc_addr.parse()?)?;

    let mut sync_interval = stream::interval(Duration::from_secs(15));

    loop {
        select! {
            _ = sync_interval.next().fuse() => if let Err(error) = sync_blocks(state.clone(), &mut swarm).await {
                log::error!("Sync failed: {error:?}");
            },
            result = miner::trying(state.clone(), &secret_key, &public_key, args.network, args.mining).fuse() => mining_handler(state.clone(), &mut swarm, result).await?,
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Swarm listening on {address:?}");
                },
                SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received{ peer_id, info })) => {
                    if info.protocol_version != swarm::protocol_version() {
                        swarm.ban_peer_id(peer_id);
                        log::warn!("Protocol version does not match: {info:?}");
                    }
                },
                SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        log::info!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        log::info!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(request_response::Event::Message { peer, message })) => match message {
                    request_response::Message::Request { request, channel, .. } => if let Err(error) = sync_request(state.clone(), &mut swarm, request, channel).await {
                        log::error!("Sync request failed: {error:?}");
                    },
                    request_response::Message::Response { response, .. } => if let Err(error) = sync_response(state.clone(), &mut swarm, peer, response).await {
                        log::error!("Sync response failed: {error:?}");
                    },
                },
                SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    message,
                    ..
                })) => if let Err(error) = gossipsub_handler(state.clone(), &mut swarm, message).await {
                    log::error!("Gossipsub failed: {error:?}");
                },
                _ => {}
            }
        }
    }
}
