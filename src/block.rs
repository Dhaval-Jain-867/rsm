use std::collections::VecDeque;

use chrono::Utc;
use sha2::{Sha256, Digest};
use hex;
use borsh;

use crate::transaction::{Transaction, TransactionEnvelope};
use crate::balances::Balance;
use crate::hash;

#[derive(Clone)]
pub struct Block {
    pub index: u128,
    pub timestamp: i64,
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

    pub fn add_block(&mut self, block: Block) -> Result<String, String>{
        for transaction in &block.data {
            if !transaction.is_valid(&self.balance) {
                return Err(String::from("Invalid transaction"));
            }
        }

        for transaction in &block.data {
            self.balance.transfer(&transaction.payload);
        }

        if block.is_valid() {
            self.chain.push(block);
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
}


impl Block {
    pub fn new(index: u128, timestamp: i64, data: Vec<TransactionEnvelope>, previous_hash: String) -> Self {
        Self {
            index: index,
            timestamp: timestamp,
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
        hash::generate_hash(&self.data, self.timestamp, self.index, Some(&self.previous_hash), nonce)
    }
}