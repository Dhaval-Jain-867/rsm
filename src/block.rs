use chrono::Utc;
use sha2::{Sha256, Digest};
use hex;
use borsh::BorshSerialize;

use crate::transaction::{self, Transaction};
use crate::balances::Balance;


fn generate_hash(data: &Vec<Transaction>, timestamp: i64, index: u128, previous_hash: Option<&str>, nonce: u128) -> String {
    let last_hash = previous_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let mut hasher = Sha256::new();
    let data_bytes = borsh::to_vec(data).expect("Failed to serialize transactions");
    hasher.update(data_bytes);
    hasher.update(timestamp.to_be_bytes());
    hasher.update(index.to_be_bytes());
    hasher.update(last_hash);
    hasher.update(nonce.to_be_bytes());
    let hash_result = hasher.finalize();
    let hex_hash = hex::encode(hash_result);
    return hex_hash;
}

fn is_hash_valid(hash: &str, difficulty: u8) -> bool {
    let mut test_str = String::from("");
    for _i in 0..difficulty {
        test_str.push('0');
    }
    let mut hash_str = String::from("");
    for i in 0..difficulty {
        hash_str.push(hash.chars().nth(i as usize).unwrap());  
    }
    if hash_str == test_str {
        return true;
    }
    return false;
}

pub struct Block {
    pub index: u128,
    pub timestamp: i64,
    pub data: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u128,
}

pub struct Blockchain {
    pub balance: Balance,
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            balance: Balance::new(),
            chain: Vec::new()
        }
    }

    pub fn add_block(&mut self, data: Vec<Transaction>) -> Result<String, String>{
        // check for each transaction
        // trying to implement all the transactions
        let mut state_clone = self.balance.clone();
        for transaction in &data {
            if !transaction.is_valid(&state_clone) {
                return Err(String::from("An invalid transaction was present in the block"));
            }
            state_clone.transfer(transaction);
        }
        
        let last_block = self.chain.last();
        let timestamp = Utc::now().timestamp();

        match last_block {
            Some(l_block) => {
                let mut new_block = Block::new(l_block.index + 1, timestamp, data.clone(), l_block.hash.clone());
                new_block.mine();
                self.chain.push(new_block);
            }
            None => {
                let mut new_block = Block::new(0, timestamp, data.clone(), String::from("0000000000000000000000000000000000000000000000000000000000000000"));
                new_block.mine();
                self.chain.push(new_block);
            }
        }

        self.balance = state_clone;
        return Ok(String::from("Block successfully mined"));
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
    pub fn new(index: u128, timestamp: i64, data: Vec<Transaction>, previous_hash: String) -> Self {
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
        if self.hash == self.calculate_hash(self.nonce) && is_hash_valid(&self.hash, 3) {
            return true;
        }
        return false;
    }

    pub fn calculate_hash(&self, nonce: u128) -> String {
        generate_hash(&self.data, self.timestamp, self.index, Some(&self.previous_hash), nonce)
    }

    pub fn mine(&mut self) {
        let mut nonce: u128 = 0;
        loop {
            let our_hash = self.calculate_hash(nonce);
            if is_hash_valid(&our_hash, 3) {
                self.hash = our_hash;
                self.nonce = nonce;
                break;
            }
            nonce += 1;
        }
    }
}