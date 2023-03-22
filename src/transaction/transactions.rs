use crate::{primitive::*, transaction::Transaction};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

impl Transactions {
    /// Getting the number of transactions
    pub fn len(&self) -> u64 {
        self.transactions.len() as u64
    }

    /// Getting a Vec<Hash> vector of transaction hashes
    pub fn to_vec_hash(&self) -> Result<Vec<Hash>> {
        let mut result = vec![];

        for transaction in self.transactions.iter() {
            let hash = transaction.hash()?;

            result.push(hash);
        }

        Ok(result)
    }

    /// Getting a Vec<u8> vector of transaction hashes
    pub fn to_vec_hash_bytes(&self) -> Result<Vec<u8>> {
        let mut result = vec![];

        for transaction in self.transactions.iter() {
            let hash = transaction.hash()?;

            result.extend_from_slice(&hash);
        }

        Ok(result)
    }

    /// Adding a transaction
    pub fn push(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }
}
