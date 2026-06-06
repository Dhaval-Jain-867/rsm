use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use getrandom;

use crate::{block::{Block, Blockchain}, hash, transaction::TransactionEnvelope};

const MAX_TX_PER_BLOCK: usize = 3;

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

    pub fn mine_block(&self, blockchain: &mut Blockchain) -> (Block, usize) {
        let mut state_clone = blockchain.balance.clone();
        let mut valid_count = 0;
        let mut total_count = 0;
        let mut block_data: Vec<TransactionEnvelope> = Vec::new();
        for transaction_envelope in blockchain.mempool.iter() {
            // let transaction_envelope = blockchain.mempool.pop_front().unwrap();
            total_count += 1;
            if transaction_envelope.is_valid(&state_clone) {
                state_clone.transfer(&transaction_envelope.payload);
                block_data.push(transaction_envelope.clone());
                valid_count += 1;
            }

            if valid_count == MAX_TX_PER_BLOCK {
                break;
            }
        }

        let last_block = blockchain.chain.last();
        let timestamp = Utc::now().timestamp();

        let mut new_block = Block::new(0, timestamp, block_data, String::from("0000000000000000000000000000000000000000000000000000000000000000"));
        match last_block {
            Some(l_block) => {
                // let new_block = Block::new(l_block.index + 1, timestamp, block_data, l_block.hash.clone());
                new_block.index = l_block.index + 1;
                new_block.previous_hash = l_block.hash.clone();
            }
            None => {

            }
        }
        let final_block = Miner::hash_block(&mut new_block);

        return (final_block.clone(), total_count as usize);
    }

    pub fn hash_block(block: &mut Block) -> &Block {
        let mut nonce = 0;
        loop {
            let our_hash = block.calculate_hash(nonce);
            if hash::is_hash_valid(&our_hash, 3) {
                block.hash = our_hash;
                block.nonce = nonce;
                break;
            }
            nonce += 1;
        }
        block
    }
}