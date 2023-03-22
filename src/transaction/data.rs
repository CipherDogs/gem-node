use crate::primitive::*;
use serde::{Deserialize, Serialize};

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
    pub fn type_id(&self) -> u8 {
        match self {
            Self::RotatePublicKey { .. } => 1,
            Self::Transfer { .. } => 2,
        }
    }
}
