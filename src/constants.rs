pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

// Swarm topics
pub const BLOCK_TOPIC: &str = "block";
pub const TRANSACTION_TOPIC: &str = "transaction";

// RocksDB column family
pub const BLOCK_HEADERS: &str = "block_headers";
pub const BLOCK_HEADERS_HASH: &str = "block_headers_hash";
pub const BLOCK_TRANSACTIONS: &str = "block_transactions";
pub const TRANSACTIONS: &str = "transactions";
pub const ACCOUNTS: &str = "accounts";
pub const ACCOUNTS_PUBLIC_KEY: &str = "account_public_key";
