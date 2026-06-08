use chrono::Utc;
use ed25519_dalek::SigningKey;
use getrandom;
use std::env;

use crate::{
    block::{Block, Blockchain},
    hash,
    transaction::{CoinbaseTransaction, TransactionEnvelope},
};

// const MAX_TX_PER_BLOCK: usize = 3;
// const PER_TX_REWARD: u64 = 50;
// const NONCE_DIFFICULTY: u8 = 3;

pub struct Miner {
    pub public_key: [u8; 32],
    pub private_key: [u8; 32],
}

impl Miner {
    pub fn new() -> Self {
        let mut secret_bytes = [0u8; 32];
        getrandom::fill(&mut secret_bytes).expect("OS failed to generate random bytes");

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let public_key = signing_key.verifying_key();

        Self {
            public_key: public_key.to_bytes(),
            private_key: secret_bytes,
        }
    }

    pub fn mine_block(&self, blockchain: &mut Blockchain) -> Result<(Block, usize), String> {
        let mut state_clone = blockchain.balance.clone();
        let mut valid_count = 0;
        let mut total_count = 0;
        let mut block_data: Vec<TransactionEnvelope> = Vec::new();
        let mut coinbase_transaction = CoinbaseTransaction {
            receiver: self.public_key,
            amount: env::var("PER_TX_REWARD").unwrap().parse().unwrap(),
        };
        let max_tx: usize = env::var("MAX_TX_PER_BLOCK").unwrap().parse().unwrap();
        for transaction_envelope in blockchain.mempool.iter() {
            total_count += 1;
            if transaction_envelope.is_valid(&state_clone) {
                state_clone.transfer(&transaction_envelope.payload);
                block_data.push(transaction_envelope.clone());
                valid_count += 1;
                coinbase_transaction.amount = coinbase_transaction
                    .amount
                    .checked_add(transaction_envelope.payload.fees)
                    .unwrap();
            }

            if valid_count == max_tx {
                break;
            }
        }

        let last_block = blockchain.chain.last();
        let timestamp = Utc::now().timestamp();

        let mut new_block = Block::new(
            0,
            timestamp,
            coinbase_transaction,
            block_data,
            String::from("0000000000000000000000000000000000000000000000000000000000000000"),
        );
        match last_block {
            Some(l_block) => {
                // let new_block = Block::new(l_block.index + 1, timestamp, block_data, l_block.hash.clone());
                new_block.index = l_block.index + 1;
                new_block.previous_hash = l_block.hash.clone();
            }
            // safety check, although impossible to reach state
            None => {
                return Err(String::from("Genesis block was not created"));
            }
        }
        let final_block = hash::hash_block(&mut new_block);

        return Ok((final_block.clone(), total_count as usize));
    }

    
}
