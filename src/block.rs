use std::collections::VecDeque;

use chrono::Utc;
use sha2::{Sha256, Digest};
use hex;
use borsh;

use crate::transaction::{CoinbaseTransaction, Transaction, TransactionEnvelope};
use crate::balances::Balance;
use crate::hash;

const PER_TX_REWARD: u64 = 50;

#[derive(Clone)]
pub struct Block {
    pub index: u128,
    pub timestamp: i64,
    pub reward: CoinbaseTransaction,
    pub data: Vec<TransactionEnvelope>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u128,
}

pub struct Blockchain {
    pub balance: Balance,
    pub chain: Vec<Block>,
    pub mempool: VecDeque<TransactionEnvelope>
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            balance: Balance::new(),
            chain: Vec::new(),
            mempool: VecDeque::new()
        }
    }

    pub fn add_block(&mut self, block: Block, total_count: usize) -> Result<String, String>{
        let mut state_clone = self.balance.clone();
        for transaction in &block.data {
            if !transaction.is_valid(&state_clone) {
                return Err(String::from("An invalid transaction was present in the block"));
            }
            state_clone.transfer(&transaction.payload);
        }

        if block.is_valid() && block.reward.amount == PER_TX_REWARD {
            *state_clone.accounts.entry(block.reward.receiver).or_insert(0) += block.reward.amount;
            self.chain.push(block);
            self.balance = state_clone;
            self.mempool.drain(0..total_count);
            return Ok(String::from("Block successfully mined"));
        }

        return Err(String::from("Block is invalid"));
    }

    pub fn is_valid(&self) -> bool {
        let chain_length = self.chain.len();
        for i in 0..chain_length {
            let curr_block = &self.chain[i];
            if i != 0 {
                if curr_block.previous_hash == (&self.chain[i-1]).hash && curr_block.is_valid() {
                    continue;
                } else {
                    return false;
                }
            } else {
                if curr_block.is_valid() {
                    continue;
                } else {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn mint(&mut self, amount: u64, address: [u8; 32]) {
        *self.balance.accounts.entry(address).or_insert(0) += amount;
    }

    pub fn submit_transaction(&mut self, transaction_envelope: TransactionEnvelope) -> Result<String, String>{
        if transaction_envelope.is_valid(&self.balance.clone()) {
            self.mempool.push_back(transaction_envelope);
            return Ok(String::from("Transaction added to mempool"));
        }
        return Err(String::from("Can't enter mempool since transaction was invalid"));
    }
}


impl Block {
    pub fn new(index: u128, timestamp: i64, reward: CoinbaseTransaction, data: Vec<TransactionEnvelope>, previous_hash: String) -> Self {
        Self {
            index: index,
            timestamp: timestamp,
            reward: reward,
            data: data,
            previous_hash: previous_hash,
            hash: String::from(""),
            nonce: 0
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.hash == self.calculate_hash(self.nonce) && hash::is_hash_valid(&self.hash, 3) {
            return true;
        }
        return false;
    }

    pub fn calculate_hash(&self, nonce: u128) -> String {
        hash::generate_hash(&self.reward, &self.data, self.timestamp, self.index, Some(&self.previous_hash), nonce)
    }
}