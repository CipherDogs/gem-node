use crate::{
    block::Block,
    primitive::*,
    rpc::RpcError,
    transaction::{Data, Transaction},
};
use base58::ToBase58;
use jsonrpc_core::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BlockResponse {
    id: String,
    pow_hash: String,
    height: u64,
    timestamp: u128,
    prev_block: String,
    generator: String,
    generator_public_key: String,
    reward: u64,
    root: String,
    transactions_count: u64,
    n_bits: u32,
    nonce: u64,
    signature: String,
    transactions: Vec<TransactionResponse>,
}

impl BlockResponse {
    pub fn from_block(block: &Block) -> Result<Self> {
        let id = block
            .header
            .hash()
            .map_err(|_| RpcError::HashCalculate.to_error())?;

        let mut transactions = vec![];
        for transaction in &block.transactions.0 {
            let transaction_response = TransactionResponse::from_transaction(transaction)?;
            transactions.push(transaction_response);
        }

        Ok(Self {
            id: id.to_base58(),
            pow_hash: block.header.pow_hash.to_base58(),
            height: block.header.height,
            timestamp: block.header.timestamp,
            prev_block: block.header.prev_block.to_base58(),
            generator: block.header.generator.to_base58(),
            generator_public_key: block.header.generator_public_key.to_base58(),
            reward: block.header.reward,
            root: block.header.root.to_base58(),
            transactions_count: block.header.transactions_count,
            n_bits: block.header.n_bits,
            nonce: block.header.nonce,
            signature: block.header.signature.to_base58(),
            transactions,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    id: String,
    sender: String,
    sender_public_key: String,
    sequence_number: u64,
    fee: u64,
    timestamp: u128,
    data: DataResponse,
    signature: String,
}

impl TransactionResponse {
    pub fn from_transaction(transaction: &Transaction) -> Result<Self> {
        let id = transaction
            .hash()
            .map_err(|_| RpcError::HashCalculate.to_error())?;

        let data = DataResponse::from_data(&transaction.data);

        Ok(Self {
            id: id.to_base58(),
            sender: transaction.sender.to_base58(),
            sender_public_key: transaction.sender_public_key.to_base58(),
            sequence_number: transaction.sequence_number,
            fee: transaction.fee,
            timestamp: transaction.timestamp,
            data,
            signature: transaction.signature.to_base58(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub enum DataResponse {
    RotatePublicKey {
        public_key: String,
    },
    Transfer {
        recipient: String,
        amount: u64,
        attachment: String,
    },
}

impl DataResponse {
    pub fn from_data(data: &Data) -> Self {
        match data {
            Data::RotatePublicKey { public_key } => DataResponse::RotatePublicKey {
                public_key: public_key.to_base58(),
            },
            Data::Transfer {
                recipient,
                amount,
                attachment,
            } => DataResponse::Transfer {
                recipient: recipient.to_base58(),
                amount: *amount,
                attachment: attachment.to_string(),
            },
        }
    }
}
