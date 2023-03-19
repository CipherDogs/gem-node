pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// RandomX change key
pub const RANDOMX_CHANGE_KEY: u64 = 8640;

/// Swarm topics
pub const BLOCK_TOPIC: &str = "block";
pub const TRANSACTION_TOPIC: &str = "transaction";

/// RocksDB column family
pub const BLOCK_HEADERS: &str = "block_headers";
pub const BLOCK_HEADERS_HASH: &str = "block_headers_hash";
pub const BLOCK_TRANSACTIONS: &str = "block_transactions";
pub const TRANSACTIONS: &str = "transactions";
pub const ACCOUNTS: &str = "accounts";
pub const ACCOUNTS_PUBLIC_KEY: &str = "account_public_key";
pub const ACCOUNTS_TRANSACTIONS: &str = "accounts_transactions";
pub const INFO: &str = "info";
