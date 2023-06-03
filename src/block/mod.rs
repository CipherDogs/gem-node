pub mod genesis;
mod header;

use crate::{
    primitive::*,
    state::State,
    transaction::{MerkleTree, Transactions},
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub use header::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: Header,
    pub transactions: Transactions,
}

impl Validation for Block {
    fn is_valid(&self, state: &State) -> Result<()> {
        if let Err(error) = self.header.is_valid(state) {
            Err(anyhow!("Block is not valid: {error:?}"))
        } else if self.header.transactions_count != self.transactions.len() {
            Err(anyhow!(
                "Number of transactions does not match: {:?}",
                self.header
            ))
        } else if !MerkleTree::verify(&self.transactions, self.header.root)? {
            Err(anyhow!(
                "Merkle tree hash does not match: {:?}",
                self.header
            ))
        } else {
            for transaction in &self.transactions.0 {
                transaction.is_valid(state)?;
            }

            Ok(())
        }
    }
}
