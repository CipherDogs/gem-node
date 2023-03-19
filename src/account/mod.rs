use crate::{tx::Transaction, types::*};
use anyhow::Result;
use blake2::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Account {
    pub address: Address,
    pub public_key: PublicKey,
    pub balance: u64,
    sequence_number: u64,
}

impl Account {
    pub fn from_address(address: Address, balance: u64) -> Self {
        Self {
            address,
            public_key: [0u8; 32],
            balance,
            sequence_number: 0,
        }
    }

    pub fn from_public_key(public_key: PublicKey) -> Self {
        let mut hasher = Blake2b256::new();
        hasher.update(public_key);
        let address = hasher.finalize().into();

        Self {
            address,
            public_key,
            balance: 0,
            sequence_number: 0,
        }
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn inc_sequence_number(&mut self) {
        self.sequence_number += 1;
    }
}

pub struct AccountTransactions {
    pub txs: Vec<Transaction>,
}

impl Default for AccountTransactions {
    fn default() -> Self {
        Self {
            txs: Default::default(),
        }
    }
}

impl AccountTransactions {
    pub fn to_vec_hash_bytes(&self) -> Result<Vec<u8>> {
        let mut result = vec![];

        for tx in self.txs.iter() {
            let hash = tx.hash()?;

            result.extend_from_slice(&hash);
        }

        Ok(result)
    }
}
