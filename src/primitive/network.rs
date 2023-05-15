use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Network {
    Testnet = 0,
    Mainnet = 1,
}
