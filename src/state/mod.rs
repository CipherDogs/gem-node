mod database;

use crate::{
    account::Account,
    block::{genesis, Block, Header},
    constants::*,
    pow::{
        lwma::Lwma1,
        randomx::{RandomXFactory, RandomXVMInstance},
    },
    primitive::*,
    transaction::Transaction,
};
use anyhow::{anyhow, Result};
use base58::ToBase58;
use database::Database;
use std::{collections::HashMap, str::FromStr};

pub struct State {
    pub database: Database,
    mempool: Vec<Transaction>,
    randomx: RandomXFactory,
    pub lwma1: Lwma1,
    pub last_header: Header,
    network: Network,
    pub is_sync: bool,
}

impl State {
    /// Blockchain state initialization
    pub fn new(path: &str, network: Network) -> Result<Self> {
        let database = Database::new(path);

        // Initializing RandomX
        let randomx = RandomXFactory::default();

        // Initializing LWMA-1
        let mut lwma1 = Lwma1::default();
        // TODO: These values are taken to verify the performance of PoW from the ZCash network
        match network {
            Network::Mainnet => lwma1.set_pow_limit(U256::from_str(
                "0007ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            )?),
            Network::Testnet => lwma1.set_pow_limit(U256::from_str(
                "07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            )?),
        }

        let last_header = match database.get_last_block_header() {
            Ok(header) => header,
            Err(_) => {
                let block = genesis::simple();

                let mut batch = database.create_batch();

                database.put_block_header(&mut batch, &block.header)?;
                database.put_block_transactions(
                    &mut batch,
                    block.header.hash()?,
                    &block.transactions,
                )?;

                database.write(batch)?;

                block.header
            }
        };

        let height = last_header.height;

        let mut state = State {
            database,
            mempool: vec![],
            randomx,
            lwma1,
            last_header,
            network,
            is_sync: false,
        };

        Self::lwma_calculate(&mut state, height)?;

        Ok(state)
    }

    /// Put a block to the state of the blockchain
    pub fn put_block(&mut self, block: &Block) -> Result<()> {
        let mut batch = self.database.create_batch();

        let mut accounts: HashMap<Address, Account> = HashMap::new();
        let fees = 0;

        // TODO: Create Diff all transactions in block
        // Verification of all transactions:
        // * Signature
        // * Existence of the sender and availability of the required amount of money
        // * Fee check
        // Create or Update account (balance or pub key)

        // TODO: Apply Diff to DB

        // Аccrue a reward to the miner
        let mut generator = if let Some(account) = accounts.get(&block.header.generator) {
            account.clone()
        } else if let Ok(account) = self
            .database
            .get_account_from_address(block.header.generator)
        {
            account
        } else {
            Account::from_public_key(block.header.generator_public_key, self.network)
        };

        generator.balance += block.header.reward;
        generator.balance += fees;

        accounts.insert(generator.address, generator);

        for account in accounts.values() {
            self.database.put_account(&mut batch, account)?;
        }

        // Adding block and block transactions to the database
        self.database.put_block_header(&mut batch, &block.header)?;
        self.database.put_block_transactions(
            &mut batch,
            block.header.hash()?,
            &block.transactions,
        )?;

        // Writing to the database
        self.database.write(batch)?;
        // Update the last block
        self.last_header = block.header.clone();

        // Mining difficulty recalculation
        self.lwma_calculate(self.last_header.height)?;

        Ok(())
    }

    /// Put a transaction to the mempool of the blockchain
    pub fn put_transaction_mempool(&mut self, transaction: Transaction) -> Result<()> {
        let id = transaction
            .hash()
            .map_err(|error| anyhow!("Failed to calculate hash transactions: {error:?}"))?
            .to_base58();

        self.mempool.push(transaction);
        log::trace!("Added transaction to a mempool: {}", id);

        Ok(())
    }

    /// Creating a new RandomX instance from height
    pub fn create_randomx_vm_from_height(&self, height: u64) -> Result<RandomXVMInstance> {
        let height = height.div_euclid(RANDOMX_CHANGE_KEY) * RANDOMX_CHANGE_KEY;
        let header = self.database.get_block_header_from_height(height)?;
        self.randomx.create(&header.hash()?)
    }

    /// Calculation of a new difficulty target
    fn lwma_calculate(&mut self, height: u64) -> Result<()> {
        let count = std::cmp::min(height, LWMA_NUMBER_BLOCKS);
        let headers = self.database.get_block_headers_from_height(height, count)?;

        self.lwma1.calculate(headers)
    }
}
