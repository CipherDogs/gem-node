use crate::types::*;
use anyhow::{anyhow, Result};
use blake2::Digest;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    sender_public_key: PublicKey,
    sequence_number: u64,
    fee: u64,
    timestamp: u64,
    data: TransactionData,
    #[serde(with = "BigArray")]
    signature: Signature,
}

impl Transaction {
    pub fn type_id(&self) -> u8 {
        self.data.type_id()
    }

    pub fn sender_public_key(&self) -> PublicKey {
        self.sender_public_key
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn amount(&self) -> u64 {
        match self.data {
            TransactionData::Transfer { amount, .. } => amount,
            _ => 0,
        }
    }

    pub fn fee(&self) -> u64 {
        self.fee
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn hash(&self) -> Result<Hash> {
        let mut hasher = Blake2b256::new();

        hasher.update(self.sender_public_key);
        hasher.update(self.sequence_number.to_le_bytes());
        hasher.update(self.fee.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());

        let data = bincode::serialize(&self.data)
            .map_err(|error| anyhow!("Failed to serialize data: {error:?}"))?;

        hasher.update(data);

        Ok(hasher.finalize().into())
    }

    pub fn signature_verify(&self) -> Result<()> {
        let public_key = ed25519_dalek::PublicKey::from_bytes(&self.sender_public_key)
            .map_err(|error| anyhow!("Public key serialization failed: {error:?}"))?;

        let mut message: Vec<u8> = vec![];
        message.extend_from_slice(&self.sender_public_key);
        message.extend_from_slice(&self.sequence_number.to_le_bytes());
        message.extend_from_slice(&self.fee.to_le_bytes());
        message.extend_from_slice(&self.timestamp.to_le_bytes());

        let data = bincode::serialize(&self.data)
            .map_err(|error| anyhow!("Failed to serialize data: {error:?}"))?;

        message.extend_from_slice(&data);

        let signature = ed25519_dalek::Signature::from_bytes(&self.signature)
            .map_err(|error| anyhow!("Signature serialization failed: {error:?}"))?;

        public_key
            .verify_strict(message.as_slice(), &signature)
            .map_err(|_error| anyhow!("Transaction has no valid signature"))
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
