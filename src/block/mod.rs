pub mod genesis;

use crate::{
    constants::*, pow::randomx::RandomXVMInstance, primitive::*, transaction::Transactions,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: Header,
    pub transactions: Transactions,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Header {
    pub height: u64,
    pub timestamp: u128,
    pub prev_block: Hash,
    pub generator: Address,
    pub generator_public_key: PublicKey,
    pub reward: u64,
    pub root: Hash,
    pub transactions_count: u64,
    pub pow_hash: Hash,
    pub n_bits: u32,
    pub nonce: u64,
    #[serde(with = "BigArray")]
    pub signature: Signature,
}

impl Header {
    pub fn new(
        height: u64,
        timestamp: u128,
        prev_block: Hash,
        generator: Address,
        generator_public_key: PublicKey,
        reward: u64,
        root: Hash,
        transactions_count: u64,
    ) -> Self {
        Self {
            height,
            timestamp,
            prev_block,
            generator,
            generator_public_key,
            reward,
            root,
            transactions_count,
            pow_hash: EMPTY_HASH,
            n_bits: 0,
            nonce: 0,
            signature: EMPTY_SIGNATURE,
        }
    }

    /// Block header PoW hash calculation
    pub fn pow_hash(&mut self, randomx_vm: &RandomXVMInstance) -> Result<()> {
        let bytes = randomx_vm.calculate_hash(&self.hash()?)?;

        let mut hash = EMPTY_HASH;
        hash.copy_from_slice(bytes.as_slice());
        self.pow_hash = hash;

        Ok(())
    }
}

impl Cryptography for Header {
    fn signer_public_key(&self) -> PublicKey {
        self.generator_public_key
    }

    fn signature(&self) -> Signature {
        self.signature
    }

    fn update_signature(&mut self, signature: Signature) {
        self.signature = signature
    }

    fn as_data_for_signing(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];

        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.prev_block);
        bytes.extend_from_slice(&self.generator);
        bytes.extend_from_slice(&self.generator_public_key);
        bytes.extend_from_slice(&self.reward.to_le_bytes());
        bytes.extend_from_slice(&self.root);
        bytes.extend_from_slice(&self.transactions_count.to_le_bytes());
        bytes.extend_from_slice(&self.n_bits.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{account::Account, primitive::Network, wallet};

    #[test]
    fn signature_verify() {
        let (secret_key, public_key) = wallet::generate();
        let account = Account::from_public_key(public_key, Network::Testnet);

        let mut header = Header::new(
            0,
            0,
            EMPTY_HASH,
            account.address,
            public_key,
            1024,
            EMPTY_HASH,
            0,
        );
        header.sign(&secret_key).unwrap();

        assert!(header.signature_verify().is_ok());
    }
}
