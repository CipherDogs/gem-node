use crate::primitive::*;
use serde::{Deserialize, Serialize};

/// Data specific to a particular transaction type
#[derive(Serialize, Deserialize, Debug)]
pub enum Data {
    RotatePublicKey {
        public_key: PublicKey,
    },
    Transfer {
        recipient: Address,
        amount: u64,
        attachment: String,
    },
}

impl Data {
    /// Getting the type_id depending on the transaction type
    pub fn type_id(&self) -> u8 {
        match self {
            Self::RotatePublicKey { .. } => 1,
            Self::Transfer { .. } => 2,
        }
    }
}
