use crate::{constants::*, primitive::*, transaction::Transactions};
use anyhow::Result;
use blake2::Digest;

pub struct MerkleTree {
    pub nodes: Vec<Hash>,
    pub levels: usize,
}

impl MerkleTree {
    /// Constructs a Merkle tree from given transactions
    pub fn construct(transactions: &Transactions) -> Result<MerkleTree> {
        let mut transactions_hashes = transactions.to_vec_hash()?;

        if transactions_hashes.is_empty() {
            transactions_hashes.push(EMPTY_HASH);
            transactions_hashes.push(EMPTY_HASH);
        } else if transactions_hashes.len() % 2 != 0 {
            transactions_hashes.push(EMPTY_HASH);
        }

        let mut hashes = vec![transactions_hashes.clone()];
        let mut last_level = &hashes[0];

        let num_levels = (transactions_hashes.len() as f64).log2() as usize;

        for _ in 0..num_levels {
            let mut next_level = vec![MerkleTree::construct_level_up(last_level)];
            hashes.append(&mut next_level);
            last_level = &hashes[hashes.len() - 1];
        }

        Ok(MerkleTree {
            nodes: hashes.into_iter().flatten().collect(),
            levels: num_levels + 1,
        })
    }

    fn construct_level_up(level: &[Hash]) -> Vec<Hash> {
        level
            .chunks(2)
            .map(|pair| hash_concat(pair[0], pair[1]))
            .collect()
    }

    /// Returns the root hash of the Merkle tree
    pub fn root_hash(&self) -> Hash {
        self.nodes[self.nodes.len() - 1]
    }

    /// Verifies that the correct root hash is obtained from the input transactions
    pub fn verify(transactions: &Transactions, root_hash: Hash) -> Result<bool> {
        let merkle_tree = MerkleTree::construct(transactions)?;
        Ok(merkle_tree.root_hash() == root_hash)
    }
}

fn hash_concat(h1: Hash, h2: Hash) -> Hash {
    let mut hasher = Blake2b256::new();

    hasher.update(h1.as_slice());
    hasher.update(h2.as_slice());

    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        transaction::{Data, Transaction},
        wallet,
    };

    #[test]
    fn verify_transactions_default() {
        let transactions = Transactions::default();
        let mut hashes = transactions.to_vec_hash().unwrap();

        for _ in 1..3 {
            hashes.push(EMPTY_HASH);
        }

        let root_hash = hash_concat(hashes[0], hashes[1]);
        assert!(MerkleTree::verify(&transactions, root_hash).unwrap());
    }

    #[test]
    fn verify_one_transactions() {
        let data = Data::RotatePublicKey {
            public_key: EMPTY_PUBLIC_KEY,
        };
        let mut transaction = Transaction::new(EMPTY_ADDRESS, 0, 1024, 0, data);

        let (secret_key, _) = wallet::generate();
        transaction.sign(&secret_key).unwrap();

        let mut transactions = Transactions::default();
        transactions.push(transaction);

        let mut hashes = transactions.to_vec_hash().unwrap();
        hashes.push(EMPTY_HASH);

        let root_hash = hash_concat(hashes[0], hashes[1]);
        assert!(MerkleTree::verify(&transactions, root_hash).unwrap());
    }
}
