use crate::{
    account::Account,
    block::{Block, Header},
    primitive::*,
    state::State,
    transaction::{MerkleTree, Transactions},
};
use anyhow::{anyhow, Result};
use async_std::sync::{Arc, RwLock};
use rand::Rng;
use std::time::SystemTime;

pub async fn trying(
    state: Arc<RwLock<State>>,
    secret_key: &SecretKey,
    public_key: &PublicKey,
    network: Network,
    mining: bool,
) -> Result<Block> {
    let state = state.read().await;

    // Block mining is disabled
    if !mining {
        return Err(anyhow!("Mining disabled"));
    }
    // Node must be synchronized to be able to mine blocks
    if !state.is_sync {
        return Err(anyhow!("Node is not synchronized"));
    }

    let randomx_vm = state.create_randomx_vm_from_height(state.last_header.height)?;

    let transactions = Transactions::default();

    // TODO: Adding transactions

    // Calculate merkle tree
    let merkle_tree = MerkleTree::construct(&transactions)?;
    let root_hash = merkle_tree.root_hash();

    // Get the current time
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis();

    let generator = if let Ok(account) = state.database.get_account_from_public_key(*public_key) {
        account.address
    } else {
        Account::from_public_key(*public_key, network).address
    };

    // TODO: Calculating the block reward

    // Preparing the block header
    let mut header = Header::new(
        state.last_header.height + 1,
        timestamp,
        state.last_header.hash()?,
        generator,
        *public_key,
        1024,
        root_hash,
        transactions.len(),
    );

    // Get a random nonce
    let mut rng = rand::thread_rng();
    let nonce: u64 = rng.gen();

    // Add a nonce and calculate the PoW hash
    header.nonce = nonce;
    header.pow_hash(&randomx_vm)?;
    let value = U256::from(header.pow_hash.as_slice());

    if value <= state.lwma1.get_target() {
        header.n_bits = state.lwma1.get_target_u32();
        header.sign(secret_key)?;

        Ok(Block {
            header,
            transactions,
        })
    } else {
        Err(anyhow!("Value does not satisfy the target"))
    }
}
