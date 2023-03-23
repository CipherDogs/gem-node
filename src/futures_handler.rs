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
            state.put_block(&block)?;

            let block_bytes = bincode::serialize(&block)
                .map_err(|error| anyhow!("Failed to serialize header: {error:?}"))?;

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

pub fn sync_request(
    state: &State,
    swarm: &mut Swarm<Behaviour>,
    request: SyncRequest,
    channel: ResponseChannel<SyncResponse>,
) -> Result<()> {
    let mut height: u64 = bincode::deserialize(&request.0)?;

    let mut size = 0;
    let mut blocks = vec![];

    loop {
        height += 1;
        let block = state.get_block(height)?;

        size += bincode::serialize(&block)?.len();
        if size > MAX_TRANSMIT_SIZE {
            break;
        }

        blocks.push(block);
    }

    let data = bincode::serialize(&blocks)?;

    if let Err(error) = swarm
        .behaviour_mut()
        .request_response
        .send_response(channel, SyncResponse(data))
    {
        log::warn!("Send response failed: {error:?}");
    }

    Ok(())
}

pub fn sync_response(state: &mut State, response: SyncResponse) -> Result<()> {
    let blocks = bincode::deserialize::<Vec<Block>>(&response.0)?;

    for block in blocks {
        log::info!("New block received: {}", block.header.height);
        state.put_block(&block)?;
    }

    Ok(())
}

pub fn gossipsub_handler(state: &mut State, message: gossipsub::Message) -> Result<()> {
    match message.topic.as_str() {
        BLOCK_TOPIC => {
            let block = bincode::deserialize::<Block>(&message.data)?;
            log::info!("New block received: {}", block.header.height);
            state.put_block(&block)?;
        }
        TRANSACTION_TOPIC => {
            let transaction = bincode::deserialize::<Transaction>(&message.data)?;
            log::info!(
                "New transaction received: {}",
                transaction.hash()?.to_base58()
            );
            state.put_transaction_mempool(transaction);
        }
        _ => {}
    }

    Ok(())
}
