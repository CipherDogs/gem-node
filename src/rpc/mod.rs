mod response;

use crate::{constants::*, state::State};
use async_std::sync::{Arc, RwLock};
use base58::FromBase58;
use jsonrpc_core::{types::error::Error, Result};
use jsonrpc_derive::rpc;
use response::BlockResponse;

#[rpc(server)]
pub trait Rpc {
    #[rpc(name = "gem_getBalance")]
    fn get_balance(&self, address: String) -> Result<u64>;
    #[rpc(name = "gem_getBlockByNumber")]
    fn get_block_by_number(&self, height: u64) -> Result<BlockResponse>;
}

pub struct RpcHandler {
    state: Arc<RwLock<State>>,
}

impl RpcHandler {
    pub fn new(state: Arc<RwLock<State>>) -> Self {
        Self { state }
    }
}

impl Rpc for RpcHandler {
    fn get_balance(&self, address: String) -> Result<u64> {
        match self.state.try_read() {
            Some(state) => {
                if let Ok(bytes) = address.from_base58() {
                    let mut address = EMPTY_ADDRESS;
                    address.copy_from_slice(bytes.as_slice());

                    if let Ok(account) = state.database.get_account_from_address(address) {
                        Ok(account.balance)
                    } else {
                        Err(Error::internal_error())
                    }
                } else {
                    Err(Error::invalid_request())
                }
            }
            None => Err(Error::internal_error()),
        }
    }

    fn get_block_by_number(&self, height: u64) -> Result<BlockResponse> {
        match self.state.try_read() {
            Some(state) => {
                if let Ok(block) = state.database.get_block(height) {
                    Ok(BlockResponse::from_block(&block))
                } else {
                    Err(Error::internal_error())
                }
            }
            None => Err(Error::internal_error()),
        }
    }
}
