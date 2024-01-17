use crate::{
    block::{Block, Header},
    constants::*,
    transaction::Transactions,
};

pub fn simple() -> Block {
    let timestamp = 0;

    let header = Header::new(
        0,
        timestamp,
        EMPTY_HASH,
        EMPTY_ADDRESS,
        EMPTY_PUBLIC_KEY,
        0,
        EMPTY_HASH,
        0,
    );
    let transactions = Transactions::default();

    Block {
        header,
        transactions,
    }
}
