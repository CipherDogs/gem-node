use crate::{primitive::*, transaction::Transaction};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Transactions(pub Vec<Transaction>);

impl Transactions {
    /// Getting the number of transactions
    pub fn len(&self) -> u64 {
        self.0.len() as u64
    }

    /// Transactions Vec is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Getting a Vec<Hash> vector of transaction hashes
    pub fn to_vec_hash(&self) -> Result<Vec<Hash>> {
        let mut result = vec![];

        for transaction in self.0.iter() {
            let hash = transaction.hash()?;

            result.push(hash);
        }

        Ok(result)
    }

    /// Getting a Vec<u8> vector of transaction hashes
    pub fn to_vec_hash_bytes(&self) -> Result<Vec<u8>> {
        let mut result = vec![];

        for transaction in self.0.iter() {
            let hash = transaction.hash()?;

            result.extend_from_slice(&hash);
        }

        Ok(result)
    }

    /// Adding a transaction
    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}
