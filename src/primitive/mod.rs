use blake2::{digest::consts::U32, Blake2b};
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

pub type Address = [u8; 32];
pub type PublicKey = [u8; 32];
pub type SecretKey = [u8; 32];

pub type Hash = [u8; 32];
pub type Signature = [u8; 64];

pub type Blake2b256 = Blake2b<U32>;
