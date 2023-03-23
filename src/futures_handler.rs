use crate::{
    block::Block,
    constants::*,
    state::State,
    swarm::behaviour::{Behaviour, SyncRequest, SyncResponse},
    transaction::Transaction,
};
use anyhow::{anyhow, Result};
use base58::ToBase58;
use libp2p::{gossipsub, request_response::ResponseChannel, Swarm};

pub fn mining_handler(
    state: &mut State,
    swarm: &mut Swarm<Behaviour>,
    result: Result<Block>,
) -> Result<()> {
    match result {
        Ok(block) => {
            log::info!(
                "New block mined: {}, Reward: {}",
                block.header.height,
                block.header.reward
            );
            log::trace!(
                "Mined block: {}, n_bits: {}, nonce: {}",
                block.header.height,
                block.header.n_bits,
                block.header.nonce,
            );

            state.put_block(&block)?;

            let block_bytes = bincode::serialize(&block)
                .map_err(|error| anyhow!("Failed to serialize block: {error:?}"))?;

            if let Err(error) = swarm
                .behaviour_mut()
                .gossipsub
                .publish(gossipsub::IdentTopic::new(BLOCK_TOPIC), block_bytes)
            {
                log::warn!("Gossipsub publish failed: {error:?}");
            }
        }
        Err(error) => log::trace!("Mining failed: {error:?}"),
    }

    Ok(())
}

pub fn sync_blocks(state: &State, swarm: &mut Swarm<Behaviour>) -> Result<()> {
    if let Some(peer_id) = swarm.connected_peers().last().cloned() {
        let data = bincode::serialize(&state.last_header.height)
            .map_err(|error| anyhow!("Failed to serialize height for sync: {error:?}"))?;
        let sync_request = SyncRequest(data);

        swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, sync_request);
    }

    Ok(())
}

pub fn sync_request(
    state: &State,
    swarm: &mut Swarm<Behaviour>,
    request: SyncRequest,
    channel: ResponseChannel<SyncResponse>,
) -> Result<()> {
    let mut height: u64 = bincode::deserialize(&request.0)
        .map_err(|error| anyhow!("Failed to deserialize height for sync: {error:?}"))?;

    if state.last_header.height < height {
        let mut size = 0;
        let mut blocks = vec![];

        loop {
            height += 1;

            if let Ok(block) = state.get_block(height) {
                size += bincode::serialize(&block)?.len();
                if size > MAX_TRANSMIT_SIZE {
                    break;
                }

                blocks.push(block);
            } else {
                break;
            }
        }

        let data = bincode::serialize(&blocks)
            .map_err(|error| anyhow!("Failed to serialize blocks: {error:?}"))?;

        if let Err(error) = swarm
            .behaviour_mut()
            .request_response
            .send_response(channel, SyncResponse(data))
        {
            log::warn!("Send response failed: {error:?}");
        }
    }

    Ok(())
}

pub fn sync_response(state: &mut State, response: SyncResponse) -> Result<()> {
    if let Ok(blocks) = bincode::deserialize::<Vec<Block>>(response.0.as_slice()) {
        for block in blocks {
            log::info!("New block received: {}", block.header.height);
            state.put_block(&block)?;
        }

        Ok(())
    } else {
        Err(anyhow!("Failed to deserialize blocks"))
    }
}

pub fn gossipsub_handler(state: &mut State, message: gossipsub::Message) -> Result<()> {
    match message.topic.as_str() {
        BLOCK_TOPIC => {
            let block = bincode::deserialize::<Block>(&message.data)
                .map_err(|error| anyhow!("Failed to deserialize block: {error:?}"))?;

            state.put_block(&block)?;
            log::info!("New block received: {}", block.header.height);
        }
        TRANSACTION_TOPIC => {
            let transaction = bincode::deserialize::<Transaction>(&message.data)
                .map_err(|error| anyhow!("Failed to deserialize transaction: {error:?}"))?;

            state.put_transaction_mempool(transaction);
            log::info!(
                "New transaction received: {}",
                transaction.hash()?.to_base58()
            );
        }
        _ => {}
    }

    Ok(())
}
