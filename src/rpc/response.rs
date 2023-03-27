use crate::{
    block::Block,
    transaction::{Data, Transaction},
};
use base58::ToBase58;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BlockResponse {
    pub height: u64,
    pub timestamp: u128,
    pub prev_block: String,
    pub generator: String,
    pub generator_public_key: String,
    pub reward: u64,
    pub root: String,
    pub transactions_count: u64,
    pub n_bits: u32,
    pub nonce: u64,
    pub signature: String,
    pub transactions: Vec<TransactionResponse>,
}

impl BlockResponse {
    pub fn from_block(block: &Block) -> Self {
        let transactions = block
            .transactions
            .0
            .iter()
            .map(|item| TransactionResponse::from_transaction(item))
            .collect::<Vec<TransactionResponse>>();

        Self {
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
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    pub sender: String,
    pub sender_public_key: String,
    pub sequence_number: u64,
    pub fee: u64,
    pub timestamp: u128,
    pub data: DataResponse,
    pub signature: String,
}

impl TransactionResponse {
    pub fn from_transaction(transaction: &Transaction) -> Self {
        let data = DataResponse::from_data(&transaction.data);

        Self {
            sender: transaction.sender.to_base58(),
            sender_public_key: transaction.sender_public_key.to_base58(),
            sequence_number: transaction.sequence_number,
            fee: transaction.fee,
            timestamp: transaction.timestamp,
            data,
            signature: transaction.signature.to_base58(),
        }
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
