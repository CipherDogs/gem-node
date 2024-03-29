mod cryptography;
mod network;
mod validation;

use blake2::{digest::consts::U32, Blake2b};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use uint::construct_uint;

pub use cryptography::*;
pub use network::*;
pub use validation::*;

construct_uint! {
    pub struct U256(4);
}

pub type Address = [u8; 32];
pub type PublicKey = [u8; PUBLIC_KEY_LENGTH];
pub type SecretKey = [u8; SECRET_KEY_LENGTH];

pub type Hash = [u8; 32];
pub type Signature = [u8; SIGNATURE_LENGTH];

pub type Blake2b256 = Blake2b<U32>;
