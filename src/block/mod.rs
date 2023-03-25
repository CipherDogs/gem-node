pub mod genesis;

use crate::{
    constants::*, pow::randomx::RandomXVMInstance, primitive::*, transaction::Transactions,
};
use anyhow::{anyhow, Result};
use blake2::Digest;
use ed25519_dalek::Signer;
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
    pub generator_public_key: PublicKey,
    pub reward: u64,
    pub root: Hash,
    pub transactions_count: u64,
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
        reward: u64,
        root: Hash,
        transactions_count: u64,
    ) -> Self {
        Self {
            height,
            timestamp,
            prev_block,
            generator_public_key: EMPTY_PUBLIC_KEY,
            reward,
            root,
            transactions_count,
            n_bits: 0,
            nonce: 0,
            signature: EMPTY_SIGNATURE,
        }
    }

    /// Block header hash calculation
    pub fn hash(&self) -> Hash {
        let mut hasher = Blake2b256::new();

        let bytes = self.to_vec_bytes();
        hasher.update(bytes.as_slice());

        hasher.finalize().into()
    }

    /// Block header PoW hash calculation
    pub fn pow_hash(&self, randomx_vm: &RandomXVMInstance) -> Result<Hash> {
        let mut hash = EMPTY_HASH;

        let bytes = randomx_vm.calculate_hash(&self.hash())?;
        hash.copy_from_slice(bytes.as_slice());

        Ok(hash)
    }

    /// Signing a block header with a private key
    pub fn sign(&mut self, secret_key: &SecretKey) -> Result<()> {
        let secret_key = ed25519_dalek::SecretKey::from_bytes(secret_key.as_slice())
            .map_err(|error| anyhow!("Secret key serialization failed: {error:?}"))?;

        let public_key = ed25519_dalek::PublicKey::from(&secret_key);
        self.generator_public_key = public_key.to_bytes();

        let keypair = ed25519_dalek::Keypair {
            secret: secret_key,
            public: public_key,
        };

        let message = self.to_vec_bytes();

        let signature = keypair.try_sign(message.as_slice())?;
        self.signature = signature.to_bytes();

        Ok(())
    }

    /// Block header signature verification
    pub fn signature_verify(&self) -> Result<()> {
        let public_key = ed25519_dalek::PublicKey::from_bytes(&self.generator_public_key)
            .map_err(|error| anyhow!("Public key serialization failed: {error:?}"))?;

        let message = self.to_vec_bytes();

        let signature = ed25519_dalek::Signature::from_bytes(&self.signature)
            .map_err(|error| anyhow!("Signature serialization failed: {error:?}"))?;

        public_key
            .verify_strict(message.as_slice(), &signature)
            .map_err(|error| anyhow!("Block has no valid signature: {error:?}"))
    }

    fn to_vec_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.prev_block);
        bytes.extend_from_slice(&self.generator_public_key);
        bytes.extend_from_slice(&self.reward.to_le_bytes());
        bytes.extend_from_slice(&self.root);
        bytes.extend_from_slice(&self.transactions_count.to_le_bytes());
        bytes.extend_from_slice(&self.n_bits.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet;

    #[test]
    fn signature_verify() {
        let mut header = Header::new(0, 0, EMPTY_HASH, 0, EMPTY_HASH, 0);

        let (secret_key, _) = wallet::generate();
        header.sign(&secret_key).unwrap();

        assert!(header.signature_verify().is_ok());
    }
}
