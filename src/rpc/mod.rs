mod response;

use crate::{constants::*, state::State};
use async_std::sync::{Arc, RwLock};
use base58::FromBase58;
use jsonrpc_core::{
    types::error::{Error, ErrorCode},
    Result,
};
use jsonrpc_derive::rpc;
use response::BlockResponse;

#[rpc(server)]
pub trait Rpc {
    #[rpc(name = "gem_getBalance")]
    fn get_balance(&self, address: String) -> Result<u64>;
    #[rpc(name = "gem_getBlockByHash")]
    fn get_block_by_hash(&self, hash: String) -> Result<BlockResponse>;
    #[rpc(name = "gem_getBlockByNumber")]
    fn get_block_by_number(&self, height: u64) -> Result<BlockResponse>;
}

pub enum RpcError {
    StateRead,
    FromBase58,
    GetDatabase,
    HashCalculate,
}

impl RpcError {
    pub fn to_error(&self) -> Error {
        match self {
            RpcError::StateRead => Error::new(ErrorCode::ServerError(0)),
            RpcError::FromBase58 => Error::new(ErrorCode::ServerError(1)),
            RpcError::GetDatabase => Error::new(ErrorCode::ServerError(2)),
            RpcError::HashCalculate => Error::new(ErrorCode::ServerError(3)),
        }
    }
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
        let state = self
            .state
            .try_read()
            .ok_or_else(|| RpcError::StateRead.to_error())?;

        let bytes = address
            .from_base58()
            .map_err(|_| RpcError::FromBase58.to_error())?;
        let mut address = EMPTY_ADDRESS;
        address.copy_from_slice(bytes.as_slice());

        let account = state
            .database
            .get_account_from_address(address)
            .map_err(|_| RpcError::GetDatabase.to_error())?;
        Ok(account.balance)
    }

    fn get_block_by_hash(&self, hash: String) -> Result<BlockResponse> {
        let state = self
            .state
            .try_read()
            .ok_or_else(|| RpcError::StateRead.to_error())?;

        let bytes = hash
            .from_base58()
            .map_err(|_| RpcError::FromBase58.to_error())?;
        let mut hash = EMPTY_HASH;
        hash.copy_from_slice(bytes.as_slice());

        let block = state
            .database
            .get_block_from_hash(hash)
            .map_err(|_| RpcError::GetDatabase.to_error())?;

        let block_response = BlockResponse::from_block(&block)?;

        Ok(block_response)
    }

    fn get_block_by_number(&self, height: u64) -> Result<BlockResponse> {
        let state = self
            .state
            .try_read()
            .ok_or_else(|| RpcError::StateRead.to_error())?;

        let block = state
            .database
            .get_block_from_height(height)
            .map_err(|_| RpcError::GetDatabase.to_error())?;

        let block_response = BlockResponse::from_block(&block)?;

        Ok(block_response)
    }
}
