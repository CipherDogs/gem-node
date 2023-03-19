use crate::{account, block, constants::*, tx, types::*};
use anyhow::{anyhow, Result};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};

pub struct Database {
    db: DB,
}

impl Database {
    pub fn new(path: &str) -> Self {
        let mut options = Options::default();
        options.create_missing_column_families(true);
        options.create_if_missing(true);

        let db = DB::open_cf_descriptors(&options, path, Self::descriptors())
            .unwrap_or_else(|error| panic!("Failed to open the database {path}: {error}"));

        Self { db }
    }

    pub fn put_block_header(&self, header: &block::Header) -> Result<()> {
        let value = bincode::serialize(&header)
            .map_err(|error| anyhow!("Failed to serialize block: {error:?}"))?;

        self.put(BLOCK_HEADERS, &header.hash(), &value)?;
        self.put(
            BLOCK_HEADERS_HASH,
            &header.height.to_le_bytes(),
            &header.hash(),
        )?;
        self.put(INFO, b"last_header", &header.hash())?;

        Ok(())
    }

    pub fn get_block_header_from_hash(&self, hash: Hash) -> Result<block::Header> {
        let bytes = self.get(BLOCK_HEADERS, &hash)?;
        let header: block::Header = bincode::deserialize(&bytes[..])
            .map_err(|error| anyhow!("Failed to deserialize block: {error:?}"))?;

        Ok(header)
    }

    pub fn get_block_header_from_height(&self, height: u64) -> Result<block::Header> {
        let bytes = self.get(BLOCK_HEADERS_HASH, &height.to_le_bytes())?;

        let mut hash = [0u8; 32];
        hash.copy_from_slice(bytes.as_slice());

        Ok(self.get_block_header_from_hash(hash)?)
    }

    pub fn get_last_block_header(&self) -> Result<block::Header> {
        let bytes = self.get(INFO, b"last_header")?;

        let mut hash = [0u8; 32];
        hash.copy_from_slice(bytes.as_slice());

        Ok(self.get_block_header_from_hash(hash)?)
    }

    pub fn put_block_transactions(
        &self,
        block_hash: Hash,
        block_transactions: &block::BlockTransactions,
    ) -> Result<()> {
        self.put(
            BLOCK_TRANSACTIONS,
            &block_hash,
            block_transactions.to_vec_hash_bytes()?.as_slice(),
        )?;

        Ok(())
    }

    pub fn get_block_transactions(&self, hash: Hash) -> Result<block::BlockTransactions> {
        let bytes = self.get(BLOCK_TRANSACTIONS, &hash)?;

        let mut block_transactions = block::BlockTransactions::default();

        for chunk in bytes.chunks(32) {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(chunk);
            block_transactions.txs.push(self.get_transaction(hash)?);
        }

        Ok(block_transactions)
    }

    pub fn put_transaction(&self, tx: &tx::Transaction) -> Result<()> {
        let value = bincode::serialize(&tx)
            .map_err(|error| anyhow!("Failed to serialize transaction: {error:?}"))?;

        self.put(TRANSACTIONS, &tx.hash()?, &value)?;

        Ok(())
    }

    pub fn get_transaction(&self, hash: Hash) -> Result<tx::Transaction> {
        let bytes = self.get(TRANSACTIONS, &hash)?;
        let tx: tx::Transaction = bincode::deserialize(&bytes[..])
            .map_err(|error| anyhow!("Failed to deserialize transaction: {error:?}"))?;

        Ok(tx)
    }

    pub fn put_account(&self, account: &account::Account) -> Result<()> {
        let value = bincode::serialize(&account)
            .map_err(|error| anyhow!("Failed to serialize account: {error:?}"))?;

        self.put(ACCOUNTS, &account.public_key, &value)?;
        self.put(ACCOUNTS_PUBLIC_KEY, &account.address, &account.public_key)?;

        Ok(())
    }

    pub fn get_account_from_public_key(&self, public_key: PublicKey) -> Result<account::Account> {
        let bytes = self.get(ACCOUNTS, &public_key)?;
        let account: account::Account = bincode::deserialize(&bytes[..])
            .map_err(|error| anyhow!("Failed to deserialize account: {error:?}"))?;

        Ok(account)
    }

    pub fn get_account_from_address(&self, address: Address) -> Result<account::Account> {
        let bytes = self.get(ACCOUNTS_PUBLIC_KEY, &address)?;

        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(bytes.as_slice());

        Ok(self.get_account_from_public_key(public_key)?)
    }

    pub fn put_account_transactions(
        &self,
        public_key: PublicKey,
        account_transactions: &account::AccountTransactions,
    ) -> Result<()> {
        self.put(
            ACCOUNTS_TRANSACTIONS,
            &public_key,
            account_transactions.to_vec_hash_bytes()?.as_slice(),
        )?;

        Ok(())
    }

    pub fn get_account_transactions(
        &self,
        public_key: PublicKey,
    ) -> Result<account::AccountTransactions> {
        let bytes = self.get(ACCOUNTS_TRANSACTIONS, &public_key)?;

        let mut account_transactions = account::AccountTransactions::default();

        for chunk in bytes.chunks(32) {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(chunk);
            account_transactions.txs.push(self.get_transaction(hash)?);
        }

        Ok(account_transactions)
    }

    fn descriptors() -> Vec<ColumnFamilyDescriptor> {
        let options = Options::default();

        vec![
            ColumnFamilyDescriptor::new(BLOCK_HEADERS, options.clone()),
            ColumnFamilyDescriptor::new(BLOCK_HEADERS_HASH, options.clone()),
            ColumnFamilyDescriptor::new(BLOCK_TRANSACTIONS, options.clone()),
            ColumnFamilyDescriptor::new(TRANSACTIONS, options.clone()),
            ColumnFamilyDescriptor::new(ACCOUNTS, options.clone()),
            ColumnFamilyDescriptor::new(ACCOUNTS_PUBLIC_KEY, options.clone()),
            ColumnFamilyDescriptor::new(INFO, options),
        ]
    }

    fn put(&self, cf: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let cf = self
            .db
            .cf_handle(cf)
            .ok_or_else(|| anyhow!("Failed column family handle"))?;

        self.db
            .put_cf(cf, key, value)
            .map_err(|error| anyhow!("Failed to write to the database: {error:?}"))
    }

    fn get(&self, cf: &str, key: &[u8]) -> Result<Vec<u8>> {
        let cf = self
            .db
            .cf_handle(cf)
            .ok_or_else(|| anyhow!("Failed column family handle"))?;

        match self.db.get_cf(cf, key) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(anyhow!("Value not found")),
            Err(error) => Err(anyhow!("Failed to reading data from the database: {error}")),
        }
    }
}
