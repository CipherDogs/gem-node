use crate::{constants::*, primitive::*, transaction::Data};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// Transaction data. Data specific to a particular transaction type are stored in the `data` field
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub sender: Address,
    pub sender_public_key: PublicKey,
    pub sequence_number: u64,
    pub fee: u64,
    pub timestamp: u128,
    pub data: Data,
    #[serde(with = "BigArray")]
    pub signature: Signature,
}

impl Transaction {
    pub fn new(
        sender: Address,
        sender_public_key: PublicKey,
        sequence_number: u64,
        fee: u64,
        timestamp: u128,
        data: Data,
    ) -> Self {
        Self {
            sender,
            sender_public_key,
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
}

impl Cryptography for Transaction {
    fn signer_public_key(&self) -> PublicKey {
        self.sender_public_key
    }

    fn signature(&self) -> Signature {
        self.signature
    }

    fn update_signature(&mut self, signature: Signature) {
        self.signature = signature
    }

    fn as_data_for_signing(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];

        bytes.extend_from_slice(&self.sender);
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
    use crate::{account::Account, primitive::Network, wallet};

    #[test]
    fn signature_verify() {
        let (secret_key, public_key) = wallet::generate();
        let account = Account::from_public_key(public_key, Network::Testnet);

        let data = Data::RotatePublicKey {
            public_key: EMPTY_PUBLIC_KEY,
        };

        let mut transaction = Transaction::new(account.address, public_key, 0, 1024, 0, data);
        transaction.sign(&secret_key).unwrap();

        assert!(transaction.signature_verify().is_ok());
    }

    #[test]
    fn rotate_public_key() {
        let data = Data::RotatePublicKey {
            public_key: EMPTY_PUBLIC_KEY,
        };
        let transaction = Transaction::new(EMPTY_ADDRESS, EMPTY_PUBLIC_KEY, 0, 1024, 0, data);

        assert_eq!(transaction.type_id(), 1);
        assert_eq!(transaction.amount(), 0);
    }

    #[test]
    fn transfer() {
        let (_, public_key) = wallet::generate();
        let account = Account::from_public_key(public_key, Network::Testnet);

        let data = Data::Transfer {
            recipient: account.address,
            amount: 1024,
            attachment: String::from("test"),
        };
        let transaction = Transaction::new(EMPTY_ADDRESS, EMPTY_PUBLIC_KEY, 0, 1024, 0, data);

        assert_eq!(transaction.type_id(), 2);
        assert_eq!(transaction.amount(), 1024);
    }
}
