use crate::primitive::*;

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Empty primitives
pub const EMPTY_ADDRESS: Address = [0u8; 32];
pub const EMPTY_HASH: Hash = [0u8; 32];
pub const EMPTY_PUBLIC_KEY: PublicKey = [0u8; 32];
pub const EMPTY_SECRET_KEY: SecretKey = [0u8; 32];
pub const EMPTY_SIGNATURE: Signature = [0u8; 64];

/// RandomX change key
pub const RANDOMX_CHANGE_KEY: u64 = 8640;

/// Lwma-1 number of blocks
pub const LWMA_NUMBER_BLOCKS: u64 = 50;

/// Swarm topics
pub const BLOCK_TOPIC: &str = "block";
pub const TRANSACTION_TOPIC: &str = "transaction";

// Swarm request response
pub const MAX_TRANSMIT_SIZE: usize = 1_000_000;

/// RocksDB column family
pub const BLOCK_HEADERS: &str = "block_headers";
pub const BLOCK_HEADERS_HASH: &str = "block_headers_hash";
pub const BLOCK_TRANSACTIONS: &str = "block_transactions";
pub const TRANSACTIONS: &str = "transactions";
pub const ACCOUNTS: &str = "accounts";
pub const ACCOUNTS_PUBLIC_KEY: &str = "account_public_key";
pub const ACCOUNTS_TRANSACTIONS: &str = "accounts_transactions";
pub const INFO: &str = "info";
