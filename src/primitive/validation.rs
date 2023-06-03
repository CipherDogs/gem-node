use crate::{state::State, transaction::Data};
use anyhow::Result;

pub trait Validation {
    fn is_valid(&self, state: &State) -> Result<()>;

    fn reward_is_valid(&self, _reward: u64) -> bool {
        true // TODO
    }

    fn minimum_fee(&self, data: &Data) -> u64 {
        match data {
            Data::RotatePublicKey { .. } => 100000,
            Data::Transfer { .. } => 100000,
        }
    }
}
