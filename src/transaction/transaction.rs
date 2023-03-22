use crate::{constants::*, primitive::*, transaction::Data};
use anyhow::{anyhow, Result};
use blake2::Digest;
use ed25519_dalek::Signer;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// Transaction data. Data specific to a particular transaction type are stored in the `data` field
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub sender_public_key: PublicKey,
    pub sequence_number: u64,
    pub fee: u64,
    pub timestamp: u128,
    pub data: Data,
    #[serde(with = "BigArray")]
    pub signature: Signature,
}

impl Transaction {
    pub fn new(sequence_number: u64, fee: u64, timestamp: u128, data: Data) -> Self {
        Self {
            sender_public_key: EMPTY_PUBLIC_KEY,
            sequence_number,
            fee,
            timestamp,
            data,
            signature: EMPTY_SIGNATURE,
        }
    }

    /// Getting the type_id depending on the transaction type
    pub fn type_id(&self) -> u8 {
        self.data.type_id()
    }

    /// Getting the amount depending on the type of transaction
    pub fn amount(&self) -> u64 {
        match self.data {
            Data::Transfer { amount, .. } => amount,
            _ => 0,
        }
    }

    /// Transaction hash calculation
    pub fn hash(&self) -> Result<Hash> {
        let mut hasher = Blake2b256::new();

        let bytes = self.to_vec_bytes()?;
        hasher.update(bytes.as_slice());

        Ok(hasher.finalize().into())
    }

    /// Signing a transaction with a private key
    pub fn sign(&mut self, secret_key: &SecretKey) -> Result<()> {
        let secret_key = ed25519_dalek::SecretKey::from_bytes(secret_key.as_slice())
            .map_err(|error| anyhow!("Secret key serialization failed: {error:?}"))?;

        let public_key = ed25519_dalek::PublicKey::from(&secret_key);
        self.sender_public_key = public_key.to_bytes();

        let keypair = ed25519_dalek::Keypair {
            secret: secret_key,
            public: public_key,
        };

        let message = self.to_vec_bytes()?;

        let signature = keypair.try_sign(message.as_slice())?;
        self.signature = signature.to_bytes();

        Ok(())
    }

    /// Transaction signature verification
    pub fn signature_verify(&self) -> Result<()> {
        let public_key = ed25519_dalek::PublicKey::from_bytes(&self.sender_public_key)
            .map_err(|error| anyhow!("Public key serialization failed: {error:?}"))?;

        let message = self.to_vec_bytes()?;

        let signature = ed25519_dalek::Signature::from_bytes(&self.signature)
            .map_err(|error| anyhow!("Signature serialization failed: {error:?}"))?;

        public_key
            .verify_strict(message.as_slice(), &signature)
            .map_err(|error| anyhow!("Transaction has no valid signature: {error:?}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{account::Account, wallet};

    #[test]
    fn signature_verify() {
        let data = Data::RotatePublicKey {
            public_key: EMPTY_PUBLIC_KEY,
        };
        let mut transaction = Transaction::new(0, 1024, 0, data);

        let (secret_key, _) = wallet::generate();
        transaction.sign(&secret_key).unwrap();

        assert!(transaction.signature_verify().is_ok());
    }

    #[test]
    fn rotate_public_key() {
        let data = Data::RotatePublicKey {
            public_key: EMPTY_PUBLIC_KEY,
        };
        let transaction = Transaction::new(0, 1024, 0, data);

        assert_eq!(transaction.type_id(), 1);
        assert_eq!(transaction.amount(), 0);
    }

    #[test]
    fn transfer() {
        let (_, public_key) = wallet::generate();
        let account = Account::from_public_key(public_key);

        let data = Data::Transfer {
            recipient: account.address,
            amount: 1024,
            attachment: String::from("test"),
        };
        let transaction = Transaction::new(0, 1024, 0, data);

        assert_eq!(transaction.type_id(), 2);
        assert_eq!(transaction.amount(), 1024);
    }
}
