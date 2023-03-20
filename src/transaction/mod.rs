use crate::primitive::*;
use anyhow::{anyhow, Result};
use blake2::Digest;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub sender_public_key: PublicKey,
    pub sequence_number: u64,
    pub fee: u64,
    pub timestamp: u64,
    pub data: TransactionData,
    #[serde(with = "BigArray")]
    pub signature: Signature,
}

impl Transaction {
    pub fn type_id(&self) -> u8 {
        self.data.type_id()
    }

    pub fn amount(&self) -> u64 {
        match self.data {
            TransactionData::Transfer { amount, .. } => amount,
            _ => 0,
        }
    }

    pub fn hash(&self) -> Result<Hash> {
        let mut hasher = Blake2b256::new();

        let bytes = self.to_vec_bytes()?;
        hasher.update(bytes.as_slice());

        Ok(hasher.finalize().into())
    }

    pub fn signature_verify(&self) -> Result<()> {
        let public_key = ed25519_dalek::PublicKey::from_bytes(&self.sender_public_key)
            .map_err(|error| anyhow!("Public key serialization failed: {error:?}"))?;

        let message = self.to_vec_bytes()?;

        let signature = ed25519_dalek::Signature::from_bytes(&self.signature)
            .map_err(|error| anyhow!("Signature serialization failed: {error:?}"))?;

        public_key
            .verify_strict(message.as_slice(), &signature)
            .map_err(|_error| anyhow!("Transaction has no valid signature"))
    }

    fn to_vec_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];

        bytes.extend_from_slice(&self.sender_public_key);
        bytes.extend_from_slice(&self.sequence_number.to_le_bytes());
        bytes.extend_from_slice(&self.fee.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());

        let data = bincode::serialize(&self.data)
            .map_err(|error| anyhow!("Failed to serialize data: {error:?}"))?;

        bytes.extend_from_slice(&data);

        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TransactionData {
    RotatePublicKey {
        public_key: PublicKey,
    },
    Transfer {
        recipient: Address,
        amount: u64,
        attachment: String,
    },
}

impl TransactionData {
    pub fn type_id(&self) -> u8 {
        match self {
            Self::RotatePublicKey { .. } => 1,
            Self::Transfer { .. } => 2,
        }
    }
}
